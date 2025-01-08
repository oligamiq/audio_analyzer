// $ENV:RUSTFLAGS="-C target-cpu=native"
// cargo run -p audio_analyzer_inner_checker -r

use std::collections::HashSet;

use dashmap::DashMap;

pub mod fn_;
pub mod libs;
pub mod presets;

const MNIST_BASE_PATH: &'static str =
    concat!(env!("CARGO_MANIFEST_DIR"), "/datasets/AudioMNIST/data");
const BAVED_BASE_PATH: &'static str = concat!(env!("CARGO_MANIFEST_DIR"), "/datasets/BAVED/remake");

use libs::load_dataset::{load_AudioMNIST, load_BAVED};

fn main() {
    let analyzer = fn_::analyzer;

    type AudioMNISTKey = (usize, usize, usize);
    type MapType<V> = DashMap<AudioMNISTKey, V, gxhash::GxBuildHasher>;

    let save_data_path = concat!(env!("CARGO_MANIFEST_DIR"), "/audio_mnist_data.json");
    let save_data: MapType<_> = if let Ok(Ok(save_data)) = std::fs::read_to_string(save_data_path)
        .map(|v| serde_json::from_str::<HashSet<(AudioMNISTKey, _), gxhash::GxBuildHasher>>(&v))
    {
        let data = hash_set_to_dash_map(save_data);
        println!("load save data: number of keys: {}", data.len());
        data
    } else {
        let hasher = gxhash::GxBuildHasher::default();
        let save_data = DashMap::with_capacity_and_hasher(60 * 10 * 50, hasher);
        save_data
    };
    let boxed_save_data = Box::new(save_data);
    let leaked_save_data: &'static mut MapType<_> = Box::leak(boxed_save_data);
    let static_ref_save_data: &'static MapType<_> = leaked_save_data;

    ctrlc::set_handler(move || {
        let saveable_data = dash_map_to_hash_set(static_ref_save_data.clone());

        std::fs::write(
            save_data_path,
            serde_json::to_string(&saveable_data).unwrap(),
        )
        .unwrap();
        std::process::exit(0);
    })
    .unwrap();

    let data = load_AudioMNIST(MNIST_BASE_PATH, analyzer, true, leaked_save_data).unwrap();

    println!("{:?}", data);

    let data = load_BAVED(BAVED_BASE_PATH, analyzer).unwrap();

    println!("{:?}", data);
}

fn hash_set_to_dash_map<K: std::hash::Hash + std::cmp::Eq, V: std::hash::Hash + std::cmp::Eq>(
    hash_set: HashSet<(K, V), gxhash::GxBuildHasher>,
) -> DashMap<K, V, gxhash::GxBuildHasher> {
    hash_set
        .into_iter()
        .collect::<DashMap<K, V, gxhash::GxBuildHasher>>()
}

fn dash_map_to_hash_set<K: std::hash::Hash + std::cmp::Eq, V: std::hash::Hash + std::cmp::Eq>(
    dash_map: DashMap<K, V, gxhash::GxBuildHasher>,
) -> HashSet<(K, V), gxhash::GxBuildHasher> {
    dash_map
        .into_iter()
        .map(|(k, v)| (k, v))
        .collect::<HashSet<(K, V), gxhash::GxBuildHasher>>()
}
