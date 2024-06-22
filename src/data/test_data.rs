use fundsp::prelude::*;

#[derive(Debug, Clone, Copy)]
pub enum TestDataType {
    TestData1,
}

impl TestDataType {
    pub fn get_data(&self) -> impl AudioUnit {
        match self {
            TestDataType::TestData1 => sine_hz(440.0),
        }
    }
}
