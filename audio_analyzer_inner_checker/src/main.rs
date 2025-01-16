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
    ctrlc::set_handler(move || {
        CTRLC_HANDLER.lock()();
    })
    .unwrap();

    // analysis()?;

    let data = analysis_data_all()?;

    use plotters::prelude::*;

    let mut mnist_max_diff = HashMap::<&str, f64>::new();

    for name in vec!["burg", "lpc", "fft", "fft_small", "liftered"] {
        let (data, range) = data.get(name).unwrap();
        let out_file_name = format!("{name}_out.svg");
        let root = SVGBackend::new(&out_file_name, (1024, 1536)).into_drawing_area();
        // let root = SVGBackend::new(&out_file_name, (1024, 1024)).into_drawing_area();
        root.fill(&WHITE)?;
        let range_min = range.iter().cloned().reduce(f64::min).unwrap() as f32;
        let range_max = range.iter().cloned().reduce(f64::max).unwrap() as f32;
        let mut chart = ChartBuilder::on(&root)
            .caption(name, ("sans-serif", 50).into_font())
            .margin(5)
            .x_label_area_size(50)
            .y_label_area_size(50)
            .build_cartesian_2d(range_min..range_max, -0.01f32..1f32)?;

        chart
            .configure_mesh()
            .x_desc("Threshold")
            .axis_desc_style(("sans-serif", 50))
            .y_desc("FAR / FRR")
            .draw()?;

        let mut dashed_line_style = ShapeStyle {
            color: RED.to_rgba(),
            filled: false,
            stroke_width: 2,
        };

        chart
            .draw_series(LineSeries::new(
                range
                    .iter()
                    .cloned()
                    .map(|threshold| threshold as f32)
                    .zip(data.0.iter().map(|(x, _)| 1. - *x as f32)),
                dashed_line_style.clone(),
            ))?;
            // .label("mnist self")
            // .legend(move |(x, y)| {
            //     PathElement::new(vec![(x, y), (x + 20, y)], dashed_line_style.clone())
            // });

        dashed_line_style.stroke_width = 1;

        chart
            .draw_series(LineSeries::new(
                range
                    .iter()
                    .cloned()
                    .map(|threshold| threshold as f32)
                    .zip(data.0.iter().map(|(_, x)| *x as f32)),
                dashed_line_style.clone(),
            ))?;
            // .label("mnist other")
            // .legend(move |(x, y)| {
            //     PathElement::new(vec![(x, y), (x + 20, y)], dashed_line_style.clone())
            // });

        fn draw_chart_for_noise_kind<const N: usize>(
            range: &Vec<f64>,
            chart: &mut ChartContext<
                '_,
                SVGBackend<'_>,
                Cartesian2d<
                    plotters::coord::types::RangedCoordf32,
                    plotters::coord::types::RangedCoordf32,
                >,
            >,
            data: &Vec<([f64; N], f64, [[f64; N]; N])>,
            title: &str,
            color_: RGBColor,
        ) -> anyhow::Result<()> {
            // ノイズなしで学習して、その他の人の全てに対しての分類精度を見る
            // レベル1で学習して、その他の人の全てに対しての分類精度を見る
            // レベル2で学習して、その他の人の全てに対しての分類精度を見る
            let mut color = color_;
            for n in 0..N {
                chart
                    .draw_series(LineSeries::new(
                        range
                            .iter()
                            .cloned()
                            .map(|threshold| threshold as f32)
                            .zip(data.iter().map(|(x, ..)| x[n] as f32)),
                        &color.mix(0.5),
                    ))?
                    .label(format!("{title} other learn by {n}"))
                    .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], color));

                color.0 = color.0.wrapping_add(64);
            }

            // 上記3つの合計
            chart
                .draw_series(LineSeries::new(
                    range
                        .iter()
                        .cloned()
                        .map(|threshold| threshold as f32)
                        .zip(data.iter().map(|(_, x, _)| *x as f32)),
                    &color,
                ))?;
                // .label(format!("{title} other learn by n sum"))
                // .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], color));

            // type_で学習し、targetに対し確認する
            let mut color = color_;
            for type_ in 0..N {
                {
                    for target in 0..N {
                        {
                            let mut style: ShapeStyle = color.mix(0.3).into();
                            style.stroke_width = if type_ == target { 2 } else { 1 };

                            chart
                                .draw_series(LineSeries::new(
                                    range.iter().cloned().map(|threshold| threshold as f32).zip(
                                        data.iter()
                                            .map(|(_, _, x)| x[type_][target] as f32)
                                            .map(|x| if type_ == target { 1. - x } else { x }),
                                    ),
                                    style,
                                ))?
                                .label(format!("{title} self learn by {type_} -> {target}"))
                                .legend(move |(x, y)| {
                                    PathElement::new(vec![(x, y), (x + 20, y)], style)
                                });

                            color.1 = color.1.wrapping_add(32);
                        }
                        color.0 = color.0.wrapping_add(32);
                    }
                }
            }

            // let self_sum = (0..N)
            //     .map(|type_| {
            //         data.iter()
            //             .map(|(_, _, x)| x[type_][type_])
            //             .sum::<f64>()
            //     })
            //     .collect::<Vec<_>>();

            // chart
            //     .draw_series(LineSeries::new(
            //         range
            //             .iter()
            //             .cloned()
            //             .map(|threshold| threshold as f32)
            //             .zip(self_sum.iter().map(|x| *x as f32)),
            //         &color,
            //     ))?
            //     .label(format!("{title} self learn by n sum"))
            //     .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], color));

            Ok(())
        }

        // BAVED
        draw_chart_for_noise_kind(&range, &mut chart, &data.1, "baved", BLUE)?;
        // ChimeHome
        draw_chart_for_noise_kind(&range, &mut chart, &data.2, "chime home", GREEN)?;

        // chart
        //     .configure_series_labels()
        //     .background_style(&WHITE.mix(0.8))
        //     .border_style(&BLACK)
        //     .draw()?;

        root.present()?;

        let mnist_max_diff_ = data
            .0
            .iter()
            .map(|(up, down)| up - down)
            .reduce(f64::max)
            .unwrap();
        mnist_max_diff.insert(name, mnist_max_diff_);
    }

    println!("mnist_max_diff: {:?}", mnist_max_diff);

    let mnist_max_diff_string = mnist_max_diff
        .iter()
        .filter(|(s, _)| **s != "fft_small")
        .map(|(s, v)| (s.to_string(), *v as f32))
        .collect::<Vec<_>>();
    let mnist_max_diff_string_only = mnist_max_diff_string
        .iter()
        .cloned()
        .map(|(s, _)| s)
        .collect::<Vec<_>>();
    let root = SVGBackend::new("mnist_diff_histogram.svg", (1024, 1024)).into_drawing_area();
    root.fill(&WHITE)?;
    let mut chart = ChartBuilder::on(&root)
        .x_label_area_size(100)
        .y_label_area_size(100)
        .margin(5)
        .caption(
            "TAR - FAR (一致成功率 - 不一致成功率)",
            ("sans-serif", 50.0),
        )
        .build_cartesian_2d(mnist_max_diff_string_only.into_segmented(), 0f32..1f32)?;

    fn fmt(v: &SegmentValue<&std::string::String>) -> String {
        match v {
            SegmentValue::Exact(v) => v.to_string(),
            SegmentValue::CenterOf(v) => v.to_string(),
            SegmentValue::Last => "Last".to_string(),
        }
    }

    chart
        .configure_mesh()
        .x_desc("Method")
        .y_desc("TAR - FAR")
        .disable_x_mesh()
        .bold_line_style(WHITE.mix(0.3))
        .x_label_style(("sans-serif", 50))
        .x_label_formatter(&fmt)
        .axis_desc_style(("sans-serif", 50))
        .draw()?;

    chart.draw_series(
        Histogram::vertical(&chart)
            .style(RED.mix(0.5).filled())
            .data(mnist_max_diff_string.iter().map(|(s, v)| (s, *v))),
    )?;

    root.present()?;

    let root = SVGBackend::new("eer_histogram.svg", (1024, 512)).into_drawing_area();

    root.fill(&WHITE)?;

    let mnist_eer = vec![
        ("burg", 0.32),
        ("lpc", 0.31),
        ("fft", 0.435),
        ("liftered", 0.2),
    ];
    let baved_eer = vec![
        ("burg", 0.31),
        ("lpc", 0.315),
        ("fft", 0.39),
        // ("liftered", ),
    ];
    let chime_home_eer = vec![
        ("burg", 0.44),
        ("lpc", 0.47),
        ("fft", 0.5),
        ("liftered", 0.4),
    ];
    let err = vec![mnist_eer, chime_home_eer, baved_eer]
        .into_iter()
        .map(|v| v.into_iter().collect::<HashMap<&str, _>>())
        .collect::<Vec<_>>();

    let binding = ["fft", "lpc", "burg", "liftered"]
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>();

    // let binding_dataset = ["mnist", "chime home", "baved"]
    //     .iter()
    //     .map(|s| s.to_string())
    //     .collect::<Vec<_>>();

    let mut chart = ChartBuilder::on(&root)
        .x_label_area_size(100)
        .y_label_area_size(100)
        .margin(5)
        // .caption("EER", ("sans-serif", 50.0))
        .build_cartesian_2d(0f32..3.0f32, 0f32..1f32)?;

    fn fmt_n(v: &f32) -> String {
        if (v.round() - v - 0.5).abs() < 0.01 {
            ["mnist", "chime home", "baved"]
                .get(*v as usize)
                .unwrap_or(&"")
                .to_string()
        } else {
            "".to_string()
        }
    }

    chart
        .configure_mesh()
        .disable_x_mesh()
        .bold_line_style(WHITE.mix(0.3))
        .x_desc("Dataset")
        .y_desc("EER")
        .x_label_style(("sans-serif", 50))
        .x_label_formatter(&fmt_n)
        .axis_desc_style(("sans-serif", 50))
        .draw()?;

    let mut colors = vec![RED, GREEN, BLUE, YELLOW];

    for (i, name) in binding.iter().enumerate() {
        let mut color: ShapeStyle = colors.pop().unwrap().filled();
        color.stroke_width = 10;

        // chart
        //     .draw_series(
        //         Histogram::vertical(&chart).style(color).data(
        //             binding_dataset
        //                 .iter()
        //                 .zip(err.iter())
        //                 .filter_map(|(dataset_n, err)| {
        //                     err.get(name.as_str()).map(|v| (dataset_n, *v as f32))
        //                 })
        //                 .collect::<Vec<_>>(),
        //         ),
        //     )?
        //     .label(name)
        //     .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], color));

        // creating a histogram by Rectangle
        let i = i as f32;
        let num = 4.;
        let a = (0..3)
            .zip(err.iter())
            .filter_map(|(dataset_n, err)| err.get(name.as_str()).map(|v| (dataset_n, *v as f32)))
            .map(|(x, y)| {
                Rectangle::new(
                    [
                        (x as f32 + 0.1 + i / num * 0.9, y),
                        (x as f32 + (i + 1.) / num * 0.9, 0.),
                    ],
                    color,
                )
            })
            .collect::<Vec<_>>();

        chart
            .draw_series(a.into_iter())?
            .label(name)
            .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], color));
    }

    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .label_font(("sans-serif", 40))
        .border_style(&BLACK)
        .draw()?;

    root.present()?;

    Ok(())
}

