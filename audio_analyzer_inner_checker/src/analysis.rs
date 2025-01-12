use crate::libs::load_dataset::{
    AudioBAVED, AudioBAVEDEmotion, AudioChimeHome, AudioChimeHomeNoise, AudioMNISTData,
};
use const_struct::primitive::F64Ty;
use rand::Rng;
use rayon::prelude::*;

type TimeSeriesType = Vec<Vec<Option<f64>>>;
type TimeCompressedType = Vec<f64>;

pub fn analysis_time_series_audio_mnist<THRESHOLD: F64Ty>(
    data: AudioMNISTData<TimeSeriesType>,
) -> (f64, f64) {
    let AudioMNISTData { speakers } = data;

    analysis_time_series_inner::<THRESHOLD>(&speakers)
}

pub fn analysis_time_series_baved<THRESHOLD: F64Ty>(
    data: AudioBAVED<TimeSeriesType>,
) -> ([f64; 3], f64, [[(f64, f64); 3]; 3]) {
    let AudioBAVED { speakers } = data;

    let level_with_other_and_other_and_self_level_wrapper = speakers
        .par_iter()
        .enumerate()
        .map(|(i, speaker)| {
            // How many data from the training source will be employed
            let use_data_n = 10;

            fn gen_speakers_all(
                speaker: &AudioBAVEDEmotion<TimeSeriesType>,
            ) -> Vec<Vec<TimeSeriesType>> {
                vec![
                    speaker.level_0.clone(),
                    speaker.level_1.clone(),
                    speaker.level_2.clone(),
                ]
            }

            let level_with_other = gen_speakers_all(speaker)
                .iter()
                .map(|speaker| {
                    // Identify data
                    let mut rng = rand::thread_rng();
                    let mut unused_n = (0..speaker.len()).collect::<Vec<_>>();
                    let ident = (0..use_data_n)
                        .map(|_| {
                            let n = rng.gen_range(0..unused_n.len());
                            let n = unused_n.remove(n);

                            let ident = speaker[n].inner_average();

                            ident
                        })
                        .collect::<Vec<_>>()
                        .inner_average();

                    // other speaker's identify data
                    let other_data = speakers
                        .iter()
                        .enumerate()
                        .filter(|(j, _)| i != *j)
                        .flat_map(|(_, speaker)| {
                            gen_speakers_all(speaker)
                                .into_iter()
                                .flat_map(|v| v)
                                .map(|v| v.get_all_some_vec())
                                .collect::<Vec<_>>()
                        })
                        .flatten()
                        .collect::<Vec<_>>();

                    let other_data_len_sum = other_data.custom_get_length();
                    let other_matching_probability = other_data
                        .iter()
                        .map(|v| compare::<THRESHOLD>(&ident, &v) as u8 as u64)
                        .sum::<u64>();

                    (other_matching_probability, other_data_len_sum)
                })
                .collect::<Vec<_>>();

            let level_with_other: [(u64, u64); 3] = level_with_other.try_into().unwrap();

            let other = {
                let other_matching_probability = level_with_other
                    .iter()
                    .map(|(matching_probability, _)| matching_probability)
                    .sum::<u64>();
                let other_data_len_sum = level_with_other
                    .iter()
                    .map(|(_, data_len_sum)| data_len_sum)
                    .sum::<u64>();

                (other_matching_probability, other_data_len_sum)
            };

            let mut self_level_wrapper = LevelWrapper::<(_, _)>::default();

            let speakers = vec![
                speaker.level_0.clone(),
                speaker.level_1.clone(),
                speaker.level_2.clone(),
            ];

            for (level, speaker) in speakers.iter().enumerate() {
                let mut rng = rand::thread_rng();
                let mut unused_n = (0..speaker.len()).collect::<Vec<_>>();
                let ident = (0..use_data_n)
                    .map(|_| {
                        let n = rng.gen_range(0..speaker.len());
                        let n = unused_n.remove(n);
                        let ident = speaker[n].inner_average();

                        ident
                    })
                    .collect::<Vec<_>>()
                    .inner_average();

                for (j, speaker) in speakers.iter().enumerate() {
                    let other_data = speaker;

                    let other_data_len_sum = other_data.custom_get_length();
                    let other_matching_probability = other_data
                        .iter()
                        .flat_map(|audio_data| {
                            audio_data
                                .get_all_some_vec()
                                .into_iter()
                                .map(|v| compare::<THRESHOLD>(&ident, &v) as u8 as u64)
                        })
                        .sum::<u64>();

                    self_level_wrapper.0[level][j] =
                        (other_matching_probability, other_data_len_sum);
                }
            }

            (level_with_other, other, self_level_wrapper)
        })
        .collect::<Vec<_>>();

    analysis_baved_post_processing(level_with_other_and_other_and_self_level_wrapper)
}

