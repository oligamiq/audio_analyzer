use audio_analyzer_core::prelude::TestData;
use bench_data::get_lifters;
use ndarray::prelude::*;
use rand::Rng as _;
use rayon::prelude::*;

pub fn main() -> anyhow::Result<()> {
    let first_time = std::time::Instant::now();

    let (correct_self_rate, correct_other_rate) = bench_data()?;

    println!("correct_self_rate: {}", correct_self_rate);
    println!("correct_other_rate: {}", correct_other_rate);

    let elapsed = first_time.elapsed();

    println!("elapsed: {:?}", elapsed);

    Ok(())
}

// 50ms
const ANALYZE_SECOND: f64 = 0.05;
const THRESHOLD: f64 = 50.;

pub fn bench_data() -> anyhow::Result<(f64, f64)> {
    let datas = get_all_data()?;

    let correct = (1..=60)
        .into_par_iter()
        .map(|i| {
            let (self_datas, other_datas) = get_ident_data(&datas, i)?;

            let ave_lifter = gen_ident_data(&datas, i, 3)?;

            check_speaker(ave_lifter.view(), self_datas, other_datas)
        })
        .collect::<Result<Vec<_>, _>>()?;

    let correct_self_rate = correct.iter().map(|(a, _)| a).sum::<f64>() / 60.;
    let correct_other_rate = correct.iter().map(|(_, b)| b).sum::<f64>() / 60.;

    Ok((correct_self_rate, correct_other_rate))
}

pub fn check_speaker(
    ave_lifter: ArrayView1<f64>,
    self_datas: Vec<Array1<f64>>,
    other_datas: Vec<Array1<f64>>,
) -> anyhow::Result<(f64, f64)> {
    let get_distance = |lifter: &Array1<f64>| {
        ave_lifter
            .iter()
            .zip(lifter.iter())
            .map(|(a, b)| (a - b).abs())
            .sum::<f64>()
    };

    let correct_self = self_datas
        .iter()
        .filter(|lifter| {
            let lifter = lifter.to_owned().to_owned();
            get_distance(&lifter) < THRESHOLD
        })
        .count();

    let correct_self_rate = correct_self as f64 / self_datas.len() as f64;

    let correct_other = other_datas
        .iter()
        .filter(|lifter| {
            let lifter = lifter.to_owned().to_owned();
            get_distance(&lifter) > THRESHOLD
        })
        .count();

    let correct_other_rate = correct_other as f64 / other_datas.len() as f64;

    Ok((correct_self_rate, correct_other_rate))
}

pub fn get_all_data() -> anyhow::Result<Vec<Vec<Array1<f64>>>> {
    let base_path = "AudioMNIST/data";

    let gen_path = |speaker_n: usize, say_n: usize, num_n: usize| {
        assert!(say_n <= 9);

        assert!(num_n <= 49);

        let path = format!("{base_path}/{speaker_n:02}/{say_n}_{speaker_n:02}_{num_n}.wav");

        path
    };

    let get_lifter = |path: &str| {
        let mut data = TestData::new_with_path(path.to_owned());

        data.start();

        let lifter = get_lifters(data, ANALYZE_SECOND);

        //平均を取る
        let lifter = (0..lifter[0].len())
            .map(|i| {
                lifter.iter().map(|lifter| lifter[i].clone()).sum::<f64>() / lifter.len() as f64
            })
            .collect::<Array1<_>>();

        lifter
    };

    let datas = (1..=60)
        .into_par_iter()
        .map(|speaker_n| {
            (0..9)
                .into_iter()
                .flat_map(|j| {
                    (0..50).into_iter().map(move |k| {
                        let path = gen_path(speaker_n, j, k);

                        get_lifter(&path)
                    })
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    Ok(datas)
}

// 本人のデータと他人のデータを取得する
pub fn get_ident_data(
    datas: &Vec<Vec<Array1<f64>>>,
    speaker_n: usize,
) -> anyhow::Result<(Vec<Array1<f64>>, Vec<Array1<f64>>)> {
    let self_datas = datas[speaker_n - 1].clone();

    let mut other_datas = Vec::new();

    for i in 0..60 {
        if i == speaker_n - 1 {
            continue;
        }

        other_datas.extend(datas[i].clone());
    }

    Ok((self_datas, other_datas))
}

// 本人のデータからランダムにn個のデータを取得し、平均を取って、個人を識別するためのデータとする
pub fn gen_ident_data(
    datas: &Vec<Vec<Array1<f64>>>,
    speaker_n: usize,
    n: usize,
) -> anyhow::Result<Array1<f64>> {
    let self_datas = &datas[speaker_n - 1];

    let mut rng = rand::thread_rng();

    let mut ave_lifter = Array1::zeros(self_datas[0].len());

    for _ in 0..n {
        let lifter = &self_datas[rng.gen_range(0..self_datas.len())];

        ave_lifter += lifter;
    }

    ave_lifter /= n as f64;

    Ok(ave_lifter)
}
