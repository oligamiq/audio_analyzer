use std::collections::HashMap;

use audio_analyzer_core::prelude::TestData;
use dashmap::DashMap;
use rayon::prelude::*;
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

#[allow(non_snake_case)]
pub fn load_AudioMNIST<T: Send + Sync + ToOwned<Owned = T>>(
    base_path: &str,
    analyzer: impl Fn(&mut TestData, u32) -> T + Send + Sync,
    save_data: &DashMap<(usize, usize, usize), T, gxhash::GxBuildHasher>,
    stopper: &std::sync::atomic::AtomicBool,
) -> anyhow::Result<AudioMNISTData<T>> {
    let gen_path = |speaker_n: usize, say_n: usize, num_n: usize| {
        assert!(say_n <= 9);

        assert!(num_n <= 49);

        let path = format!("{base_path}/{speaker_n:02}/{say_n}_{speaker_n:02}_{num_n}.wav");

        path
    };

    let current_loaded = save_data.len();
    let progress = parking_lot::Mutex::new(pbr::ProgressBar::new(
        (60 * 10 * 50 - current_loaded) as u64,
    ));
    {
        let mut progress = progress.lock();
        progress.message("Loading AudioMNIST dataset...");
        progress.message(&format!("current loaded: {current_loaded}"));
        progress.message("analyzing...");
    }

    (1..=60).into_par_iter().for_each(|speaker_n| {
        for j in 0..9 {
            for k in 0..50 {
                if save_data.contains_key(&(speaker_n, j, k)) {
                    continue;
                }

                if stopper.load(std::sync::atomic::Ordering::Relaxed) {
                    break;
                }

                let path = gen_path(speaker_n, j, k);

                save_data.insert((speaker_n, j, k), analyzer.get_data(&path));

                if stopper.load(std::sync::atomic::Ordering::Relaxed) {
                    break;
                }

                progress.lock().inc();
            }
        }
    });

    if stopper.load(std::sync::atomic::Ordering::Relaxed) {
        return Ok(AudioMNISTData {
            speakers: [const { Vec::new() }; 60],
        });
    }

    progress
        .lock()
        .finish_println("Finished loading AudioMNIST dataset.");

    let now = std::time::Instant::now();

    let mut speakers = [const { Vec::new() }; 60];

    let speaker_data = (1..=60)
        .into_par_iter()
        .map(|speaker_n| {
            let mut data = Vec::with_capacity(9 * 50);

            for j in 0..9 {
                for k in 0..50 {
                    let data_s = save_data.get(&(speaker_n, j, k)).unwrap().to_owned();

                    data.push(data_s);
                }
            }

            data
        })
        .collect::<Vec<_>>();

    println!("\n\rto Vec: {:?}", now.elapsed());

    for (i, speaker) in speaker_data.into_iter().enumerate() {
        speakers[i] = speaker;
    }

    println!("to array: {:?}", now.elapsed());

    return Ok(AudioMNISTData { speakers });
}

#[derive(Debug, Clone)]
pub struct AudioBAVED<T> {
    pub speakers: [AudioBAVEDEmotion<T>; 60],
}

/// emotion level 0: neutral
#[derive(Debug, Clone)]
pub struct AudioBAVEDEmotion<T> {
    pub level_0: Vec<T>,
    pub level_1: Vec<T>,
    pub level_2: Vec<T>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct BAVEDPattern {
    place: usize,
    speaker_id: usize,
    speaker_gender: char,
    speaker_age: usize,
    spoken_word: usize,
    spoken_emotion: usize,
    record_id: usize,
}

#[allow(non_snake_case)]
pub fn load_BAVED<T: Send + Sync + ToOwned<Owned = T>>(
    base_path: &str,
    analyzer: impl Fn(&mut TestData, u32) -> T + Send + Sync,
    save_data: &DashMap<BAVEDPattern, T, gxhash::GxBuildHasher>,
    stopper: &std::sync::atomic::AtomicBool,
) -> anyhow::Result<AudioBAVED<T>> {
    let gen_path_full = |pattern: &BAVEDPattern| {
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
    };

    let save_level_files_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/audio_baved_data_level_files.bincode"
    );
    let level_files = if let Ok(level_files) =
        bincode::deserialize_from(std::fs::File::open(save_level_files_path)?)
    {
        level_files
    } else {
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

        bincode::serialize_into(std::fs::File::create(save_level_files_path)?, &level_files)?;
        level_files
    };

    let files_len = level_files.iter().map(|x| x.len()).sum::<usize>();
    let current_loaded = save_data.len();
    let progress =
        parking_lot::Mutex::new(pbr::ProgressBar::new((files_len - current_loaded) as u64));
    {
        let mut progress = progress.lock();
        progress.message("Loading BAVED dataset...");
        progress.message(&format!("current loaded: {current_loaded}"));
        progress.message("analyzing...");
    }

    level_files
        .par_iter()
        // .into_iter()
        .for_each(|level| {
            for (emotion, patterns) in level.iter() {
                for pattern in patterns.iter() {
                    if save_data.contains_key(&pattern) {
                        continue;
                    }

                    if stopper.load(std::sync::atomic::Ordering::Relaxed) {
                        break;
                    }

                    let path = gen_path_full(&pattern);

                    save_data.insert(pattern.clone(), analyzer.get_data(&path));

                    if stopper.load(std::sync::atomic::Ordering::Relaxed) {
                        break;
                    }

                    progress.lock().inc();
                }
            }
        });

    if stopper.load(std::sync::atomic::Ordering::Relaxed) {
        return Ok(AudioBAVED {
            speakers: [const {
                AudioBAVEDEmotion {
                    level_0: Vec::new(),
                    level_1: Vec::new(),
                    level_2: Vec::new(),
                }
            }; 60],
        });
    }

    progress
        .lock()
        .finish_println("Finished loading BAVED dataset.");

    let now = std::time::Instant::now();

    let mut speakers = [const {
        AudioBAVEDEmotion {
            level_0: Vec::new(),
            level_1: Vec::new(),
            level_2: Vec::new(),
        }
    }; 60];

    for key_and_value in save_data.iter() {
        let pattern = key_and_value.key();
        let data: T = key_and_value.value().to_owned();

        let emotion = pattern.spoken_emotion;

        let speaker_id = pattern.speaker_id;

        let speaker = &mut speakers[speaker_id];

        match emotion {
            0 => speaker.level_0.push(data),
            1 => speaker.level_1.push(data),
            2 => speaker.level_2.push(data),
            _ => panic!("Invalid emotion level: {}", emotion),
        }
    }

    Ok(AudioBAVED { speakers })
}

pub trait GetAnalyzedData<T>
where
    Self: Fn(&mut TestData, u32) -> T + Send + Sync,
{
    fn get_data<S: AsRef<str>>(&self, path: S) -> T;
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

// impl<