pub(super) fn analysis_time_series_inner<THRESHOLD: F64Ty>(
    speakers: &[Vec<TimeSeriesType>],
) -> (f64, f64) {
    let (
        (self_matching_probability, self_data_len_sum),
        (other_matching_probability, other_data_len_sum),
    ) = speakers
        .par_iter()
        .enumerate()
        .map(|(i, speaker)| {
            // How many data from the training source will be employed
            let use_data_n = 10;

            // Identify data
            let mut rng = rand::thread_rng();
            let mut unused_n = (0..speaker.len()).collect::<Vec<_>>();
            let ident = (0..use_data_n)
                .map(|_| {
                    let n = rng.gen_range(0..unused_n.len());
                    let n = unused_n.remove(n);

                    let ident = speaker[n].inner_average();

                    ident
                })
                .collect::<Vec<_>>()
                .inner_average();

            // other speaker's identify data
            let other_data = speakers
                .iter()
                .enumerate()
                .filter(|(j, _)| i != *j)
                .map(|(_, speaker)| speaker)
                .collect::<Vec<_>>();

            let other_data_len_sum = other_data.custom_get_length();
            let other_matching_probability = other_data
                .iter()
                .flat_map(|speaker| {
                    speaker.iter().flat_map(|audio_data| {
                        audio_data
                            .get_all_some_vec()
                            .into_iter()
                            .map(|v| compare::<THRESHOLD>(&ident, &v) as u8 as u64)
                    })
                })
                .sum::<u64>();

            let self_data = &speakers[i];

            let self_data_len_sum = self_data.custom_get_length();

            let self_matching_probability = self_data
                .iter()
                .flat_map(|audio_data| {
                    audio_data
                        .get_all_some_vec()
                        .into_iter()
                        .map(|v| compare::<THRESHOLD>(&ident, &v) as u8 as u64)
                })
                .sum::<u64>();

            (
                (self_matching_probability, self_data_len_sum),
                (other_matching_probability, other_data_len_sum),
            )
        })
        .collect::<((Vec<_>, Vec<_>), (Vec<_>, Vec<_>))>();

    let self_matching_probability = self_matching_probability.iter().sum::<u64>();
    let self_data_len_sum = self_data_len_sum.iter().sum::<u64>();

    let other_matching_probability = other_matching_probability.iter().sum::<u64>();
    let other_data_len_sum = other_data_len_sum.iter().sum::<u64>();

    (
        self_matching_probability as f64 / self_data_len_sum as f64,
        other_matching_probability as f64 / other_data_len_sum as f64,
    )
}

