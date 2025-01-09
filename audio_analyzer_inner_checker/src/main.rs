// $ENV:RUSTFLAGS="-C target-cpu=native"
// cargo run -p audio_analyzer_inner_checker -r

use std::{io::Read, sync::atomic::AtomicBool};

use dashmap::DashMap;

pub mod brotli_system;
pub mod deserialize;
pub mod fn_;
pub mod libs;
pub mod presets;

const MNIST_BASE_PATH: &'static str =
    concat!(env!("CARGO_MANIFEST_DIR"), "/datasets/AudioMNIST/data");
const BAVED_BASE_PATH: &'static str = concat!(env!("CARGO_MANIFEST_DIR"), "/datasets/BAVED/remake");

use deserialize::DashMapWrapper;
use libs::load_dataset::{load_AudioMNIST, load_BAVED};

fn main() {
    let analyzer = fn_::analyzer;

    type AudioMNISTKey = (usize, usize, usize);
    type MapType<V> = DashMap<AudioMNISTKey, V, gxhash::GxBuildHasher>;

    let save_data_path = concat!(env!("CARGO_MANIFEST_DIR"), "/audio_mnist_data.json.br");
    let save_data: MapType<_> = match std::fs::File::open(save_data_path).map(|f| {
        let now = std::time::Instant::now();
        let reader = std::io::BufReader::new(f);
        let mut decompressor = brotli::Decompressor::new(reader, 4096);
        let mut str = String::new();
        let str = decompressor.read_to_string(&mut str).map(|_| str);
        println!("decompress time: {:?}", now.elapsed());
        str.map(|str| {
                let now = std::time::Instant::now();
                let str = serde_json::from_str::<
                    DashMapWrapper<AudioMNISTKey, _, gxhash::GxBuildHasher>,
                > (&str);
                println!("parse time: {:?}", now.elapsed());
                str
            })
    }) {
        Ok(Ok(Ok(save_data))) => {
            println!("loaded save data");
            save_data.dash_map
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
    let leaked_save_data_mut: &'static mut MapType<_> = Box::leak(Box::new(save_data));
    let leaked_save_data: &'static MapType<_> = leaked_save_data_mut;

    let leaked_stopper_mut: &'static mut AtomicBool = Box::leak(Box::new(AtomicBool::new(false)));
    let leaked_stopper: &'static AtomicBool = leaked_stopper_mut;

    let (blocker, waiter) = std::sync::mpsc::channel::<()>();

    ctrlc::set_handler(move || {
        save_with_compress_file(
            leaked_save_data,
            save_data_path,
            Some(leaked_stopper),
            Some(&blocker),
        );
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

    ctrlc::set_handler(|| {}).unwrap();

    save_with_compress_file(leaked_save_data, save_data_path, None, None);

    fn release<T>(ptr: &T) {
        let ptr = ptr as *const T;
        std::mem::drop(unsafe { Box::<T>::from_raw(ptr as *mut T) });
    }
    release(leaked_save_data_mut);
    release(leaked_stopper_mut);
    std::mem::drop(waiter);

    println!("{:?}", data);

    let data = load_BAVED(BAVED_BASE_PATH, analyzer).unwrap();

    println!("{:?}", data);
}

fn dash_map_to_vec<
    K: Eq + std::hash::Hash + Clone,
    V: Clone,
    Hasher: Default + std::hash::BuildHasher + Clone,
>(
    dash_map: &DashMap<K, V, Hasher>,
) -> Vec<(K, V)> {
    dash_map
        .iter()
        .map(|multi| (multi.key().clone(), multi.value().clone()))
        .collect::<Vec<(K, V)>>()
}

fn save_with_compress_file<
    K: Eq + std::hash::Hash + Clone + serde::Serialize,
    V: Clone + serde::Serialize,
    Hasher: Default + std::hash::BuildHasher + Clone,
>(
    save_data: &DashMap<K, V, Hasher>,
    save_data_path: &str,
    stopper: Option<&AtomicBool>,
    blocker: Option<&std::sync::mpsc::Sender<()>>,
) {
    if let Some(stopper) = stopper {
        if stopper.load(std::sync::atomic::Ordering::Relaxed) {
            std::process::exit(0);
        }
        stopper.store(true, std::sync::atomic::Ordering::SeqCst);
    }
    println!("\n\rsaving...");
    let now = std::time::Instant::now();
    let saveable_data = dash_map_to_vec(save_data);
    let str = serde_json::to_string(&saveable_data).unwrap();
    let mut params = brotli::enc::BrotliEncoderParams::default();
    params.quality = 4;

    let bytes = str.as_bytes().to_owned();
    std::mem::drop(str);

    let compress_now = std::time::Instant::now();

    println!("compressing...");

    let out_data = match brotli_system::compress_multi_thread(&params, bytes) {
        Ok(out_data) => {
            println!("save success");
            out_data
        }
        Err(e) => {
            println!("failed to save: {:?}", e);
            return;
        }
    };

    println!("compress time: {:?}", compress_now.elapsed());

    std::fs::write(save_data_path, out_data).unwrap();
    println!("save time: {:?}", now.elapsed());
    blocker.map(|blocker| blocker.send(()));
}
