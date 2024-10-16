use config::{ConfigNodes, FftSizeNode};
use egui::Ui;
use egui_editable_num::EditableOnText;
use egui_snarl::{
    ui::{PinInfo, SnarlViewer},
    Snarl,
};
use layer::{LayerNodes, STFTLayerNode};
use ndarray::{Array1, Array2};
use num_complex::Complex;

pub mod config;
pub mod layer;
pub mod utils;

#[derive(serde::Serialize, serde::Deserialize)]
pub enum FlowNodes {
    LayerNodes(LayerNodes),
    ConfigNodes(config::ConfigNodes),
}

impl FlowNodes {
    fn name(&self) -> &str {
        match self {
            FlowNodes::LayerNodes(node) => node.name(),
            FlowNodes::ConfigNodes(node) => node.name(),
        }
    }
}

pub struct FlowNodesViewer;

impl SnarlViewer<FlowNodes> for FlowNodesViewer {
    // #[inline]
    // fn connect(
    //     &mut self,
    //     from: &egui_snarl::OutPin,
    //     to: &egui_snarl::InPin,
    //     snarl: &mut Snarl<FlowNodes>,
    // ) {
    //     match (&snarl[from.id.node], &snarl[to.id.node]) {
    //         (FlowNodes::LayerNodes(from), FlowNodes::LayerNodes(to)) => {
    //             if !from.validate_connections(to) {
    //                 return;
    //             }
    //         }
    //     }

    //     for &remote in &to.remotes {
    //         snarl.disconnect(remote, to.id);
    //     }

    //     snarl.connect(from.id, to.id);
    // }

    fn title(&mut self, node: &FlowNodes) -> String {
        node.name().to_string()
    }

    fn inputs(&mut self, node: &FlowNodes) -> usize {
        match node {
            FlowNodes::LayerNodes(node) => node.inputs(),
            FlowNodes::ConfigNodes(node) => node.inputs(),
        }
    }

    fn outputs(&mut self, node: &FlowNodes) -> usize {
        match node {
            FlowNodes::LayerNodes(node) => node.outputs(),
            FlowNodes::ConfigNodes(node) => node.outputs(),
        }
    }

    // inputを表示する
    fn show_input(
        &mut self,
        pin: &egui_snarl::InPin,
        ui: &mut egui::Ui,
        scale: f32,
        snarl: &mut egui_snarl::Snarl<FlowNodes>,
    ) -> egui_snarl::ui::PinInfo {
        enum InputConfig {
            FftSize,
        }

        let mut input_config = None;

        let info = match &snarl[pin.id.node] {
            FlowNodes::LayerNodes(layer_nodes) => match layer_nodes {
                LayerNodes::STFTLayer(node) => match pin.id.input {
                    0 => {
                        if let Some(pin) = pin.remotes.get(0) {
                            let remote = &snarl[pin.node];
                            if let FlowNodes::ConfigNodes(ConfigNodes::FftSizeNode(FftSizeNode {
                                fft_size,
                            })) = remote
                            {
                                ui.label(format!("fft_size: {}", fft_size));
                                return PinInfo::star();
                            }
                        }

                        input_config = Some(InputConfig::FftSize);

                        PinInfo::star().with_fill(egui::Color32::from_rgb(0, 0, 0))
                    }
                    1 => {
                        ui.label("input STFTLayer");
                        PinInfo::star().with_fill(egui::Color32::from_rgb(0, 0, 0))
                    }
                    2 => {
                        ui.label("input STFTLayer");
                        PinInfo::star().with_fill(egui::Color32::from_rgb(0, 0, 0))
                    }
                    _ => unreachable!(),
                },
                LayerNodes::MelLayer(_) => todo!(),
                LayerNodes::SpectrogramDensityLayer(_) => todo!(),
            },
            FlowNodes::ConfigNodes(config_nodes) => todo!(),
        };

        match input_config {
            Some(InputConfig::FftSize) => {
                let node = &mut snarl[pin.id.node];

                let node = if let FlowNodes::LayerNodes(LayerNodes::STFTLayer(node)) = node {
                    node
                } else {
                    unreachable!()
                };

                let STFTLayerNode { fft_size, .. } = node;
                ui.label("fft_size");
                let response = egui::TextEdit::singleline(fft_size)
                    .clip_text(false)
                    .desired_width(0.0)
                    .margin(ui.spacing().item_spacing)
                    .show(ui)
                    .response;

                if response.lost_focus() {
                    node.fft_size.fmt();
                    node.update();
                } else if response.changed() {
                    if node.fft_size.try_update() {
                        node.update();
                    }
                }
            }
            None => {}
        }

        info
    }

    fn show_output(
        &mut self,
        pin: &egui_snarl::OutPin,
        ui: &mut egui::Ui,
        scale: f32,
        snarl: &mut egui_snarl::Snarl<FlowNodes>,
    ) -> egui_snarl::ui::PinInfo {
        match &snarl[pin.id.node] {
            FlowNodes::LayerNodes(layer_nodes) => match layer_nodes {
                LayerNodes::STFTLayer(_) => {
                    ui.label("output STFTLayer");
                    PinInfo::star().with_fill(egui::Color32::from_rgb(0, 0, 0))
                }
                LayerNodes::MelLayer(_) => todo!(),
                LayerNodes::SpectrogramDensityLayer(_) => todo!(),
            },
            FlowNodes::ConfigNodes(config_nodes) => todo!(),
        }
    }

    fn has_graph_menu(&mut self, _pos: egui::Pos2, _snarl: &mut Snarl<FlowNodes>) -> bool {
        true
    }

    fn show_graph_menu(
        &mut self,
        pos: egui::Pos2,
        ui: &mut Ui,
        _scale: f32,
        snarl: &mut Snarl<FlowNodes>,
    ) {
        ui.label("Add node");
        if ui.button("STFTLayer").clicked() {
            snarl.insert_node(
                pos,
                FlowNodes::LayerNodes(LayerNodes::STFTLayer(STFTLayerNode::default())),
            );
            ui.close_menu();
        }
    }
}
