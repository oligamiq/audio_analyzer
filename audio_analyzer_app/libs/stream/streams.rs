use audio_analyzer_core::{
    data::RawDataStreamLayer,
    layer::{layers::MultipleLayers, Layer as _},
};
use log::debug;
use ndarray::{Array1, Axis as npAxis};
use std::fmt::Debug;
use std::{
    any::Any,
    thread,
    time::{Duration, Instant},
};

use crate::Result;

//   Vec<f32>
// 音声ストリーム -> スペクトル -> メルスペクトル -> メルケプストラム

pub struct Streamer {
    input_stream: Box<dyn RawDataStreamLayer>,
    layer: MultipleLayers,
    mel_psd_data: Vec<(f64, f64)>,
}

impl Streamer {
    pub fn new(input_stream: Box<dyn RawDataStreamLayer>, layer: MultipleLayers) -> Self {
        Self {
            input_stream,
            layer,
            mel_psd_data: vec![],
        }
    }

    pub fn apply(&mut self) {
        if let Some(data) = self.input_stream.try_recv() {
            if let Ok(mel_data) = self.layer.through(&data as &dyn Any) {
                if mel_data.len() != 0 {
                    // debug!("{:?}", mel_data);
                }

                if let Some(mel_data) = mel_data
                    .iter()
                    .last()
                    .map(|x| x.downcast_ref::<Array1<(f64, f64)>>().unwrap())
                    .clone()
                {
                    // debug!("{:?}", mel_data);

                    self.mel_psd_data = mel_data.to_vec();

                    // debug!("{:?}", self.mel_psd_data);
                }
                // debug!(mel_data);
            }
        }
    }
}