pub fn analysis_time_series_chime_home<THRESHOLD: F64Ty>(
    data: AudioChimeHome<TimeSeriesType>,
) -> ([f64; 8], f64, [[(f64, f64); 8]; 8]) {
    let AudioChimeHome {
        father,
        mother,
        child,
    } = data;

    let speakers = [father, mother, child];

    #[derive(Default)]
    struct LevelWrapper<T>(pub [[T; 8]; 8]);

    let noise_with_other_and_other_and_self_level_wrapper = speakers
        .par_iter()
        .enumerate()
        .map(|(i, speaker)| {
            // How many data from the training source will be employed
            let use_data_n = 10;

            fn gen_speakers_all(
                speaker: &AudioChimeHomeNoise<TimeSeriesType>,
            ) -> Vec<Vec<TimeSeriesType>> {
                vec![
                    speaker.none.clone(),
                    speaker.human_activity.clone(),
                    speaker.television.clone(),
                    speaker.household_appliance.clone(),
                    speaker.human_activity_and_television.clone(),
                    speaker.human_activity_and_household_appliance.clone(),
                    speaker.television_and_household_appliance.clone(),
                    speaker.all.clone(),
                ]
            }
            let speakers_all = gen_speakers_all(speaker);

            let noise_with_other = speakers_all
                .iter()
                .map(|speaker| {
                    // Identify data
                    let mut rng = rand::thread_rng();
                    let mut unused_n = (0..speaker.len()).collect::<Vec<_>>();
                    let ident = (0..use_data_n)
                        .map(|_| {
                            let n = rng.gen_range(0..unused_n.len());
                            let n = unused_n.remove(n);

                            let ident = speaker[n].inner_average();

                            ident
                        })
                        .collect::<Vec<_>>()
                        .inner_average();

                    // other speaker's identify data
                    let other_data = speakers
                        .iter()
                        .enumerate()
                        .filter(|(j, _)| i != *j)
                        .flat_map(|(_, speaker)| {
                            gen_speakers_all(speaker)
                                .into_iter()
                                .flat_map(|v| v)
                                .collect::<Vec<_>>()
                        })
                        .flatten()
                        .collect::<Vec<_>>();

                    let other_data_len_sum = other_data.custom_get_length();
                    let other_matching_probability = other_data
                        .get_all_some_vec()
                        .iter()
                        .map(|v| compare::<THRESHOLD>(&ident, &v) as u8 as u64)
                        .sum::<u64>();

                    (other_matching_probability, other_data_len_sum)
                })
                .collect::<Vec<_>>();

            let noise_with_other: [(u64, u64); 8] = noise_with_other.try_into().unwrap();

            let other = {
                let other_matching_probability = noise_with_other
                    .iter()
                    .map(|(matching_probability, _)| matching_probability)
                    .sum::<u64>();
                let other_data_len_sum = noise_with_other
                    .iter()
                    .map(|(_, data_len_sum)| data_len_sum)
                    .sum::<u64>();

                (other_matching_probability, other_data_len_sum)
            };

            let mut self_level_wrapper = LevelWrapper::<(_, _)>::default();

            for (type_, speaker) in speakers_all.iter().enumerate() {
                let mut rng = rand::thread_rng();
                let mut unused_n = (0..speaker.len()).collect::<Vec<_>>();
                let ident = (0..use_data_n)
                    .map(|_| {
                        let n = rng.gen_range(0..speaker.len());
                        let n = unused_n.remove(n);
                        let ident = speaker[n].inner_average();

                        ident
                    })
                    .collect::<Vec<_>>()
                    .inner_average();

                for (j, speaker) in speakers_all.iter().enumerate() {
                    let other_data = speaker;

                    let other_data_len_sum = other_data.custom_get_length();
                    let other_matching_probability = other_data
                        .iter()
                        .flat_map(|audio_data| {
                            audio_data
                                .get_all_some_vec()
                                .into_iter()
                                .map(|v| compare::<THRESHOLD>(&ident, &v) as u8 as u64)
                        })
                        .sum::<u64>();

                    self_level_wrapper.0[type_][j] =
                        (other_matching_probability, other_data_len_sum);
                }
            }

            (noise_with_other, other, self_level_wrapper)
        })
        .collect::<Vec<_>>();

    let noise_with_other = noise_with_other_and_other_and_self_level_wrapper
        .iter()
        .map(|(noise_with_other, _, _)| noise_with_other)
        .collect::<Vec<_>>();

    let other = noise_with_other_and_other_and_self_level_wrapper
        .iter()
        .map(|(_, other, _)| other)
        .collect::<Vec<_>>();

    let self_level_wrapper = noise_with_other_and_other_and_self_level_wrapper
        .iter()
        .map(|(_, _, self_level_wrapper)| self_level_wrapper)
        .collect::<Vec<_>>();

    let noise_with_other_matching_probability: [_; 8] = noise_with_other
        .iter()
        .map(|noise_with_other| {
            let noise_with_other_matching_probability = noise_with_other
                .iter()
                .map(|(matching_probability, _)| matching_probability)
                .sum::<u64>();
            let noise_with_other_data_len_sum = noise_with_other
                .iter()
                .map(|(_, data_len_sum)| data_len_sum)
                .sum::<u64>();

            noise_with_other_matching_probability as f64 / noise_with_other_data_len_sum as f64
        })
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();

    let other_matching_probability = other
        .iter()
        .map(|(matching_probability, _)| matching_probability)
        .sum::<u64>();
    let other_data_len_sum = other
        .iter()
        .map(|(_, data_len_sum)| data_len_sum)
        .sum::<u64>();

    let other_matching_probability = other_matching_probability as f64 / other_data_len_sum as f64;

    let self_level_wrapper: [[_; 8]; 8] = self_level_wrapper
        .iter()
        .map(|self_level_wrapper| {
            let self_level_matching_probability: [(f64, f64); 8] = self_level_wrapper
                .0
                .iter()
                .map(|v| {
                    let self_matching_probability = v
                        .iter()
                        .map(|(matching_probability, _)| matching_probability)
                        .sum::<u64>();
                    let self_data_len_sum =
                        v.iter().map(|(_, data_len_sum)| data_len_sum).sum::<u64>();

                    (
                        self_matching_probability as f64 / self_data_len_sum as f64,
                        self_data_len_sum as f64,
                    )
                })
                .collect::<Vec<_>>()
                .try_into()
                .unwrap();

            self_level_matching_probability
        })
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();

    (
        noise_with_other_matching_probability,
        other_matching_probability,
        self_level_wrapper,
    )
}

