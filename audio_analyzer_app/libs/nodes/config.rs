use crate::prelude::nodes::*;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum ConfigNodes {
    NumberNode(NumberNode),
}

impl ConfigNodes {
    pub fn name(&self) -> &str {
        match self {
            Self::NumberNode(_) => "NumberNode",
        }
    }

    pub fn inputs(&self) -> usize {
        match self {
            Self::NumberNode(_) => NumberNodeInfo.inputs(),
        }
    }

    pub fn outputs(&self) -> usize {
        match self {
            Self::NumberNode(_) => NumberNodeInfo.outputs(),
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct NumberNode {
    pub name: String,
    pub number: EditableOnText<f64>,
}

pub struct NumberNodeInfo;

impl NodeInfo for NumberNodeInfo {
    fn name(&self) -> &str {
        "NumberNode"
    }

    fn inputs(&self) -> usize {
        0
    }

    fn outputs(&self) -> usize {
        1
    }

    fn input_types(&self) -> Vec<NodeInfoTypes> {
        vec![]
    }

    fn output_types(&self) -> Vec<NodeInfoTypes> {
        vec![NodeInfoTypes::Number]
    }

    fn flow_node(&self) -> super::editor::FlowNodes {
        super::editor::FlowNodes::ConfigNodes(ConfigNodes::NumberNode(Default::default()))
    }
}

impl Default for NumberNode {
    fn default() -> Self {
        Self {
            name: "NumberNode".to_string(),
            number: EditableOnText::new(1024.),
        }
    }
}

impl NumberNode {
    pub const fn inputs() -> usize {
        0
    }

    pub const fn outputs() -> usize {
        1
    }

    pub fn to_info(&self) -> NumberNodeInfo {
        NumberNodeInfo
    }

    pub fn update(&self) {}
}
