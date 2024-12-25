use crate::prelude::nodes::*;

use super::layer::extract_snarl_ui_pin_member;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum IterNodes {
    EnumerateIterNode(EnumerateIterNode),
}

#[derive(Debug, serde::Serialize)]
pub struct EnumerateIterNode {
    pub start: EditableOnText<usize>,
    pub step: EditableOnText<usize>,
    pub end: EditableOnText<usize>,

    #[serde(skip)]
    iterated: Option<Array1<f64>>,
}

impl Default for EnumerateIterNode {
    fn default() -> Self {
        Self::new(0, 1, 10)
    }
}

impl<'de> serde::Deserialize<'de> for EnumerateIterNode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct EnumerateIterNodeHelper {
            start: EditableOnText<usize>,
            step: EditableOnText<usize>,
            end: EditableOnText<usize>,
        }

        let helper = EnumerateIterNodeHelper::deserialize(deserializer)?;

        Ok(Self::new(
            helper.start.get(),
            helper.step.get(),
            helper.end.get(),
        ))
    }
}

impl FlowNodesViewerTrait for EnumerateIterNode {
    fn show_input(
        &self,
        _ctx: &FlowNodesViewerCtx,
        pin: &egui_snarl::InPin,
        ui: &mut egui::Ui,
        _scale: f32,
        snarl: &egui_snarl::Snarl<FlowNodes>,
    ) -> Box<dyn Fn(&mut Snarl<FlowNodes>, &mut egui::Ui) -> PinInfo> {
        let pin_id = pin.id;

        match pin_id.input {
            0 => {
                extract_snarl_ui_pin_member!(
                    snarl,
                    ui,
                    pin,
                    FlowNodes::IterNodes(IterNodes::EnumerateIterNode(node)),
                    node,
                    start
                );
            }
            1 => {
                extract_snarl_ui_pin_member!(
                    snarl,
                    ui,
                    pin,
                    FlowNodes::IterNodes(IterNodes::EnumerateIterNode(node)),
                    node,
                    step
                );
            }
            2 => {
                extract_snarl_ui_pin_member!(
                    snarl,
                    ui,
                    pin,
                    FlowNodes::IterNodes(IterNodes::EnumerateIterNode(node)),
                    node,
                    end
                );
            }
            _ => unreachable!(),
        }
    }
}

pub struct EnumerateIterNodeInfo;

impl NodeInfo for EnumerateIterNodeInfo {
    fn name(&self) -> &'static str {
        "EnumerateIterNode"
    }

    fn inputs(&self) -> usize {
        3
    }

    fn outputs(&self) -> usize {
        1
    }

    fn input_types(&self) -> Vec<NodeInfoTypes> {
        vec![
            NodeInfoTypes::Number,
            NodeInfoTypes::Number,
            NodeInfoTypes::Number,
        ]
    }

    fn output_types(&self) -> Vec<NodeInfoTypes> {
        vec![NodeInfoTypes::Array1F64]
    }

    fn flow_node(&self) -> super::editor::FlowNodes {
        FlowNodes::IterNodes(IterNodes::EnumerateIterNode(EnumerateIterNode::default()))
    }
}

impl EnumerateIterNode {
    pub fn new(start: usize, step: usize, end: usize) -> Self {
        let mut sl = Self {
            start: EditableOnText::new(start),
            step: EditableOnText::new(step),
            end: EditableOnText::new(end),
            iterated: None,
        };

        sl.update();

        sl
    }

    pub fn get_result(&self) -> Option<Array1<f64>> {
        self.iterated.clone()
    }
}

impl GraphNode for EnumerateIterNode {
    type NodeInfoType = EnumerateIterNodeInfo;

    fn to_info(&self) -> Self::NodeInfoType {
        EnumerateIterNodeInfo
    }

    fn update(&mut self) {
        let start = self.start.get();
        let step = self.step.get();
        let end = self.end.get();

        self.iterated = Some((start..end).step_by(step).map(|x| x as f64).collect());
    }
}
