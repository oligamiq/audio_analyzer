use config::NumberNodeInfo;
use layer::{MelLayerNodeInfo, STFTLayerNodeInfo, SpectrogramDensityLayerNodeInfo};
use ndarray::{Array1, Array2};
use raw_input::{FileInputNodeInfo, MicrophoneInputNodeInfo};
use viewer::DataPlotterNodeInfo;

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
    AnyInput,
}

impl NodeInfoTypes {
    pub fn contains_out(&self, other: &[NodeInfoTypes]) -> bool {
        other
            .iter()
            .any(|x| x == self || x == &NodeInfoTypes::AnyInput)
    }

    pub fn positions_out(&self, other: &[NodeInfoTypes]) -> Vec<usize> {
        other
            .iter()
            .enumerate()
            .filter_map(|(i, x)| {
                if x == self || x == &NodeInfoTypes::AnyInput {
                    Some(i)
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn contains_in(&self, other: &[NodeInfoTypes]) -> bool {
        other.iter().any(|x| x == self) || (self == &NodeInfoTypes::AnyInput && !other.is_empty())
    }

    pub fn positions_in(&self, other: &[NodeInfoTypes]) -> Vec<usize> {
        if self == &NodeInfoTypes::AnyInput {
            other.iter().enumerate().map(|(i, _)| i).collect()
        } else {
            other
                .iter()
                .enumerate()
                .filter_map(|(i, x)| if x == self { Some(i) } else { None })
                .collect()
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub enum NodeInfoTypesWithData {
    Number(f64),
    VecF32(Vec<f32>),
    Array1TupleF64F64(Array1<(f64, f64)>),
    Array2F64(Array2<f64>),
    Array1ComplexF64(Array1<num_complex::Complex<f64>>),
}

pub struct NodeInfos;

impl NodeInfos {
    pub fn all() -> Vec<Box<dyn NodeInfo>> {
        vec![
            Box::new(NumberNodeInfo),
            Box::new(STFTLayerNodeInfo),
            Box::new(MelLayerNodeInfo),
            Box::new(SpectrogramDensityLayerNodeInfo),
            Box::new(MicrophoneInputNodeInfo),
            Box::new(FileInputNodeInfo),
            Box::new(DataPlotterNodeInfo),
        ]
    }
}

pub trait SerdeClone: serde::Serialize + serde::de::DeserializeOwned {
    fn serde_clone(&self) -> Self {
        serde_json::from_str(&serde_json::to_string(self).unwrap()).unwrap()
    }
}

impl<T> SerdeClone for T where T: serde::Serialize + serde::de::DeserializeOwned {}
