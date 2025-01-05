use audio_analyzer_core::prelude::TestData;
pub mod libs;

fn main() {
    let mnist_base_path = r"datasets/AudioMNIST/data";
    let analyzer = |data: &TestData, sample_rate| sample_rate;

    let data = libs::load_dataset::load_AudioMNIST(mnist_base_path, analyzer).unwrap();

    println!("{:?}", data);

    let baved_base_path = r"datasets/BAVED/remake";

    let data = libs::load_dataset::load_BAVED(baved_base_path, analyzer).unwrap();

    println!("{:?}", data);
}
