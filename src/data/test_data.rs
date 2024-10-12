use symphonia::core::{
    codecs::{Decoder, DecoderOptions},
    formats::{FormatOptions, FormatReader},
    io::MediaSourceStream,
    meta::MetadataOptions,
    probe,
};
use tracing::{error, warn};

use super::RawDataStreamLayer;

#[derive(Debug, Clone, Copy)]
pub enum TestDataType {
    TestData1,
}

pub struct TestData {
    pub test_data_type: TestDataType,
    sample_rate: Option<u32>,
    format: Option<Box<dyn FormatReader>>,
    track_id: Option<u32>,
    decoder: Option<Box<dyn Decoder>>,
}

impl TestData {
    pub fn new(test_data_type: TestDataType) -> Self {
        TestData {
            test_data_type,
            sample_rate: None,
            format: None,
            track_id: None,
            decoder: None,
        }
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
        let format = probed.format;

        // Get the default track.
        let track = format.default_track().unwrap();

        let sample_rate = track.codec_params.sample_rate.unwrap();
        self.sample_rate = Some(sample_rate);

        // Create a decoder for the track.
        let decoder = symphonia::default::get_codecs()
            .make(&track.codec_params, &decoder_opts)
            .unwrap();

        // Store the track identifier, we'll use it to filter packets.
        let track_id = track.id;

        self.format = Some(format);
        self.track_id = Some(track_id);
        self.decoder = Some(decoder);
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate.unwrap()
    }
}

impl RawDataStreamLayer for TestData {
    fn start(&mut self) {
        self.start();
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate()
    }

    fn try_recv(&mut self) -> Option<Vec<f32>> {
        let Self {
            format,
            track_id,
            decoder,
            ..
        } = self;

        let format = format.as_mut()?;
        let track_id = *track_id.as_ref()?;
        let decoder = decoder.as_mut()?;

        // Get the next packet from the format reader.
        let packet = match format.next_packet() {
            Ok(packet) => packet,
            Err(e) => {
                error!("###Error: {:?}", e);
                return None;
            }
        };

        // If the packet does not belong to the selected track, skip it.
        if packet.track_id() != track_id {
            return None;
        }

        // Decode the packet into audio samples, ignoring any decode errors.
        match decoder.decode(&packet) {
            Ok(audio_buf) => {
                let buf = match audio_buf {
                    symphonia::core::audio::AudioBufferRef::F32(buf) => buf,
                    _ => {
                        warn!("###This is not f32");
                        return None;
                    }
                };

                let vec = buf.planes().planes()[0].to_vec();
                Some(vec)
            }
            Err(symphonia::core::errors::Error::DecodeError(_)) => {
                warn!("###Decode error");
                return None;
            }
            Err(_) => return None,
        }
    }
}
