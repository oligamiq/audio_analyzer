use audio_analyzer_core::prelude::*;
use ndarray::*;
use num_complex::Complex;

// lifterをしてから呼び出す
fn calculate_log_power_spectrum_envelope(cepstrum: Array1<f64>, n: usize) -> Array1<f64> {
    let fft = rustfft::FftPlanner::new().plan_fft_forward(n);

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

    let mut n = 0;
    let count = {
        let mut data = data.clone();
        data.start();
        let mut count = 0;
        while let Some(_) = data.try_recv() {
            count += 1;
        }

        count
    };
    while let Some(data) = data.try_recv() {
        {
            n += 1;
            if n != count / 2 {
                continue;
            }
        }

        println!("n: {:?}", n);
        println!("data: {:?}", data);
    }
}
