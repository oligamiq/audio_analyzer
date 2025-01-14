// $ENV:RUSTFLAGS="-C target-cpu=native"
// cargo run -p audio_analyzer_inner_checker -r

use std::{collections::HashMap, io::Write};

use analysis::Analysis;

// pub mod brotli_system;
pub mod analysis;
pub mod deserialize;
pub mod fn_;
pub mod libs;
pub mod presets;

const MNIST_BASE_PATH: &'static str =
    concat!(env!("CARGO_MANIFEST_DIR"), "/datasets/AudioMNIST/data");
const BAVED_BASE_PATH: &'static str = concat!(env!("CARGO_MANIFEST_DIR"), "/datasets/BAVED/remake");
const CHIME_HOME_BASE_PATH: &'static str =
    concat!(env!("CARGO_MANIFEST_DIR"), "/datasets/ChimeHome/chunks");

use deserialize::DashMapWrapperRef;
use libs::load_dataset::{
    AudioBAVED, AudioChimeHome, AudioChimeHomePattern, AudioMNISTData, BAVEDPattern,
    GetAnalyzedData,
};
use rayon::iter::IntoParallelRefIterator;

static CTRLC_HANDLER: std::sync::LazyLock<
    parking_lot::Mutex<Box<dyn Fn() + 'static + Send + Sync>>,
> = std::sync::LazyLock::new(|| {
    parking_lot::Mutex::new(Box::new(move || {
        std::process::exit(0);
    }))
});

fn set_handler_boxed(handler: Box<dyn Fn() + 'static + Send + Sync>) {
    *CTRLC_HANDLER.lock() = handler;
}

fn main() -> anyhow::Result<()> {
    let now = std::time::Instant::now();

    ctrlc::set_handler(move || {
        CTRLC_HANDLER.lock()();
    })
    .unwrap();

    // analysis()?;

    const USE_DATA_N: usize = 10;

    let range = (1..=100).map(|n| n as f64 / 100.).collect::<Vec<_>>();

    // let data = load_data::<Vec<f64>, _>("burg")?;
    let analysis_data = analysis_load_data::<Vec<f64>, _, USE_DATA_N>("burg", &range)?;
    println!("analysis_data: {:?}", analysis_data);

    // let data = load_data::<Vec<Vec<Option<f64>>>, _>("burg_uncompress")?;
    // let analysis_data =
    //     analysis_load_data::<Vec<Vec<Option<f64>>>, _, USE_DATA_N>("burg_uncompress")?;
    // println!("analysis_data: {:?}", analysis_data);

    // let data = load_data::<Vec<f64>, _>("lpc")?;
    let analysis_data = analysis_load_data::<Vec<f64>, _, USE_DATA_N>("lpc", &range)?;
    println!("analysis_data: {:?}", analysis_data);

    // let data = load_data::<Vec<Vec<Option<f64>>>, _>("lpc_uncompress")?;
    // let analysis_data =
    //     analysis_load_data::<Vec<Vec<Option<f64>>>, _, USE_DATA_N>("lpc_uncompress")?;
    // println!("analysis_data: {:?}", analysis_data);

    // let data = load_data::<Vec<f64>, _>("fft")?;
    let analysis_data = analysis_load_data::<Vec<f64>, _, USE_DATA_N>("fft", &range)?;
    println!("analysis_data: {:?}", analysis_data);

    let range = (0..=20).map(|n| (n * 10000 + 100000) as f64).collect::<Vec<_>>();
    // let range = vec![0.5];
    // let data = load_data::<Vec<f64>, _>("liftered")?;
    let analysis_data = analysis_load_data::<Vec<f64>, _, USE_DATA_N>("liftered", &range)?;
    println!("analysis_data: {:?}", analysis_data);

    println!("last elapsed: {:?}", now.elapsed());

    Ok(())
}

#[allow(unused)]
fn analysis() -> anyhow::Result<()> {
    let analyzer = fn_::analyzer;

    // load AudioMNIST
    let data = analyzer.load_and_analysis::<AudioMNISTData<_>, _, _, gxhash::GxBuildHasher>(
        MNIST_BASE_PATH,
        concat!(env!("CARGO_MANIFEST_DIR"), "/tmp/"),
        set_handler_boxed,
    )?;

    // load AudioBAVED
    let data = analyzer.load_and_analysis::<AudioBAVED<_>, _, _, gxhash::GxBuildHasher>(
        BAVED_BASE_PATH,
        concat!(env!("CARGO_MANIFEST_DIR"), "/tmp/"),
        set_handler_boxed,
    )?;

    // load AudioChimeHome
    let data = analyzer.load_and_analysis::<AudioChimeHome<_>, _, _, gxhash::GxBuildHasher>(
        CHIME_HOME_BASE_PATH,
        concat!(env!("CARGO_MANIFEST_DIR"), "/tmp/"),
        set_handler_boxed,
    )?;

    Ok(())
}

