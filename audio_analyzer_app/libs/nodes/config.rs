use egui_editable_num::EditableOnText;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum ConfigNodes {
    FftSizeNode(FftSizeNode),
    HopSizeNode(HopSizeNode),
}

impl ConfigNodes {
    pub fn name(&self) -> &str {
        match self {
            ConfigNodes::FftSizeNode(_) => "FftSize",
            ConfigNodes::HopSizeNode(_) => "HopSize",
        }
    }

    pub const fn inputs(&self) -> usize {
        match self {
            ConfigNodes::FftSizeNode(_) => FftSizeNode::inputs(),
            ConfigNodes::HopSizeNode(_) => HopSizeNode::inputs(),
        }
    }

    pub const fn outputs(&self) -> usize {
        match self {
            ConfigNodes::FftSizeNode(_) => FftSizeNode::outputs(),
            ConfigNodes::HopSizeNode(_) => HopSizeNode::outputs(),
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct FftSizeNode {
    pub fft_size: EditableOnText<usize>,
}

impl Default for FftSizeNode {
    fn default() -> Self {
        Self {
            fft_size: EditableOnText::new(1024),
        }
    }
}

impl FftSizeNode {
    pub const fn inputs() -> usize {
        0
    }

    pub const fn outputs() -> usize {
        1
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct HopSizeNode {
    pub hop_size: EditableOnText<usize>,
}

impl Default for HopSizeNode {
    fn default() -> Self {
        Self {
            hop_size: EditableOnText::new(512),
        }
    }
}

impl HopSizeNode {
    pub const fn inputs() -> usize {
        0
    }

    pub const fn outputs() -> usize {
        1
    }
}
