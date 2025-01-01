use egui_snarl::Snarl;

use crate::prelude::nodes::*;

pub fn analysis(snarl: Snarl<FlowNodes>) {
    let mut nodes_iter = snarl.nodes();

    let wires = snarl.wires();

    let abstract_input = nodes_iter.filter(|node| matches!(node, FlowNodes::AbstractInputNode(_)));
}
