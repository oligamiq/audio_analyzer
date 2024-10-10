// PSD layer

use std::{fmt::Debug, sync::Arc, thread};

use crossbeam_channel::{unbounded, Receiver, Sender};
use ndarray::{Array1, Array2};
use parking_lot::Mutex;

use crate::layer::Layer;

// https://gochikika.ntt.com/Visualization_and_EDA/spectral_visualization.html
// This use Welch's method.
// STFTの結果を二乗して周波数毎に平均を取る
#[derive(Debug)]
pub struct ToPowerSpectralDensityLayer {
    handles: Option<Vec<std::thread::JoinHandle<()>>>,
    result_sender: Arc<Mutex<Vec<Sender<Array1<f64>>>>>,
    input_receiver: Option<Receiver<Array2<f64>>>,
}

impl ToPowerSpectralDensityLayer {
    pub fn new() -> Self {
        Self {
            handles: Some(Vec::new()),
            result_sender: Arc::new(Mutex::new(Vec::new())),
            input_receiver: None,
        }
    }

    pub fn start(&mut self) {
        let result_sender = self.result_sender.clone();

        if let Some(input_receiver) = self.input_receiver.take() {
            let fft_handle = thread::spawn(move || {
                while let Ok(data) = input_receiver.recv() {
                    let mut psd = Array1::zeros(data.shape()[1]);

                    for i in 0..data.shape()[1] {
                        let mut sum = 0.0;
                        for j in 0..data.shape()[0] {
                            sum += data[[j, i]].powi(2);
                        }
                        psd[i] = sum / data.shape()[0] as f64;
                    }

                    result_sender.lock().retain(|x| x.send(psd.clone()).is_ok());
                }
            });

            self.handles.as_mut().unwrap().push(fft_handle);
        } else {
            panic!("Input stream not set");
        }
    }
}

impl Layer for ToPowerSpectralDensityLayer {
    type InputType = Array2<f64>;

    type OutputType = Array1<f64>;

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
