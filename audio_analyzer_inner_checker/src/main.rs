// $ENV:RUSTFLAGS="-C target-cpu=native"
// cargo run -p audio_analyzer_inner_checker -r

use std::{collections::HashMap, sync::atomic::AtomicBool};

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
use libs::load_dataset::{AudioBAVED, AudioMNISTData, BAVEDPattern, GetAnalyzedData};
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

fn set_handler_boxed(handler: Box<dyn Fn() + 'static + Send + Sync>) {
    *CTRLC_HANDLER.lock() = handler;
}

fn main() -> anyhow::Result<()> {
    let analyzer = fn_::analyzer;

    ctrlc::set_handler(move || {
        CTRLC_HANDLER.lock()();
    })
    .unwrap();

    // load AudioMNIST
    // let data = load_and_analysis_old(analyzer)?;
    let data = analyzer.load_and_analysis::<AudioMNISTData<_>, _, _, gxhash::GxBuildHasher>(
        MNIST_BASE_PATH,
        env!("CARGO_MANIFEST_DIR"),
        set_handler_boxed,
    )?;

    let data = analyzer.load_and_analysis::<AudioBAVED<_>, _, _, gxhash::GxBuildHasher>(
        BAVED_BASE_PATH,
        env!("CARGO_MANIFEST_DIR"),
        set_handler_boxed,
    )?;

    // load BAVED

    // let data = load_BAVED(BAVED_BASE_PATH, analyzer).unwrap();

    // println!("{:?}", data);

    Ok(())
}

pub trait LoadAndAnalysis<T, F>
where
    T: Send + Sync + Clone + 'static,
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

    fn get_all_pattern(base_path: &str) -> anyhow::Result<Self::AllPattern>;

    fn get_all_pattern_count(all_pattern: &Self::AllPattern) -> usize;

    fn iterate<U: Send + Sync>(
        patterns: &Self::AllPattern,
        f: impl Fn(&Self::Key) -> anyhow::Result<U> + Send + Sync,
    ) -> anyhow::Result<Vec<U>>;

    fn to_self(pattern_and_data: Vec<(Self::Key, T)>) -> Self;
}

impl<T, F> LoadAndAnalysis<T, F> for AudioMNISTData<T>
where
    T: Send + Sync + Clone + 'static,
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

    fn get_all_pattern(base_path: &str) -> anyhow::Result<Self::AllPattern> {
        use rayon::prelude::*;

        let all_pattern = (1..=60)
            .into_par_iter()
            .map(|speaker_n| {
                (0..10)
                    .into_iter()
                    .flat_map(|say_n| {
                        (0..50)
                            .into_iter()
                            .filter_map(|num_n| {
                                let path = <Self as LoadAndAnalysis<T, F>>::gen_path_from_key(
                                    &(speaker_n, say_n, num_n),
                                    base_path,
                                );

                                if std::path::Path::new(&path).exists() {
                                    Some((speaker_n, say_n, num_n))
                                } else {
                                    panic!("not found: {}", path);
                                }
                            })
                            .collect::<Vec<Self::Key>>()
                    })
                    .collect::<Vec<Self::Key>>()
            })
            .collect::<Vec<Vec<Self::Key>>>();

        Ok(all_pattern)
    }

    fn get_all_pattern_count(all_pattern: &Self::AllPattern) -> usize {
        all_pattern.iter().map(|v| v.len()).sum::<usize>()
    }

    fn iterate<U: Send + Sync>(
        patterns: &Self::AllPattern,
        f: impl Fn(&Self::Key) -> anyhow::Result<U> + Send + Sync,
    ) -> anyhow::Result<Vec<U>> {
        use rayon::iter::ParallelIterator;

        patterns
            .par_iter()
            .map(|pattern| {
                pattern
                    .iter()
                    .map(|key| f(key))
                    .collect::<anyhow::Result<Vec<U>>>()
            })
            .collect::<anyhow::Result<Vec<Vec<U>>>>()
            .map(|v| v.into_iter().flatten().collect())
    }

    fn to_self(pattern_and_data: Vec<(Self::Key, T)>) -> Self {
        let mut ret_data = Self {
            speakers: [const { Vec::<T>::new() }; 60],
        };

        for ((speaker_n, _, _), data) in pattern_and_data {

            ret_data.speakers[speaker_n].push(data);
        }

        ret_data
    }
}

