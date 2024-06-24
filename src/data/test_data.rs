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
                receiver
            }
        }
    }

    pub fn start(&mut self) {
        match self {
            TestDataType::TestData1 => {
                // Start the test data
            }
        }
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
