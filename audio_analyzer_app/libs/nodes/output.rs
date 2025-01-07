use crate::prelude::nodes::*;

// use super::layer::extract_snarl_ui_pin_member;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum OutputNodes {
    OutputNode(OutputNode),
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct OutputNode {
}

impl Default for OutputNode {
    fn default() -> Self {
        Self {
        }
    }
}

impl FlowNodesViewerTrait for OutputNode {
    fn show_input(
        &self,
        ctx: &FlowNodesViewerCtx,
        pin: &egui_snarl::InPin,
        ui: &mut egui::Ui,
        _scale: f32,
        _snarl: &egui_snarl::Snarl<FlowNodes>,
    ) -> Box<dyn Fn(&mut Snarl<FlowNodes>, &mut egui::Ui) -> MyPinInfo> {
        let pin_id = pin.id;

        match pin_id.input {
            _ => {
                ui.label("input");

                if !ctx.running {
                    return Box::new(move |_, _| CustomPinInfo::none_status());
                }
            }
        }

        Box::new(move |_, _| CustomPinInfo::none_status())
    }
}

impl OutputNode {
    pub fn new() -> Self {
        Self {
        }
    }
}

pub struct OutputNodeInfo;

impl NodeInfo for OutputNodeInfo {
    fn name(&self) -> &'static str {
        "Output"
    }

    fn inputs(&self) -> usize {
        1
    }

    fn outputs(&self) -> usize {
        0
    }

    fn input_types(&self) -> Vec<NodeInfoTypes> {
        vec![NodeInfoTypes::Array1F64]
    }

    fn output_types(&self) -> Vec<NodeInfoTypes> {
        vec![]
    }

    fn flow_node(&self) -> super::editor::FlowNodes {
        FlowNodes::OutputNodes(OutputNodes::OutputNode(OutputNode::default()))
    }
}

impl GraphNode for OutputNode {
    type NodeInfoType = OutputNodeInfo;

    fn to_info(&self) -> Self::NodeInfoType {
        OutputNodeInfo
    }
}
