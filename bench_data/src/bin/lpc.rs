use audio_analyzer_core::prelude::*;
use bench_data::lifter;
use ndarray::*;
use num_complex::Complex;
use plotters::prelude::*;

// lifterをしてから呼び出す
// idct
fn calculate_log_power_spectrum_envelope(cepstrum: Array1<f64>, n: usize) -> Array1<f64> {
    let fft = rustfft::FftPlanner::new().plan_fft_inverse(n);

    let mut fft_input = cepstrum
        .iter()
        .map(|x| Complex::new(*x, 0.0))
        .collect::<Vec<_>>();

    fft.process(&mut fft_input);

    let spectral_envelope = fft_input.iter().map(|x| x.re).collect::<Array1<_>>();

    // データをサイズで割って正規化
    let spectral_envelope = spectral_envelope.mapv(|x| x / n as f64);

    spectral_envelope
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

    let mut n = 0;
    while let Some(data) = data.try_recv() {
        {
            n += 1;
            if n != count / 2 && n % 50 != 0 {
                continue;
            }
        }

        analyze_data(
            data.iter().map(|x| *x as f64).collect::<Array1<f64>>(),
            sample_rate as f64,
            n,
        );

        // println!("n: {:?}", n);
        // println!("data: {:?}", data);
    }
}

fn fft(data: Array1<f64>) -> Array1<f64> {
    let fft = rustfft::FftPlanner::new().plan_fft_forward(data.len());

    let mut fft_input = data
        .iter()
        .map(|x| Complex::new(*x, 0.0))
        .collect::<Vec<_>>();

    fft.process(&mut fft_input);

    let fft_output = fft_input.iter().map(|x| x.re).collect::<Array1<_>>();

    fft_output
}

fn plot_view_data(view_data: Vec<(Vec<(f64, f64)>, String)>, x_max: f64, salt: usize) {
    let file_name = format!("out/{salt}.svg");
    let root_area = SVGBackend::new(&file_name, (1024, 768)).into_drawing_area();
    root_area.fill(&WHITE).unwrap();

    let root_area = root_area.titled("FFT", ("sans-serif", 60)).unwrap();

    let mut chart = ChartBuilder::on(&root_area)
        .margin(5)
        .caption("FFT", ("sans-serif", 50).into_font())
        .x_label_area_size(40)
        .y_label_area_size(40)
        .build_cartesian_2d(0.0..x_max, -40.0..40.0)
        .unwrap();

    chart.configure_mesh().draw().unwrap();

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
        .border_style(&BLACK)
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
    let fft_ = fft(hanning.clone());
    let frequencies = (0..fft_.len())
        .map(|x| x as f64 * frame_rate / fft_.len() as f64)
        .collect::<Vec<_>>();
    let log_power = fft_.mapv(|x| x.abs().log10() * 20.0);

    view_data.push((
        frequencies
            .iter()
            .zip(log_power.iter())
            .map(|(x, y)| (*x, *y))
            .collect(),
        "log_power".to_string(),
    ));

    let mut cepstrum = fft(log_power.clone());
    // let liftered = lifter(cepstrum.clone(), 30);
    println!("cepstrum: {:?}", cepstrum.len());
    for i in 0..cepstrum.len() {
        if i > 30 {
            cepstrum[i] = 0.0;
        }
    }
    let spectral_envelope = calculate_log_power_spectrum_envelope(cepstrum, log_power.len());

    view_data.push((
        frequencies
            .iter()
            .zip(spectral_envelope.iter())
            .map(|(x, y)| (*x, *y))
            .collect(),
        "spectral_envelope".to_string(),
    ));

    // plot_view_data(view_data, frame_rate / 2.0, salt);
    plot_view_data(view_data, 4000.0, salt);
}
