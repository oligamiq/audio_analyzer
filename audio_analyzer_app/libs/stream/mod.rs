use audio_analyzer_core::{
    data::{device_stream, RawDataStreamLayer as _},
    layer::layers::MultipleLayers,
    mel_layer::{
        fft_layer::{FftConfig, ToSpectrogramLayer},
        spectral_density::{ToPowerSpectralDensityLayer, ToPowerSpectralDensityLayerConfig},
        to_mel_layer::ToMelSpectrogramLayer,
    },
};
use log::debug;
use mel_spec::config::MelConfig;
use streams::Streamer;

pub mod streams;

pub fn new_stream() -> Streamer {
    #[cfg(target_family = "wasm")]
    let mut raw_data_layer = audio_analyzer_core::data::web_stream::WebAudioStream::new();
    #[cfg(not(target_family = "wasm"))]
    let mut raw_data_layer = device_stream::Device::new();
    // let mut raw_data_layer = TestData::new(TestDataType::TestData1);

    raw_data_layer.start();

    let sample_rate = raw_data_layer.sample_rate();

    debug!("sample rate: {}", sample_rate);
    let fft_layer = ToSpectrogramLayer::new(FftConfig::new(400, 160));
    let mel_layer = ToMelSpectrogramLayer::new(MelConfig::new(400, 160, 80, sample_rate.into()));
    let psd_layer = ToPowerSpectralDensityLayer::new(ToPowerSpectralDensityLayerConfig {
        sample_rate: sample_rate.into(),
        time_range: 20,
        n_mels: 80,
    });

    let mut layers = MultipleLayers::default();
    layers.push_layer(fft_layer);
    layers.push_layer(mel_layer);
    layers.push_layer(psd_layer);
    debug!("{:?}", layers);

    Streamer::new(Box::new(raw_data_layer), layers)
}
