use audio_analyzer_core::prelude::TestData;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct AudioMNISTData<T> {
    pub speakers: [Vec<T>; 60],
}

impl<'de, T: for<'b> Deserialize<'b>> Deserialize<'de> for AudioMNISTData<T> {
    fn deserialize<D>(deserializer: D) -> Result<AudioMNISTData<T>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let speakers_array = Vec::<Vec<T>>::deserialize(deserializer)?;

        let mut speakers = [const { Vec::new() }; 60];

        for (i, speaker) in speakers_array.into_iter().enumerate() {
            speakers[i] = speaker;
        }

        Ok(AudioMNISTData { speakers })
    }
}

impl<T: Serialize> Serialize for AudioMNISTData<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.speakers.serialize(serializer)
    }
}

// #[allow(non_snake_case)]
// pub fn load_AudioMNIST<T: Send + Sync + ToOwned<Owned = T>>(
//     base_path: &str,
//     analyzer: impl Fn(&mut TestData, u32) -> T + Send + Sync,
//     save_data: &DashMap<(usize, usize, usize), T, gxhash::GxBuildHasher>,
//     stopper: &std::sync::atomic::AtomicBool,
// ) -> anyhow::Result<AudioMNISTData<T>> {
//     let gen_path = |speaker_n: usize, say_n: usize, num_n: usize| {
//         assert!(say_n <= 9);

//         assert!(num_n <= 49);

//         let path = format!("{base_path}/{speaker_n:02}/{say_n}_{speaker_n:02}_{num_n}.wav");

//         path
//     };

//     let current_loaded = save_data.len();
//     let progress = parking_lot::Mutex::new(pbr::ProgressBar::new(
//         (60 * 10 * 50 - current_loaded) as u64,
//     ));
//     {
//         let mut progress = progress.lock();
//         progress.message("Loading AudioMNIST dataset...");
//         progress.message(&format!("current loaded: {current_loaded}"));
//         progress.message("analyzing...");
//     }

//     (1..=60).into_par_iter().for_each(|speaker_n| {
//         for j in 0..9 {
//             for k in 0..50 {
//                 if save_data.contains_key(&(speaker_n, j, k)) {
//                     continue;
//                 }

//                 if stopper.load(std::sync::atomic::Ordering::Relaxed) {
//                     break;
//                 }

//                 let path = gen_path(speaker_n, j, k);

//                 save_data.insert((speaker_n, j, k), analyzer.get_data(&path));

//                 if stopper.load(std::sync::atomic::Ordering::Relaxed) {
//                     break;
//                 }

//                 progress.lock().inc();
//             }
//         }
//     });

//     if stopper.load(std::sync::atomic::Ordering::Relaxed) {
//         return Ok(AudioMNISTData {
//             speakers: [const { Vec::new() }; 60],
//         });
//     }

//     progress
//         .lock()
//         .finish_println("Finished loading AudioMNIST dataset.");

//     let now = std::time::Instant::now();

//     let mut speakers = [const { Vec::new() }; 60];

//     let speaker_data = (1..=60)
//         .into_par_iter()
//         .map(|speaker_n| {
//             let mut data = Vec::with_capacity(9 * 50);

//             for j in 0..9 {
//                 for k in 0..50 {
//                     let data_s = save_data.get(&(speaker_n, j, k)).unwrap().to_owned();

//                     data.push(data_s);
//                 }
//             }

//             data
//         })
//         .collect::<Vec<_>>();

//     println!("\n\rto Vec: {:?}", now.elapsed());

//     for (i, speaker) in speaker_data.into_iter().enumerate() {
//         speakers[i] = speaker;
//     }

//     println!("to array: {:?}", now.elapsed());

//     return Ok(AudioMNISTData { speakers });
// }

#[derive(Debug, Clone)]
pub struct AudioBAVED<T> {
    pub speakers: [AudioBAVEDEmotion<T>; 60],
}

impl<T> Serialize for AudioBAVED<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.speakers.serialize(serializer)
    }
}

