// stft layer

use std::{fmt::Debug, sync::Arc, thread};

use crossbeam_channel::{unbounded, Receiver, Sender};
use mel_spec::stft::Spectrogram;
use ndarray::Array1;
use num_complex::Complex;
use parking_lot::Mutex;

use crate::layer::Layer;

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
    mel_config: FftConfig,
    handles: Option<Vec<std::thread::JoinHandle<()>>>,
    result_sender: Arc<Mutex<Vec<Sender<Array1<Complex<f64>>>>>>,
    input_receiver: Option<Receiver<Vec<f32>>>,
}

impl Debug for ToSpectrogramLayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ToSpectrogramLayer")
            .field("handles", &self.handles)
            .field("result_sender", &self.result_sender)
            .field("input_receiver", &self.input_receiver)
            .finish()
    }
}

impl Default for ToSpectrogramLayer {
    fn default() -> Self {
        Self {
            mel_config: FftConfig::default(),
            handles: Some(Vec::new()),
            result_sender: Arc::new(Mutex::new(Vec::new())),
            input_receiver: None,
        }
    }
}

impl ToSpectrogramLayer {
    pub fn new(mel_config: FftConfig) -> Self {
        Self {
            mel_config,
            handles: Some(Vec::new()),
            result_sender: Arc::new(Mutex::new(Vec::new())),
            input_receiver: None,
        }
    }

    pub fn start(&mut self) {
        let Self { mel_config, .. } = self;

        let fft_size = mel_config.fft_size;
        let hop_size = mel_config.hop_size;

        let result_sender = self.result_sender.clone();

        if let Some(input_receiver) = self.input_receiver.take() {
            let fft_handle = thread::spawn(move || {
                let mut fft = Spectrogram::new(fft_size, hop_size);

                let mut kept_data = Vec::new();
                while let Ok(data) = input_receiver.recv() {
                    kept_data.extend(data);

                    if kept_data.len() < hop_size {
                        continue;
                    }

                    while kept_data.len() >= hop_size {
                        // hop_sizeと一緒のサイズに調整する
                        let kept_data = kept_data.drain(..hop_size).collect::<Vec<_>>();

                        let fft_result = fft.add(&kept_data);
                        if let Some(fft_result) = fft_result {
                            result_sender
                                .lock()
                                .retain(|x| x.send(fft_result.clone()).is_ok());
                        }
                    }
                }
            });

            self.handles.as_mut().unwrap().push(fft_handle);
        } else {
            panic!("Input stream not set");
        }
    }
}

impl Layer for ToSpectrogramLayer {
    type InputType = Vec<f32>;

    type OutputType = Array1<Complex<f64>>;

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