type AnalyzedDataType = (
    Vec<(f64, f64)>,
    Vec<([f64; 3], f64, [[f64; 3]; 3])>,
    Vec<([f64; 8], f64, [[f64; 8]; 8])>,
);

#[allow(unused)]
fn analysis_data_all(
) -> anyhow::Result<HashMap<compact_str::CompactString, (AnalyzedDataType, Vec<f64>)>> {
    let now = std::time::Instant::now();

    const USE_DATA_N: usize = 10;

    let range = (1..=250).map(|n| n as f64 / 100.).collect::<Vec<_>>();

    let mut analysis_data_map =
        HashMap::<compact_str::CompactString, (AnalyzedDataType, Vec<f64>)>::new();

    // let data = load_data::<Vec<f64>, _>("burg")?;
    let analysis_data = analysis_load_data::<Vec<f64>, _, USE_DATA_N>("burg", &range, "")?;
    // println!("analysis_data: {:?}", analysis_data);
    analysis_data_map.insert("burg".into(), (analysis_data, range.clone()));

    // let data = load_data::<Vec<Vec<Option<f64>>>, _>("burg_uncompress")?;
    // let analysis_data =
    //     analysis_load_data::<Vec<Vec<Option<f64>>>, _, USE_DATA_N>("burg_uncompress")?;
    // println!("analysis_data: {:?}", analysis_data);

    // let data = load_data::<Vec<f64>, _>("lpc")?;
    let analysis_data = analysis_load_data::<Vec<f64>, _, USE_DATA_N>("lpc", &range, "")?;
    // println!("analysis_data: {:?}", analysis_data);
    analysis_data_map.insert("lpc".into(), (analysis_data, range.clone()));

    // let data = load_data::<Vec<Vec<Option<f64>>>, _>("lpc_uncompress")?;
    // let analysis_data =
    //     analysis_load_data::<Vec<Vec<Option<f64>>>, _, USE_DATA_N>("lpc_uncompress")?;
    // println!("analysis_data: {:?}", analysis_data);

    let range = (1..=250).map(|n| n as f64 / 100.).collect::<Vec<_>>();
    // let data = load_data::<Vec<f64>, _>("fft")?;
    let analysis_data = analysis_load_data::<Vec<f64>, _, USE_DATA_N>("fft", &range, "_small")?;
    // println!("analysis_data: {:?}", analysis_data);
    analysis_data_map.insert("fft_small".into(), (analysis_data, range.clone()));

    let range = (1..=500).map(|n| n as f64 / 10.).collect::<Vec<_>>();
    // let data = load_data::<Vec<f64>, _>("fft")?;
    let analysis_data = analysis_load_data::<Vec<f64>, _, USE_DATA_N>("fft", &range, "")?;
    // println!("analysis_data: {:?}", analysis_data);
    analysis_data_map.insert("fft".into(), (analysis_data, range.clone()));

    let range = (0..=15)
        .map(|n| (n * 10000 + 100000) as f64)
        .collect::<Vec<_>>();
    // let range = vec![0.5];
    // let data = load_data::<Vec<f64>, _>("liftered")?;
    let analysis_data = analysis_load_data::<Vec<f64>, _, USE_DATA_N>("liftered", &range, "")?;
    // println!("analysis_data: {:?}", analysis_data);
    analysis_data_map.insert("liftered".into(), (analysis_data, range.clone()));

    println!("last elapsed: {:?}", now.elapsed());

    Ok(analysis_data_map)
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
    unique_salt: &str,
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

    fn saving_file_name(
        place: &str,
        unique_id: &str,
        use_data_n: usize,
        unique_salt: &str,
    ) -> String {
        format!("{place}/{unique_id}{unique_salt}_{use_data_n}.bin")
    }

    fn analysis_load_data<T, const USE_DATA_N: usize, Dataset, F, Hasher>(
        place: &str,
        range: &Vec<f64>,
        fake_analyzer: F,
        unique_salt: &str,
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
            unique_salt,
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
                    out
                })
                .collect::<Vec<_>>();

            progress.lock().finish();

            bincode::serialize_into(
                std::fs::File::create(saving_file_name(place, unique_id, USE_DATA_N, unique_salt))?,
                &mnist_analyzed_data,
            )?;

            mnist_analyzed_data
        };

        Ok(mnist_data)
    }

    let mnist_analyzed_data = analysis_load_data::<
        T,
        USE_DATA_N,
        AudioMNISTData<T>,
        _,
        gxhash::GxBuildHasher,
    >(&place, &range, &fake_analyzer, unique_salt)?;

    let baved_analyzed_data = analysis_load_data::<
        T,
        USE_DATA_N,
        AudioBAVED<T>,
        _,
        gxhash::GxBuildHasher,
    >(&place, &range, &fake_analyzer, unique_salt)?;

    let chime_home_analyzed_data = analysis_load_data::<
        T,
        USE_DATA_N,
        AudioChimeHome<T>,
        _,
        gxhash::GxBuildHasher,
    >(&place, &range, &fake_analyzer, unique_salt)?;

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
