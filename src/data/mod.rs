use crossbeam_channel::Receiver;
use test_data::TestDataType;

pub mod test_data;

pub trait RawDataLayer {
    fn voice_stream_receiver(&self) -> Receiver<Vec<f32>>;
}
