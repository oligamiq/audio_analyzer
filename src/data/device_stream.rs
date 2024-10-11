use std::{sync::Arc, thread};

use cpal::traits::{DeviceTrait as _, HostTrait, StreamTrait as _};
use parking_lot::Mutex;

use crate::trace_dbg;

use super::RawDataStreamLayer;

pub struct Device {
    _host: cpal::Host,
    device: Option<cpal::Device>,
    sample_rate: Option<u32>,
    sender: Arc<Mutex<Vec<crossbeam_channel::Sender<Vec<f32>>>>>,
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
            sender: Arc::new(Mutex::new(Vec::new())),
            handles: None,
        }
    }

    pub fn get_sample_rate(&self) -> Option<u32> {
        self.sample_rate
    }

    pub fn run(&mut self) {
        let device = self.device.take().unwrap();

        trace_dbg!(device.name().unwrap());

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

        // dbg!(&config);

        self.sample_rate = Some(config.sample_rate().0);

        let err_fn = move |err| {
            eprintln!("an error occurred on stream: {}", err);
        };

        let sender = self.sender.clone();

        let handle = thread::spawn(move || {
            let stream = match config.sample_format() {
                cpal::SampleFormat::F32 => device.build_input_stream(
                    &config.into(),
                    move |data, _: &_| {
                        let data = data.to_vec();
                        if data.len() == 0 {
                            return;
                        }
                        // trace_dbg!(data.len());
                        sender.lock().retain(|x| x.send(data.clone()).is_ok());
                    },
                    err_fn,
                    None,
                ),
                _ => panic!("Unsupported format"),
            }
            .unwrap();

            stream.play().unwrap();

            loop {}
        });

        self.handles = Some(vec![handle]);
    }
}

impl RawDataStreamLayer for Device {
    fn voice_stream_receiver(&self) -> crossbeam_channel::Receiver<Vec<f32>> {
        let (sender, receiver) = crossbeam_channel::unbounded();
        self.sender.lock().push(sender);
        receiver
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
