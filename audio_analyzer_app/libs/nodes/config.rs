use egui_editable_num::EditableOnText;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum ConfigNodes {
    NumberNode(NumberNode),
}

impl ConfigNodes {
    pub fn name(&self) -> &str {
        match self {
            ConfigNodes::NumberNode(_) => "NumberNode",
        }
    }

    pub const fn inputs(&self) -> usize {
        match self {
            ConfigNodes::NumberNode(_) => NumberNode::inputs(),
        }
    }

    pub const fn outputs(&self) -> usize {
        match self {
            ConfigNodes::NumberNode(_) => NumberNode::outputs(),
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct NumberNode {
    pub name: String,
    pub number: EditableOnText<f64>,
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
}