fn analysis_load_data<T, S, const USE_DATA_N: usize>(
    place: S,
    range: &Vec<f64>,
) -> anyhow::Result<(
    Vec<<AudioMNISTData<T> as Analysis<T>>::Output>,
    Vec<<AudioBAVED<T> as Analysis<T>>::Output>,
    Vec<<AudioChimeHome<T> as Analysis<T>>::Output>,
)>
where
    T: ToOwned<Owned = T>
        + Send
        + Sync
        + Clone
        + Default
        + serde::Serialize
        + for<'de> serde::Deserialize<'de>
        + 'static,
    S: AsRef<str>,
    AudioMNISTData<T>: Analysis<T>,
    AudioBAVED<T>: Analysis<T>,
    AudioChimeHome<T>: Analysis<T>,
    <AudioMNISTData<T> as Analysis<T>>::Output:
        serde::Serialize + for<'de> serde::Deserialize<'de> + Send,
    <AudioBAVED<T> as Analysis<T>>::Output:
        serde::Serialize + for<'de> serde::Deserialize<'de> + Send,
    <AudioChimeHome<T> as Analysis<T>>::Output:
        serde::Serialize + for<'de> serde::Deserialize<'de> + Send,
{
    let now = std::time::Instant::now();

    let place = place.as_ref();

    println!("loading... {}", place);

    let fake_analyzer = |_: &mut audio_analyzer_core::prelude::TestData, _: u32| -> T {
        panic!("not implemented");
    };

    let place = format!("{}{place}/", concat!(env!("CARGO_MANIFEST_DIR"), "/tmp/"));

    const fn get_unique_id<U, T, F>(_: &F) -> &'static str
    where
        U: LoadAndAnalysis<T, F>,
        T: Send + Sync + Clone + 'static,
        F: Fn(&mut audio_analyzer_core::prelude::TestData, u32) -> T + Send + Sync,
    {
        <U as LoadAndAnalysis<T, F>>::UNIQUE_ID
    }

    fn saving_file_name(place: &str, unique_id: &str, use_data_n: usize) -> String {
        format!("{place}/{unique_id}_{use_data_n}.bin")
    }

    fn analysis_load_data<T, const USE_DATA_N: usize, Dataset, F, Hasher>(
        place: &str,
        range: &Vec<f64>,
        fake_analyzer: F,
    ) -> anyhow::Result<Vec<<Dataset as Analysis<T>>::Output>>
    where
        T: ToOwned<Owned = T>
            + Send
            + Sync
            + Clone
            + Default
            + serde::Serialize
            + for<'de> serde::Deserialize<'de>
            + 'static,
        Dataset: LoadAndAnalysis<T, F>
            + serde::Serialize
            + for<'de> serde::Deserialize<'de>
            + Analysis<T>
            + Sync,
        Hasher: Default + std::hash::BuildHasher + Clone + Send + Sync + 'static,
        for<'de> <Dataset as LoadAndAnalysis<T, F>>::Key:
            serde::Serialize + serde::Deserialize<'de> + 'static,
        for<'de> <Dataset as LoadAndAnalysis<T, F>>::AllPattern:
            serde::Serialize + serde::Deserialize<'de>,
        <Dataset as Analysis<T>>::Output:
            serde::Serialize + for<'de> serde::Deserialize<'de> + Send,
        F: Fn(&mut audio_analyzer_core::prelude::TestData, u32) -> T + Send + Sync,
    {
        let mnist_data = if let Ok(Ok(mnist_data)) = std::fs::File::open(saving_file_name(
            place,
            get_unique_id::<Dataset, _, _>(&fake_analyzer),
            USE_DATA_N,
        ))
        .map(|f| {
            bincode::deserialize_from(f)
                .map_err(|e| {
                    println!("failed to load save data: {:?}", e);
                })
                .map(|data| {
                    println!("loaded save data");
                    data
                })
        })
        .map_err(|e| {
            println!("failed to load save data: {:?}", e);
        }) {
            mnist_data
        } else {
            let unique_id = get_unique_id::<Dataset, _, _>(&fake_analyzer);

            let mnist_data = fake_analyzer.load_and_analysis::<Dataset, _, _, Hasher>(
                MNIST_BASE_PATH,
                place,
                set_handler_boxed,
            )?;

            println!("loaded {unique_id}");

            let progress = parking_lot::Mutex::new(pbr::ProgressBar::new(range.len() as u64));
            {
                let mut progress = progress.lock();
                progress.message("analysis");
                progress.flush().unwrap();
            }

            #[allow(unused)]
            use rayon::prelude::*;

            let mnist_analyzed_data = range
                // .par_iter()
                .iter()
                .cloned()
                .map(|threshold| {
                    let out = mnist_data.analysis::<USE_DATA_N>(threshold as f64);
                    let mut progress = progress.lock();
                    progress.inc();
                    progress.flush().unwrap();
                    println!("counted {}", progress.total);
                    out
                })
                .collect::<Vec<_>>();

            progress.lock().finish();

            bincode::serialize_into(
                std::fs::File::create(saving_file_name(place, unique_id, USE_DATA_N))?,
                &mnist_analyzed_data,
            )?;

            mnist_analyzed_data
        };

        Ok(mnist_data)
    }

    let mnist_analyzed_data =
        analysis_load_data::<T, USE_DATA_N, AudioMNISTData<T>, _, gxhash::GxBuildHasher>(
            &place,
            &range,
            &fake_analyzer,
        )?;

    let baved_analyzed_data =
        analysis_load_data::<T, USE_DATA_N, AudioBAVED<T>, _, gxhash::GxBuildHasher>(
            &place,
            &range,
            &fake_analyzer,
        )?;

    let chime_home_analyzed_data =
        analysis_load_data::<T, USE_DATA_N, AudioChimeHome<T>, _, gxhash::GxBuildHasher>(
            &place,
            &range,
            &fake_analyzer,
        )?;

    println!("elapsed: {:?}", now.elapsed());
    let now =
        chrono::Utc::now().with_timezone(chrono::FixedOffset::east_opt(9 * 3600).as_ref().unwrap());
    println!("{}", now.format("%Y-%m-%d %H:%M:%S").to_string());

    Ok((
        mnist_analyzed_data,
        baved_analyzed_data,
        chime_home_analyzed_data,
    ))
}

