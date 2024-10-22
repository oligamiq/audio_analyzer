use audio_analyzer_core::prelude::*;
use mel_spec::mel::{hz_to_mel, mel_to_hz};
use ndarray::prelude::*;
use num_complex::Complex;
use plotters::prelude::*;

pub mod line_graph;

fn main() {
    let mut data = TestData::new_with_path("../test_data/jfk_f32le.wav".into());
    data.start();

    // 50ms
    let analyze_second = 0.05;
    // let analyze_second = 5.;

    let sample_rate = data.sample_rate();

    let fft_size = (analyze_second * sample_rate as f64) as usize;
    let hop_size = fft_size / 5;
    let fft_size = hop_size * 5;

    let mut fft_layer = ToSpectrogramLayer::new(FftConfig { fft_size, hop_size });

    let mut mel_layer =
        ToMelSpectrogramLayer::new(MelConfig::new(fft_size, hop_size, 64, sample_rate as f64));

    println!("fft_size: {:?}", fft_size);
    println!("hop_size: {:?}", hop_size);

    let mut n = 0;

    if std::fs::exists("out").unwrap() {
        std::fs::remove_dir_all("out").ok();

        std::fs::create_dir("out").unwrap();
    } else {
        std::fs::create_dir("out").unwrap();
    }

    let mut plot = Plot {
        log_amplitude_spectrum: vec![],
        log_mel_amplitude_spectrum: vec![],
        spectral_envelope: vec![],
    };

    while let Some(data) = data.try_recv() {
        // println!("{:?}", data.len());

        if let Ok(fft) = fft_layer.through_inner(&data) {
            for fft in fft {
                // println!("{:?}", fft.len());

                let mel = mel_layer.through_inner(&fft).unwrap().unwrap();

                let log_mel_amplitude_spectrum = log_mel_amplitude_spectrum(&mel.view());

                let log_amplitude_spectrum = log_amplitude_spectrum(&fft.view());

                let quefrency = quefrency(log_amplitude_spectrum.clone());

                let liftered = lifter(quefrency, 15);

                let spectral_envelope = spectral_envelope(liftered, log_amplitude_spectrum.len());

                let log_amplitude_spectrum_harf =
                    log_amplitude_spectrum.slice(s![..log_amplitude_spectrum.len() / 2]);

                let spectral_envelope_harf =
                    spectral_envelope.slice(s![..spectral_envelope.len() / 2]);

                n += 1;
                plot.plot(
                    log_amplitude_spectrum_harf.to_owned(),
                    log_mel_amplitude_spectrum,
                    spectral_envelope_harf.to_owned(),
                    &format!("out/mel_{n}.png"),
                    sample_rate as f64,
                    (800, 600),
                    false,
                )
                .unwrap();
            }
        }
    }

    plot.plot_3d("out/3d.svg", sample_rate as f64, (8000, 6000))
        .unwrap();
}

fn log_amplitude_spectrum(fft: &ArrayView1<Complex<f64>>) -> Array1<f64> {
    fft.mapv(|x| 10.0 * x.norm_sqr().log10())
}

fn quefrency(log_amplitude_spectrum: Array1<f64>) -> Array1<f64> {
    let ifft = rustfft::FftPlanner::new().plan_fft_inverse(log_amplitude_spectrum.len());

    let mut ifft_input = log_amplitude_spectrum
        .iter()
        .map(|x| Complex::new(*x, 0.0))
        .collect::<Vec<_>>();

    ifft.process(&mut ifft_input);

    let quefrency = ifft_input.iter().map(|x| x.re).collect::<Array1<_>>();

    quefrency
}

fn lifter(quefrency: Array1<f64>, index: usize) -> Array1<f64> {
    let mut quefrency = quefrency.clone();

    for i in 0..quefrency.len() {
        if i > index && i < quefrency.len() - index {
            quefrency[i] = 0.0;
        }
    }

    quefrency
}

