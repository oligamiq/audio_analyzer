use crate::libs::load_dataset::{
    AudioBAVED, AudioBAVEDEmotion, AudioChimeHome, AudioChimeHomeNoise, AudioMNISTData,
};
use rand::Rng;
use rayon::prelude::*;

type TimeSeriesType = Vec<Vec<Option<f64>>>;
type TimeCompressedType = Vec<f64>;

pub trait Analysis<T> {
    type Output;

    fn analysis<const USE_DATA_N: usize>(self, threshold: f64) -> Self::Output;
}

impl Analysis<TimeSeriesType> for AudioMNISTData<TimeSeriesType> {
    type Output = (f64, f64);

    fn analysis<const USE_DATA_N: usize>(self, threshold: f64) -> (f64, f64) {
        analysis_time_series_audio_mnist::<USE_DATA_N>(self, threshold)
    }
}

impl Analysis<TimeSeriesType> for AudioBAVED<TimeSeriesType> {
    type Output = ([f64; 3], f64, [[f64; 3]; 3]);

    fn analysis<const USE_DATA_N: usize>(
        self,
        threshold: f64,
    ) -> ([f64; 3], f64, [[f64; 3]; 3]) {
        analysis_time_series_baved::<USE_DATA_N>(self, threshold)
    }
}

impl Analysis<TimeSeriesType> for AudioChimeHome<TimeSeriesType> {
    type Output = ([f64; 8], f64, [[f64; 8]; 8]);

    fn analysis<const USE_DATA_N: usize>(
        self,
        threshold: f64,
    ) -> ([f64; 8], f64, [[f64; 8]; 8]) {
        analysis_time_series_chime_home::<USE_DATA_N>(self, threshold)
    }
}

impl Analysis<TimeCompressedType> for AudioMNISTData<TimeCompressedType> {
    type Output = (f64, f64);

    fn analysis<const USE_DATA_N: usize>(self, threshold: f64) -> (f64, f64) {
        analysis_audio_mnist::<USE_DATA_N>(self, threshold)
    }
}

impl Analysis<TimeCompressedType> for AudioBAVED<TimeCompressedType> {
    type Output = ([f64; 3], f64, [[f64; 3]; 3]);

    fn analysis<const USE_DATA_N: usize>(
        self,
        threshold: f64,
    ) -> ([f64; 3], f64, [[f64; 3]; 3]) {
        analysis_baved::<USE_DATA_N>(self, threshold)
    }
}

impl Analysis<TimeCompressedType> for AudioChimeHome<TimeCompressedType> {
    type Output = ([f64; 8], f64, [[f64; 8]; 8]);

    fn analysis<const USE_DATA_N: usize>(
        self,
        threshold: f64,
    ) -> ([f64; 8], f64, [[f64; 8]; 8]) {
        analysis_chime_home::<USE_DATA_N>(self, threshold)
    }
}

pub fn analysis_time_series_audio_mnist<const USE_DATA_N: usize>(
    data: AudioMNISTData<TimeSeriesType>,
    threshold: f64,
) -> (f64, f64) {
    let AudioMNISTData { speakers } = data;

    analysis_time_series_inner::<USE_DATA_N>(&speakers, threshold)
}

pub fn analysis_time_series_baved<const USE_DATA_N: usize>(
    data: AudioBAVED<TimeSeriesType>,
    threshold: f64,
) -> ([f64; 3], f64, [[f64; 3]; 3]) {
    let AudioBAVED { speakers } = data;

    let level_with_other_and_other_and_self_level_wrapper = common_pre_processing::<
        3,
        TimeSeriesType,
        AudioBAVEDEmotion<TimeSeriesType>,
        USE_DATA_N,
    >(&speakers, threshold);

    common_post_processing(level_with_other_and_other_and_self_level_wrapper)
}