impl<T, F> LoadAndAnalysis<T, F> for AudioBAVED<T>
where
    T: Send + Sync + Clone + 'static,
    F: Fn(&mut audio_analyzer_core::prelude::TestData, u32) -> T + Send + Sync,
{
    const UNIQUE_ID: &'static str = "AudioBAVED";

    type Key = BAVEDPattern;

    type AllPattern = Vec<HashMap<usize, Vec<BAVEDPattern>>>;

    fn gen_path_from_key(pattern: &Self::Key, base_path: &str) -> String {
        let BAVEDPattern {
            place,
            speaker_id,
            speaker_gender,
            speaker_age,
            spoken_word,
            spoken_emotion,
            record_id,
        } = pattern;

        let path = format!(
            "{base_path}/{place}/{speaker_id}-{speaker_gender}-{speaker_age}-{spoken_word}-{spoken_emotion}-{record_id}.wav"
        );

        path
    }

    fn get_all_pattern(base_path: &str) -> anyhow::Result<Self::AllPattern> {
        let analysis_file_name = |place: usize| -> Vec<BAVEDPattern> {
            walkdir::WalkDir::new(format!("{base_path}/{place}"))
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file())
                .filter_map(|e| {
                    let path = e.path();

                    let file_name = path.file_name()?.to_str()?;

                    let parts = file_name.split('-').collect::<Vec<_>>();

                    if parts.len() != 6 {
                        panic!("Invalid file name: {}", file_name);
                    }

                    let speaker_id = parts[0].parse::<usize>().unwrap();
                    let speaker_gender = parts[1].chars().next().unwrap();
                    let speaker_age = parts[2].parse::<usize>().unwrap();
                    let spoken_word = parts[3].parse::<usize>().unwrap();
                    let spoken_emotion = parts[4].parse::<usize>().unwrap();
                    let record_id = parts[5].split('.').next()?.parse::<usize>().unwrap();

                    Some(BAVEDPattern {
                        place,
                        speaker_id,
                        speaker_gender,
                        speaker_age,
                        spoken_word,
                        spoken_emotion,
                        record_id,
                    })
                })
                .collect()
        };

        let files: Vec<BAVEDPattern> = (0..=6)
            .into_par_iter()
            .map(|i| analysis_file_name(i))
            .flatten()
            .collect();

        let speaker_files: Vec<Vec<BAVEDPattern>> = files
            .into_iter()
            .fold(HashMap::new(), |mut acc, pattern| {
                let speaker_id = pattern.speaker_id;
                let gender = pattern.speaker_gender;
                let speaker_age = pattern.speaker_age;

                acc.entry((speaker_id, gender, speaker_age))
                    .or_insert_with(Vec::new)
                    .push(pattern);

                acc
            })
            .into_iter()
            .map(|(_, v)| v)
            .collect();

        use rayon::prelude::*;

        let level_files: Vec<HashMap<usize, Vec<BAVEDPattern>>> = speaker_files
            .into_par_iter()
            .map(|patterns| {
                patterns
                    .into_iter()
                    .fold(HashMap::new(), |mut acc, pattern| {
                        let spoken_emotion = pattern.spoken_emotion;

                        acc.entry(spoken_emotion)
                            .or_insert_with(Vec::new)
                            .push(pattern);

                        acc
                    })
            })
            .collect();

        Ok(level_files)
    }

    fn get_all_pattern_count(all_pattern: &Self::AllPattern) -> usize {
        all_pattern.iter().map(|v| v.iter().map(|(_, v)| v.len()).sum::<usize>()).sum::<usize>()
    }

    fn iterate<U: Send + Sync>(
        patterns: &Self::AllPattern,
        f: impl Fn(&Self::Key) -> anyhow::Result<U> + Send + Sync,
    ) -> anyhow::Result<Vec<U>> {
        use rayon::iter::ParallelIterator;

        patterns
            .par_iter()
            .map(|level| {
                level
                    .iter()
                    .flat_map(|(_, patterns)| patterns.iter().map(|pattern| f(pattern)))
                    .collect::<anyhow::Result<Vec<U>>>()
            })
            .collect::<anyhow::Result<Vec<Vec<U>>>>()
            .map(|v| v.into_iter().flatten().collect())
    }

    fn to_self(pattern_and_data: Vec<(Self::Key, T)>) -> Self {
        use crate::libs::load_dataset::AudioBAVEDEmotion;

        let mut speakers = [const {
            AudioBAVEDEmotion {
                level_0: Vec::new(),
                level_1: Vec::new(),
                level_2: Vec::new(),
            }
        }; 60];

        let mut speakers_inner = HashMap::new();

        for (pattern, data) in pattern_and_data.into_iter() {
            let data = data.clone();

            let emotion = pattern.spoken_emotion;

            let speaker_id = pattern.speaker_id;

            let speaker =
                &mut speakers_inner
                    .entry(speaker_id)
                    .or_insert_with(|| AudioBAVEDEmotion {
                        level_0: Vec::new(),
                        level_1: Vec::new(),
                        level_2: Vec::new(),
                    });

            match emotion {
                0 => speaker.level_0.push(data),
                1 => speaker.level_1.push(data),
                2 => speaker.level_2.push(data),
                _ => panic!("Invalid emotion level: {}", emotion),
            }
        }

        for (n, (_, speaker)) in speakers_inner.into_iter().enumerate() {
            speakers[n] = speaker;
        }

        Self { speakers }
    }
}

