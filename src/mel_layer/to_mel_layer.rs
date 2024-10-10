use std::{fmt::Debug, sync::Arc, thread};

use crossbeam_channel::{unbounded, Receiver, Sender};
use mel_spec::{config::MelConfig, mel::MelSpectrogram};
use ndarray::{Array1, Array2};
use num_complex::Complex;
use parking_lot::Mutex;

use crate::layer::Layer;

pub struct ToMelSpectrogramLayer {
    mel_settings: MelConfig,
    handles: Option<Vec<std::thread::JoinHandle<()>>>,
    result_sender: Arc<Mutex<Vec<Sender<Array2<f64>>>>>,
    input_receiver: Option<Receiver<Array1<Complex<f64>>>>,
}

impl Debug for ToMelSpectrogramLayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ToMelSpectrogramLayer")
            .field("handles", &self.handles)
            .field("result_sender", &self.result_sender)
            .field("input_receiver", &self.input_receiver)
            .finish()
    }
}

impl Default for ToMelSpectrogramLayer {
    fn default() -> Self {
        Self {
            mel_settings: MelConfig::new(400, 160, 80, 16000.0),
            handles: Some(Vec::new()),
            result_sender: Arc::new(Mutex::new(Vec::new())),
            input_receiver: None,
        }
    }
}

impl ToMelSpectrogramLayer {
    pub fn new(mel_config: MelConfig) -> Self {
        Self {
            mel_settings: mel_config,
            handles: Some(Vec::new()),
            result_sender: Arc::new(Mutex::new(Vec::new())),
            input_receiver: None,
        }
    }

    pub fn start(&mut self) {
        let Self { mel_settings, .. } = self;

        let fft_size = mel_settings.fft_size();
        let n_mels = mel_settings.n_mels();
        let sampling_rate = mel_settings.sampling_rate();

        let result_sender = self.result_sender.clone();

        if let Some(input_receiver) = self.input_receiver.take() {
            let mel_handle = thread::spawn(move || {
                let mut mel = MelSpectrogram::new(fft_size, sampling_rate, n_mels);

                while let Ok(fft_result) = input_receiver.recv() {
                    let mel_spec = mel.add(&fft_result);
                    result_sender
                        .lock()
                        .retain(|s| s.send(mel_spec.clone()).is_ok());
                }
            });

            self.handles.as_mut().unwrap().push(mel_handle);
        } else {
            panic!("Input stream not set");
        }
    }
}

impl Layer for ToMelSpectrogramLayer {
    type InputType = Array1<Complex<f64>>;

    type OutputType = Array2<f64>;

    fn get_result_stream(&self) -> Receiver<Self::OutputType> {
        let (sender, receiver) = unbounded();
        self.result_sender.lock().push(sender);
        receiver
    }

    fn set_input_stream(&mut self, input_stream: Receiver<Self::InputType>) {
        self.input_receiver = Some(input_stream);
    }

    fn handle(&mut self) -> Vec<std::thread::JoinHandle<()>> {
        self.handles.take().expect("Handles already taken")
    }

    fn start(&mut self) {
        self.start();
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
