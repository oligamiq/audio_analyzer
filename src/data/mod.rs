use crossbeam_channel::Receiver;

pub mod device_stream;
pub mod test_data;

pub trait RawDataStreamLayer {
    fn voice_stream_receiver(&self) -> Receiver<Vec<f32>>;
    fn sample_rate(&self) -> u32;
    fn start(&mut self);
    fn handle(&mut self) -> Vec<std::thread::JoinHandle<()>>;
}
