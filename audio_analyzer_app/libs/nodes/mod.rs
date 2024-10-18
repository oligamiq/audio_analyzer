use config::NumberNodeInfo;
use layer::{MelLayerNodeInfo, STFTLayerNodeInfo, SpectrogramDensityLayerNodeInfo};

pub mod config;
pub mod editor;
pub mod layer;
pub mod pin_info;
pub mod raw_input;
pub mod utils;
pub mod viewer;

pub trait NodeInfo {
    fn name(&self) -> &str;
    fn inputs(&self) -> usize;
    fn outputs(&self) -> usize;
    fn input_types(&self) -> Vec<NodeInfoTypes>;
    fn output_types(&self) -> Vec<NodeInfoTypes>;
    fn flow_node(&self) -> editor::FlowNodes;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeInfoTypes {
    Number,
    VecF32,
    Array1TupleF64F64,
    Array2F64,
    Array1ComplexF64,
}

pub struct NodeInfos;

impl NodeInfos {
    pub fn all() -> Vec<Box<dyn NodeInfo>> {
        vec![
            Box::new(NumberNodeInfo),
            Box::new(STFTLayerNodeInfo),
            Box::new(MelLayerNodeInfo),
            Box::new(SpectrogramDensityLayerNodeInfo),
            Box::new(raw_input::MicrophoneInputNodeInfo),
            Box::new(raw_input::FileInputNodeInfo),
        ]
    }
}

pub trait SerdeClone: serde::Serialize + serde::de::DeserializeOwned {
    fn serde_clone(&self) -> Self {
        serde_json::from_str(&serde_json::to_string(self).unwrap()).unwrap()
    }
}

impl<T> SerdeClone for T where T: serde::Serialize + serde::de::DeserializeOwned {}
