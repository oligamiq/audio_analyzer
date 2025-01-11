use crate::prelude::nodes::*;

pub mod config;
pub mod editor;
pub mod expr;
pub mod frame_queue;
pub mod idct;
pub mod iter;
pub mod layer;
pub mod lifter;
pub mod lpc;
pub mod output;
pub mod pin_info;
pub mod raw_input;
pub mod unknown;
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
    Array1TupleF64F64,
    Array1F64,
    Array2F64,
    Array1ComplexF64,
    AnyInput,
    AnyOutput,
}

impl NodeInfoTypes {
    pub fn eq(&self, other: &NodeInfoTypes) -> bool {
        self == other
            || other == &NodeInfoTypes::AnyInput
            || self == &NodeInfoTypes::AnyInput
            || other == &NodeInfoTypes::AnyOutput
            || self == &NodeInfoTypes::AnyOutput
    }

    pub fn contains_out(&self, other: &[NodeInfoTypes]) -> bool {
        other
            .iter()
            .any(|x| x == self || x == &NodeInfoTypes::AnyInput)
            || (self == &NodeInfoTypes::AnyOutput && !other.is_empty())
    }

    pub fn positions_out(&self, other: &[NodeInfoTypes]) -> Vec<usize> {
        if self == &NodeInfoTypes::AnyOutput {
            other.iter().enumerate().map(|(i, _)| i).collect()
        } else {
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
    }

    pub fn contains_in(&self, other: &[NodeInfoTypes]) -> bool {
        other
            .iter()
            .any(|x| x == self || x == &NodeInfoTypes::AnyOutput)
            || (self == &NodeInfoTypes::AnyInput && !other.is_empty())
    }

    pub fn positions_in(&self, other: &[NodeInfoTypes]) -> Vec<usize> {
        if self == &NodeInfoTypes::AnyInput {
            other.iter().enumerate().map(|(i, _)| i).collect()
        } else {
            other
                .iter()
                .enumerate()
                .filter_map(|(i, x)| {
                    if x == self || x == &NodeInfoTypes::AnyOutput {
                        Some(i)
                    } else {
                        None
                    }
                })
                .collect()
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub enum NodeInfoTypesWithData {
    Number(f64),
    Array1TupleF64F64(Array1<(f64, f64)>),
    Array1F64(Array1<f64>),
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
            Box::new(AbstractInputNodeInfo),
            Box::new(DataPlotterNodeInfo),
            Box::new(SchemaViewerNodeInfo),
            Box::new(ExprNodeInfo),
            Box::new(FrameQueueNodeInfo),
            Box::new(CycleBufferNodeInfo),
            Box::new(IFFTNodeInfo),
            Box::new(FFTNodeInfo),
            Box::new(LifterNodeInfo),
            Box::new(EnumerateIterNodeInfo),
            Box::new(LpcNodeInfo),
            Box::new(BurgNodeInfo),
            Box::new(OutputNodeInfo),
        ]
    }
}

pub trait SerdeClone: serde::Serialize + serde::de::DeserializeOwned {
    fn serde_clone(&self) -> Self {
        serde_json::from_str(&serde_json::to_string(self).unwrap()).unwrap()
    }
}

impl<T> SerdeClone for T where T: serde::Serialize + serde::de::DeserializeOwned {}

pub trait GraphNode {
    type NodeInfoType: NodeInfo;

    fn to_info(&self) -> Self::NodeInfoType;

    fn update(&mut self) {}
}
