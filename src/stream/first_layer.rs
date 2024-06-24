use std::sync::Arc;

use color_eyre::eyre::ContextCompat;
use crossbeam_channel::{Receiver, Sender};
use mel_spec::{config::MelConfig, vad::DetectionSettings};
use mel_spec_pipeline::pipeline::{AudioConfig, MelFrame, Pipeline, PipelineConfig};

use super::MelLayer;

pub struct DefaultMelLayer {
    mel_config: Option<MelConfig>,
    detection_settings: Option<DetectionSettings>,
    audio_config: Option<AudioConfig>,
    pipeline: Option<Arc<Pipeline>>,
    handles: Option<Vec<std::thread::JoinHandle<()>>>,
    sender: Option<Sender<Vec<f32>>>,
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
    fn handle(&mut self) -> Vec<std::thread::JoinHandle<()>> {
        self.handles.take().unwrap()
    }
    fn start(&mut self) {
        self.start();
    }
}

impl DefaultMelLayer {
    pub fn new() -> Self {
        DefaultMelLayer {
            mel_config: Some(MelConfig::new(400, 160, 80, 16000.0)),
            detection_settings: Some(DetectionSettings::new(1.0, 3, 6, 0)),
            audio_config: Some(AudioConfig::new(32, 16000.0)),
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

    pub fn borrow_audio_config(&mut self) -> &mut AudioConfig {
        self.audio_config.as_mut().unwrap()
    }

    pub fn start(&mut self) {
        let Self {
            mel_config,
            detection_settings,
            audio_config,
            ..
        } = self;

        let config = PipelineConfig::new(
            audio_config.take().unwrap(),
            mel_config.take().unwrap(),
            detection_settings.take().unwrap(),
        );

        let mut pipeline = Pipeline::new(config);

        let mut handles = pipeline.start();

        let pipeline = Arc::new(pipeline);

        self.pipeline = Some(pipeline.clone());

        let (sender, receiver) = crossbeam_channel::unbounded();
        self.sender = Some(sender);
        let handle = std::thread::spawn(move || loop {
            let data = receiver.recv().unwrap();
            pipeline.send_pcm(data.as_slice()).unwrap();
        });
        handles.push(handle);

        self.handles = Some(handles);
    }
}
