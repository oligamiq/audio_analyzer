use crate::{libs::nodes::layer::extract_snarl_ui_pin_member, prelude::nodes::*};
use audio_analyzer_core::prelude::*;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum RawInputNodes {
    MicrophoneInputNode(MicrophoneInputNode),
    FileInputNode(FileInputNode),
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AbstractInputNode {
    hop_size: EditableOnText<usize>,

    input: RawInputNodes,
}

impl Default for AbstractInputNode {
    fn default() -> Self {
        AbstractInputNode {
            hop_size: EditableOnText::new(160),
            input: RawInputNodes::MicrophoneInputNode(MicrophoneInputNode::default()),
        }
    }
}

impl FlowNodesViewerTrait for AbstractInputNode {
    fn show_input(
        &self,
        ctx: &FlowNodesViewerCtx,
        pin: &egui_snarl::InPin,
        ui: &mut egui::Ui,
        _: f32,
        snarl: &egui_snarl::Snarl<FlowNodes>,
    ) -> Box<dyn Fn(&mut Snarl<FlowNodes>, &mut egui::Ui) -> MyPinInfo> {
        let pin_id = pin.id;

        match pin_id.input {
            0 => {
                if !ctx.running {
                    ui.label("order");

                    return Box::new(move |_, _| CustomPinInfo::none_status());
                }

                extract_snarl_ui_pin_member!(
                    snarl,
                    ui,
                    pin,
                    FlowNodes::AbstractInputNode(node),
                    node,
                    hop_size
                );
            }
            _ => unreachable!(),
        }
    }
}

impl AbstractInputNode {
    pub fn name(&self) -> &str {
        "AbstractInputNode"
    }

    pub fn get_sample_rate(&self) -> u32 {
        match &self.input {
            RawInputNodes::MicrophoneInputNode(node) => node.get_sample_rate(),
            RawInputNodes::FileInputNode(node) => node.get_sample_rate(),
        }
    }

    pub fn get(&self) -> Option<Array1<f64>> {
        match &self.input {
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

pub struct AbstractInputNodeInfo;

impl NodeInfo for AbstractInputNodeInfo {
    fn name(&self) -> &str {
        "AbstractInputNode"
    }

    fn inputs(&self) -> usize {
        1
    }

    fn outputs(&self) -> usize {
        2
    }

    fn input_types(&self) -> Vec<super::NodeInfoTypes> {
        vec![NodeInfoTypes::Number]
    }

    fn output_types(&self) -> Vec<super::NodeInfoTypes> {
        vec![NodeInfoTypes::Array1F64, NodeInfoTypes::Number]
    }

    fn flow_node(&self) -> super::editor::FlowNodes {
        super::editor::FlowNodes::AbstractInputNode(AbstractInputNode::default())
    }
}

impl GraphNode for AbstractInputNode {
    type NodeInfoType = AbstractInputNodeInfo;

    fn to_info(&self) -> Self::NodeInfoType {
        AbstractInputNodeInfo
    }

    fn update(&mut self) {
        match &mut self.input {
            RawInputNodes::MicrophoneInputNode(node) => match node {
                #[cfg(not(target_family = "wasm"))]
                MicrophoneInputNode::Device(node, vec) => {
                    *vec = node.try_recv().map(|x| x.into_iter().collect())
                }
                #[cfg(target_family = "wasm")]
                MicrophoneInputNode::WebAudioStream(node, vec) => {
                    *vec = node.try_recv().map(|x| x.into_iter().collect())
                }
            },
            RawInputNodes::FileInputNode(node) => {
                node.vec = node.data.try_recv().map(|x| x.into_iter().collect());
            }
        }
    }
}

#[derive(Debug)]
pub enum MicrophoneInputNode {
    #[cfg(not(target_family = "wasm"))]
    Device(Device, Option<Array1<f64>>),
    #[cfg(target_family = "wasm")]
    WebAudioStream(WebAudioStream, Option<Array1<f64>>),
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
            Variant::Device => Ok(MicrophoneInputNode::Device(Device::new(), None)),
            #[cfg(target_family = "wasm")]
            Variant::WebAudioStream => Ok(MicrophoneInputNode::WebAudioStream(
                WebAudioStream::new(),
                None,
            )),
            _ => {
                #[cfg(not(target_family = "wasm"))]
                {
                    Ok(MicrophoneInputNode::Device(Device::new(), None))
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
            MicrophoneInputNode::Device(_, _) => Variant::Device,
            #[cfg(target_family = "wasm")]
            MicrophoneInputNode::WebAudioStream(_, _) => Variant::WebAudioStream,
        };

        variant.serialize(serializer)
    }
}

impl MicrophoneInputNode {
    fn get_sample_rate(&self) -> u32 {
        match self {
            #[cfg(not(target_family = "wasm"))]
            MicrophoneInputNode::Device(node, _) => {
                node.get_sample_rate().expect("please start stream")
            }
            #[cfg(target_family = "wasm")]
            MicrophoneInputNode::WebAudioStream(node, _) => node.sample_rate(),
        }
    }

    // fn start(&mut self) {
    //     match self {
    //         #[cfg(not(target_family = "wasm"))]
    //         MicrophoneInputNode::Device(node, _) => node.start(),
    //         #[cfg(target_family = "wasm")]
    //         MicrophoneInputNode::WebAudioStream(node, _) => node.start(),
    //     }
    // }
}

#[derive(Debug, serde::Serialize)]
pub struct FileInputNode {
    pub file_path: EditableOnText<String>,

    vec: Option<Array1<f64>>,

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
    fn new(file_path: String) -> Self {
        Self {
            file_path: EditableOnText::new(file_path.clone()),
            data: TestData::new_with_path(file_path),
            vec: None,
        }
    }

    fn get_sample_rate(&self) -> u32 {
        self.data.sample_rate()
    }

    fn start(&mut self) {
        self.data.start();
    }
}
