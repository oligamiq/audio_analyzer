use egui_snarl::Snarl;

use crate::prelude::nodes::*;

pub fn analysis(
    snarl: Snarl<FlowNodes>,
) {
    let nodes = snarl.nodes();

    let wires = snarl.wires();

    // let abstract_input = nodes.get("AbstractInput").unwrap();
}