#[derive(Default)]
struct LevelWrapper<T>(pub [[T; 3]; 3]);

fn analysis_baved_post_processing(
    level_with_other_and_other_and_self_level_wrapper: Vec<(
        [(u64, u64); 3],
        (u64, u64),
        LevelWrapper<(u64, u64)>,
    )>,
) -> ([f64; 3], f64, [[(f64, f64); 3]; 3]) {
    let level_with_other = level_with_other_and_other_and_self_level_wrapper
        .iter()
        .map(|(level_with_other, _, _)| level_with_other)
        .collect::<Vec<_>>();

    let other = level_with_other_and_other_and_self_level_wrapper
        .iter()
        .map(|(_, other, _)| other)
        .collect::<Vec<_>>();

    let self_level_wrapper = level_with_other_and_other_and_self_level_wrapper
        .iter()
        .map(|(_, _, self_level_wrapper)| self_level_wrapper)
        .collect::<Vec<_>>();

    let level_with_other_matching_probability: [_; 3] = level_with_other
        .iter()
        .map(|level_with_other| {
            let level_with_other_matching_probability = level_with_other
                .iter()
                .map(|(matching_probability, _)| matching_probability)
                .sum::<u64>();
            let level_with_other_data_len_sum = level_with_other
                .iter()
                .map(|(_, data_len_sum)| data_len_sum)
                .sum::<u64>();

            level_with_other_matching_probability as f64 / level_with_other_data_len_sum as f64
        })
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();

    let other_matching_probability = other
        .iter()
        .map(|(matching_probability, _)| matching_probability)
        .sum::<u64>();
    let other_data_len_sum = other
        .iter()
        .map(|(_, data_len_sum)| data_len_sum)
        .sum::<u64>();

    let other_matching_probability = other_matching_probability as f64 / other_data_len_sum as f64;

    let self_level_wrapper: [[_; 3]; 3] = self_level_wrapper
        .iter()
        .map(|self_level_wrapper| {
            let self_level_matching_probability: [(f64, f64); 3] = self_level_wrapper
                .0
                .iter()
                .map(|v| {
                    let self_matching_probability = v
                        .iter()
                        .map(|(matching_probability, _)| matching_probability)
                        .sum::<u64>();
                    let self_data_len_sum =
                        v.iter().map(|(_, data_len_sum)| data_len_sum).sum::<u64>();

                    (
                        self_matching_probability as f64 / self_data_len_sum as f64,
                        self_data_len_sum as f64,
                    )
                })
                .collect::<Vec<_>>()
                .try_into()
                .unwrap();

            self_level_matching_probability
        })
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();

    (
        level_with_other_matching_probability,
        other_matching_probability,
        self_level_wrapper,
    )
}

