use std::fmt::Debug;

use mel_spec::{config::MelConfig, mel::MelSpectrogram};
use ndarray::Array1;
use num_complex::Complex;

use anyhow::Result;

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

    pub fn through_inner<'a>(
        &mut self,
        data: &'a Array1<Complex<f64>>,
    ) -> Result<Option<Array1<f64>>> {
        let Self { mel } = self;

        let mel_spec: Array1<f64> = mel.add(data);

        Ok(Some(mel_spec))
    }
}
