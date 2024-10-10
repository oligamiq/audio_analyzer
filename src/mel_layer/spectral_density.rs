// PSD layer

use std::{collections::VecDeque, fmt::Debug, sync::Arc, thread};

use crossbeam_channel::{unbounded, Receiver, Sender};
use ndarray::{Array1, Array2, Axis};
use parking_lot::Mutex;
use tracing::debug;

use crate::layer::Layer;

#[derive(Debug, Clone)]
pub struct ToPowerSpectralDensityLayerConfig {
    pub sample_rate: f64,
    pub time_range: usize,
    pub n_mels: usize,
}

// https://gochikika.ntt.com/Visualization_and_EDA/spectral_visualization.html
// This use Welch's method.
// 今はピリオドグラムを使っているが、Welch's methodを使う
// STFTの結果を二乗して周波数毎に平均を取る
#[derive(Debug)]
pub struct ToPowerSpectralDensityLayer {
    config: ToPowerSpectralDensityLayerConfig,
    handles: Option<Vec<std::thread::JoinHandle<()>>>,
    result_sender: Arc<Mutex<Vec<Sender<Array1<(f64, f64)>>>>>,
    input_receiver: Option<Receiver<Array2<f64>>>,
}

impl ToPowerSpectralDensityLayer {
    pub fn new(config: ToPowerSpectralDensityLayerConfig) -> Self {
        Self {
            config,
            handles: Some(Vec::new()),
            result_sender: Arc::new(Mutex::new(Vec::new())),
            input_receiver: None,
        }
    }

    pub fn start(&mut self) {
        let result_sender = self.result_sender.clone();

        let config = self.config.clone();

        if let Some(input_receiver) = self.input_receiver.take() {
            let fft_handle = thread::spawn(move || {
                let mut holder: Array2<f64> = Array2::zeros((0, config.n_mels));

                while let Ok(data) = input_receiver.recv() {
                    debug!("Data: {:?}", data);

                    assert!(data.len() == 1);

                    holder = Array2::from_shape_vec(
                        (holder.len() + 1, config.n_mels),
                        data.outer_iter().map(|x| x[0]).collect(),
                    )
                    .unwrap();

                    if holder.len() < config.time_range {
                        continue;
                    }

                    for _ in 0..holder.len() - config.time_range {
                        holder.swap_axes(0, 1);
                        holder
                            .slice_axis_mut(Axis(0), (0..config.time_range).into())
                            .fill(0.0);
                        holder.swap_axes(0, 1);
                    }

                    debug!("Holder: {:?}", holder);

                    // use holder to calculate PSD
                    let mut psd = Array1::default(config.n_mels);
                    for i in 0..holder.len() {
                        let mut sum = 0.0;
                        for j in 0..holder.shape()[1] {
                            sum += holder[[i, j]].powi(2);
                        }
                        let sum = sum / holder.shape()[1] as f64;
                        let freq = i as f64 * config.sample_rate / holder.len() as f64;

                        debug!("Freq: {}, Sum: {}", freq, sum);

                        psd[i] = (freq, sum);
                    }

                    // let mut psd = Vec::new();
                    // for i in 0..holder[0].len() {
                    //     let mut sum = 0.0;
                    //     for j in 0..holder.len() {
                    //         sum += holder[j][i].powi(2);
                    //     }
                    //     let sum = sum / holder.len() as f64;
                    //     let freq = i as f64 * config.sample_rate / holder[0].len() as f64;

                    //     debug!("Freq: {}, Sum: {}", freq, sum);

                    //     psd.push((freq, sum));
                    // }

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

    type OutputType = Array1<(f64, f64)>;

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
