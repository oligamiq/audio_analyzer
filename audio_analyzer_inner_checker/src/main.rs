// $ENV:RUSTFLAGS="-C target-cpu=native"
// cargo run -p audio_analyzer_inner_checker -r

use std::{io::Read, sync::atomic::AtomicBool};

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

    let save_data_path = concat!(env!("CARGO_MANIFEST_DIR"), "/audio_mnist_data.json.br");
    let save_data: MapType<_> = match std::fs::File::open(save_data_path).map(|f| {
        let reader = std::io::BufReader::new(f);
        let mut decompressor = brotli::Decompressor::new(reader, 4096);
        let mut str = String::new();
        let str = decompressor.read_to_string(&mut str).map(|_| str);
        str.map(|str| json5::from_str(&str))
    }) {
        Ok(Ok(Ok(save_data))) => {
            let data = vec_to_dash_map(save_data);
            println!("load save data: number of keys: {}", data.len());
            data
        }
        Ok(Ok(Err(e))) => {
            println!("failed to load save data: {:?}", e);
            DashMap::with_capacity_and_hasher(60 * 10 * 50, gxhash::GxBuildHasher::default())
        }
        Ok(Err(e)) | Err(e) => {
            println!("failed to load save data on decompress: {:?}", e);
            DashMap::with_capacity_and_hasher(60 * 10 * 50, gxhash::GxBuildHasher::default())
        }
    };
    let boxed_save_data = Box::new(save_data);
    let leaked_save_data: &'static MapType<_> = Box::leak(boxed_save_data);

    let stopper = AtomicBool::new(false);
    let boxed_stopper = Box::new(stopper);
    let leaked_stopper: &'static AtomicBool = Box::leak(boxed_stopper);

    let (blocker, waiter) = std::sync::mpsc::channel::<()>();

    ctrlc::set_handler(move || {
        leaked_stopper.store(true, std::sync::atomic::Ordering::SeqCst);
        println!("\n\rsaving...");
        let saveable_data = dash_map_to_vec(leaked_save_data.clone());
        let str = json5::to_string(&saveable_data).unwrap();
        let mut params = brotli::enc::BrotliEncoderParams::default();
        params.quality = 4;
        match brotli::BrotliCompress(
            &mut str.as_bytes(),
            &mut std::fs::File::create(save_data_path).unwrap(),
            &params,
        ) {
            Ok(_) => println!("save success"),
            Err(e) => println!("failed to save: {:?}", e),
        };
        blocker.send(()).unwrap();
        std::process::exit(0);
    })
    .unwrap();

    let data = load_AudioMNIST(
        MNIST_BASE_PATH,
        analyzer,
        true,
        leaked_save_data,
        &leaked_stopper,
    )
    .unwrap();
    if leaked_stopper.load(std::sync::atomic::Ordering::Relaxed) {
        waiter.recv().unwrap();
        return;
    }

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
