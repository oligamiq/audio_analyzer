use std::thread;

use crossbeam_channel::Receiver;

use super::RawDataLayer;

#[derive(Debug, Clone, Copy)]
pub enum TestDataType {
    TestData1,
}

pub struct TestData {
    pub test_data_type: TestDataType,
    sample_rate: Option<u32>,
    voice_stream_receiver: Option<Receiver<Vec<f32>>>,
    handles: Option<Vec<thread::JoinHandle<()>>>,
}

impl TestData {
    pub fn new(test_data_type: TestDataType) -> Self {
        TestData {
            test_data_type,
            sample_rate: None,
            voice_stream_receiver: None,
            handles: None,
        }
    }

    pub fn voice_stream_receiver(&self) -> Receiver<Vec<f32>> {

    }

    pub fn start(&mut self) {
        let file_path = match self.test_data_type {
            TestDataType::TestData1 => "test_data/jfk_f32le.wav",
        };
        let (sender, receiver) = crossbeam_channel::unbounded();
    }

    pub fn sample_rate(&self) -> u32 {
        match self {
            TestDataType::TestData1 => 48000,
        }
    }
}

impl RawDataLayer for TestDataType {
    fn voice_stream_receiver(&self) -> Receiver<Vec<f32>> {
        self.voice_stream_receiver()
    }
    fn start(&mut self) {
        self.start();
    }
    fn sample_rate(&self) -> u32 {
        self.sample_rate()
    }

}
