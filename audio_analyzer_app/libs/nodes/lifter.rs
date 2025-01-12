use ndarray::ArrayView1;

use crate::prelude::nodes::*;

use super::layer::extract_snarl_ui_pin_member;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum FilterNodes {
    LifterNode(LifterNode),
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct LifterNode {
    pub size: EditableOnText<usize>,

    #[serde(skip)]
    liftered: Option<Array1<f64>>,
}

impl FlowNodesViewerTrait for LifterNode {
    fn show_input(
        &self,
        ctx: &FlowNodesViewerCtx,
        pin: &egui_snarl::InPin,
        ui: &mut egui::Ui,
        _scale: f32,
        snarl: &egui_snarl::Snarl<FlowNodes>,
    ) -> Box<dyn Fn(&mut Snarl<FlowNodes>, &mut egui::Ui) -> MyPinInfo> {
        let pin_id = pin.id;

        if !ctx.running {
            return Box::new(|_, _| CustomPinInfo::none_status());
        }

        match pin_id.input {
            0 => {
                extract_snarl_ui_pin_member!(
                    snarl,
                    ui,
                    pin,
                    FlowNodes::FilterNodes(FilterNodes::LifterNode(node)),
                    node,
                    size
                );
            }
            1 => {
                ui.label("stream");

                if !ctx.running {
                    return Box::new(|_, _| CustomPinInfo::none_status());
                }

                let remote_node = pin.remotes.get(0);

                if let Some(remote_node) = remote_node {
                    let data =
                        snarl[remote_node.node].to_node_info_types_with_data(remote_node.output);

                    if let Some(NodeInfoTypesWithData::Array1F64(data)) = data {
                        return Box::new(move |snarl, _ui| {
                            extract_node!(
                                &mut snarl[pin_id.node],
                                FlowNodes::FilterNodes(FilterNodes::LifterNode(node)) => {
                                    node.through_inner(data.view());
                                }
                            );

                            CustomPinInfo::ok_status()
                        });
                    }
                }

                return Box::new(|_, _| CustomPinInfo::ng_status());
            }
            _ => unreachable!(),
        }
    }
}

impl Default for LifterNode {
    fn default() -> Self {
        let size = EditableOnText::new(13);
        let liftered = None;

        Self { size, liftered }
    }
}

pub struct LifterNodeInfo;

impl NodeInfo for LifterNodeInfo {
    fn name(&self) -> &str {
        "Lifter"
    }

    fn inputs(&self) -> usize {
        2
    }

    fn outputs(&self) -> usize {
        1
    }

    fn input_types(&self) -> Vec<NodeInfoTypes> {
        vec![NodeInfoTypes::Number, NodeInfoTypes::Array1F64]
    }

    fn output_types(&self) -> Vec<NodeInfoTypes> {
        vec![NodeInfoTypes::Array1F64]
    }

    fn flow_node(&self) -> super::editor::FlowNodes {
        FlowNodes::FilterNodes(FilterNodes::LifterNode(LifterNode::default()))
    }
}

impl GraphNode for LifterNode {
    type NodeInfoType = LifterNodeInfo;

    fn to_info(&self) -> Self::NodeInfoType {
        LifterNodeInfo
    }
}

impl LifterNode {
    fn through_inner(&mut self, quefrency: ArrayView1<f64>) {
        let mut quefrency = quefrency.to_owned();
        let index = self.size.get();

        // 中間報告ではこっちだった
        // for i in 0..quefrency.len() {
        //     if i > index && i < quefrency.len() - index {
        //         quefrency[i] = 0.0;
        //     }
        // }
        for i in 0..quefrency.len() {
            if i < index || i > quefrency.len() - index {
                quefrency[i] = 0.0;
            }
        }

        self.liftered = Some(quefrency);
    }

    pub fn get_result(&self) -> Option<Array1<f64>> {
        self.liftered.clone()
    }
}
