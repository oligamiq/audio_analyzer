// $ENV:RUSTFLAGS="-C target-cpu=native"
// cargo run -p audio_analyzer_inner_checker -r

use std::sync::atomic::AtomicBool;

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
use libs::load_dataset::{load_AudioMNIST, load_BAVED, AudioMNISTData};
use rayon::iter::IntoParallelRefIterator;

static CTRLC_HANDLER: std::sync::LazyLock<
    parking_lot::Mutex<Box<dyn Fn() + 'static + Send + Sync>>,
> = std::sync::LazyLock::new(|| {
    parking_lot::Mutex::new(Box::new(move || {
        std::process::exit(0);
    }))
});

fn set_handler<F: Fn() + 'static + Send + Sync>(handler: F) {
    *CTRLC_HANDLER.lock() = Box::new(handler);
}

fn main() -> anyhow::Result<()> {
    let analyzer = fn_::analyzer;

    ctrlc::set_handler(move || {
        CTRLC_HANDLER.lock()();
    })
    .unwrap();

    // load AudioMNIST
    let data = load_and_analysis(analyzer)?;

    // load BAVED

    // let data = load_BAVED(BAVED_BASE_PATH, analyzer).unwrap();

    // println!("{:?}", data);

    Ok(())
}

pub trait LoadAndAnalysis<T, F>
where
    T: Send
        + Sync
        + ToOwned<Owned = T>
        + Clone
        + serde::Serialize
        + for<'de> serde::Deserialize<'de>
        + 'static,
    F: Fn(&mut audio_analyzer_core::prelude::TestData, u32) -> T + Send + Sync,
    Self: Sized,
{
    const UNIQUE_ID: &'static str;

    /// The key type for the data
    /// maybe pattern
    type Key: Eq + std::hash::Hash + Clone + serde::Serialize + Send + Sync;

    /// The top iterator item type
    /// Person level
    type AllPattern;

    fn gen_path_from_key(key: &Self::Key, base_path: &str) -> String;

    fn iterate<U: Send + Sync>(
        patterns: &Self::AllPattern,
        f: impl Fn(&Self::Key) -> anyhow::Result<U> + Send + Sync,
    ) -> anyhow::Result<Vec<U>>;

    fn to_self(pattern_and_data: Vec<(Self::Key, T)>) -> Self;
}

impl<T, F> LoadAndAnalysis<T, F> for AudioMNISTData<T>
where
    T: Send
        + Sync
        + ToOwned<Owned = T>
        + Clone
        + serde::Serialize
        + for<'de> serde::Deserialize<'de>
        + 'static,
    F: Fn(&mut audio_analyzer_core::prelude::TestData, u32) -> T + Send + Sync,
{
    const UNIQUE_ID: &'static str = "AudioMNISTData";

    type Key = (usize, usize, usize);

    type AllPattern = Vec<Vec<Self::Key>>;

    fn gen_path_from_key(key: &Self::Key, base_path: &str) -> String {
        let (speaker_n, say_n, num_n) = key.clone();

        assert!(say_n <= 9);

        assert!(num_n <= 49);

        let path = format!("{base_path}/{speaker_n:02}/{say_n}_{speaker_n:02}_{num_n}.wav");

        path
    }

    fn iterate<U: Send + Sync>(
        patterns: &Self::AllPattern,
        f: impl Fn(&Self::Key) -> anyhow::Result<U> + Send + Sync,
    ) -> anyhow::Result<Vec<U>> {
        use rayon::iter::ParallelIterator;

        patterns.par_iter().map(|pattern| {
            pattern
                .iter()
                .map(|key| f(key))
                .collect::<anyhow::Result<Vec<U>>>()
        }).collect::<anyhow::Result<Vec<Vec<U>>>>().map(|v| v.into_iter().flatten().collect())
    }

    fn to_self(pattern_and_data: Vec<(Self::Key, T)>) -> Self {
        let mut ret_data = Self {
            speakers: [const { Vec::<T>::new() }; 60],
        };

        for (key, data) in pattern_and_data {
            let (speaker_n, say_n, num_n) = key;

            ret_data.speakers[speaker_n].push(data);
        }

        ret_data
    }
}

fn load_and_analysis<
    T: Send
        + Sync
        + ToOwned<Owned = T>
        + Clone
        + serde::Serialize
        + for<'de> serde::Deserialize<'de>
        + 'static,
    // K: Eq + std::hash::Hash + Clone + serde::Serialize + Send + Sync,
