use crossbeam_channel::Receiver;

use super::RawDataLayer;

#[derive(Debug, Clone, Copy)]
pub enum TestDataType {
    TestData1,
}

impl TestDataType {
    pub fn voice_stream_receiver(&self) -> Receiver<Vec<f32>> {
        match self {
            TestDataType::TestData1 => {
                let (sender, receiver) = crossbeam_channel::unbounded();
                sender.send(vec![1.0, 2.0, 3.0]).unwrap();
                receiver
            }
        }
    }
}

impl RawDataLayer for TestDataType {
    fn voice_stream_receiver(&self) -> Receiver<Vec<f32>> {
        self.voice_stream_receiver()
    }
}