fn common_pre_processing<'b, const N: usize, T, Dataset, const UseDataN: usize, THRESHOLD>(
    speakers: &[Dataset],
) -> Vec<([(u64, u64); N], (u64, u64), [[(u64, u64); N]; N])>
where
    T: InnerAverage + std::iter::Iterator + Clone,
    Dataset: GenSpeakersAll<T> + 'b,
    Vec<Vec<T>>: CustomFlat<Vec<Vec<Vec<f64>>>>,
    Vec<T>: CustomFlat<Vec<Vec<f64>>> + CustomGetLength,
    THRESHOLD: F64Ty,
    for<'a> &'a [Dataset]: IntoParallelIterator<Item = Dataset>,
    for<'a> <&'a [Dataset] as IntoParallelIterator>::Iter: Iterator<Item = &'b Dataset>,
{
    let noise_with_other_and_other_and_self_level_wrapper = speakers
        .par_iter()
        .enumerate()
        .map(|(i, speaker)| {
            let speakers_all = speaker.gen_speakers_all();

            let noise_with_other = speakers_all
                .iter()
                .map(|speaker| {
                    // Identify data
                    let mut rng = rand::thread_rng();
                    let mut unused_n = (0..speaker.len()).collect::<Vec<_>>();
                    let ident = (0..UseDataN)
                        .map(|_| {
                            let n = rng.gen_range(0..unused_n.len());
                            let n = unused_n.remove(n);

                            let ident = speaker[n].inner_average();

                            ident
                        })
                        .collect::<Vec<_>>()
                        .inner_average();

                    // other speaker's identify data
                    let other_data = speakers
                        .iter()
                        .enumerate()
                        .filter(|(j, _)| i != *j)
                        .flat_map(|(_, speaker)| speaker.gen_speakers_all().custom_flat())
                        .flatten()
                        .collect::<Vec<_>>();

                    let other_data_len_sum = other_data.custom_get_length();
                    let other_matching_probability = other_data
                        .iter()
                        .map(|v| compare::<THRESHOLD>(&ident, &v) as u8 as u64)
                        .sum::<u64>();

                    (other_matching_probability, other_data_len_sum)
                })
                .collect::<Vec<_>>();

            let noise_with_other: [(u64, u64); N] = noise_with_other.try_into().unwrap();

            let other = {
                let other_matching_probability = noise_with_other
                    .iter()
                    .map(|(matching_probability, _)| matching_probability)
                    .sum::<u64>();
                let other_data_len_sum = noise_with_other
                    .iter()
                    .map(|(_, data_len_sum)| data_len_sum)
                    .sum::<u64>();

                (other_matching_probability, other_data_len_sum)
            };

            let mut self_level_wrapper: [[(u64, u64); N]; N] = [[(0, 0); N]; N];

            for (type_, speaker) in speakers_all.iter().enumerate() {
                let mut rng = rand::thread_rng();
                let mut unused_n = (0..speaker.len()).collect::<Vec<_>>();
                let ident = (0..UseDataN)
                    .map(|_| {
                        let n = rng.gen_range(0..speaker.len());
                        let n = unused_n.remove(n);
                        let ident = speaker[n].inner_average();

                        ident
                    })
                    .collect::<Vec<_>>()
                    .inner_average();

                for (j, speaker) in speakers_all.iter().enumerate() {
                    let other_data = speaker;

                    let other_data_len_sum = other_data.custom_get_length();
                    let other_matching_probability =
                        CustomFlat::<Vec<Vec<f64>>>::custom_flat(other_data.clone())
                        .iter()
                        .map(|v| compare::<THRESHOLD>(&ident, &v) as u8 as u64)
                        .sum::<u64>();

                    self_level_wrapper[type_][j] =
                        (other_matching_probability, other_data_len_sum);
                }
            }

            (noise_with_other, other, self_level_wrapper)
        })
        .collect::<Vec<_>>();

    noise_with_other_and_other_and_self_level_wrapper
}

