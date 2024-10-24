use audio_analyzer_core::prelude::TestData;
use bench_data::get_lifters;
use ndarray::prelude::*;
use rand::Rng as _;
use rayon::prelude::*;

pub fn main() -> anyhow::Result<()> {
    let first_time = std::time::Instant::now();

    // bench_data(1)?;

    let correct = (1..=60)
        .into_par_iter()
        .map(|i| bench_data(i))
        .collect::<Result<Vec<_>, _>>()?;

    let correct_self_rate = correct.iter().map(|(a, _)| a).sum::<f64>() / 60.;
    let correct_other_rate = correct.iter().map(|(_, b)| b).sum::<f64>() / 60.;

    println!("correct_self_rate: {}", correct_self_rate);
    println!("correct_other_rate: {}", correct_other_rate);

    let elapsed = first_time.elapsed();

    println!("elapsed: {:?}", elapsed);

    Ok(())
}

// 50ms
const ANALYZE_SECOND: f64 = 0.05;
const THRESHOLD: f64 = 50.;

pub fn bench_data(speaker_n: usize) -> anyhow::Result<(f64, f64)> {
    assert!(speaker_n > 0);
    assert!(speaker_n <= 60);

    let mut rng = rand::thread_rng();

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

    let mut gen_landom_lifter = || {
        let say_n = rng.gen_range(0..10);
        let num_n = rng.gen_range(0..50);

        let path = gen_path(speaker_n, say_n, num_n);

        get_lifter(&path)
    };

    // 5 sample
    let mut data = Vec::new();
    for _ in 0..5 {
        data.push(gen_landom_lifter());
    }

    // 平均を取る
    let ave_lifter = (0..data[0].len())
        .map(|i| data.iter().map(|lifter| lifter[i].clone()).sum::<f64>() / 50.0)
        .collect::<Vec<_>>();

    // ave_lifterとの距離を計算し、本人か確認するための関数
    let get_distance = |lifter: &Array1<f64>| {
        ave_lifter
            .iter()
            .zip(lifter.iter())
            .map(|(a, b)| (a - b).abs())
            .sum::<f64>()
    };

    let get_distance_by_path = |path: &str| {
        let lifter = get_lifter(path);

        get_distance(&lifter)
    };

    let check_speaker = |path: &str| {
        let distance = get_distance_by_path(path);

        // println!("distance: {}", distance);

        distance < THRESHOLD
    };

    // 自身のデータに対して、本人か確認する
    let mut correct_self = 0;
    for i in 0..9 {
        for j in 0..50 {
            let path = gen_path(speaker_n, i, j);

            if check_speaker(&path) {
                correct_self += 1;
            }
        }
    }
    // 自身への正答率
    let correct_self_rate = correct_self as f64 / 450.;

    // その他のデータに対して、他人か確認する
    let mut correct_other = 0;
    for i in 1..=60 {
        if i == speaker_n {
            continue;
        }

        for j in 0..9 {
            for k in 0..50 {
                let path = gen_path(i, j, k);

                if !check_speaker(&path) {
                    correct_other += 1;
                }
            }
        }
    }

    // 他人への正答率
    let correct_other_rate = correct_other as f64 / (59. * 9. * 50.);

    println!("correct_self_rate: {}", correct_self_rate);
    println!("correct_other_rate: {}", correct_other_rate);

    Ok((correct_self_rate, correct_other_rate))
}

pub fn check_speaker(
    ave_lifter: ArrayView1<f64>,
    self_datas: Vec<ArrayView1<f64>>,
    other_datas: Vec<ArrayView1<f64>>,
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

pub fn get_all_data