// fn load_and_analysis_old<
//     T: Send
//         + Sync
//         + ToOwned<Owned = T>
//         + Clone
//         + serde::Serialize
//         + for<'de> serde::Deserialize<'de>
//         + 'static,
//     // K: Eq + std::hash::Hash + Clone + serde::Serialize + Send + Sync,
// >(
//     analyzer: impl Fn(&mut audio_analyzer_core::prelude::TestData, u32) -> T + Send + Sync,
// ) -> anyhow::Result<AudioMNISTData<T>> {
//     type Hasher = gxhash::GxBuildHasher;
//     type MapType<K, V> = DashMap<K, V, Hasher>;

//     let save_data_path_mid = concat!(env!("CARGO_MANIFEST_DIR"), "/audio_mnist_data_mid.bincode");
//     let save_data_path = concat!(env!("CARGO_MANIFEST_DIR"), "/audio_mnist_data.bincode");

//     let data = if let Ok(Ok(data)) = {
//         std::fs::File::open(save_data_path)
//             .map(|f| {
//                 println!("loading...");
//                 let now = std::time::Instant::now();
//                 let mut reader = std::io::BufReader::new(f);
//                 let data = bincode::deserialize_from::<_, AudioMNISTData<_>>(&mut reader);
//                 data.map_err(|e| {
//                     println!("failed to load save audio_mnist_complete data: {:?}", e);
//                 })
//                 .map(|data| {
//                     println!("loaded save audio_mnist_complete data: {:?}", now.elapsed());
//                     data
//                 })
//             })
//             .map_err(|e| {
//                 println!("failed to load save audio_mnist_complete data: {:?}", e);
//             })
//     } {
//         data
//     } else {
//         let save_data: MapType<_, _> = match std::fs::File::open(save_data_path_mid).map(|f| {
//             let now = std::time::Instant::now();
//             println!("loading...");
//             let mut reader = std::io::BufReader::new(f);
//             let save_data =
//                 bincode::deserialize_from::<_, DashMapWrapper<_, _, Hasher>>(&mut reader);
//             println!("load time: {:?}", now.elapsed());

//             save_data
//         }) {
//             Ok(Ok(save_data)) => {
//                 println!("loaded save data");
//                 save_data.dash_map
//             }
//             Err(e) => {
//                 println!("failed to load save data: {:?}", e);
//                 DashMap::with_capacity_and_hasher(60 * 10 * 50, Hasher::default())
//             }
//             Ok(Err(e)) => {
//                 println!("failed to load save data: {:?}", e);
//                 DashMap::with_capacity_and_hasher(60 * 10 * 50, Hasher::default())
//             }
//         };
//         let leaked_save_data_mut: &'static mut MapType<_, _> = Box::leak(Box::new(save_data));
//         let leaked_save_data: &'static MapType<_, _> = leaked_save_data_mut;

//         let leaked_stopper_mut: &'static mut AtomicBool =
//             Box::leak(Box::new(AtomicBool::new(false)));
//         let leaked_stopper: &'static AtomicBool = leaked_stopper_mut;