fn spectral_envelope(quefrency: Array1<f64>, n: usize) -> Array1<f64> {
    let fft = rustfft::FftPlanner::new().plan_fft_forward(n);

    let mut fft_input = quefrency
        .iter()
        .map(|x| Complex::new(*x, 0.0))
        .collect::<Vec<_>>();

    fft.process(&mut fft_input);

    let spectral_envelope = fft_input.iter().map(|x| x.re).collect::<Array1<_>>();

    // データをサイズで割って正規化
    let spectral_envelope = spectral_envelope.mapv(|x| x / n as f64);

    spectral_envelope
}

fn log_mel_amplitude_spectrum(
    mel: &ArrayBase<ndarray::ViewRepr<&f64>, Dim<[usize; 1]>>,
) -> Array1<f64> {
    mel.mapv(|x| 10.0 * x.log10())
}

struct Plot {
    log_amplitude_spectrum: Vec<Array1<f64>>,
    log_mel_amplitude_spectrum: Vec<Array1<f64>>,
    spectral_envelope: Vec<Array1<f64>>,
}

impl Plot {
    pub fn plot(
        &mut self,
        log_amplitude_spectrum: Array1<f64>,
        log_mel_amplitude_spectrum: Array1<f64>,
        spectral_envelope: Array1<f64>,
        name: &str,
        sample_rate: f64,
        (width, height): (u32, u32),
        is_plot: bool,
    ) -> anyhow::Result<()> {
        let Self {
            log_amplitude_spectrum: _log_amplitude_spectrum,
            log_mel_amplitude_spectrum: _log_mel_amplitude_spectrum,
            spectral_envelope: _spectral_envelope,
        } = self;

        _log_amplitude_spectrum.push(log_amplitude_spectrum.clone());
        _log_mel_amplitude_spectrum.push(log_mel_amplitude_spectrum.clone());
        _spectral_envelope.push(spectral_envelope.clone());

        if is_plot {
            Self::plot_inner(
                log_amplitude_spectrum,
                log_mel_amplitude_spectrum,
                spectral_envelope,
                name,
                sample_rate,
                (width, height),
            )
        } else {
            Ok(())
        }
    }