impl<'de, T: for<'b> Deserialize<'b>> Deserialize<'de> for AudioBAVED<T> {
    fn deserialize<D>(deserializer: D) -> Result<AudioBAVED<T>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let speakers = Vec::<AudioBAVEDEmotion<T>>::deserialize(deserializer)?;

        let mut speakers_array = [const {
            AudioBAVEDEmotion {
                level_0: Vec::new(),
                level_1: Vec::new(),
                level_2: Vec::new(),
            }
        }; 60];

        for (i, speaker) in speakers.into_iter().enumerate() {
            speakers_array[i] = speaker;
        }

        Ok(AudioBAVED {
            speakers: speakers_array,
        })
    }
}

/// emotion level 0: neutral
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct AudioBAVEDEmotion<T> {
    pub level_0: Vec<T>,
    pub level_1: Vec<T>,
    pub level_2: Vec<T>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct BAVEDPattern {
    pub place: usize,
    pub speaker_id: usize,
    pub speaker_gender: char,
    pub speaker_age: usize,
    pub spoken_word: usize,
    pub spoken_emotion: usize,
    pub record_id: usize,
}

// #[allow(non_snake_case)]
// pub fn load_BAVED<T: Send + Sync + ToOwned<Owned = T>>(
//     base_path: &str,
//     analyzer: impl Fn(&mut TestData, u32) -> T + Send + Sync,
//     save_data: &DashMap<BAVEDPattern, T, gxhash::GxBuildHasher>,
//     stopper: &std::sync::atomic::AtomicBool,
// ) -> anyhow::Result<AudioBAVED<T>> {
//     let gen_path_full = |pattern: &BAVEDPattern| {
//         let BAVEDPattern {
//             place,
//             speaker_id,
//             speaker_gender,
//             speaker_age,
//             spoken_word,
//             spoken_emotion,
//             record_id,
//         } = pattern;

//         let path = format!(
//             "{base_path}/{place}/{speaker_id}-{speaker_gender}-{speaker_age}-{spoken_word}-{spoken_emotion}-{record_id}.wav"
//         );

//         path
//     };

//     let save_level_files_path = concat!(
//         env!("CARGO_MANIFEST_DIR"),
//         "/audio_baved_data_level_files.bincode"
//     );
//     let level_files = if let Ok(level_files) =
//         bincode::deserialize_from(std::fs::File::open(save_level_files_path)?)
//     {
//         level_files
//     } else {

//     };

//     let files_len = level_files.iter().map(|x| x.len()).sum::<usize>();
//     let current_loaded = save_data.len();
//     let progress =
//         parking_lot::Mutex::new(pbr::ProgressBar::new((files_len - current_loaded) as u64));
//     {
//         let mut progress = progress.lock();
//         progress.message("Loading BAVED dataset...");
//         progress.message(&format!("current loaded: {current_loaded}"));
//         progress.message("analyzing...");
//     }

//     level_files
//         .par_iter()
//         // .into_iter()
//         .for_each(|level| {
//             for (emotion, patterns) in level.iter() {
//                 for pattern in patterns.iter() {
//                     if save_data.contains_key(&pattern) {
//                         continue;
//                     }

//                     if stopper.load(std::sync::atomic::Ordering::Relaxed) {
//                         break;
//                     }

//                     let path = gen_path_full(&pattern);

//                     save_data.insert(pattern.clone(), analyzer.get_data(&path));

//                     if stopper.load(std::sync::atomic::Ordering::Relaxed) {
//                         break;
//                     }

//                     progress.lock().inc();
//                 }
//             }
//         });

//     if stopper.load(std::sync::atomic::Ordering::Relaxed) {
//         return Ok(AudioBAVED {
//             speakers: [const {
//                 AudioBAVEDEmotion {
//                     level_0: Vec::new(),
//                     level_1: Vec::new(),
//                     level_2: Vec::new(),
//                 }
//             }; 60],
//         });
//     }

//     progress
//         .lock()
//         .finish_println("Finished loading BAVED dataset.");

//     let now = std::time::Instant::now();

//     let mut speakers = [const {
//         AudioBAVEDEmotion {
//             level_0: Vec::new(),
//             level_1: Vec::new(),
//             level_2: Vec::new(),
//         }
//     }; 60];

//     for key_and_value in save_data.iter() {
//         let pattern = key_and_value.key();
//         let data: T = key_and_value.value().to_owned();

//         let emotion = pattern.spoken_emotion;

//         let speaker_id = pattern.speaker_id;

//         let speaker = &mut speakers[speaker_id];

//         match emotion {
//             0 => speaker.level_0.push(data),
//             1 => speaker.level_1.push(data),
//             2 => speaker.level_2.push(data),
//             _ => panic!("Invalid emotion level: {}", emotion),
//         }
//     }

//     Ok(AudioBAVED { speakers })
// }

use crate::LoadAndAnalysis;
pub trait GetAnalyzedData<T>
where
    Self: Fn(&mut TestData, u32) -> T + Send + Sync,
{
    fn get_data<S: AsRef<str>>(&self, path: S) -> T;

    fn load_and_analysis<Dataset: LoadAndAnalysis<T, Self>, P, D, Hasher>(
        &self,
        base_path: P,
        save_data_dir: D,
        set_handler: impl Fn(Box<dyn Fn() + 'static + Send + Sync>) + Send + Sync,
    ) -> anyhow::Result<Dataset>
    where
        Dataset: LoadAndAnalysis<T, Self> + serde::Serialize + for<'de> serde::Deserialize<'de>,
        T: ToOwned<Owned = T>
            + Send
            + Sync
            + Clone
            + serde::Serialize
            + for<'de> serde::Deserialize<'de>
            + 'static,
        P: AsRef<str>,
        D: AsRef<str>,
        Hasher: Default + std::hash::BuildHasher + Clone + Send + Sync + 'static,
        Dataset::Key: serde::Serialize + for<'de> serde::Deserialize<'de> + 'static,
        Dataset::AllPattern: serde::Serialize + for<'de> serde::Deserialize<'de>,
        Self: Sized,
    {
        use crate::deserialize::DashMapWrapper;
        use convert_case::{Case, Casing as _};
        use std::sync::atomic::AtomicBool;

        let first_now = std::time::Instant::now();

        let base_path = base_path.as_ref();
        let save_data_dir = save_data_dir.as_ref();

        let unique_id = <Dataset as LoadAndAnalysis<T, Self>>::UNIQUE_ID;
        let snake_unique_id = unique_id.to_case(Case::Snake);
        let save_data_path_mid = format!("{save_data_dir}/{snake_unique_id}_mid.bincode");
        let save_data_path = format!("{save_data_dir}/{snake_unique_id}.bincode");
        let save_data_all_pattern_path =
            format!("{save_data_dir}/{snake_unique_id}_all_pattern.bincode");

        let data: Dataset = if let Ok(Ok(data)) = std::fs::File::open(&save_data_path)
            .map(|f| {
                println!("loading...");
                let now = std::time::Instant::now();
                let mut reader = std::io::BufReader::new(f);
                let data = bincode::deserialize_from::<_, Dataset>(&mut reader);
                data.map_err(|e| {
                    println!("failed to load save {snake_unique_id} data: {:?}", e);
                })
                .map(|data| {
                    println!("loaded save {snake_unique_id} data: {:?}", now.elapsed());
                    data
                })
            })
            .map_err(|e| {
                println!("failed to load save {snake_unique_id} data: {:?}", e);
            }) {
            data
        } else {
            let load_all_pattern = || {
                if let Ok(Ok(all_data)) = {
                    std::fs::File::open(&save_data_all_pattern_path)
                        .map(|f| {
                            let now = std::time::Instant::now();
                            println!("loading...");
                            let mut reader = std::io::BufReader::new(f);
                            let all_data = bincode::deserialize_from::<
                                _,
                                <Dataset as LoadAndAnalysis<T, Self>>::AllPattern,
                            >(&mut reader);
                            println!("loaded save {snake_unique_id} all pattern data: {:?}", now.elapsed());

                            all_data
                        })
                        .map_err(|e| {
                            println!("failed to load save {snake_unique_id} data about all pattern: {:?}", e);
                        })
                } {
                    Ok(all_data)
                } else {
                    Dataset::get_all_pattern(base_path).map(|all_pattern| {
                        if let Ok(file) = std::fs::File::create(&save_data_all_pattern_path) {
                            let mut wtr = std::io::BufWriter::new(file);
                            if let Err(e) = bincode::serialize_into(&mut wtr, &all_pattern) {
                                println!(
                                    "failed to save {snake_unique_id} all pattern data: {:?}",
                                    e
                                );
                            }
                        }

                        all_pattern
                    })
                }
            };

            let (save_data, all_data) = match std::fs::File::open(&save_data_path_mid).map(|f| {
                let now = std::time::Instant::now();
                println!("loading...");
                let mut reader = std::io::BufReader::new(f);
                let save_data =
                    bincode::deserialize_from::<_, DashMapWrapper<_, _, Hasher>>(&mut reader);
                println!("load time: {:?}", now.elapsed());

                save_data
            }) {
                Ok(Ok(save_data)) => {
                    println!("loaded {snake_unique_id} save data");
                    (save_data.dash_map, load_all_pattern()?)
                }
                Err(e) => {
                    println!("failed to load {snake_unique_id} save data: {:?}", e);
                    let all_data = load_all_pattern()?;
                    let len = Dataset::get_all_pattern_count(&all_data);
                    (
                        DashMap::with_capacity_and_hasher(len, Hasher::default()),
                        all_data,
                    )
                }
                Ok(Err(e)) => {
                    println!("failed to load {snake_unique_id} save data: {:?}", e);
                    let all_data = load_all_pattern()?;
                    let len = Dataset::get_all_pattern_count(&all_data);
                    (
                        DashMap::with_capacity_and_hasher(len, Hasher::default()),
                        all_data,
                    )
                }
            };

            let leaked_save_data_mut: &'static mut _ = Box::leak(Box::new(save_data));
            let leaked_save_data: &'static _ = leaked_save_data_mut;

            let leaked_stopper_mut: &'static mut AtomicBool =
                Box::leak(Box::new(AtomicBool::new(false)));
            let leaked_stopper: &'static AtomicBool = leaked_stopper_mut;

            let (blocker, waiter) = std::sync::mpsc::channel::<()>();

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
                println!("save with compress file");

                if let Some(stopper) = stopper {
                    if stopper.load(std::sync::atomic::Ordering::Relaxed) {
                        std::process::exit(0);
                    }
                    stopper.store(true, std::sync::atomic::Ordering::SeqCst);
                }
                println!("\n\rsaving...");

                let save_data_path = save_data_path.to_string();

                std::thread::spawn(move || {
                    let saveable_data = crate::DashMapWrapperRef {
                        dash_map: save_data,
                    };
                    let compress_now = std::time::Instant::now();

                    let mut file = std::fs::File::create(save_data_path).unwrap();
                    let mut wtr = std::io::BufWriter::new(&mut file);
                    bincode::serialize_into(&mut wtr, &saveable_data).unwrap();

                    println!("save time: {:?}", compress_now.elapsed());

                    std::mem::drop(wtr);
                    let file_size = file.metadata().unwrap().len();
                    println!(
                        "file size: {} bytes",
                        si_scale::helpers::bibytes2(file_size as f64)
                    );
                })
                .join()
                .unwrap();

                blocker.map(|blocker| blocker.send(()));
            }

            let save_data_path_mid_clone = save_data_path_mid.clone();
            set_handler(Box::new(move || {
                save_with_compress_file(
                    leaked_save_data,
                    &save_data_path_mid_clone,
                    Some(leaked_stopper),
                    Some(&blocker),
                );
            }));

            let data = {
                let save_data = leaked_save_data;
                let stopper = leaked_stopper;

                let current_loaded = save_data.len() as i64;

                let progress = parking_lot::Mutex::new(pbr::ProgressBar::new(
                    (Dataset::get_all_pattern_count(&all_data) as i64 - current_loaded) as u64,
                ));
                {
                    let mut progress = progress.lock();
                    progress.message("Loading dataset...");
                    progress.message(&format!("current loaded: {current_loaded}"));
                    progress.message("analyzing...");
                }

                if let Err(e) =
                    Dataset::iterate(&all_data, |pattern: &Dataset::Key| -> anyhow::Result<()> {
                        if save_data.contains_key(pattern) {
                            return Ok(());
                        }

                        if stopper.load(std::sync::atomic::Ordering::Relaxed) {
                            return Err(anyhow::anyhow!("stopped"));
                        }

                        let path = Dataset::gen_path_from_key(pattern, base_path);

                        save_data.insert(pattern.clone(), self.get_data(&path));

                        if stopper.load(std::sync::atomic::Ordering::Relaxed) {
                            return Err(anyhow::anyhow!("stopped"));
                        }

                        progress.lock().inc();

                        Ok(())
                    })
                {
                    if stopper.load(std::sync::atomic::Ordering::Relaxed) {
                        // The whole program should stop eventually.
                        waiter.recv().unwrap();
                    } else {
                        save_with_compress_file(leaked_save_data, &save_data_path_mid, None, None);
                    }

                    set_handler(Box::new(move || {
                        std::process::exit(0);
                    }));

                    release(leaked_save_data);
                    release(leaked_stopper);

                    return Err(e);
                }

                progress.lock().finish_println("Finished loading dataset.");

                set_handler(Box::new(move || {
                    std::process::exit(0);
                }));

                let data_from = Dataset::iterate(
                    &all_data,
                    |pattern: &Dataset::Key| -> anyhow::Result<(Dataset::Key, T)> {
                        let data = save_data.get(pattern).unwrap().to_owned();

                        let pattern = pattern.clone();

                        Ok((pattern, data))
                    },
                )?;

                let data = Dataset::to_self(data_from);

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

            println!("analyzed all: {:?}", first_now.elapsed());

            data
        };

        fn release<T>(ptr: &T) {
            let ptr = ptr as *const T;
            std::mem::drop(unsafe { Box::<T>::from_raw(ptr as *mut T) });
        }

        Ok(data)
    }
}

impl<F, T> GetAnalyzedData<T> for F
where
    F: Fn(&mut TestData, u32) -> T + Send + Sync,
{
    fn get_data<S: AsRef<str>>(&self, path: S) -> T {
        let mut data = TestData::new_with_path(path.as_ref().to_owned());

        data.start();

        let sample_rate = data.sample_rate();

        self(&mut data, sample_rate)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Default)]
pub struct AudioChimeHome<T> {
    pub father: AudioChimeHomeNoise<T>,
    pub mother: AudioChimeHomeNoise<T>,
    pub child: AudioChimeHomeNoise<T>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Default)]
pub struct AudioChimeHomeNoise<T> {
    pub none: Vec<T>,
    pub human_activity: Vec<T>,
    pub television: Vec<T>,
    pub household_appliance: Vec<T>,
    pub human_activity_and_television: Vec<T>,
    pub human_activity_and_household_appliance: Vec<T>,
    pub television_and_household_appliance: Vec<T>,
    pub all: Vec<T>,
}

/// https://paperswithcode.com/dataset/chime-home
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Default)]
pub struct AudioChimeHomePattern {
    // f: father, m: mother, c: child
    pub speaker_id: char,

    // p
    pub human_activity_noise: bool,
    // b
    pub television_noise: bool,
    // o
    pub household_appliance_noise: bool,

    // file name
    pub file_name: String,
}
