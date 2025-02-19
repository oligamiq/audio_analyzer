use audio_analyzer_core::prelude::*;
use bench_data::lifter;
use linear_predictive_coding::{calc_lpc_by_burg, calc_lpc_by_levinson_durbin};
use ndarray::*;
use num_complex::Complex;
use plotters::prelude::*;

// lifterをしてから呼び出す
// idct
fn calculate_log_power_spectrum_envelope_fft_re(cepstrum: Array1<f64>, n: usize) -> Array1<f64> {
    let spectral_envelope = fft_re(cepstrum);

    // データをサイズで割って正規化
    let spectral_envelope = spectral_envelope.mapv(|x| x / n as f64);

    spectral_envelope
}

fn calculate_log_power_spectrum_envelope_fft(cepstrum: Array1<Complex<f64>>, n: usize) -> Array1<f64> {
    let cepstrum = cepstrum.mapv(|x| x / n as f64);

    let spectral_envelope = fft(cepstrum);

    // データをサイズで割って正規化

    spectral_envelope.into_iter().map(|x| 0. - x.norm()).collect::<Array1<_>>()
}

/// LPC係数から対数パワースペクトルを求める
pub fn calculate_log_power_spectrum_envelope_lpc(
    lpc: ArrayView1<f64>,
    e: Option<f64>,
    n: usize,
    fft_size: usize,
) -> Vec<f64> {
    // インパルス応答（LPCフィルタ）
    let mut impulse_response = vec![0.0; fft_size];
    impulse_response[0] = 1.0; // インパルス
    for i in 0..n {
        impulse_response[i + 1] = -lpc[i];
    }

    let mut spectrum: Vec<Complex<f64>> = impulse_response
        .iter()
        .map(|&x| Complex::new(x, 0.0))
        .collect();

    let mut fft_planner = rustfft::FftPlanner::new();
    let fft = fft_planner.plan_fft_forward(fft_size);
    fft.process(&mut spectrum);

    // let spectrum = spectrum.iter().map(|&x| x / (fft_size as f64).sqrt()).collect::<Vec<_>>();

    // 振幅スペクトル
    let amp: Vec<f64> = spectrum
        .iter()
        .map(|c| c.norm())
        .collect();
    // let mut amp: Vec<f64> = spectrum.iter().map(|c| c.re).collect();

    // 片側スペクトル（ナイキスト周波数まで）
    // amp.truncate(fft_size / 2 + 1);

    let e = e.map(|x| x.sqrt().log10()).unwrap_or(0.0);

    // dBスケール変換
    let env = amp
        .iter()
        .map(|&x| (e - x.log10()) * 20.)
        .collect::<Vec<f64>>();

    env
}

fn main() {
    let now_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let mut data = TestData::new_with_path(format!("{now_dir}/../test_data/jfk_f32le.wav"));
    println!("{:?}", now_dir);
    data.start();

    // 50ms
    let analyze_second = 0.05;
    // let analyze_second = 5.;

    let sample_rate = data.sample_rate();

    let fft_size = (analyze_second * sample_rate as f64) as usize;
    let hop_size = fft_size / 5;
    let fft_size = hop_size * 5;

    println!("fft_size: {:?}", fft_size);
    println!("hop_size: {:?}", hop_size);

    if std::fs::exists("out").unwrap() {
        std::fs::remove_dir_all("out").ok();

        std::fs::create_dir("out").unwrap();
    } else {
        std::fs::create_dir("out").unwrap();
    }

    let count = {
        let mut data = data.clone();
        data.start();
        let mut count = 0;
        while let Some(_) = data.try_recv() {
            count += 1;
        }
        count
    };

    let mut min__ = None;
    let mut max__ = 0.;

    let mut n = 0;
    while let Some(data) = data.try_recv() {
        {
            n += 1;
            if n != count / 2 && n % 50 != 0 {
                continue;
            }
        }

        // -1.1 ~ 1.1 to -1 ~ 1
        let data = data.iter().map(|x| *x as f64 / 2.).collect::<Vec<_>>();

        let max = data.iter().fold(0.0, |acc, x| if acc < *x { *x } else { acc });
        let min = data.iter().fold(0.0, |acc, x| if acc > *x { *x } else { acc });
        // if max > 1.0 {
        //     println!("max: {:?}", max);
        //     panic!();
        // }
        if max__ < max {
            max__ = max;
        }
        if min__ == None {
            min__ = Some(min);
        } else {
            if min__ > Some(min) {
                min__ = Some(min);
            }
        }
        println!("max: {:?}", max);

        analyze_data(
            data.iter().map(|x| *x as f64).collect::<Array1<f64>>(),
            sample_rate as f64,
            n,
        );

        // println!("n: {:?}", n);
        // println!("data: {:?}", data);
    }

    println!("max__: {:?}", max__);
    println!("min__: {:?}", min__);
}

fn ifft(data: Array1<f64>) -> Array1<Complex<f64>> {
    let fft = rustfft::FftPlanner::new().plan_fft_inverse(data.len());

    let mut fft_input = data
        .iter()
        .map(|x| Complex::new(*x, 0.0))
        .collect::<Vec<_>>();

    fft.process(&mut fft_input);

    fft_input.into_iter().collect::<Array1<_>>()
}

fn fft(data: Array1<Complex<f64>>) -> Array1<Complex<f64>> {
    let fft = rustfft::FftPlanner::new().plan_fft_forward(data.len());

    let mut fft_input = data.into_iter().collect::<Vec<_>>();

    fft.process(&mut fft_input);

    fft_input.into_iter().collect::<Array1<_>>()
}

