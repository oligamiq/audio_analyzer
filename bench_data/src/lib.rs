pub mod line_graph;

use audio_analyzer_core::{
    data::RawDataStreamLayer as _,
    prelude::{FftConfig, TestData, ToSpectrogramLayer},
};
use ndarray::prelude::*;
use num_complex::Complex;

pub fn get_lifters(mut data: TestData, analyze_second: f64) -> Vec<Array1<f64>> {
    let sample_rate = data.sample_rate();

    let fft_size = (analyze_second * sample_rate as f64) as usize;
    let hop_size = fft_size / 5;
    let fft_size = hop_size * 5;

    let mut fft_layer = ToSpectrogramLayer::new(FftConfig { fft_size, hop_size });

    let mut liftereds = vec![];
    while let Some(data) = data.try_recv() {
        if let Ok(fft) = fft_layer.through_inner(&data) {
            for fft in fft {
                let log_amplitude_spectrum = log_amplitude_spectrum(&fft.view());

                let quefrency = quefrency(log_amplitude_spectrum.clone());

                let liftered = lifter(quefrency, 15);

                liftereds.push(liftered.clone());
            }
        }
    }

    liftereds
}

pub fn log_amplitude_spectrum(fft: &ArrayView1<Complex<f64>>) -> Array1<f64> {
    fft.mapv(|x| 10.0 * x.norm_sqr().log10())
}

pub fn quefrency(log_amplitude_spectrum: Array1<f64>) -> Array1<f64> {
    let ifft = rustfft::FftPlanner::new().plan_fft_inverse(log_amplitude_spectrum.len());

    let mut ifft_input = log_amplitude_spectrum
        .iter()
        .map(|x| Complex::new(*x, 0.0))
        .collect::<Vec<_>>();

    ifft.process(&mut ifft_input);

    let quefrency = ifft_input.iter().map(|x| x.re).collect::<Array1<_>>();

    quefrency
}

pub fn lifter(quefrency: Array1<f64>, index: usize) -> Array1<f64> {
    let mut quefrency = quefrency.clone();

    for i in 0..quefrency.len() {
        if i > index && i < quefrency.len() - index {
            quefrency[i] = 0.0;
        }
    }

    quefrency
}
