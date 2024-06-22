use fundsp::prelude::*;
use test_data::TestDataType;

pub mod test_data;

#[derive(Debug, Clone, Copy)]
pub enum RawDataType {
    Test(TestDataType),
}

impl RawDataType {
    pub fn get_data(&self) -> impl AudioUnit {
        match self {
            RawDataType::Test(a) => a.get_data(),
        }
    }
}