fn fft_re(data: Array1<f64>) -> Array1<f64> {
    let fft = rustfft::FftPlanner::new().plan_fft_forward(data.len());

    let mut fft_input = data
        .iter()
        .map(|x| Complex::new(*x, 0.0))
        .collect::<Vec<_>>();

    fft.process(&mut fft_input);

    let fft_output = fft_input.iter().map(|x| x.re).collect::<Array1<_>>();

    fft_output
}

fn fft_norm(data: Array1<f64>) -> Array1<f64> {
    let fft = rustfft::FftPlanner::new().plan_fft_forward(data.len());

    let mut fft_input = data
        .iter()
        .map(|x| Complex::new(*x, 0.0))
        .collect::<Vec<_>>();

    fft.process(&mut fft_input);

    let fft_output = fft_input.iter().map(|x| x.norm()).collect::<Array1<_>>();

    fft_output
}

fn plot_view_data(view_data: Vec<(Vec<(f64, f64)>, String)>, x_max: f64, salt: usize) {
    let file_name = format!("out/{salt}.svg");
    let root_area = SVGBackend::new(&file_name, (1024, 768)).into_drawing_area();
    root_area.fill(&WHITE).unwrap();

    // let root_area = root_area.titled("FFT", ("sans-serif", 60)).unwrap();

    let mut chart = ChartBuilder::on(&root_area)
        .margin(20)
        // .caption("FFT", ("sans-serif", 50).into_font())
        .x_label_area_size(90)
        .y_label_area_size(110)
        .build_cartesian_2d(0.0..x_max, -110.0..30.0)
        .unwrap();

    chart
        .configure_mesh()
        .x_desc("Frequency [Hz]")
        .y_desc("Power [dB]")
        .x_label_style(("sans-serif", 30))
        .y_label_style(("sans-serif", 30))
        .draw()
        .unwrap();

    for (i, (data, title)) in view_data.iter().enumerate() {
        chart
            .draw_series(LineSeries::new(
                data.iter().map(|(x, y)| (*x, *y)),
                &Palette99::pick(i),
            ))
            .unwrap()
            .label(title)
            .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &Palette99::pick(i)));
    }

    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .position(SeriesLabelPosition::UpperRight)
        .border_style(&BLACK)
        .label_font(("sans-serif", 30))
        .draw()
        .unwrap();

    if let Err(e) = root_area.present() {
        eprintln!("error: {}", e);
    }
}

fn analyze_data(data: Array1<f64>, frame_rate: f64, salt: usize) {
    let mut view_data: Vec<(Vec<(f64, f64)>, String)> = Vec::new();

    let hanning = Array1::from_shape_fn(data.len(), |i| {
        0.5 - 0.5 * (2.0 * std::f64::consts::PI * i as f64 / data.len() as f64).cos()
    });
    let hanning = hanning * data;
    let fft_ = fft_norm(hanning.clone());
    let frequencies = (0..fft_.len())
        .map(|x| x as f64 * frame_rate / fft_.len() as f64)
        .collect::<Vec<_>>();
    let log_power = fft_.mapv(|x| x.log10() * 20.0);

    view_data.push((
        frequencies
            .iter()
            .zip(log_power.iter())
            .map(|(x, y)| (*x, *y))
            .collect(),
        "(1) log_power".to_string(),
    ));

    // let mut cepstrum = fft_re(log_power.clone());
    let mut cepstrum = ifft(log_power.clone());
    // let liftered = lifter(cepstrum.clone(), 30);
    println!("cepstrum: {:?}", cepstrum.len());
    for i in 0..cepstrum.len() {
        if i > 30 {
            // cepstrum[i] = 0.0;
            cepstrum[i] = Complex::new(0.0, 0.0);
        }
    }
    let spectral_envelope = calculate_log_power_spectrum_envelope_fft(cepstrum, log_power.len());

    view_data.push((
        frequencies
            .iter()
            .zip(spectral_envelope.iter())
            .map(|(x, y)| (*x, *y))
            .collect(),
        "(2) spectral_envelope by cepstrum".to_string(),
    ));

    // let depth = hanning.len() - 1;
    let depth = 32;

    // let lpc = calc_lpc_by_levinson_durbin(hanning.view(), log_power.len() - 1).unwrap();
    let (lpc, e) = calc_lpc_by_levinson_durbin(hanning.view(), depth).unwrap();

    let levinson_durbin_log_power =
        calculate_log_power_spectrum_envelope_lpc(lpc.view(), Some(e), depth, fft_.len());

    view_data.push((
        frequencies
            .iter()
            .zip(levinson_durbin_log_power.iter())
            .map(|(x, y)| (*x, *y))
            .collect(),
        "(3) spectral_envelope by levinson-durbin".to_string(),
    ));

    let lpc = calc_lpc_by_burg(hanning.view(), depth).unwrap();

    let burg_log_power =
        calculate_log_power_spectrum_envelope_lpc(lpc.view(), None, depth, fft_.len());

    view_data.push((
        frequencies
            .iter()
            .zip(burg_log_power.iter())
            .map(|(x, y)| (*x, *y))
            .collect(),
        "(4) spectral_envelope by burg".to_string(),
    ));

    // plot_view_data(view_data, frame_rate, salt);
    plot_view_data(view_data, frame_rate / 2.0, salt);
}
