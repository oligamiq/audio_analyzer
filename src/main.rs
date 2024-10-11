use std::io::stdout;
use std::time::Duration;

use app::App;
use clap::Parser as _;
use color_eyre::Result;
use command::Args;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand as _;
use data::test_data::{TestData, TestDataType};
use data::RawDataStreamLayer as _;
use layer::layers::{layer, MultipleLayers};
use layer::Layer as _;
use mel_layer::fft_layer::{FftConfig, ToSpectrogramLayer};
use mel_layer::spectral_density::{ToPowerSpectralDensityLayer, ToPowerSpectralDensityLayerConfig};
use mel_layer::to_mel_layer::ToMelSpectrogramLayer;
use mel_spec::config::MelConfig;
use ndarray::{Array1, Array2};
use num_complex::Complex;
use ratatui::prelude::*;
use symphonia::core::sample;
use tracing::debug;
use utils::debug::initialize_logging;

// pub mod console;
// pub mod plot;
pub mod app;
pub mod command;
pub mod data;
pub mod layer;
pub mod mel_layer;
pub mod tui;
pub mod utils;

fn main() -> Result<()> {
    initialize_logging()?;

    let args = Args::parse();

    // setup terminal
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    // layer config
    let mut raw_data_layer = data::device_stream::Device::new();
    // let mut raw_data_layer = TestData::new(TestDataType::TestData1);

    raw_data_layer.start();

    let sample_rate = raw_data_layer.sample_rate();

    debug!("sample rate: {}", sample_rate);

    let spec = hound::WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(".data/voice.wav", spec).unwrap();
    let receiver = raw_data_layer.voice_stream_receiver();
    let amplitude = i32::MAX as f32;
    let handle = std::thread::spawn(move || loop {
        let data = receiver.recv().unwrap();
        let ave = data.iter().sum::<f32>() / data.len() as f32;
        debug!("writing to file: {}", ave * amplitude);
        data.iter().for_each(|&sample| {
            writer.write_sample((sample * amplitude) as i32).unwrap();
        });
    });

    let mut fft_layer = ToSpectrogramLayer::new(FftConfig::new(400, 160));
    fft_layer.set_input_stream(raw_data_layer.voice_stream_receiver());
    let mel_layer = ToMelSpectrogramLayer::new(MelConfig::new(400, 160, 80, sample_rate.into()));
    let psd_layer = ToPowerSpectralDensityLayer::new(ToPowerSpectralDensityLayerConfig {
        sample_rate: sample_rate.into(),
        time_range: 20,
        n_mels: 80,
    });

    let layers = layer(fft_layer);
    let layers = layers.add_layer(mel_layer);
    let layers = layers.add_layer(psd_layer);
    // debug!("{:?}", std::any::type_name_of_val(&layers));
    debug!("{:?}", layers);

    // create app and run it
    let tick_rate = Duration::from_millis(50);

    let app = App::new(layers);
    app.run(&mut terminal, tick_rate)?;

    // plot::plot().unwrap();

    // match config.sample_format() {
    //     cpal::SampleFormat::F32 => run::<f32>(&device, &config.into()).unwrap(),
    //     cpal::SampleFormat::I16 => run::<i16>(&device, &config.into()).unwrap(),
    //     cpal::SampleFormat::U16 => run::<u16>(&device, &config.into()).unwrap(),
    //     _ => panic!("Unsupported format"),
    // }

    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}