    fn plot_inner(
        log_amplitude_spectrum: Array1<f64>,
        log_mel_amplitude_spectrum: Array1<f64>,
        spectral_envelope: Array1<f64>,
        name: &str,
        sample_rate: f64,
        (width, height): (u32, u32),
    ) -> anyhow::Result<()> {
        let mut root_area_buf = vec![0; (width * height * 3) as usize];

        {
            let x_fft_axis = (0.0..1.0 as f64)
                .step(1.0f64 / log_amplitude_spectrum.len() as f64)
                .values()
                .into_iter()
                .map(|x| x * sample_rate / 2.0)
                .collect::<Vec<_>>();
            let max_mel_hz = hz_to_mel(sample_rate / 2.0, false);
            let x_mel_axis = Array1::linspace(0.0, max_mel_hz, log_mel_amplitude_spectrum.len())
                .iter()
                .map(|x| mel_to_hz(*x, false))
                .collect::<Vec<_>>();

            let x_spectral_envelope_axis = (0.0..1.0 as f64)
                .step(1.0f64 / spectral_envelope.len() as f64)
                .values()
                .into_iter()
                .map(|x| x * sample_rate / 2.0)
                .collect::<Vec<_>>();

            let root_area =
                BitMapBackend::with_buffer(&mut root_area_buf, (width, height)).into_drawing_area();

            root_area.fill(&WHITE)?;

            let root_area = root_area.titled("Spectrogram", ("sans-serif", 60))?;

            let max_y = log_amplitude_spectrum
                .iter()
                .cloned()
                .fold(0.0f64, |acc, x| acc.max(x.abs()));

            let mut cc = ChartBuilder::on(&root_area)
                .margin(5)
                .set_all_label_area_size(50)
                .caption("Mel Spectrogram", ("sans-serif", 40))
                .build_cartesian_2d(
                    -0.1..sample_rate / 2.0f64 + 0.1,
                    -0.1f64 - max_y..max_y + 0.1,
                )
                .unwrap();

            cc.configure_mesh()
                .x_labels(20)
                .x_desc("Hz")
                .y_labels(10)
                .y_desc("dB")
                .disable_mesh()
                .x_label_formatter(&|v| format!("{:.1}", v))
                .y_label_formatter(&|v| format!("{:.1}", v))
                .draw()?;

            cc.draw_series(LineSeries::new(
                x_fft_axis
                    .iter()
                    .cloned()
                    .zip(log_amplitude_spectrum.iter().cloned()),
                &RED,
            ))?
            .label("fft spectrogram (dB)")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], RED));

            cc.draw_series(LineSeries::new(
                x_mel_axis
                    .iter()
                    .cloned()
                    .zip(log_mel_amplitude_spectrum.iter().cloned()),
                &BLUE,
            ))?
            .label("mel spectrogram (dB)")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], BLUE));

            cc.draw_series(LineSeries::new(
                x_spectral_envelope_axis
                    .iter()
                    .cloned()
                    .zip(spectral_envelope.iter().cloned()),
                &GREEN,
            ))?
            .label("spectral envelope")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], GREEN));

            cc.configure_series_labels().border_style(BLACK).draw()?;

            root_area.present()?;
        }

        let img = image::RgbImage::from_raw(width, height, root_area_buf).unwrap();

        img.save(name)?;

        Ok(())
    }

    pub fn plot_3d(
        &self,
        name: &str,
        sample_rate: f64,
        (width, height): (u32, u32),
    ) -> anyhow::Result<()> {
        // let mut root_area_buf = vec![0; (width * height * 3) as usize];
        let mut root_area_buf = String::new();
        {
            let Self {
                log_amplitude_spectrum,
                log_mel_amplitude_spectrum,
                spectral_envelope,
            } = self;

            // 最初の100フレームだけを表示
            let log_amplitude_spectrum = log_amplitude_spectrum
                .iter()
                .take(50)
                .map(|x| x.to_owned())
                .collect::<Vec<_>>();
            let log_mel_amplitude_spectrum = log_mel_amplitude_spectrum
                .iter()
                .take(50)
                .map(|x| x.to_owned())
                .collect::<Vec<_>>();
            let spectral_envelope = spectral_envelope
                .iter()
                .take(50)
                .map(|x| x.to_owned())
                .collect::<Vec<_>>();

            let max_y = log_amplitude_spectrum[0]
                .iter()
                .cloned()
                .fold(0.0f64, |acc, x| acc.max(x.abs()));

            let x_range = -0.1..sample_rate / 2.0f64 + 0.1;
            let y_range = -0.1f64 - max_y..max_y + 0.1;
            let z_range = 0.0f64..(log_amplitude_spectrum.len() as f64);

            // let root_area =
            //     BitMapBackend::with_buffer(&mut root_area_buf, (width, height)).into_drawing_area();

            let root_area =
                SVGBackend::with_string(&mut root_area_buf, (width, height)).into_drawing_area();

            root_area.fill(&WHITE)?;

            let root_area = root_area.titled("Spectrogram", ("sans-serif", 60))?;

            let mut cc = ChartBuilder::on(&root_area)
                .margin(5)
                .set_all_label_area_size(50)
                .caption("Mel Spectrogram", ("sans-serif", 40))
                .build_cartesian_3d(x_range, y_range, z_range)
                .unwrap();

            cc.with_projection(|mut p| {
                p.pitch = std::f64::consts::PI / 12.0;
                p.yaw = std::f64::consts::PI / 6.0;
                p.scale = 1.2;
                p.into_matrix()
            });

            cc.configure_axes()
                .x_labels(20)
                .y_labels(10)
                .z_labels(10)
                .light_grid_style(BLACK.mix(0.15))
                .max_light_lines(3)
                .draw()?;

            // let time_max = log_amplitude_spectrum.len() as f64
            //     * log_amplitude_spectrum[0].len() as f64
            //     / sample_rate;

            let fft_line_graphs = self
                .log_amplitude_spectrum
                .iter()
                .map(|x| {
                    let points = x
                        .iter()
                        .enumerate()
                        .map(|(i, y)| line_graph::Point {
                            x: i as f64 * sample_rate / 2.0 / x.len() as f64,
                            y: *y,
                        })
                        .collect::<Vec<_>>();

                    line_graph::LineGraph::new(points)
                })
                .collect::<Vec<_>>();

            let log_mel_line_graphs = self
                .log_mel_amplitude_spectrum
                .iter()
                .map(|x| {
                    let points = x
                        .iter()
                        .enumerate()
                        .map(|(i, y)| line_graph::Point {
                            x: i as f64 * sample_rate / 2.0 / x.len() as f64,
                            y: *y,
                        })
                        .collect::<Vec<_>>();

                    line_graph::LineGraph::new(points)
                })
                .collect::<Vec<_>>();

            let spector_envelope_line_graphs = self
                .spectral_envelope
                .iter()
                .map(|x| {
                    let points = x
                        .iter()
                        .enumerate()
                        .map(|(i, y)| line_graph::Point {
                            x: i as f64 * sample_rate / 2.0 / x.len() as f64,
                            y: *y,
                        })
                        .collect::<Vec<_>>();

                    line_graph::LineGraph::new(points)
                })
                .collect::<Vec<_>>();

            cc.draw_series(
                SurfaceSeries::xoz(
                    // まずはx軸の値を生成
                    ArcArray::linspace(0.0, sample_rate / 2.0f64, log_amplitude_spectrum[0].len())
                        .iter()
                        .cloned(),
                    // 次にz軸の値を生成
                    (0..log_amplitude_spectrum.len()).map(|z| z as f64),
                    // 最後にy軸の値をx,zの組み合わせに対して生成
                    |x, z| {
                        let z = z as usize;

                        let y = fft_line_graphs[z].get_y(x).unwrap_or_default();

                        y
                    },
                )
                .style(BLUE.mix(0.1).filled()),
            )?
            .label("fft spectrogram (dB)")
            .legend(|(x, y)| {
                Rectangle::new([(x + 5, y - 5), (x + 15, y + 5)], BLUE.mix(0.5).filled())
            });

            // cc.draw_series(
            //     SurfaceSeries::xoz(
            //         // まずはx軸の値を生成
            //         ArcArray::linspace(
            //             0.0,
            //             sample_rate / 2.0f64,
            //             log_mel_amplitude_spectrum[0].len(),
            //         )
            //         .iter()
            //         .cloned(),
            //         // 次にz軸の値を生成
            //         (0..log_mel_amplitude_spectrum.len()).map(|z| z as f64),
            //         // 最後にy軸の値をx,zの組み合わせに対して生成
            //         |x, z| {
            //             let z = z as usize;

            //             let y = log_mel_line_graphs[z].get_y(x).unwrap_or_default();

            //             y
            //         },
            //     )
            //     .style(RED.mix(0.1).filled()),
            // )?
            // .label("mel spectrogram (dB)")
            // .legend(|(x, y)| {
            //     Rectangle::new([(x + 5, y - 5), (x + 15, y + 5)], RED.mix(0.5).filled())
            // });

            cc.draw_series(
                SurfaceSeries::xoz(
                    // まずはx軸の値を生成
                    ArcArray::linspace(0.0, sample_rate / 2.0f64, spectral_envelope[0].len())
                        .iter()
                        .cloned(),
                    // 次にz軸の値を生成
                    (0..spectral_envelope.len()).map(|z| z as f64),
                    // 最後にy軸の値をx,zの組み合わせに対して生成
                    |x, z| {
                        let z = z as usize;

                        let y = spector_envelope_line_graphs[z].get_y(x).unwrap_or_default();

                        y
                    },
                )
                .style(GREEN.mix(0.1).filled()),
            )?
            .label("spectral envelope")
            .legend(|(x, y)| {
                Rectangle::new([(x + 5, y - 5), (x + 15, y + 5)], GREEN.mix(0.5).filled())
            });

            root_area.present()?;
        }

        // let img = image::RgbImage::from_raw(width, height, root_area_buf).unwrap();

        if std::fs::exists(name).unwrap() {
            std::fs::remove_file(name).ok();
        }

        // img.save(name)?;

        std::fs::write(name, root_area_buf)?;

        Ok(())
    }
}
