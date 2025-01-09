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

use deserialize::{DashMapWrapper, DashMapWrapperRef};
use libs::load_dataset::{load_AudioMNIST, load_BAVED};

fn main() {
    let analyzer = fn_::analyzer;

    type AudioMNISTKey = (usize, usize, usize);
    type MapType<V> = DashMap<AudioMNISTKey, V, gxhash::GxBuildHasher>;

    let save_data_path = concat!(env!("CARGO_MANIFEST_DIR"), "/audio_mnist_data.bincode");
    let save_data: MapType<_> = match std::fs::File::open(save_data_path).map(|f| {
        let now = std::time::Instant::now();
        let mut reader = std::io::BufReader::new(f);
        // let mut reader = snap::read::FrameDecoder::new(reader);
        let save_data = bincode::deserialize_from::<_, DashMapWrapper<AudioMNISTKey, _, gxhash::GxBuildHasher>>(&mut reader);
        // let mut reader = std::io::BufReader::new(reader);
        // let save_data = serde_json::from_reader::<_, DashMapWrapper<AudioMNISTKey, _, gxhash::GxBuildHasher>>(reader);
        println!("decompress time: {:?}", now.elapsed());

        save_data
    }) {
        Ok(Ok(save_data)) => {
            println!("loaded save data");
            save_data.dash_map
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
    let saveable_data = DashMapWrapperRef { dash_map: save_data };
    let compress_now = std::time::Instant::now();

    println!("compressing...");

    let mut file = std::fs::File::create(save_data_path).unwrap();

    // let mut wtr = snap::write::FrameEncoder::new(
    //     &mut file,
    // );

    // let mut wtr = std::io::BufWriter::with_capacity(1024 * 1024 * 10, wtr);
    let mut wtr = std::io::BufWriter::new(&mut file);
    bincode::serialize_into(wtr, &saveable_data).unwrap();
    // serde_json::to_writer(&mut wtr, &saveable_data).unwrap();

    println!("compress and save time: {:?}", compress_now.elapsed());

    // let file = std::fs::File::open(save_data_path).unwrap();
    let file_size = file.metadata().unwrap().len();
    println!("file size: {} bytes", si_scale::helpers::bibytes2(file_size as f64));

    // let file2 = std::fs::File::create(format!("{}.s", save_data_path)).unwrap();
    // let mut file2 = std::io::BufWriter::with_capacity(1024 * 1024 * 10, file2);
    // bincode::serialize_into(file2, &saveable_data).unwrap();
    // serde_json::to_writer(&mut file2, &saveable_data).unwrap();

    // let file2 = std::fs::File::open(format!("{}.s", save_data_path)).unwrap();
    // let file_size = file2.metadata().unwrap().len();
    // println!("file size: {} bytes", si_scale::helpers::bibytes2(file_size as f64));

    blocker.map(|blocker| blocker.send(()));
}
