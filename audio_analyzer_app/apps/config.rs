use std::collections::HashMap;

use crate::prelude::{nodes::*, snarl::*, utils::*};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Serialize, Debug, serde::Deserialize)]
pub struct Config {
    pub snarl: Snarl<FlowNodes>,
    pub stop: bool,
}

// impl<'de> serde::Deserialize<'de> for Config {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: serde::Deserializer<'de>,
//     {
//         let

//         let mut snarl: Snarl<FlowNodes> = Snarl::new();

//         let mut id_map = HashMap::new();

//         for (id, node) in helper.snarl.nodes_ids_data() {
//             match &node.value {
//                 FlowNodesHelper::FlowNodes(flow_nodes) => {
//                     let new_id = snarl.insert_node(node.pos, flow_nodes.serde_clone());

//                     id_map.insert(id, new_id);
//                 },
//                 FlowNodesHelper::Unknown => {
//                     log::warn!("Unknown node type");
//                 },
//             }
//         }

//         for (out_pin, in_pin) in helper.snarl.wires() {
//             if let (Some(mapped_out_pin), Some(mapped_in_pin)) = (
//                 id_map.get(&out_pin.node),
//                 id_map.get(&in_pin.node),
//             ) {
//                 snarl.connect(
//                     egui_snarl::OutPinId {
//                         node: *mapped_out_pin,
//                         output: out_pin.output,
//                     },
//                     egui_snarl::InPinId {
//                         node: *mapped_in_pin,
//                         input: in_pin.input,
//                     },
//                 );
//             }
//         }

//         Ok(Self {
//             snarl,
//             stop: helper.stop,
//         })
//     }
// }

impl Default for Config {
    fn default() -> Self {
        Self {
            snarl: Snarl::new(),
            stop: false,
        }
    }
}

impl Config {
    pub fn deserialize(data: &str) -> Result<Self, serde_json::Error> {
        let value: serde_json::Value = serde_json::from_str(data)?;

        let stop = value["stop"].as_bool().unwrap_or(false);
        let snarl_part = value["snarl"].clone();

        let coarse_snarl: Snarl<serde_json::Value> = serde_json::from_value(snarl_part)?;

        let wires_map = coarse_snarl.wires().fold(
            (
                HashMap::<NodeId, usize>::new(),
                HashMap::<NodeId, usize>::new(),
            ),
            |(mut acc_in, mut acc_out), (out_pin, in_pin)| {
                acc_out
                    .entry(out_pin.node)
                    .and_modify(|e| *e = std::cmp::max(*e, out_pin.output))
                    .or_insert(out_pin.output);
                acc_in
                    .entry(in_pin.node)
                    .and_modify(|e| *e = std::cmp::max(*e, in_pin.input))
                    .or_insert(in_pin.input);

                (acc_in, acc_out)
            },
        );

        let mut snarl: Snarl<FlowNodes> = Snarl::new();

        let mut id_map = HashMap::new();

        for (id, node) in coarse_snarl.nodes_ids_data() {
            let new_id = match serde_json::from_value::<FlowNodes>(node.value.clone()) {
                Ok(flow_nodes) => snarl.insert_node(node.pos, flow_nodes),
                Err(e) => {
                    log::warn!("Unknown node type: {:?}", e);

                    let value = node.value.clone();
                    let name = value
                        .as_object()
                        .map(|o| o.keys().next().cloned())
                        .flatten()
                        .unwrap_or(String::from("Unknown"));

                    snarl.insert_node(
                        node.pos,
                        FlowNodes::UnknownNode(UnknownNode {
                            name: name,
                            input_num: wires_map.0.get(&id).map(|x| *x + 1).unwrap_or(0),
                            output_num: wires_map.1.get(&id).map(|x| *x + 1).unwrap_or(0),
                        }),
                    )
                }
            };

            id_map.insert(id, new_id);
        }

        for (out_pin, in_pin) in coarse_snarl.wires() {
            if let (Some(mapped_out_pin), Some(mapped_in_pin)) =
                (id_map.get(&out_pin.node), id_map.get(&in_pin.node))
            {
                snarl.connect(
                    egui_snarl::OutPinId {
                        node: *mapped_out_pin,
                        output: out_pin.output,
                    },
                    egui_snarl::InPinId {
                        node: *mapped_in_pin,
                        input: in_pin.input,
                    },
                );
            }
        }

        Ok(Self { snarl, stop })
    }

    pub fn from_ref(snarl: &Snarl<FlowNodes>) -> Self {
        Self {
            snarl: snarl.serde_clone(),
            stop: false,
        }
    }

    pub const SNARL_STYLE: SnarlStyle = {
        let mut style = SnarlStyle::new();

        style.bg_pattern = Some(egui_snarl::ui::BackgroundPattern::Grid(
            egui_snarl::ui::Grid {
                spacing: egui::Vec2::new(50.0, 50.0),
                angle: 1.0,
            },
        ));

        style
    };
}