//         let (blocker, waiter) = std::sync::mpsc::channel::<()>();

//         set_handler(move || {
//             println!("save with compress file");

//             save_with_compress_file(
//                 leaked_save_data,
//                 save_data_path_mid,
//                 Some(leaked_stopper),
//                 Some(&blocker),
//             );
//         });

//         let data =
//             load_AudioMNIST(MNIST_BASE_PATH, analyzer, leaked_save_data, &leaked_stopper).unwrap();

//         if leaked_stopper.load(std::sync::atomic::Ordering::Relaxed) {
//             waiter.recv().unwrap();

//             set_handler(move || {
//                 std::process::exit(0);
//             });

//             release(leaked_save_data);
//             release(leaked_stopper);

//             return Err(anyhow::anyhow!("stop"));
//         }

//         set_handler(move || {
//             std::process::exit(0);
//         });

//         let now = std::time::Instant::now();
//         println!("saving... audio mnist data");
//         let mut file = std::fs::File::create(save_data_path).unwrap();
//         let mut wtr = std::io::BufWriter::new(&mut file);
//         bincode::serialize_into(&mut wtr, &data).unwrap();

//         println!("save time: {:?}", now.elapsed());

//         release(leaked_save_data_mut);
//         release(leaked_stopper_mut);
//         std::mem::drop(waiter);

//         data
//     };

//     Ok(data)
// }

// fn save_with_compress_file<
//     K: Eq + std::hash::Hash + Clone + serde::Serialize + Send + Sync,
//     V: Clone + serde::Serialize + Send + Sync,
//     Hasher: Default + std::hash::BuildHasher + Clone + Send + Sync,
// >(
//     save_data: &'static DashMap<K, V, Hasher>,
//     save_data_path: &str,
//     stopper: Option<&AtomicBool>,
//     blocker: Option<&std::sync::mpsc::Sender<()>>,
// ) {
//     if let Some(stopper) = stopper {
//         if stopper.load(std::sync::atomic::Ordering::Relaxed) {
//             std::process::exit(0);
//         }
//         stopper.store(true, std::sync::atomic::Ordering::SeqCst);
//     }
//     println!("\n\rsaving...");

//     let save_data_path = save_data_path.to_string();

//     std::thread::spawn(move || {
//         let saveable_data = DashMapWrapperRef {
//             dash_map: save_data,
//         };
//         let compress_now = std::time::Instant::now();

//         let mut file = std::fs::File::create(save_data_path).unwrap();

//         // let mut wtr = snap::write::FrameEncoder::new(
//         //     &mut file,
//         // );

//         // let mut wtr = std::io::BufWriter::with_capacity(1024 * 1024 * 10, wtr);
//         let mut wtr = std::io::BufWriter::new(&mut file);
//         bincode::serialize_into(&mut wtr, &saveable_data).unwrap();
//         // serde_json::to_writer(&mut wtr, &saveable_data).unwrap();

//         println!("save time: {:?}", compress_now.elapsed());

//         // let file = std::fs::File::open(save_data_path).unwrap();
//         std::mem::drop(wtr);
//         let file_size = file.metadata().unwrap().len();
//         println!(
//             "file size: {} bytes",
//             si_scale::helpers::bibytes2(file_size as f64)
//         );

//         // let file2 = std::fs::File::create(format!("{}.s", save_data_path)).unwrap();
//         // let mut file2 = std::io::BufWriter::with_capacity(1024 * 1024 * 10, file2);
//         // bincode::serialize_into(file2, &saveable_data).unwrap();
//         // serde_json::to_writer(&mut file2, &saveable_data).unwrap();

//         // let file2 = std::fs::File::open(format!("{}.s", save_data_path)).unwrap();
//         // let file_size = file2.metadata().unwrap().len();
//         // println!("file size: {} bytes", si_scale::helpers::bibytes2(file_size as f64));
//     })
//     .join()
//     .unwrap();

//     blocker.map(|blocker| blocker.send(()));
// }

// fn release<T>(ptr: &T) {
//     let ptr = ptr as *const T;
//     std::mem::drop(unsafe { Box::<T>::from_raw(ptr as *mut T) });
// }
