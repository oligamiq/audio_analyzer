use anyhow::Context;
use egui_snarl::{InPinId, OutPinId, Snarl};

use crate::prelude::nodes::*;
use quote::quote;

pub fn analysis(snarl: &Snarl<FlowNodes>) -> anyhow::Result<()> {
    let abstract_input_node_id = snarl
        .nodes_ids_data()
        .find(|(_, node)| matches!(node.value, FlowNodes::AbstractInputNode(_)))
        .with_context(|| "No abstract input node found")?
        .0;

    log::info!("Abstract input node id: {:?}", abstract_input_node_id);

    let next_nodes = get_next_nodes(snarl, abstract_input_node_id);

    log::info!("Output input node id: {:?}", next_nodes);

    let first_code = match snarl.get_node(abstract_input_node_id) {
        Some(FlowNodes::AbstractInputNode(node)) => {
            log::info!("Abstract input node: {:?}", node);

            quote! {
                
            }
        }
        _ => unreachable!(),
    };

    Ok(())
}

fn get_next_nodes(snarl: &Snarl<FlowNodes>, node: NodeId) -> Vec<(OutPinId, InPinId)> {
    let mut next_nodes: Vec<(OutPinId, InPinId)> = snarl
        .wires()
        .filter(|(out, _)| out.node == node)
        .collect::<Vec<_>>();

    next_nodes.sort_by(|(_, a_out), (_, b_out)| b_out.input.cmp(&a_out.input));

    next_nodes
}
