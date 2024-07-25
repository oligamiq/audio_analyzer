use crossbeam_channel::{Receiver, Sender};
use mel_spec_pipeline::pipeline::{MelFrame, VadResult};

pub mod first_layer;

pub trait MelLayer {
    fn voice_stream_sender(&self) -> Sender<Vec<f32>>;
    fn mel_frame_stream_receiver(&self) -> Receiver<MelFrame>;
    fn vad_rx_stream_receiver(&self) -> Receiver<VadResult>;
    fn handle(&mut self) -> Vec<std::thread::JoinHandle<()>>;
    fn start(&mut self);
    fn set_sampling_rate(&mut self, sampling_rate: f64);
}
