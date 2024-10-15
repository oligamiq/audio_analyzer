#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

pub mod apps;
pub mod libs;

pub type Result<T> = color_eyre::Result<T>;

// fn main() -> Result<()> {
//     initialize_logging()?;

//     let args = Args::parse();

//     // setup terminal
//     stdout().execute(EnterAlternateScreen)?;
//     enable_raw_mode()?;
//     let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
//     terminal.clear()?;

//     // layer config
//     let mut raw_data_layer = device_stream::Device::new();
//     // let mut raw_data_layer = TestData::new(TestDataType::TestData1);

//     raw_data_layer.start();

//     let sample_rate = raw_data_layer.sample_rate();

//     debug!("sample rate: {}", sample_rate);

//     // let spec = hound::WavSpec {
//     //     channels: 1,
//     //     sample_rate,
//     //     bits_per_sample: 32,
//     //     sample_format: hound::SampleFormat::Int,
//     // };
//     // let mut writer = hound::WavWriter::create(".data/voice.wav", spec).unwrap();
//     // let receiver = raw_data_layer.voice_stream_receiver();
//     // let amplitude = i32::MAX as f32;
//     // let handle = std::thread::spawn(move || loop {
//     //     let data = receiver.recv().unwrap();
//     //     let ave = data.iter().sum::<f32>() / data.len() as f32;
//     //     debug!("writing to file: {}", ave * amplitude);
//     //     data.iter().for_each(|&sample| {
//     //         writer.write_sample((sample * amplitude) as i32).unwrap();
//     //     });
//     // });

//     let fft_layer = ToSpectrogramLayer::new(FftConfig::new(400, 160));
//     let mel_layer = ToMelSpectrogramLayer::new(MelConfig::new(400, 160, 80, sample_rate.into()));
//     let psd_layer = ToPowerSpectralDensityLayer::new(ToPowerSpectralDensityLayerConfig {
//         sample_rate: sample_rate.into(),
//         time_range: 20,
//         n_mels: 80,
//     });

//     let mut layers = MultipleLayers::default();
//     layers.push_layer(fft_layer);
//     layers.push_layer(mel_layer);
//     layers.push_layer(psd_layer);
//     // debug!("{:?}", std::any::type_name_of_val(&layers));
//     debug!("{:?}", layers);

//     // create app and run it
//     let tick_rate = Duration::from_millis(50);

//     let app = App::new(Box::new(raw_data_layer), layers);
//     app.run(&mut terminal, tick_rate)?;

//     // plot::plot().unwrap();

//     // match config.sample_format() {
//     //     cpal::SampleFormat::F32 => run::<f32>(&device, &config.into()).unwrap(),
//     //     cpal::SampleFormat::I16 => run::<i16>(&device, &config.into()).unwrap(),
//     //     cpal::SampleFormat::U16 => run::<u16>(&device, &config.into()).unwrap(),
//     //     _ => panic!("Unsupported format"),
//     // }

//     stdout().execute(LeaveAlternateScreen)?;
//     disable_raw_mode()?;
//     Ok(())
// }

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    use egui_tracing::tracing_subscriber::layer::SubscriberExt;
    use egui_tracing::tracing_subscriber::util::SubscriberInitExt;
    use egui_tracing::{egui, tracing_subscriber};

    let collector = egui_tracing::EventCollector::default();

    tracing_subscriber::registry()
        .with(tracing_subscriber::filter::filter_fn(|event| {
            if let Some(module) = event.module_path() {
                let mut bool = (*event.level() == tracing_core::Level::TRACE
                    || *event.level() == tracing_core::Level::DEBUG
                    || *event.level() == tracing_core::Level::INFO)
                    && (module.starts_with("eframe")
                        || module.starts_with("wgpu")
                        || module.starts_with("naga"));
                bool |= (*event.level() == tracing_core::Level::TRACE
                    || *event.level() == tracing_core::Level::DEBUG)
                    && module.starts_with("egui");
                !bool
            } else {
                true
            }
        }))
        .with(collector.clone())
        .init();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_min_inner_size([300.0, 220.0])
            .with_icon(
                // NOTE: Adding an icon is optional
                eframe::icon_data::from_png_bytes(&include_bytes!("../../assets/icon-256.png")[..])
                    .expect("Failed to load icon"),
            ),
        ..Default::default()
    };
    eframe::run_native(
        "eframe template",
        native_options,
        Box::new(|cc| Ok(Box::new(apps::app::App::new(cc, collector.clone())))),
    )
}

// When compiling to web using trunk:
#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::wasm_bindgen::JsCast as _;
    use egui_tracing::tracing_subscriber;
    use egui_tracing::tracing_subscriber::layer::SubscriberExt;
    use egui_tracing::tracing_subscriber::util::SubscriberInitExt;

    console_error_panic_hook::set_once();

    let collector = egui_tracing::EventCollector::default();

    tracing_subscriber::registry()
        .with(tracing_subscriber::filter::filter_fn(|event| {
            if let Some(module) = event.module_path() {
                let mut bool = (*event.level() == tracing_core::Level::TRACE
                    || *event.level() == tracing_core::Level::DEBUG
                    || *event.level() == tracing_core::Level::INFO)
                    && (module.starts_with("eframe")
                        || module.starts_with("wgpu")
                        || module.starts_with("naga"));
                bool |= (*event.level() == tracing_core::Level::TRACE
                    || *event.level() == tracing_core::Level::DEBUG)
                    && module.starts_with("egui");
                !bool
            } else {
                true
            }
        }))
        .with(collector.clone())
        .init();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window()
            .expect("No window")
            .document()
            .expect("No document");

        let canvas = document
            .get_element_by_id("the_canvas_id")
            .expect("Failed to find the_canvas_id")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("the_canvas_id was not a HtmlCanvasElement");

        audio_analyzer_core::data::web_stream::init_on_web_struct().await;

        let start_result = eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(move |cc| Ok(Box::new(apps::app::App::new(cc, collector.clone())))),
            )
            .await;

        // Remove the loading text and spinner:
        if let Some(loading_text) = document.get_element_by_id("loading_text") {
            match start_result {
                Ok(_) => {
                    loading_text.remove();
                }
                Err(e) => {
                    loading_text.set_inner_html(
                        "<p> The app has crashed. See the developer \
                                                 console for details. </p>",
                    );
                    panic!("Failed to start eframe: {e:?}");
                }
            }
        }
    });
}
