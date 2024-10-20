// stft layer

use std::{any::Any, fmt::Debug};

use color_eyre::eyre::eyre;
use mel_spec::stft::Spectrogram;
use ndarray::Array1;
use num_complex::Complex;

use crate::layer::Layer;
use crate::Result;

#[derive(Debug)]
pub struct FftConfig {
    pub fft_size: usize,
    pub hop_size: usize,
}

impl FftConfig {
    pub fn new(fft_size: usize, hop_size: usize) -> Self {
        Self { fft_size, hop_size }
    }
}

impl Default for FftConfig {
    fn default() -> Self {
        Self {
            fft_size: 400,
            hop_size: 160,
        }
    }
}

// To FFT Frame
pub struct ToSpectrogramLayer {
    hop_size: usize,
    kept_data: Vec<f32>,
    fft: Spectrogram,
}

impl Debug for ToSpectrogramLayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ToSpectrogramLayer")
            .field("hop_size", &self.hop_size)
            .field("kept_data", &self.kept_data)
            .finish()
    }
}

impl ToSpectrogramLayer {
    pub fn new(mel_config: FftConfig) -> Self {
        let FftConfig { fft_size, hop_size } = mel_config;

        let fft = Spectrogram::new(fft_size, hop_size);

        Self {
            fft,
            hop_size,
            kept_data: Vec::new(),
        }
    }

    pub fn through_inner<'a>(&mut self, input: &'a Vec<f32>) -> Result<Vec<Array1<Complex<f64>>>> {
        let Self {
            fft,
            hop_size,
            kept_data,
        } = self;

        let hop_size = *hop_size;

        kept_data.extend(input);

        if kept_data.len() < hop_size {
            return Ok(Vec::new());
        }

        let mut ret = Vec::new();
        while kept_data.len() >= hop_size {
            // hop_sizeと一緒のサイズに調整する
            let kept_data = kept_data.drain(..hop_size).collect::<Vec<_>>();

            let fft_result: Option<Array1<Complex<f64>>> = fft.add(&kept_data);
            if let Some(fft_result) = fft_result {
                ret.push(fft_result);
            }
        }

        Ok(ret)
    }
}

impl Layer for ToSpectrogramLayer {
    fn through<'a>(
        &mut self,
        input: &'a dyn std::any::Any,
    ) -> Result<Vec<Box<(dyn Any + 'static)>>> {
        let input = input
            .downcast_ref::<Vec<f32>>()
            .ok_or_else(|| eyre!("Invalid input type"))?;

        let ret = self.through_inner(input)?;

        Ok(ret
            .into_iter()
            .map(|x| Box::new(x) as Box<dyn Any>)
            .collect())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn input_type(&self) -> &'static str {
        "Vec<f32>"
    }

    fn output_type(&self) -> &'static str {
        "Array1<Complex<f64>>"
    }
}