pub(super) fn analysis_time_series_inner<const USE_DATA_N: usize>(
    speakers: &[Vec<TimeSeriesType>],
    threshold: f64,
) -> (f64, f64) {
    let (
        (self_matching_probability, self_data_len_sum),
        (other_matching_probability, other_data_len_sum),
    ) = speakers
        .par_iter()
        .enumerate()
        .filter_map(|(i, speaker)| {
            // How many data from the training source will be employed
            let (other_matching_probability, other_data_len_sum, ident) =
                common_pre_processing_inner_other::<TimeSeriesType, _, USE_DATA_N>(
                    i, speakers, speaker, threshold,
                )?;

            let self_data = &speakers[i];

            let self_data_len_sum = self_data.custom_get_length();

            let self_matching_probability = self_data
                .iter()
                .flat_map(|audio_data| {
                    audio_data
                        .get_all_some_vec()
                        .into_iter()
                        .map(|v| compare(&ident, &v, threshold) as u8 as u64)
                })
                .sum::<u64>();

            Some((
                (self_matching_probability, self_data_len_sum),
                (other_matching_probability, other_data_len_sum),
            ))
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

pub fn analysis_time_series_chime_home<const USE_DATA_N: usize>(
    data: AudioChimeHome<TimeSeriesType>,
    threshold: f64,
) -> ([f64; 8], f64, [[f64; 8]; 8]) {
    let AudioChimeHome {
        father,
        mother,
        child,
    } = data;

    let speakers = [father, mother, child];

    let noise_with_other_and_other_and_self_level_wrapper = common_pre_processing::<
        8,
        TimeSeriesType,
        AudioChimeHomeNoise<TimeSeriesType>,
        USE_DATA_N,
    >(&speakers, threshold);

    common_post_processing(noise_with_other_and_other_and_self_level_wrapper)
}

fn common_post_processing<const N: usize>(
    level_with_other_and_other_and_self_level_wrapper: Vec<(
        [(u64, u64); N],
        (u64, u64),
        [[(u64, u64); N]; N],
    )>,
) -> ([f64; N], f64, [[f64; N]; N]) {
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

    assert!(level_with_other.iter().all(|level| level.len() == N));
    let level_with_other_matching_probability = (0..N)
        .into_iter()
        .map(|i| {
            let level_with_other_matching_probability = level_with_other
                .iter()
                .map(|level_with_other| level_with_other[i].0)
                .sum::<u64>();
            let level_with_other_data_len_sum = level_with_other
                .iter()
                .map(|level_with_other| level_with_other[i].1)
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

    assert!(self_level_wrapper.iter().all(|level| level.len() == N));
    assert!(self_level_wrapper
        .iter()
        .all(|level| level.iter().all(|level| level.len() == N)));
    let self_level_wrapper: [[_; N]; N] = (0..N)
        .into_iter()
        .map(|i| {
            (0..N)
                .into_iter()
                .map(|j| {
                    let other_matching_probability = self_level_wrapper
                        .iter()
                        .map(|level| level[i][j].0)
                        .sum::<u64>();
                    let other_data_len_sum = self_level_wrapper
                        .iter()
                        .map(|level| level[i][j].1)
                        .sum::<u64>();

                    other_matching_probability as f64 / other_data_len_sum as f64
                })
                .collect::<Vec<_>>()
                .try_into()
                .unwrap()
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

fn common_pre_processing_inner_other<T, Dataset, const USE_DATA_N: usize>(
    i: usize,
    speakers: &[Dataset],
    speaker: &Vec<T>,
    threshold: f64,
) -> Option<(u64, u64, Vec<f64>)>
where
    T: InnerAverage + Clone,
    Dataset: GenSpeakersAll<T> + Sync,
    Vec<Vec<T>>: CustomFlat<Vec<Vec<Vec<f64>>>>,
    Vec<T>: CustomFlat<Vec<Vec<f64>>> + CustomGetLength,
{
    // Identify data
    let mut rng = rand::thread_rng();
    let mut unused_n = (0..speaker.len()).collect::<Vec<_>>();
    let ident = (0..USE_DATA_N)
        .map(|_| {
            if unused_n.is_empty() {
                return None;
            }

            let n = rng.gen_range(0..unused_n.len());
            let n = unused_n.remove(n);

            let ident = speaker[n].inner_average();

            Some(ident)
        })
        .collect::<Option<Vec<_>>>()?
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
        .map(|v| compare(&ident, &v, threshold) as u8 as u64)
        .sum::<u64>();

    Some((other_matching_probability, other_data_len_sum, ident))
}

fn common_pre_processing<'a, 'b, const N: usize, T, Dataset, const USE_DATA_N: usize>(
    speakers: &'a [Dataset],
    threshold: f64,
) -> Vec<([(u64, u64); N], (u64, u64), [[(u64, u64); N]; N])>
where
    T: InnerAverage + Clone,
    Dataset: GenSpeakersAll<T> + 'b + Sync,
    Vec<Vec<T>>: CustomFlat<Vec<Vec<Vec<f64>>>>,
    Vec<T>: CustomFlat<Vec<Vec<f64>>> + CustomGetLength,
    &'a [Dataset]: IntoParallelIterator<Item = &'b Dataset>,
    <&'a [Dataset] as rayon::iter::IntoParallelIterator>::Iter:
        ParallelIterator<Item = &'b Dataset> + IndexedParallelIterator,
{
    let noise_with_other_and_other_and_self_level_wrapper = speakers
        .par_iter()
        .enumerate()
        .map(|(i, speaker): (usize, &Dataset)| {
            let speakers_all = speaker.gen_speakers_all();

            let noise_with_other: [(u64, u64); N] = speakers_all
                .iter()
                .map(|speaker| {
                    if let Some((other_matching_probability, other_data_len_sum, _)) =
                        common_pre_processing_inner_other::<T, Dataset, USE_DATA_N>(
                            i, speakers, speaker, threshold,
                        )
                    {
                        (other_matching_probability, other_data_len_sum)
                    } else {
                        (0, 0)
                    }
                })
                .collect::<Vec<_>>()
                .try_into()
                .unwrap();

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
                let ident = match (0..USE_DATA_N)
                    .map(|_| {
                        if unused_n.is_empty() {
                            return None;
                        }

                        let n = rng.gen_range(0..unused_n.len());
                        let n = unused_n.remove(n);
                        let ident = speaker[n].inner_average();

                        Some(ident)
                    })
                    .collect::<Option<Vec<_>>>()
                {
                    Some(ident) => ident.inner_average(),
                    None => continue,
                };

                for (j, speaker) in speakers_all.iter().enumerate() {
                    let other_data = speaker;

                    let other_data_len_sum = other_data.custom_get_length();
                    let other_matching_probability =
                        CustomFlat::<Vec<Vec<f64>>>::custom_flat(other_data.clone())
                            .iter()
                            .map(|v| compare(&ident, &v, threshold) as u8 as u64)
                            .sum::<u64>();

                    self_level_wrapper[type_][j] = (other_matching_probability, other_data_len_sum);
                }
            }

            (noise_with_other, other, self_level_wrapper)
        })
        .collect::<Vec<_>>();

    noise_with_other_and_other_and_self_level_wrapper
}

pub fn analysis_audio_mnist<const USE_DATA_N: usize>(
    data: AudioMNISTData<TimeCompressedType>,
    threshold: f64,
) -> (f64, f64) {
    let AudioMNISTData { speakers } = data;

    analysis_inner::<USE_DATA_N>(&speakers, threshold)
}

pub(super) fn analysis_inner<const USE_DATA_N: usize>(
    speakers: &[Vec<TimeCompressedType>],
    threshold: f64,
) -> (f64, f64) {
    let (
        (self_matching_probability, self_data_len_sum),
        (other_matching_probability, other_data_len_sum),
    ) = speakers
        .par_iter()
        .enumerate()
        .filter_map(|(i, speaker)| {
            // How many data from the training source will be employed
            let (other_matching_probability, other_data_len_sum, ident) =
                common_pre_processing_inner_other::<TimeCompressedType, _, USE_DATA_N>(
                    i, speakers, speaker, threshold,
                )?;

            let self_data = &speakers[i];

            let self_data_len_sum = self_data.custom_get_length();

            let self_matching_probability = self_data
                .iter()
                .map(|v| compare(&ident, &v, threshold) as u8 as u64)
                .sum::<u64>();

            Some((
                (self_matching_probability, self_data_len_sum),
                (other_matching_probability, other_data_len_sum),
            ))
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

pub fn analysis_baved<const USE_DATA_N: usize>(
    data: AudioBAVED<TimeCompressedType>,
    threshold: f64,
) -> ([f64; 3], f64, [[f64; 3]; 3]) {
    let AudioBAVED { speakers } = data;

    let level_with_other_and_other_and_self_level_wrapper = common_pre_processing::<
        3,
        TimeCompressedType,
        AudioBAVEDEmotion<TimeCompressedType>,
        USE_DATA_N,
    >(&speakers, threshold);

    common_post_processing(level_with_other_and_other_and_self_level_wrapper)
}

pub fn analysis_chime_home<const USE_DATA_N: usize>(
    data: AudioChimeHome<TimeCompressedType>,
    threshold: f64,
) -> ([f64; 8], f64, [[f64; 8]; 8]) {
    let AudioChimeHome {
        father,
        mother,
        child,
    } = data;

    let speakers = [father, mother, child];

    let noise_with_other_and_other_and_self_level_wrapper = common_pre_processing::<
        8,
        TimeCompressedType,
        AudioChimeHomeNoise<TimeCompressedType>,
        USE_DATA_N,
    >(&speakers, threshold);

    common_post_processing(noise_with_other_and_other_and_self_level_wrapper)
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

impl<T: Clone> GenSpeakersAll<T> for Vec<T> {
    fn gen_speakers_all(&self) -> Vec<Vec<T>> {
        vec![self.clone()]
    }
}

fn compare(a: &Vec<f64>, b: &Vec<f64>, threshold: f64) -> bool {
    fn get_distance(a: &Vec<f64>, b: &Vec<f64>) -> f64 {
        a.iter()
            .zip(b.iter())
            .map(|(a, b)| (a - b).abs())
            .sum::<f64>()
    }

    let distance = get_distance(a, b);

    distance < threshold
}
