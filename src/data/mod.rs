pub mod device_stream;
pub mod test_data;

pub trait RawDataStreamLayer {
    fn try_recv(&mut self) -> Option<Vec<f32>>;
    fn sample_rate(&self) -> u32;
    fn start(&mut self);
}
