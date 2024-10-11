use std::{sync::Arc, thread};

use crossbeam_channel::Receiver;
use parking_lot::Mutex;
use symphonia::core::{
    audio::Signal, codecs::DecoderOptions, formats::FormatOptions, io::MediaSourceStream,
    meta::MetadataOptions, probe,
};
use tracing::warn;

use crate::trace_dbg;

use super::RawDataStreamLayer;

#[derive(Debug, Clone, Copy)]
pub enum TestDataType {
    TestData1,
}

pub struct TestData {
    pub test_data_type: TestDataType,
    sample_rate: Option<u32>,
    sender: Arc<Mutex<Vec<crossbeam_channel::Sender<Vec<f32>>>>>,
    handles: Option<Vec<thread::JoinHandle<()>>>,
}

impl TestData {
    pub fn new(test_data_type: TestDataType) -> Self {
        TestData {
            test_data_type,
            sample_rate: None,
            sender: Arc::new(Mutex::new(Vec::new())),
            handles: None,
        }
    }

    pub fn voice_stream_receiver(&self) -> Receiver<Vec<f32>> {
        let (sender, receiver) = crossbeam_channel::unbounded();
        self.sender.lock().push(sender);
        receiver
    }

    pub fn start(&mut self) {
        let file_path = match self.test_data_type {
            TestDataType::TestData1 => "test_data/jfk_f32le.wav",
        };

        let file = Box::new(std::fs::File::open(file_path).unwrap());

        let mss = MediaSourceStream::new(file, Default::default());

        let hint = probe::Hint::new();

        let format_opts: FormatOptions = Default::default();
        let metadata_opts: MetadataOptions = Default::default();
        let decoder_opts: DecoderOptions = Default::default();

        let probed = symphonia::default::get_probe()
            .format(&hint, mss, &format_opts, &metadata_opts)
            .unwrap();

        // Get the format reader yielded by the probe operation.
        let mut format = probed.format;

        // Get the default track.
        let track = format.default_track().unwrap();

        let sample_rate = track.codec_params.sample_rate.unwrap();
        self.sample_rate = Some(sample_rate);

        // Create a decoder for the track.
        let mut decoder = symphonia::default::get_codecs()
            .make(&track.codec_params, &decoder_opts)
            .unwrap();

        // Store the track identifier, we'll use it to filter packets.
        let track_id = track.id;

        let sender = self.sender.clone();

        let handle = thread::spawn(move || {
            loop {
                // Get the next packet from the format reader.
                let packet = match format.next_packet() {
                    Ok(packet) => packet,
                    Err(e) => {
                        eprintln!("Error reading packet: {}", e);
                        break;
                    }
                };

                // If the packet does not belong to the selected track, skip it.
                if packet.track_id() != track_id {
                    continue;
                }

                // Decode the packet into audio samples, ignoring any decode errors.
                match decoder.decode(&packet) {
                    Ok(audio_buf) => {
                        let buf = match audio_buf {
                            symphonia::core::audio::AudioBufferRef::F32(buf) => {
                                // let frames = String::from(format!(
                                //     "###Decoded packet with {} samples",
                                //     buf.frames()
                                // ));
                                // trace_dbg!(frames);
                                buf
                            }
                            _ => {
                                warn!("###This is not f32");
                                continue;
                            }
                        };

                        let vec = buf.planes().planes()[0].to_vec();
                        sender.lock().retain(|x| x.send(vec.clone()).is_ok());
                    }
                    Err(symphonia::core::errors::Error::DecodeError(_)) => (),
                    Err(_) => break,
                }
            }
        });

        self.handles = Some(vec![handle]);
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate.unwrap()
    }
}

impl RawDataStreamLayer for TestData {
    fn voice_stream_receiver(&self) -> Receiver<Vec<f32>> {
        self.voice_stream_receiver()
    }
    fn start(&mut self) {
        self.start();
    }
    fn sample_rate(&self) -> u32 {
        self.sample_rate()
    }
    fn handle(&mut self) -> Vec<std::thread::JoinHandle<()>> {
        self.handles.take().unwrap()
    }
}
