use ndarray::ArrayView1;

use crate::prelude::nodes::*;

use super::layer::extract_snarl_ui_pin_member;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum LpcNodes {
    LpcNode(LpcNode),
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct LpcNode {
    pub order: EditableOnText<usize>,

    #[serde(skip)]
    lpc: Option<Array1<f64>>,
}

impl Default for LpcNode {
    fn default() -> Self {
        Self {
            order: EditableOnText::new(10),
            lpc: None,
        }
    }
}

impl FlowNodesViewerTrait for LpcNode {
    fn show_input(
        &self,
        ctx: &FlowNodesViewerCtx,
        pin: &egui_snarl::InPin,
        ui: &mut egui::Ui,
        _scale: f32,
        snarl: &egui_snarl::Snarl<FlowNodes>,
    ) -> Box<dyn Fn(&mut Snarl<FlowNodes>, &mut egui::Ui) -> PinInfo> {
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
                    FlowNodes::LpcNodes(LpcNodes::LpcNode(node)),
                    node,
                    order
                );
            }
            1 => {
                ui.label("input");

                if !ctx.running {
                    return Box::new(move |_, _| CustomPinInfo::none_status());
                }

                if let Some(out_pin) = pin.remotes.get(0) {
                    let data = snarl[out_pin.node].to_node_info_types_with_data(out_pin.output);

                    if let Some(NodeInfoTypesWithData::Array1F64(data)) = data {
                        return Box::new(move |snarl, _ui| {
                            extract_node!(
                                &mut snarl[pin_id.node],
                                FlowNodes::LpcNodes(LpcNodes::LpcNode(node)) => {
                                    node.through_inner(data.view());
                                }
                            );

                            // log::info!("LpcNode: input ok: {:?}", data);

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

impl LpcNode {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_result(&self) -> Option<Array1<f64>> {
        self.lpc.clone()
    }

    pub fn through_inner(&mut self, input: ArrayView1<f64>) {
        let order = self.order.get();

        self.lpc = Some(linear_predictive_coding::calc_lpc_by_levinson_durbin(
            input, order,
        ));
    }
}

pub struct LpcNodeInfo;

impl NodeInfo for LpcNodeInfo {
    fn name(&self) -> &'static str {
        "LpcNode"
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

    fn flow_node(&self) -> FlowNodes {
        FlowNodes::LpcNodes(LpcNodes::LpcNode(LpcNode::default()))
    }
}

impl GraphNode for LpcNode {
    type NodeInfoType = LpcNodeInfo;

    fn to_info(&self) -> Self::NodeInfoType {
        LpcNodeInfo
    }

    fn update(&mut self) {
        self.lpc = None;
    }
}
