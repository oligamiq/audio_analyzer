use std::sync::Arc;

use color_eyre::eyre::ContextCompat;
use crossbeam_channel::{Receiver, Sender};
use mel_spec::{config::MelConfig, vad::DetectionSettings};
use mel_spec_pipeline::pipeline::{AudioConfig, MelFrame, Pipeline, PipelineConfig, VadResult};

use crate::trace_dbg;

use super::MelLayer;

pub struct DefaultMelLayer {
    mel_config: Option<MelConfig>,
    detection_settings: Option<DetectionSettings>,
    audio_config: Option<AudioConfigBuilder>,
    pipeline: Option<Arc<Pipeline>>,
    handles: Option<Vec<std::thread::JoinHandle<()>>>,
    sender: Option<Sender<Vec<f32>>>,
}

#[derive(Clone)]
pub struct AudioConfigBuilder {
    bit_depth: usize,
    sampling_rate: f64,
}

impl MelLayer for DefaultMelLayer {
    fn voice_stream_sender(&self) -> Sender<Vec<f32>> {
        self.sender
            .clone()
            .wrap_err("Sender is not initialized")
            .unwrap()
    }
    fn mel_frame_stream_receiver(&self) -> Receiver<MelFrame> {
        let receiver = self.pipeline.as_ref().unwrap().mel_rx();
        receiver
    }
    fn vad_rx_stream_receiver(&self) -> Receiver<VadResult> {
        let receiver = self.pipeline.as_ref().unwrap().vad_rx();
        receiver
    }
    fn handle(&mut self) -> Vec<std::thread::JoinHandle<()>> {
        self.handles.take().unwrap()
    }
    fn start(&mut self) {
        self.start();
    }
    fn set_sampling_rate(&mut self, sampling_rate: f64) {
        self.set_sampling_rate(sampling_rate);
    }
}

impl DefaultMelLayer {
    pub fn new() -> Self {
        DefaultMelLayer {
            mel_config: Some(MelConfig::new(400, 160, 80, 16000.0)),
            detection_settings: Some(DetectionSettings::new(1.0, 3, 6, 0)),
            audio_config: Some(AudioConfigBuilder { bit_depth: 32, sampling_rate: 16000.0 }),
            pipeline: None,
            handles: None,
            sender: None,
        }
    }

    pub fn borrow_mel_config(&mut self) -> &mut MelConfig {
        self.mel_config.as_mut().unwrap()
    }

    pub fn borrow_detection_settings(&mut self) -> &mut DetectionSettings {
        self.detection_settings.as_mut().unwrap()
    }

    pub fn borrow_audio_config(&mut self) -> &mut AudioConfigBuilder {
        self.audio_config.as_mut().unwrap()
    }

    pub fn set_sampling_rate(&mut self, sampling_rate: f64) {
        self.audio_config.as_mut().unwrap().sampling_rate = sampling_rate;

        let mel_config = self.mel_config.as_mut().unwrap();
        let new_mel_config = MelConfig::new(
            mel_config.fft_size(),
            mel_config.hop_size(),
            mel_config.n_mels(),
            sampling_rate,
        );
        *mel_config = new_mel_config;
    }

    pub fn start(&mut self) {
        let Self {
            mel_config,
            detection_settings,
            audio_config,
            ..
        } = self;

        let audio_config = audio_config.clone().unwrap();

        let config = PipelineConfig::new(
            AudioConfig::new(audio_config.bit_depth, audio_config.sampling_rate),
            mel_config.take().unwrap(),
            detection_settings.take().unwrap(),
        );

        let mut pipeline = Pipeline::new(config);

        let mut handles = pipeline.start();

        let pipeline = Arc::new(pipeline);

        self.pipeline = Some(pipeline.clone());

        let (sender, receiver) = crossbeam_channel::unbounded();
        self.sender = Some(sender);

        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: self.audio_config.as_ref().unwrap().sampling_rate as u32,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut writer = hound::WavWriter::create("sine.wav", spec).unwrap();

        let handle = std::thread::spawn(move || loop {
            let data = receiver.recv().unwrap();
            // trace_dbg!(data.len());
            if data.len() == 0 {
                continue;
            }

            trace_dbg!(&data);
            for sample in &data {
                let amplitude = i16::MAX as f32;
                writer.write_sample((*sample * amplitude) as i16).unwrap();
            }

            match pipeline.send_pcm(data.as_slice()) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Error sending data to pipeline: {}", e);
                }
            }
        });
        handles.push(handle);

        self.handles = Some(handles);
    }
}
