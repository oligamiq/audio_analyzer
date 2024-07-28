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
use data::device_stream::Device;
use data::test_data::{TestData, TestDataType};
use debug_util::initialize_logging;
use mel_spec::config::MelConfig;
use ratatui::prelude::*;
use stream::first_layer::DefaultMelLayer;

// pub mod console;
// pub mod plot;
pub mod app;
pub mod command;
pub mod data;
pub mod debug_util;
pub mod stream;
pub mod tui;

fn main() -> Result<()> {
    initialize_logging()?;

    let args = Args::parse();

    // setup terminal
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    // layer config
    // let raw_data_layer = Device::new();
    let raw_data_layer = TestData::new(TestDataType::TestData1);

    let mel_layer = {
        let mut mel_layer = DefaultMelLayer::new();
        std::mem::swap(
            mel_layer.borrow_mel_config(),
            &mut MelConfig::new(400, 160, args.mels, 16000.0),
        );
        mel_layer
    };

    // create app and run it
    let tick_rate = Duration::from_millis(250);

    let app = App::new(raw_data_layer, mel_layer);
    app.run(&mut terminal, tick_rate)?;

    // let host = cpal::default_host();

    // let device = host
    //     .default_output_device()
    //     .expect("Failed to find a default output device");

    // let config = device.default_output_config().unwrap();

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

// fn run<T>(device: &cpal::Device, config: &cpal::StreamConfig) -> Result<()>
// where
//     T: SizedSample + FromSample<f64>,
// {
//     let sample_rate = config.sample_rate.0 as f64;
//     let channels = config.channels as usize;

//     let c = 0.2 * (organ_hz(midi_hz(57.0)) + organ_hz(midi_hz(61.0)) + organ_hz(midi_hz(64.0)));

//     let c = c >> pan(0.0);

//     // Add chorus.
//     let c = c >> (chorus(0, 0.0, 0.01, 0.2) | chorus(1, 0.0, 0.01, 0.2));

//     let mut c = c
//         >> (declick() | declick())
//         >> (dcblock() | dcblock())
//         //>> (multipass() & 0.2 * reverb_stereo(10.0, 3.0, 1.0))
//         >> limiter_stereo(1.0, 5.0);

//     c.set_sample_rate(sample_rate);
//     c.allocate();

//     let mut next_value = move || assert_no_alloc(|| c.get_stereo());

//     let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

//     let stream = device.build_output_stream(
//         config,
//         move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
//             write_data(data, channels, &mut next_value)
//         },
//         err_fn,
//         None,
//     )?;
//     stream.play()?;

//     std::thread::sleep(std::time::Duration::from_millis(50000));

//     Ok(())
// }

// fn write_data<T>(output: &mut [T], channels: usize, next_sample: &mut dyn FnMut() -> (f32, f32))
// where
//     T: SizedSample + FromSample<f64>,
// {
//     for frame in output.chunks_mut(channels) {
//         let sample = next_sample();
//         let left = T::from_sample(sample.0 as f64);
//         let right: T = T::from_sample(sample.1 as f64);

//         for (channel, sample) in frame.iter_mut().enumerate() {
//             if channel & 1 == 0 {
//                 *sample = left;
//             } else {
//                 *sample = right;
//             }
//         }
//     }
// }