#[allow(unused)]
fn load_data<T, S: AsRef<str>>(
    place: S,
) -> anyhow::Result<(AudioMNISTData<T>, AudioBAVED<T>, AudioChimeHome<T>)>
where
    T: ToOwned<Owned = T>
        + Send
        + Sync
        + Clone
        + Default
        + serde::Serialize
        + for<'de> serde::Deserialize<'de>
        + 'static,
{
    let place = place.as_ref();

    println!("loading... {}", place);

    let fake_analyzer = |_: &mut audio_analyzer_core::prelude::TestData, _: u32| -> T {
        panic!("not implemented");
    };

    let place = format!("{}{place}/", concat!(env!("CARGO_MANIFEST_DIR"), "/tmp/"));

    let mnist_data = fake_analyzer
        .load_and_analysis::<AudioMNISTData<_>, _, _, gxhash::GxBuildHasher>(
            MNIST_BASE_PATH,
            &place,
            set_handler_boxed,
        )?;

    let baved_data = fake_analyzer
        .load_and_analysis::<AudioBAVED<_>, _, _, gxhash::GxBuildHasher>(
            BAVED_BASE_PATH,
            &place,
            set_handler_boxed,
        )?;

    let chime_home_data = fake_analyzer
        .load_and_analysis::<AudioChimeHome<_>, _, _, gxhash::GxBuildHasher>(
            CHIME_HOME_BASE_PATH,
            &place,
            set_handler_boxed,
        )?;

    Ok((mnist_data, baved_data, chime_home_data))
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
            ret_data.speakers[speaker_n - 1].push(data);
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
        all_pattern
            .iter()
            .map(|v| v.iter().map(|(_, v)| v.len()).sum::<usize>())
            .sum::<usize>()
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

impl<T, F> LoadAndAnalysis<T, F> for AudioChimeHome<T>
where
    T: Send + Sync + Clone + 'static + Default,
    F: Fn(&mut audio_analyzer_core::prelude::TestData, u32) -> T + Send + Sync,
{
    const UNIQUE_ID: &'static str = "AudioChimeHome";

    type Key = AudioChimeHomePattern;

    type AllPattern = Vec<Self::Key>;

    fn gen_path_from_key(key: &Self::Key, base_path: &str) -> String {
        let AudioChimeHomePattern { file_name, .. } = key;

        let path = format!("{base_path}/{file_name}.48kHz.wav");

        path
    }

    fn get_all_pattern(base_path: &str) -> anyhow::Result<Self::AllPattern> {
        use rayon::prelude::*;

        Ok(walkdir::WalkDir::new(format!("{base_path}"))
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter(|f| {
                f.path()
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .ends_with(".csv")
            })
            .collect::<Vec<_>>()
            .par_iter()
            .filter_map(|e| {
                let path = e.path();
                let file_name_with_csv = path.file_name().unwrap().to_str().unwrap();
                let file_name = &file_name_with_csv[..file_name_with_csv.len() - 4];

                // csv file
                let mut rdr = csv::Reader::from_path(path).unwrap();

                let records = rdr
                    .records()
                    .map(|r| r.unwrap())
                    .filter(|r| {
                        r.get(0)
                            .map_or_else(|| false, |v| v.starts_with("annotation"))
                    })
                    .map(|r| r.get(1).unwrap().to_string())
                    .collect::<Vec<_>>();

                if records.iter().all(|r| {
                    r.contains("f") as u8 + r.contains("m") as u8 + r.contains("c") as u8 == 1
                }) {
                    let count = records.iter().count() as u8;
                    let sum = records
                        .iter()
                        .map(|r| {
                            r.contains("f") as u8
                                + r.contains("m") as u8 * 2
                                + r.contains("c") as u8 * 3
                        })
                        .sum::<u8>();

                    let human_activity_noise = records.iter().any(|r| r.contains("p"));
                    let television_noise = records.iter().any(|r| r.contains("b"));
                    let household_appliance_noise = records.iter().any(|r| r.contains("o"));

                    let speaker_id = match sum {
                        _ if 1 * count == sum => char::from(b"f"[0]),
                        _ if 2 * count == sum => char::from(b"m"[0]),
                        _ if 3 * count == sum => char::from(b"c"[0]),
                        _ => return None,
                    };

                    return Some(AudioChimeHomePattern {
                        file_name: file_name.to_string(),
                        speaker_id,
                        human_activity_noise,
                        television_noise,
                        household_appliance_noise,
                    });
                }

                return None;
            })
            .collect::<Vec<_>>())
    }

    fn get_all_pattern_count(all_pattern: &Self::AllPattern) -> usize {
        all_pattern.len()
    }

    fn iterate<U: Send + Sync>(
        patterns: &Self::AllPattern,
        f: impl Fn(&Self::Key) -> anyhow::Result<U> + Send + Sync,
    ) -> anyhow::Result<Vec<U>> {
        use rayon::prelude::*;

        // patterns
        //     .par_iter()
        //     .map(|pattern| f(pattern))
        //     .collect::<anyhow::Result<Vec<U>>>()

        // divide 8
        patterns
            .chunks(patterns.len() / 8)
            .collect::<Vec<_>>()
            .into_par_iter()
            .map(|patterns| {
                patterns
                    .iter()
                    .map(|pattern| f(pattern))
                    .collect::<anyhow::Result<Vec<U>>>()
            })
            .collect::<anyhow::Result<Vec<Vec<U>>>>()
            .map(|v| v.into_iter().flatten().collect())
    }

    fn to_self(pattern_and_data: Vec<(Self::Key, T)>) -> Self {
        let mut ret_data = Self::default();

        for (pattern, data) in pattern_and_data.into_iter() {
            let data = data.clone();

            let AudioChimeHomePattern {
                speaker_id,
                human_activity_noise,
                television_noise,
                household_appliance_noise,
                ..
            } = pattern;

            let speaker = match speaker_id {
                'f' => &mut ret_data.father,
                'm' => &mut ret_data.mother,
                'c' => &mut ret_data.child,
                _ => unreachable!("Invalid speaker id: {}", speaker_id),
            };

            match (
                human_activity_noise,
                television_noise,
                household_appliance_noise,
            ) {
                (true, true, true) => speaker.all.push(data),
                (true, true, false) => speaker.human_activity_and_television.push(data),
                (true, false, true) => speaker.human_activity_and_household_appliance.push(data),
                (true, false, false) => speaker.human_activity.push(data),
                (false, true, true) => speaker.television_and_household_appliance.push(data),
                (false, true, false) => speaker.television.push(data),
                (false, false, true) => speaker.household_appliance.push(data),
                (false, false, false) => speaker.none.push(data),
            }
        }

        ret_data
    }
}
