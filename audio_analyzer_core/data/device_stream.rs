use std::sync::Arc;

use cpal::traits::{DeviceTrait as _, HostTrait, StreamTrait as _};
use log::debug;
use log::error;
use parking_lot::Mutex;

use super::RawDataStreamLayer;

pub struct Device {
    _host: cpal::Host,
    device: Option<cpal::Device>,
    data: Arc<Mutex<Vec<f32>>>,
    sample_rate: Option<u32>,
    stream: Option<cpal::Stream>,
}

impl Device {
    pub fn new() -> Self {
        let host = cpal::default_host();
        let device = host.default_input_device().unwrap();

        Device {
            _host: host,
            device: Some(device),
            sample_rate: None,
            data: Arc::new(Mutex::new(Vec::new())),
            stream: None,
        }
    }

    pub fn get_sample_rate(&self) -> Option<u32> {
        self.sample_rate
    }

    pub fn run(&mut self) {
        let device = self.device.take().unwrap();

        debug!("{:?}", device.name().unwrap());

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
            error!("an error occurred on stream: {}", err);
        };

        let data = self.data.clone();

        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => device.build_input_stream(
                &config.into(),
                move |_data: &[f32], _: &_| {
                    let _data = _data.to_vec();
                    if _data.len() == 0 {
                        return;
                    }

                    // tracing::debug!("##data: {:?}", _data);

                    let mut data = data.lock();
                    data.extend(_data);
                    std::mem::drop(data);
                },
                err_fn,
                None,
            ),
            _ => panic!("Unsupported format"),
        }
        .unwrap();

        stream.play().unwrap();

        self.stream = Some(stream);
    }
}

impl RawDataStreamLayer for Device {
    fn sample_rate(&self) -> u32 {
        self.sample_rate.unwrap()
    }

    fn start(&mut self) {
        self.run();
    }

    fn try_recv(&mut self) -> Option<Vec<f32>> {
        // tracing::debug!("try_recv: {:?}", self.data);

        let mut data = self.data.lock();
        let _void = Vec::new();
        let data = std::mem::replace(&mut *data, _void);

        if data.len() == 0 {
            return None;
        }

        Some(data)
    }
}
