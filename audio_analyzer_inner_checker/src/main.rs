// $ENV:RUSTFLAGS="-C target-cpu=native"
// cargo run -p audio_analyzer_inner_checker -r

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
    let save_data: MapType<_> = match std::fs::read_to_string(save_data_path)
        .map(|v| json5::from_str::<Vec<(AudioMNISTKey, _)>>(&v))
    {
        Ok(Ok(save_data)) => {
            let data = vec_to_dash_map(save_data);
            println!("load save data: number of keys: {}", data.len());
            data
        }
        Err(e) => {
            println!("failed to load save data: {:?}", e);
            DashMap::with_capacity_and_hasher(60 * 10 * 50, gxhash::GxBuildHasher::default())
        }
        Ok(Err(e)) => {
            println!("failed to load save data: {:?}", e);
            DashMap::with_capacity_and_hasher(60 * 10 * 50, gxhash::GxBuildHasher::default())
        }
    };
    let boxed_save_data = Box::new(save_data);
    let leaked_save_data: &'static mut MapType<_> = Box::leak(boxed_save_data);
    let static_ref_save_data: &'static MapType<_> = leaked_save_data;

    ctrlc::set_handler(move || {
        let saveable_data = dash_map_to_vec(static_ref_save_data.clone());

        std::fs::write(
            save_data_path,
            json5::to_string(&saveable_data).unwrap(),
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

fn vec_to_dash_map<K: Eq + std::hash::Hash + Clone, V: Clone>(
    vec: Vec<(K, V)>,
) -> DashMap<K, V, gxhash::GxBuildHasher> {
    vec.into_iter()
        .collect::<DashMap<K, V, gxhash::GxBuildHasher>>()
}

fn dash_map_to_vec<K: Eq + std::hash::Hash + Clone, V: Clone>(
    dash_map: DashMap<K, V, gxhash::GxBuildHasher>,
) -> Vec<(K, V)> {
    dash_map
        .into_iter()
        .map(|(k, v)| (k, v))
        .collect::<Vec<(K, V)>>()
}
