#[cfg(not(target_family = "wasm"))]
use audio_analyzer_core::data::device_stream::Device;
use audio_analyzer_core::data::{test_data::TestData, RawDataStreamLayer};

#[cfg(target_family = "wasm")]
use audio_analyzer_core::data::web_stream::WebAudioStream;

use egui_editable_num::EditableOnText;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum RawInputNodes {
    MicrophoneInputNode(MicrophoneInputNode),
    FileInputNode(FileInputNode),
}

impl RawInputNodes {
    pub fn name(&self) -> &str {
        match self {
            RawInputNodes::MicrophoneInputNode(_) => "MicrophoneInputNode",
            RawInputNodes::FileInputNode(_) => "FileInputNode",
        }
    }

    pub fn get_sample_rate(&mut self) -> u32 {
        match self {
            RawInputNodes::MicrophoneInputNode(node) => node.get_sample_rate(),
            RawInputNodes::FileInputNode(node) => node.get_sample_rate(),
        }
    }

    pub const fn inputs(&self) -> usize {
        match self {
            RawInputNodes::MicrophoneInputNode(node) => node.inputs(),
            RawInputNodes::FileInputNode(node) => node.inputs(),
        }
    }

    pub const fn outputs(&self) -> usize {
        1
    }
}

#[derive(Debug)]
pub enum MicrophoneInputNode {
    #[cfg(not(target_family = "wasm"))]
    Device(Device),
    #[cfg(target_family = "wasm")]
    WebAudioStream(WebAudioStream),
}

impl Default for MicrophoneInputNode {
    fn default() -> Self {
        #[cfg(not(target_family = "wasm"))]
        {
            MicrophoneInputNode::Device(Device::new())
        }
        #[cfg(target_family = "wasm")]
        {
            MicrophoneInputNode::WebAudioStream(WebAudioStream::new())
        }
    }
}

impl<'a> serde::Deserialize<'a> for MicrophoneInputNode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'a>,
    {
        #[derive(serde::Deserialize)]
        enum Variant {
            Device,
            WebAudioStream,
        }

        let variant = Variant::deserialize(deserializer)?;

        match variant {
            #[cfg(not(target_family = "wasm"))]
            Variant::Device => Ok(MicrophoneInputNode::Device(Device::new())),
            #[cfg(target_family = "wasm")]
            Variant::WebAudioStream => {
                Ok(MicrophoneInputNode::WebAudioStream(WebAudioStream::new()))
            }
            _ => {
                #[cfg(not(target_family = "wasm"))]
                {
                    Ok(MicrophoneInputNode::Device(Device::new()))
                }
                #[cfg(target_family = "wasm")]
                {
                    Ok(MicrophoneInputNode::WebAudioStream(WebAudioStream::new()))
                }
            }
        }
    }
}

impl<'a> serde::Serialize for MicrophoneInputNode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[derive(serde::Serialize)]
        enum Variant {
            Device,
            WebAudioStream,
        }

        let variant = match self {
            #[cfg(not(target_family = "wasm"))]
            MicrophoneInputNode::Device(_) => Variant::Device,
            #[cfg(target_family = "wasm")]
            MicrophoneInputNode::WebAudioStream(_) => Variant::WebAudioStream,
        };

        variant.serialize(serializer)
    }
}

impl MicrophoneInputNode {
    pub fn get_sample_rate(&mut self) -> u32 {
        match self {
            #[cfg(not(target_family = "wasm"))]
            MicrophoneInputNode::Device(node) => {
                node.get_sample_rate().expect("please start stream")
            }
            #[cfg(target_family = "wasm")]
            MicrophoneInputNode::WebAudioStream(node) => node.sample_rate(),
        }
    }

    pub fn start(&mut self) {
        match self {
            #[cfg(not(target_family = "wasm"))]
            MicrophoneInputNode::Device(node) => node.start(),
            #[cfg(target_family = "wasm")]
            MicrophoneInputNode::WebAudioStream(node) => node.start(),
        }
    }

    pub const fn inputs(&self) -> usize {
        0
    }

    pub const fn outputs(&self) -> usize {
        1
    }

    pub fn try_recv(&mut self) -> Option<Vec<f32>> {
        match self {
            #[cfg(not(target_family = "wasm"))]
            MicrophoneInputNode::Device(node) => node.try_recv(),
            #[cfg(target_family = "wasm")]
            MicrophoneInputNode::WebAudioStream(node) => node.try_recv(),
        }
    }
}

#[derive(Debug, serde::Serialize)]
pub struct FileInputNode {
    pub file_path: EditableOnText<String>,

    #[serde(skip)]
    data: TestData,
}

impl Default for FileInputNode {
    fn default() -> Self {
        Self::new("jfk_f32le.wav".to_string())
    }
}

impl<'a> serde::Deserialize<'a> for FileInputNode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'a>,
    {
        #[derive(serde::Deserialize)]
        struct FileInputNodeHelper {
            file_path: EditableOnText<String>,
        }

        let helper = FileInputNodeHelper::deserialize(deserializer)?;

        Ok(FileInputNode::new(helper.file_path.as_ref().clone()))
    }
}

impl FileInputNode {
    pub fn new(file_path: String) -> Self {
        Self {
            file_path: EditableOnText::new(file_path.clone()),
            data: TestData::new_with_path(file_path),
        }
    }

    pub fn get_sample_rate(&mut self) -> u32 {
        self.data.sample_rate()
    }

    pub fn start(&mut self) {
        self.data.start();
    }

    pub const fn inputs(&self) -> usize {
        1
    }

    pub const fn outputs(&self) -> usize {
        1
    }

    pub fn try_recv(&mut self) -> Option<Vec<f32>> {
        self.data.try_recv()
    }
}
