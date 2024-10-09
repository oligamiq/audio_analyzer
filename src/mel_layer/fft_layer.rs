// stft layer

use std::{fmt::Debug, thread};

use crossbeam_channel::{unbounded, Receiver, Sender};
use mel_spec::stft::Spectrogram;
use ndarray::Array1;
use num_complex::Complex;

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

pub struct ToSpectrogramLayer {
    mel_config: FftConfig,
    handles: Option<Vec<std::thread::JoinHandle<()>>>,
    result_sender: Option<Sender<Array1<Complex<f64>>>>,
    input_receiver: Option<Receiver<Vec<f32>>>,
    result_receiver: Receiver<Array1<Complex<f64>>>,
}

impl Debug for ToSpectrogramLayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ToSpectrogramLayer")
            .field("handles", &self.handles)
            .field("result_sender", &self.result_sender)
            .field("input_receiver", &self.input_receiver)
            .field("result_receiver", &self.result_receiver)
            .finish()
    }
}

impl Default for ToSpectrogramLayer {
    fn default() -> Self {
        let (sender, receiver) = unbounded();

        Self {
            mel_config: FftConfig::default(),
            handles: Some(Vec::new()),
            result_sender: Some(sender),
            input_receiver: None,
            result_receiver: receiver,
        }
    }
}

impl ToSpectrogramLayer {
    pub fn new(mel_config: FftConfig) -> Self {
        let (sender, receiver) = unbounded();

        Self {
            mel_config,
            handles: Some(Vec::new()),
            result_sender: Some(sender),
            input_receiver: None,
            result_receiver: receiver,
        }
    }

    pub fn start(&mut self) {
        let Self { mel_config, .. } = self;

        let fft_size = mel_config.fft_size;
        let hop_size = mel_config.hop_size;

        let result_sender = self.result_sender.take().expect("Result sender not set");

        if let Some(input_receiver) = self.input_receiver.take() {
            let fft_handle = thread::spawn(move || {
                let mut fft = Spectrogram::new(fft_size, hop_size);

                while let Ok(data) = input_receiver.recv() {
                    let fft_result = fft.add(&data);
                    if let Some(fft_result) = fft_result {
                        result_sender.send(fft_result).unwrap();
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
        self.result_receiver.clone()
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