pub fn analysis_baved<THRESHOLD: F64Ty>(
    data: AudioBAVED<TimeCompressedType>,
) -> ([f64; 3], f64, [[(f64, f64); 3]; 3]) {
    let AudioBAVED { speakers } = data;

    let level_with_other_and_other_and_self_level_wrapper = speakers
        .par_iter()
        .enumerate()
        .map(|(i, speaker)| {
            // How many data from the training source will be employed
            let use_data_n = 10;

            fn gen_speakers_all(
                speaker: &AudioBAVEDEmotion<TimeCompressedType>,
            ) -> Vec<Vec<TimeCompressedType>> {
                vec![
                    speaker.level_0.clone(),
                    speaker.level_1.clone(),
                    speaker.level_2.clone(),
                ]
            }

            let level_with_other = gen_speakers_all(speaker)
                .iter()
                .map(|speaker| {
                    // Identify data
                    let mut rng = rand::thread_rng();
                    let mut unused_n = (0..speaker.len()).collect::<Vec<_>>();
                    let ident = (0..use_data_n)
                        .map(|_| {
                            let n = rng.gen_range(0..unused_n.len());
                            let n = unused_n.remove(n);

                            speaker[n].clone()
                        })
                        .collect::<Vec<_>>()
                        .inner_average();

                    // other speaker's identify data
                    let other_data = speakers
                        .iter()
                        .enumerate()
                        .filter(|(j, _)| i != *j)
                        .flat_map(|(_, speaker)| gen_speakers_all(speaker))
                        .flatten()
                        .collect::<Vec<_>>();

                    let other_data_len_sum = other_data.custom_get_length();
                    let other_matching_probability = other_data
                        .iter()
                        .map(|v| compare::<THRESHOLD>(&ident, &v) as u8 as u64)
                        .sum::<u64>();

                    (other_matching_probability, other_data_len_sum)
                })
                .collect::<Vec<_>>();

            let level_with_other: [(u64, u64); 3] = level_with_other.try_into().unwrap();

            let other = {
                let other_matching_probability = level_with_other
                    .iter()
                    .map(|(matching_probability, _)| matching_probability)
                    .sum::<u64>();
                let other_data_len_sum = level_with_other
                    .iter()
                    .map(|(_, data_len_sum)| data_len_sum)
                    .sum::<u64>();

                (other_matching_probability, other_data_len_sum)
            };

            let mut self_level_wrapper = LevelWrapper::<(_, _)>::default();

            let speakers = vec![
                speaker.level_0.clone(),
                speaker.level_1.clone(),
                speaker.level_2.clone(),
            ];

            for (level, speaker) in speakers.iter().enumerate() {
                let mut rng = rand::thread_rng();
                let mut unused_n = (0..speaker.len()).collect::<Vec<_>>();
                let ident = (0..use_data_n)
                    .map(|_| {
                        let n = rng.gen_range(0..speaker.len());
                        let n = unused_n.remove(n);
                        let ident = speaker[n].clone();

                        ident
                    })
                    .collect::<Vec<_>>()
                    .inner_average();

                for (j, speaker) in speakers.iter().enumerate() {
                    let other_data = speaker;

                    let other_data_len_sum = other_data.custom_get_length();
                    let other_matching_probability = other_data
                        .iter()
                        .map(|v| compare::<THRESHOLD>(&ident, &v) as u8 as u64)
                        .sum::<u64>();

                    self_level_wrapper.0[level][j] =
                        (other_matching_probability, other_data_len_sum);
                }
            }

            (level_with_other, other, self_level_wrapper)
        })
        .collect::<Vec<_>>();

    analysis_baved_post_processing(level_with_other_and_other_and_self_level_wrapper)
}

trait CustomGetLength {
    fn custom_get_length(&self) -> u64;
}

impl CustomGetLength for Vec<Vec<Option<f64>>> {
    fn custom_get_length(&self) -> u64 {
        self.iter()
            .filter(|v| v.iter().all(|v| v.is_some()))
            .count() as u64
    }
}

impl CustomGetLength for Vec<Vec<f64>> {
    fn custom_get_length(&self) -> u64 {
        self.len() as u64
    }
}

