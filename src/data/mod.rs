use test_data::TestDataType;

pub mod test_data;

#[derive(Debug, Clone, Copy)]
pub enum RawDataType {
    Test(TestDataType),
}