>(
    analyzer: impl Fn(&mut audio_analyzer_core::prelude::TestData, u32) -> T + Send + Sync,
) -> anyhow::Result<AudioMNISTData<T>> {
    type Hasher = gxhash::GxBuildHasher;
    type MapType<K, V> = DashMap<K, V, Hasher>;

    let save_data_path_mid = concat!(env!("CARGO_MANIFEST_DIR"), "/audio_mnist_data_mid.bincode");
    let save_data_path = concat!(env!("CARGO_MANIFEST_DIR"), "/audio_mnist_data.bincode");

    let data = if let Ok(Ok(data)) = {
        std::fs::File::open(save_data_path)
            .map(|f| {
                println!("loading...");
                let now = std::time::Instant::now();
                let mut reader = std::io::BufReader::new(f);
                let data = bincode::deserialize_from::<_, AudioMNISTData<_>>(&mut reader);
                data.map_err(|e| {
                    println!("failed to load save audio_mnist_complete data: {:?}", e);
                })
                .map(|data| {
                    println!("loaded save audio_mnist_complete data: {:?}", now.elapsed());
                    data
                })
            })
            .map_err(|e| {
                println!("failed to load save audio_mnist_complete data: {:?}", e);
            })
    } {
        data
    } else {
        let save_data: MapType<_, _> = match std::fs::File::open(save_data_path_mid).map(|f| {
            let now = std::time::Instant::now();
            println!("loading...");
            let mut reader = std::io::BufReader::new(f);
            let save_data =
                bincode::deserialize_from::<_, DashMapWrapper<_, _, Hasher>>(&mut reader);
            println!("load time: {:?}", now.elapsed());

            save_data
        }) {
            Ok(Ok(save_data)) => {
                println!("loaded save data");
                save_data.dash_map
            }
            Err(e) => {
                println!("failed to load save data: {:?}", e);
                DashMap::with_capacity_and_hasher(60 * 10 * 50, Hasher::default())
            }
            Ok(Err(e)) => {
                println!("failed to load save data: {:?}", e);
                DashMap::with_capacity_and_hasher(60 * 10 * 50, Hasher::default())
            }
        };
        let leaked_save_data_mut: &'static mut MapType<_, _> = Box::leak(Box::new(save_data));
        let leaked_save_data: &'static MapType<_, _> = leaked_save_data_mut;

        let leaked_stopper_mut: &'static mut AtomicBool =
            Box::leak(Box::new(AtomicBool::new(false)));
        let leaked_stopper: &'static AtomicBool = leaked_stopper_mut;

        let (blocker, waiter) = std::sync::mpsc::channel::<()>();

        set_handler(move || {
            println!("save with compress file");

            save_with_compress_file(
                leaked_save_data,
                save_data_path_mid,
                Some(leaked_stopper),
                Some(&blocker),
            );
        });

        let data =
            load_AudioMNIST(MNIST_BASE_PATH, analyzer, leaked_save_data, &leaked_stopper).unwrap();

        if leaked_stopper.load(std::sync::atomic::Ordering::Relaxed) {
            waiter.recv().unwrap();

            set_handler(move || {
                std::process::exit(0);
            });

            release(leaked_save_data);
            release(leaked_stopper);

            return Err(anyhow::anyhow!("stop"));
        }

        set_handler(move || {
            std::process::exit(0);
        });

        let now = std::time::Instant::now();
        println!("saving... audio mnist data");
        let mut file = std::fs::File::create(save_data_path).unwrap();
        let mut wtr = std::io::BufWriter::new(&mut file);
        bincode::serialize_into(&mut wtr, &data).unwrap();

        println!("save time: {:?}", now.elapsed());

        release(leaked_save_data_mut);
        release(leaked_stopper_mut);
        std::mem::drop(waiter);

        data
    };

    Ok(data)
}

fn save_with_compress_file<
    K: Eq + std::hash::Hash + Clone + serde::Serialize + Send + Sync,
    V: Clone + serde::Serialize + Send + Sync,
    Hasher: Default + std::hash::BuildHasher + Clone + Send + Sync,
>(
    save_data: &'static DashMap<K, V, Hasher>,
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

    let save_data_path = save_data_path.to_string();

    std::thread::spawn(move || {
        let saveable_data = DashMapWrapperRef {
            dash_map: save_data,
        };
        let compress_now = std::time::Instant::now();

        let mut file = std::fs::File::create(save_data_path).unwrap();

        // let mut wtr = snap::write::FrameEncoder::new(
        //     &mut file,
        // );

        // let mut wtr = std::io::BufWriter::with_capacity(1024 * 1024 * 10, wtr);
        let mut wtr = std::io::BufWriter::new(&mut file);
        bincode::serialize_into(&mut wtr, &saveable_data).unwrap();
        // serde_json::to_writer(&mut wtr, &saveable_data).unwrap();

        println!("save time: {:?}", compress_now.elapsed());

        // let file = std::fs::File::open(save_data_path).unwrap();
        std::mem::drop(wtr);
        let file_size = file.metadata().unwrap().len();
        println!(
            "file size: {} bytes",
            si_scale::helpers::bibytes2(file_size as f64)
        );

        // let file2 = std::fs::File::create(format!("{}.s", save_data_path)).unwrap();
        // let mut file2 = std::io::BufWriter::with_capacity(1024 * 1024 * 10, file2);
        // bincode::serialize_into(file2, &saveable_data).unwrap();
        // serde_json::to_writer(&mut file2, &saveable_data).unwrap();

        // let file2 = std::fs::File::open(format!("{}.s", save_data_path)).unwrap();
        // let file_size = file2.metadata().unwrap().len();
        // println!("file size: {} bytes", si_scale::helpers::bibytes2(file_size as f64));
    })
    .join()
    .unwrap();

    blocker.map(|blocker| blocker.send(()));
}

fn release<T>(ptr: &T) {
    let ptr = ptr as *const T;
    std::mem::drop(unsafe { Box::<T>::from_raw(ptr as *mut T) });
}
