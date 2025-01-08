use std::collections::HashMap;

use audio_analyzer_core::prelude::TestData;
use dashmap::DashMap;
use rayon::prelude::*;

#[derive(Debug, Clone)]
pub struct AudioMNISTData<T> {
    pub speakers: [Vec<T>; 60],
}

#[allow(non_snake_case)]
pub fn load_AudioMNIST<T: Send + Sync + ToOwned<Owned = T>>(
    base_path: &str,
    analyzer: impl Fn(&mut TestData, u32) -> T + Send + Sync,
    is_parallel: bool,
    save_data: &DashMap<(usize, usize, usize), T, gxhash::GxBuildHasher>,
) -> anyhow::Result<AudioMNISTData<T>> {
    let gen_path = |speaker_n: usize, say_n: usize, num_n: usize| {
        assert!(say_n <= 9);

        assert!(num_n <= 49);

        let path = format!("{base_path}/{speaker_n:02}/{say_n}_{speaker_n:02}_{num_n}.wav");

        path
    };

    let get_data = |path: &str| {
        let mut data = TestData::new_with_path(path.to_owned());

        data.start();

        let sample_rate = data.sample_rate();

        let data_s = analyzer(&mut data, sample_rate);

        data_s
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

    if is_parallel {
        (1..=60)
            .into_par_iter()
            .map(|speaker_n| {
                for j in 0..9 {
                    for k in 0..50 {
                        if save_data.contains_key(&(speaker_n, j, k)) {
                            continue;
                        }

                        let path = gen_path(speaker_n, j, k);

                        progress.lock().inc();
                        save_data.insert((speaker_n, j, k), get_data(&path));
                    }
                }
            })
            .count();

        progress
            .lock()
            .finish_println("Finished loading AudioMNIST dataset.");

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

        for (i, speaker) in speaker_data.into_iter().enumerate() {
            speakers[i] = speaker;
        }

        return Ok(AudioMNISTData { speakers });
    } else {
        let data = (1..=60)
            .into_iter()
            .map(|speaker_n| {
                (0..9)
                    .into_iter()
                    .flat_map(|j| {
                        (0..50).into_iter().map(move |k| {
                            let path = gen_path(speaker_n, j, k);

                            get_data(&path)
                        })
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        let mut speakers = [const { Vec::new() }; 60];

        for (i, speaker) in data.into_iter().enumerate() {
            speakers[i] = speaker;
        }

        return Ok(AudioMNISTData { speakers });
    }
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

#[allow(non_snake_case)]
pub fn load_BAVED<T: Send + Sync>(
    base_path: &str,
    analyzer: impl Fn(&mut TestData, u32) -> T + Send + Sync,
) -> anyhow::Result<AudioBAVED<T>> {
    struct Pattern {
        place: usize,
        speaker_id: usize,
        speaker_gender: char,
        speaker_age: usize,
        spoken_word: usize,
        spoken_emotion: usize,
        record_id: usize,
    }

    let analysis_file_name = |place: usize| -> Vec<Pattern> {
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

                Some(Pattern {
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

    let gen_path_full = |pattern: &Pattern| {
        let Pattern {
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

    let files: Vec<Pattern> = (0..=6)
        .into_par_iter()
        .map(|i| analysis_file_name(i))
        .flatten()
        .collect();

    let speaker_files: Vec<Vec<Pattern>> = files
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

    let level_files: Vec<HashMap<usize, Vec<Pattern>>> = speaker_files
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

    let data_s = level_files
        .into_par_iter()
        .map(|level| {
            let gen_data = |pattern: &Vec<Pattern>| {
                pattern
                    .into_iter()
                    .map(|pattern| {
                        let path = gen_path_full(pattern);

                        let mut data = TestData::new_with_path(path.to_owned());

                        data.start();

                        let sample_rate = data.sample_rate();

                        let data_s = analyzer(&mut data, sample_rate);

                        data_s
                    })
                    .collect::<Vec<_>>()
            };

            let level_0 = gen_data(&level.get(&0).unwrap_or(&Vec::new()));
            let level_1 = gen_data(&level.get(&1).unwrap_or(&Vec::new()));
            let level_2 = gen_data(&level.get(&2).unwrap_or(&Vec::new()));

            AudioBAVEDEmotion {
                level_0,
                level_1,
                level_2,
            }
        })
        .collect::<Vec<_>>();

    let mut speakers = [const {
        AudioBAVEDEmotion {
            level_0: Vec::new(),
            level_1: Vec::new(),
            level_2: Vec::new(),
        }
    }; 60];

    for (i, speaker) in data_s.into_iter().enumerate() {
        speakers[i] = speaker;
    }

    Ok(AudioBAVED { speakers })
}