impl<T: CustomGetLength> CustomGetLength for Vec<T> {
    fn custom_get_length(&self) -> u64 {
        self.iter().map(|v| v.custom_get_length()).sum()
    }
}

impl<T: CustomGetLength> CustomGetLength for &T {
    fn custom_get_length(&self) -> u64 {
        T::custom_get_length(*self)
    }
}

trait GetAllSomeVec {
    fn get_all_some_vec(&self) -> Vec<Vec<f64>>;
}

impl GetAllSomeVec for Vec<Vec<Option<f64>>> {
    fn get_all_some_vec(&self) -> Vec<Vec<f64>> {
        self.iter()
            .filter_map(|v| {
                if v.iter().all(|v| v.is_some()) {
                    Some(v.iter().map(|v| v.unwrap()).collect::<Vec<_>>())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
    }
}

impl GetAllSomeVec for Vec<&Vec<Option<f64>>> {
    fn get_all_some_vec(&self) -> Vec<Vec<f64>> {
        self.iter()
            .filter_map(|v| {
                if v.iter().all(|v| v.is_some()) {
                    Some(v.iter().map(|v| v.unwrap()).collect::<Vec<_>>())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
    }
}

trait InnerAverage {
    fn inner_average(&self) -> Vec<f64>;
}

impl InnerAverage for Vec<f64> {
    fn inner_average(&self) -> Vec<f64> {
        self.clone()
    }
}

impl InnerAverage for Vec<Vec<f64>> {
    fn inner_average(&self) -> Vec<f64> {
        let len = self.iter().map(|v| v.len()).min().unwrap();
        (0..len)
            .into_iter()
            .map(|i| self.iter().map(|v| v[i]).sum::<f64>() / self.len() as f64)
            .collect::<Vec<_>>()
    }
}

impl InnerAverage for Vec<Vec<Option<f64>>> {
    fn inner_average(&self) -> Vec<f64> {
        self.get_all_some_vec().inner_average()
    }
}

pub trait GenSpeakersAll<T> {
    fn gen_speakers_all(&self) -> Vec<Vec<T>>;
}

impl<T> GenSpeakersAll<T> for AudioBAVEDEmotion<T>
where
    T: Clone,
{
    fn gen_speakers_all(&self) -> Vec<Vec<T>> {
        vec![
            self.level_0.clone(),
            self.level_1.clone(),
            self.level_2.clone(),
        ]
    }
}

pub trait CustomFlat<T> {
    fn custom_flat(self) -> T;
}

impl CustomFlat<Vec<Vec<Vec<f64>>>> for Vec<Vec<Vec<Vec<Option<f64>>>>> {
    fn custom_flat(self) -> Vec<Vec<Vec<f64>>> {
        self.into_iter()
            .flat_map(|v| v)
            .map(|v| v.get_all_some_vec())
            .collect::<Vec<_>>()
    }
}

impl CustomFlat<Vec<Vec<f64>>> for Vec<Vec<Vec<Option<f64>>>> {
    fn custom_flat(self) -> Vec<Vec<f64>> {
        self.into_iter()
            .flat_map(|v| v.get_all_some_vec())
            .collect::<Vec<_>>()
    }
}

impl<T> CustomFlat<T> for T {
    fn custom_flat(self) -> T {
        self
    }
}

impl<T> GenSpeakersAll<T> for AudioChimeHomeNoise<T>
where
    T: Clone,
{
    fn gen_speakers_all(&self) -> Vec<Vec<T>> {
        vec![
            self.none.clone(),
            self.human_activity.clone(),
            self.television.clone(),
            self.household_appliance.clone(),
            self.human_activity_and_television.clone(),
            self.human_activity_and_household_appliance.clone(),
            self.television_and_household_appliance.clone(),
            self.all.clone(),
        ]
    }
}

fn compare<THRESHOLD: F64Ty>(a: &Vec<f64>, b: &Vec<f64>) -> bool {
    fn get_distance(a: &Vec<f64>, b: &Vec<f64>) -> f64 {
        a.iter()
            .zip(b.iter())
            .map(|(a, b)| (a - b).abs())
            .sum::<f64>()
    }

    let distance = get_distance(a, b);

    distance < THRESHOLD::VALUE
}
