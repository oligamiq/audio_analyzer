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
use mel_layer::to_mel_layer::ToMelSpectrogramLayer;
use mel_spec::config::MelConfig;
use ndarray::{Array1, Array2};
use num_complex::Complex;
use ratatui::prelude::*;
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

    let mut fft_layer = ToSpectrogramLayer::new(FftConfig::new(400, 160));
    fft_layer.set_input_stream(raw_data_layer.voice_stream_receiver());
    let mel_layer = ToMelSpectrogramLayer::new(MelConfig::new(400, 160, 80, 16000.0));

    let layers = layer(fft_layer);
    let layers = layers.add_layer(mel_layer);

    // create app and run it
    let tick_rate = Duration::from_millis(250);

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
