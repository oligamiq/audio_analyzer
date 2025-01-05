use audio_analyzer_core::prelude::TestData;
pub mod libs;

const MNIST_BASE_PATH: &'static str = concat!(env!("CARGO_MANIFEST_DIR"), "/datasets/AudioMNIST/data");
const BAVED_BASE_PATH: &'static str = concat!(env!("CARGO_MANIFEST_DIR"), "/datasets/BAVED/remake");

fn main() {
    let analyzer = |data: &TestData, sample_rate| sample_rate;

    let data = libs::load_dataset::load_AudioMNIST(MNIST_BASE_PATH, analyzer).unwrap();

    println!("{:?}", data);

    let data = libs::load_dataset::load_BAVED(BAVED_BASE_PATH, analyzer).unwrap();

    println!("{:?}", data);
}
