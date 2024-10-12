use std::{any::Any, fmt::Debug};

use mel_spec::{config::MelConfig, mel::MelSpectrogram};
use ndarray::{Array1, Array2};
use num_complex::Complex;

use crate::layer::Layer;
use crate::Result;

// 今までのFFTの結果を受け取り、新たなメルスペクトログラムを生成する
pub struct ToMelSpectrogramLayer {
    mel: MelSpectrogram,
}

impl Debug for ToMelSpectrogramLayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ToMelSpectrogramLayer").finish()
    }
}

impl ToMelSpectrogramLayer {
    pub fn new(mel_config: MelConfig) -> Self {
        let mel = MelSpectrogram::new(
            mel_config.fft_size(),
            mel_config.sampling_rate(),
            mel_config.n_mels(),
        );

        Self { mel }
    }
}

impl Layer for ToMelSpectrogramLayer {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn through<'a>(&mut self, input: &'a dyn Any) -> Result<Vec<Box<(dyn Any + 'static)>>> {
        let Self { mel } = self;

        let data = input.downcast_ref::<Array1<Complex<f64>>>().unwrap();

        let mel_spec: Array2<f64> = mel.add(data);

        Ok(vec![Box::new(mel_spec)])
    }

    fn input_type(&self) -> &'static str {
        "Array1<Complex<f64>>"
    }

    fn output_type(&self) -> &'static str {
        "Array2<f64>"
    }
}
