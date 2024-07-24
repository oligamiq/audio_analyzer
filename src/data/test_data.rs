use std::thread;

use crossbeam_channel::Receiver;
use symphonia::core::{audio::{AudioBuffer, SampleBuffer}, codecs::DecoderOptions, conv::IntoSample, formats::FormatOptions, io::MediaSourceStream, meta::MetadataOptions, probe};
use tracing_subscriber::fmt::format::Format;

use super::RawDataLayer;

#[derive(Debug, Clone, Copy)]
pub enum TestDataType {
    TestData1,
}

pub struct TestData {
    pub test_data_type: TestDataType,
    sample_rate: Option<u32>,
    voice_stream_receiver: Option<Receiver<Vec<f32>>>,
    handles: Option<Vec<thread::JoinHandle<()>>>,
}

impl TestData {
    pub fn new(test_data_type: TestDataType) -> Self {
        TestData {
            test_data_type,
            sample_rate: None,
            voice_stream_receiver: None,
            handles: None,
        }
    }

    pub fn voice_stream_receiver(&self) -> Receiver<Vec<f32>> {

    }

    pub fn start(&mut self) {
        let file_path = match self.test_data_type {
            TestDataType::TestData1 => "test_data/jfk_f32le.wav",
        };
        let (sender, receiver) = crossbeam_channel::unbounded();

        self.voice_stream_receiver = Some(receiver);

        let file = Box::new(std::fs::File::open(file_path).unwrap());

        let mss = MediaSourceStream::new(file, Default::default());

        let hint = probe::Hint::new();

        let format_opts: FormatOptions = Default::default();
        let metadata_opts: MetadataOptions = Default::default();
        let decoder_opts: DecoderOptions = Default::default();

        let probed =
            symphonia::default::get_probe().format(&hint, mss, &format_opts, &metadata_opts).unwrap();

        // Get the format reader yielded by the probe operation.
        let mut format = probed.format;

        // Get the default track.
        let track = format.default_track().unwrap();

        // Create a decoder for the track.
        let mut decoder =
            symphonia::default::get_codecs().make(&track.codec_params, &decoder_opts).unwrap();

        // Store the track identifier, we'll use it to filter packets.
        let track_id = track.id;

        loop {
            // Get the next packet from the format reader.
            let packet = format.next_packet().unwrap();

            // If the packet does not belong to the selected track, skip it.
            if packet.track_id() != track_id {
                continue;
            }

            // Decode the packet into audio samples, ignoring any decode errors.
            match decoder.decode(&packet) {
                Ok(audio_buf) => {
                    // If the packet was decoded successfully, send the audio samples to the
                    // receiver.
                    let samples: AudioBuffer<f32> = audio_buf.make_equivalent();
                    let mut buffer = Vec::with_capacity(samples.capacity());
                    for sample in samples {
                        buffer.push(sample[0]);
                    }
                    sender.send(buffer).unwrap();
                }
                Err(symphonia::core::errors::Error::DecodeError(_)) => (),
                Err(_) => break,
            }
        }
    }

    pub fn sample_rate(&self) -> u32 {
        match self {
            TestDataType::TestData1 => 48000,
        }
    }
}

impl RawDataLayer for TestDataType {
    fn voice_stream_receiver(&self) -> Receiver<Vec<f32>> {
        self.voice_stream_receiver()
    }
    fn start(&mut self) {
        self.start();
    }
    fn sample_rate(&self) -> u32 {
        self.sample_rate()
    }

}
