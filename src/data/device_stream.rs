use std::thread;

use cpal::traits::{DeviceTrait as _, HostTrait, StreamTrait as _};
use crossbeam_channel::Receiver;

use super::RawDataLayer;

pub struct Device {
    _host: cpal::Host,
    device: Option<cpal::Device>,
    sample_rate: Option<u32>,
    voice_stream_receiver: Option<Receiver<Vec<f32>>>,
    handles: Option<Vec<thread::JoinHandle<()>>>,
}

impl Device {
    pub fn new() -> Self {
        let host = cpal::default_host();
        let device = host.default_input_device().unwrap();

        Device {
            _host: host,
            device: Some(device),
            sample_rate: None,
            voice_stream_receiver: None,
            handles: None,
        }
    }

    pub fn run(&mut self) {
        let device = self.device.take().unwrap();

        dbg!(device.name().unwrap());

        // let mut supported_configs_range = device.supported_output_configs().wrap_err("cannot get supported config on audio device").unwrap();

        // dbg!(supported_configs_range.next().unwrap());

        // supported_configs_range.for_each(|config| {
        //     dbg!(config);
        // });

        // // let config = supported_configs_range.find_(|config| {
        // //     dbg!(config);
        // //     config.channels() == 2
        // // }).unwrap();

        let config = device.default_input_config().unwrap();

        dbg!(&config);

        self.sample_rate = Some(config.sample_rate().0);

        let (sender, receiver) = crossbeam_channel::unbounded();

        self.voice_stream_receiver = Some(receiver);

        let err_fn = move |err| {
            eprintln!("an error occurred on stream: {}", err);
        };

        let handle = thread::spawn(move || {
            let stream = match config.sample_format() {
                cpal::SampleFormat::F32 => device.build_input_stream(
                    &config.into(),
                    move |data, _: &_| {
                        let data = data.to_vec();
                        sender.send(data).unwrap();
                    },
                    err_fn,
                    None,
                ),
                _ => panic!("Unsupported format"),
            }
            .unwrap();

            stream.play().unwrap();
        });

        self.handles = Some(vec![handle]);

        println!("Device started");
    }
}

impl RawDataLayer for Device {
    fn voice_stream_receiver(&self) -> crossbeam_channel::Receiver<Vec<f32>> {
        self.voice_stream_receiver.as_ref().unwrap().clone()
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate.unwrap()
    }

    fn start(&mut self) {
        self.run();
    }

    fn handle(&mut self) -> Vec<std::thread::JoinHandle<()>> {
        self.handles.take().unwrap()
    }
}
