// PSD layer

use std::{collections::VecDeque, fmt::Debug, sync::Arc, thread};

use crossbeam_channel::{unbounded, Receiver, Sender};
use ndarray::{s, Array1, Array2, AssignElem as _, Axis, Slice};
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
                let mut holder: Array2<f64> = Array2::zeros((config.n_mels, 0));

                while let Ok(data) = input_receiver.recv() {
                    // debug!("Data: {:?}", data);

                    assert!(data.shape()[1] == 1);
                    assert!(data.shape()[0] == config.n_mels);

                    // dataから取り出す
                    // let data = data.t()

                    // holder.assign_elem(data);

                    assert!(holder.shape()[0] == config.n_mels);

                    holder = ndarray::concatenate(Axis(1), &[holder.view(), data.view()]).unwrap();

                    // holder.axis_iter_mut(Axis(1)).for_each(|mut x| {
                    //     debug!("$$ X 1: {:?}", x);

                    //     debug!("$$ X 2: {:?}", x);
                    // });

                    // debug!("$$ Holder: {:?}", holder.shape());

                    // holder = ndarray::arr2(&[holder.to_owned(), data.t().to_owned()])
                    //     .concatenate(Axis(0));

                    if holder.shape()[1] < config.time_range {
                        continue;
                    }

                    // remove first element if holder is too long

                    if holder.shape()[1] > config.time_range {
                        holder = holder.slice(s![.., 1..]).to_owned();
                    }

                    // debug!("Holder: {:?}", holder);

                    assert!(holder.shape()[1] == config.time_range);
                    assert!(holder.shape()[0] == config.n_mels);

                    // use holder to calculate PSD
                    let psd = holder
                        .axis_iter(Axis(0))
                        .enumerate()
                        .map(|(i, x)| {
                            let sum = x.mapv(|x| x.powi(2)).sum() / config.time_range as f64;
                            let freq = i as f64 * config.sample_rate / config.n_mels as f64;
                            (freq, sum)
                        })
                        .collect::<Array1<_>>();

                    debug!("PSD: {:?}", psd);

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
