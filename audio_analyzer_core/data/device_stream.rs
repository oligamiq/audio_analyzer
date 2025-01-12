use std::fmt::Debug;
use std::sync::Arc;

use cpal::traits::{DeviceTrait as _, HostTrait, StreamTrait as _};
use log::debug;
use log::error;
use parking_lot::Mutex;

use super::RawDataStreamLayer;

pub struct Device {
    _host: cpal::Host,
    device: Option<cpal::Device>,
    data: Arc<Mutex<Vec<f64>>>,
    sample_rate: Option<u32>,
    stream: Option<cpal::Stream>,
}

impl Debug for Device {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Device")
            .field("data", &self.data)
            .field("sample_rate", &self.sample_rate)
            .finish()
    }
}

impl Device {
    pub fn new() -> Self {
        let host = cpal::default_host();
        let device = host.default_input_device().map(|device| {
            let mut sl = Device {
                _host: host,
                device: Some(device),
                sample_rate: None,
                data: Arc::new(Mutex::new(Vec::new())),
                stream: None,
            };

            sl.start();

            sl
        });

        match device {
            Some(device) => device,
            None => {
                log::warn!("no input device found");
                Device {
                    _host: cpal::default_host(),
                    device: None,
                    sample_rate: None,
                    data: Arc::new(Mutex::new(Vec::new())),
                    stream: None,
                }
            }
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

        let callback = move |_data: Vec<f64>| {
            if _data.len() == 0 {
                return;
            }

            let mut data = data.lock();
            data.extend(_data);
            std::mem::drop(data);
        };

        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => device.build_input_stream(
                &config.into(),
                move |_data: &[f32], _: &_| {
                    callback(_data.iter().map(|x| *x as f64).collect());
                },
                err_fn,
                None,
            ),
            cpal::SampleFormat::F64 => device.build_input_stream(
                &config.into(),
                move |_data: &[f64], _: &_| {
                    callback(_data.to_vec());
                },
                err_fn,
                None,
            ),
            cpal::SampleFormat::I8 => device.build_input_stream(
                &config.into(),
                move |_data: &[i8], _: &_| {
                    callback(
                        _data
                            .iter()
                            .map(|x| *x as f64 / std::i8::MAX as f64)
                            .collect(),
                    );
                },
                err_fn,
                None,
            ),
            cpal::SampleFormat::U8 => device.build_input_stream(
                &config.into(),
                move |_data: &[u8], _: &_| {
                    callback(
                        _data
                            .iter()
                            .map(|x| *x as f64 / std::u8::MAX as f64)
                            .collect(),
                    );
                },
                err_fn,
                None,
            ),
            cpal::SampleFormat::I16 => device.build_input_stream(
                &config.into(),
                move |_data: &[i16], _: &_| {
                    callback(
                        _data
                            .iter()
                            .map(|x| *x as f64 / std::i16::MAX as f64)
                            .collect(),
                    );
                },
                err_fn,
                None,
            ),
            cpal::SampleFormat::U16 => device.build_input_stream(
                &config.into(),
                move |_data: &[u16], _: &_| {
                    callback(
                        _data
                            .iter()
                            .map(|x| *x as f64 / std::u16::MAX as f64)
                            .collect(),
                    );
                },
                err_fn,
                None,
            ),
            cpal::SampleFormat::I32 => device.build_input_stream(
                &config.into(),
                move |_data: &[i32], _: &_| {
                    callback(
                        _data
                            .iter()
                            .map(|x| *x as f64 / std::i32::MAX as f64)
                            .collect(),
                    );
                },
                err_fn,
                None,
            ),
            cpal::SampleFormat::U32 => device.build_input_stream(
                &config.into(),
                move |_data: &[u32], _: &_| {
                    callback(
                        _data
                            .iter()
                            .map(|x| *x as f64 / std::u32::MAX as f64)
                            .collect(),
                    );
                },
                err_fn,
                None,
            ),
            cpal::SampleFormat::I64 => device.build_input_stream(
                &config.into(),
                move |_data: &[i64], _: &_| {
                    callback(
                        _data
                            .iter()
                            .map(|x| *x as f64 / std::i64::MAX as f64)
                            .collect(),
                    );
                },
                err_fn,
                None,
            ),
            cpal::SampleFormat::U64 => device.build_input_stream(
                &config.into(),
                move |_data: &[u64], _: &_| {
                    callback(
                        _data
                            .iter()
                            .map(|x| *x as f64 / std::u64::MAX as f64)
                            .collect(),
                    );
                },
                err_fn,
                None,
            ),
            _ => unreachable!(),
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

    fn try_recv(&mut self) -> Option<Vec<f64>> {
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
