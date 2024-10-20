#[cfg(not(target_family = "wasm"))]
use audio_analyzer_core::data::device_stream::Device;
use audio_analyzer_core::data::{test_data::TestData, RawDataStreamLayer};

#[cfg(target_family = "wasm")]
use audio_analyzer_core::data::web_stream::WebAudioStream;

use egui_editable_num::EditableOnText;

use super::NodeInfo;

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

    pub fn update(&mut self) {
        match self {
            RawInputNodes::MicrophoneInputNode(node) => node.update(),
            RawInputNodes::FileInputNode(node) => node.update(),
        }
    }

    pub fn get(&self) -> Option<Vec<f32>> {
        match self {
            RawInputNodes::MicrophoneInputNode(node) => match node {
                #[cfg(not(target_family = "wasm"))]
                MicrophoneInputNode::Device(_, vec) => vec.clone(),
                #[cfg(target_family = "wasm")]
                MicrophoneInputNode::WebAudioStream(_, vec) => vec.clone(),
            },
            RawInputNodes::FileInputNode(node) => node.vec.clone(),
        }
    }
}

#[derive(Debug)]
pub enum MicrophoneInputNode {
    #[cfg(not(target_family = "wasm"))]
    Device(Device, Option<Vec<f32>>),
    #[cfg(target_family = "wasm")]
    WebAudioStream(WebAudioStream, Option<Vec<f32>>),
}

pub struct MicrophoneInputNodeInfo;

impl NodeInfo for MicrophoneInputNodeInfo {
    fn name(&self) -> &str {
        "MicrophoneInputNode"
    }

    fn inputs(&self) -> usize {
        0
    }

    fn outputs(&self) -> usize {
        1
    }

    fn input_types(&self) -> Vec<super::NodeInfoTypes> {
        vec![]
    }

    fn output_types(&self) -> Vec<super::NodeInfoTypes> {
        vec![super::NodeInfoTypes::VecF32]
    }

    fn flow_node(&self) -> super::editor::FlowNodes {
        super::editor::FlowNodes::RawInputNodes(RawInputNodes::MicrophoneInputNode(
            MicrophoneInputNode::default(),
        ))
    }
}

impl Default for MicrophoneInputNode {
    fn default() -> Self {
        #[cfg(not(target_family = "wasm"))]
        {
            MicrophoneInputNode::Device(Device::new(), None)
        }
        #[cfg(target_family = "wasm")]
        {
            MicrophoneInputNode::WebAudioStream(WebAudioStream::new(), None)
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
            Variant::WebAudioStream => Ok(MicrophoneInputNode::WebAudioStream(
                WebAudioStream::new(),
                None,
            )),
            _ => {
                #[cfg(not(target_family = "wasm"))]
                {
                    Ok(MicrophoneInputNode::Device(Device::new()))
                }
                #[cfg(target_family = "wasm")]
                {
                    Ok(MicrophoneInputNode::WebAudioStream(
                        WebAudioStream::new(),
                        None,
                    ))
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
        #[allow(dead_code)]
        #[derive(serde::Serialize)]
        enum Variant {
            Device,
            WebAudioStream,
        }

        let variant = match self {
            #[cfg(not(target_family = "wasm"))]
            MicrophoneInputNode::Device(_) => Variant::Device,
            #[cfg(target_family = "wasm")]
            MicrophoneInputNode::WebAudioStream(_, _) => Variant::WebAudioStream,
        };

        variant.serialize(serializer)
    }
}

impl MicrophoneInputNode {
    pub fn get_sample_rate(&mut self) -> u32 {
        match self {
            #[cfg(not(target_family = "wasm"))]
            MicrophoneInputNode::Device(node, _) => {
                node.get_sample_rate().expect("please start stream")
            }
            #[cfg(target_family = "wasm")]
            MicrophoneInputNode::WebAudioStream(node, _) => node.sample_rate(),
        }
    }

    pub fn start(&mut self) {
        match self {
            #[cfg(not(target_family = "wasm"))]
            MicrophoneInputNode::Device(node, _) => node.start(),
            #[cfg(target_family = "wasm")]
            MicrophoneInputNode::WebAudioStream(node, _) => node.start(),
        }
    }

    pub const fn inputs(&self) -> usize {
        0
    }

    pub const fn outputs(&self) -> usize {
        1
    }

    pub fn update(&mut self) {
        match self {
            #[cfg(not(target_family = "wasm"))]
            MicrophoneInputNode::Device(node, vec) => *vec = node.try_recv(),
            #[cfg(target_family = "wasm")]
            MicrophoneInputNode::WebAudioStream(node, vec) => *vec = node.try_recv(),
        }
    }

    pub fn to_info(&self) -> MicrophoneInputNodeInfo {
        MicrophoneInputNodeInfo
    }
}

#[derive(Debug, serde::Serialize)]
pub struct FileInputNode {
    pub file_path: EditableOnText<String>,

    vec: Option<Vec<f32>>,

    #[serde(skip)]
    data: TestData,
}

pub struct FileInputNodeInfo;

impl NodeInfo for FileInputNodeInfo {
    fn name(&self) -> &str {
        "FileInputNode"
    }

    fn inputs(&self) -> usize {
        1
    }

    fn outputs(&self) -> usize {
        1
    }

    fn input_types(&self) -> Vec<super::NodeInfoTypes> {
        vec![]
    }

    fn output_types(&self) -> Vec<super::NodeInfoTypes> {
        vec![super::NodeInfoTypes::VecF32]
    }

    fn flow_node(&self) -> super::editor::FlowNodes {
        super::editor::FlowNodes::RawInputNodes(RawInputNodes::FileInputNode(
            FileInputNode::default(),
        ))
    }
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
            vec: None,
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

    pub fn update(&mut self) {
        self.vec = self.data.try_recv();
    }

    pub fn to_info(&self) -> FileInputNodeInfo {
        FileInputNodeInfo
    }
}
