use audio_analyzer_core::prelude::*;
use mel_spec::mel::{hz_to_mel, mel_to_hz};
use ndarray::prelude::*;
use num_complex::Complex;
use plotters::prelude::*;
use rustfft::FftDirection;

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

    while let Some(data) = data.try_recv() {
        // println!("{:?}", data.len());

        if let Ok(fft) = fft_layer.through_inner(&data) {
            for fft in fft {
                println!("{:?}", fft.len());

                let mel = mel_layer.through_inner(&fft).unwrap().unwrap();

                let fft = fft.slice(s![..fft.len() / 2]);
                let log_amplitude_spectrum = log_amplitude_spectrum(&fft);

                let log_mel_amplitude_spectrum = log_mel_amplitude_spectrum(&mel.view());

                let quefrency = quefrency(log_amplitude_spectrum.clone());

                let liftered = lifter(quefrency, 1000);

                println!("{:?}", liftered);

                let spectral_envelope = spectral_envelope(liftered);

                println!("{:?}", spectral_envelope);

                n += 1;
                plot(
                    log_amplitude_spectrum,
                    log_mel_amplitude_spectrum,
                    spectral_envelope,
                    &format!("out/mel_{n}.png"),
                    sample_rate as f64,
                    (800, 600),
                )
                .unwrap();
            }
        }

        // println!("{:?}", data);
    }
}

fn log_amplitude_spectrum(fft: &ArrayView1<Complex<f64>>) -> Array1<f64> {
    fft.mapv(|x| 10.0 * x.norm_sqr().log10())
}

fn quefrency(log_amplitude_spectrum: Array1<f64>) -> Array1<f64> {
    let ifft =
        rustfft::FftPlanner::new().plan_fft(log_amplitude_spectrum.len(), FftDirection::Inverse);

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

fn spectral_envelope(quefrency: Array1<f64>) -> Array1<f64> {
    let fft = rustfft::FftPlanner::new().plan_fft(quefrency.len(), FftDirection::Forward);

    let mut fft_input = quefrency
        .iter()
        .map(|x| Complex::new(*x, 0.0))
        .collect::<Vec<_>>();

    fft.process(&mut fft_input);

    let spectral_envelope = fft_input.iter().map(|x| x.re).collect::<Array1<_>>();

    spectral_envelope
}

fn log_mel_amplitude_spectrum(
    mel: &ArrayBase<ndarray::ViewRepr<&f64>, Dim<[usize; 1]>>,
) -> Array1<f64> {
    mel.mapv(|x| 10.0 * x.log10())
}

fn plot(
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
