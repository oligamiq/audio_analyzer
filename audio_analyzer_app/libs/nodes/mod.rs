use config::{ConfigNodes, FftSizeNode, HopSizeNode};
use egui::{Pos2, Sense, Ui, Vec2};
use egui_editable_num::EditableOnText;
use egui_snarl::{
    ui::{PinInfo, SnarlViewer},
    Snarl,
};
use layer::{LayerNodes, STFTLayerNode};
use ndarray::{Array1, Array2};
use num_complex::Complex;
use pin_info::CustomPinInfo;

pub mod config;
pub mod layer;
pub mod pin_info;
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
            FftSize(Option<usize>),
            HopSize(Option<usize>),
        }

        let mut input_config = None;

        let info = match &snarl[pin.id.node] {
            FlowNodes::LayerNodes(layer_nodes) => match layer_nodes {
                LayerNodes::STFTLayer(node) => match pin.id.input {
                    0 => {
                        let mut info = CustomPinInfo::setting(8);

                        input_config = Some(InputConfig::FftSize(None));

                        if let Some(pin) = pin.remotes.get(0) {
                            let remote = &snarl[pin.node];
                            if let FlowNodes::ConfigNodes(ConfigNodes::FftSizeNode(FftSizeNode {
                                fft_size,
                            })) = remote
                            {
                                ui.label(format!("fft_size: {}", fft_size));

                                input_config = Some(InputConfig::FftSize(Some(fft_size.get())));

                                info = CustomPinInfo::lock();
                            }
                        }

                        info
                    }
                    1 => {
                        let mut info = CustomPinInfo::setting(8);

                        input_config = Some(InputConfig::HopSize(None));

                        if let Some(pin) = pin.remotes.get(0) {
                            let remote = &snarl[pin.node];
                            if let FlowNodes::ConfigNodes(ConfigNodes::HopSizeNode(HopSizeNode {
                                hop_size,
                            })) = remote
                            {
                                ui.label(format!("hop_size: {}", hop_size));

                                input_config = Some(InputConfig::HopSize(Some(hop_size.get())));

                                info = CustomPinInfo::lock();
                            }
                        }

                        info
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
            FlowNodes::ConfigNodes(_) => unreachable!(),
        };

        match input_config {
            Some(InputConfig::FftSize(edit)) => {
                let node = &mut snarl[pin.id.node];

                let node = if let FlowNodes::LayerNodes(LayerNodes::STFTLayer(node)) = node {
                    node
                } else {
                    unreachable!()
                };

                let STFTLayerNode { fft_size, .. } = node;

                if let Some(value) = edit {
                    node.fft_size.set(value);
                } else {
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
            }
            Some(InputConfig::HopSize(edit)) => {
                let node = &mut snarl[pin.id.node];

                let node = if let FlowNodes::LayerNodes(LayerNodes::STFTLayer(node)) = node {
                    node
                } else {
                    unreachable!()
                };

                let STFTLayerNode { hop_size, .. } = node;

                if let Some(value) = edit {
                    node.hop_size.set(value);
                } else {
                    ui.label("hop_size");
                    let response = egui::TextEdit::singleline(hop_size)
                        .clip_text(false)
                        .desired_width(0.0)
                        .margin(ui.spacing().item_spacing)
                        .show(ui)
                        .response;

                    if response.lost_focus() {
                        node.hop_size.fmt();
                        node.update();
                    } else if response.changed() {
                        if node.hop_size.try_update() {
                            node.update();
                        }
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
        match &mut snarl[pin.id.node] {
            FlowNodes::LayerNodes(layer_nodes) => match layer_nodes {
                LayerNodes::STFTLayer(_) => {
                    ui.label("output STFTLayer");
                    PinInfo::star().with_fill(egui::Color32::from_rgb(0, 0, 0))
                }
                LayerNodes::MelLayer(_) => todo!(),
                LayerNodes::SpectrogramDensityLayer(_) => todo!(),
            },
            FlowNodes::ConfigNodes(config_nodes) => match config_nodes {
                ConfigNodes::FftSizeNode(node) => {
                    ui.end_row();
                    if egui::TextEdit::singleline(&mut node.fft_size)
                        .clip_text(false)
                        .desired_width(0.0)
                        .margin(ui.spacing().item_spacing)
                        .show(ui)
                        .response
                        .lost_focus()
                    {
                        node.fft_size.fmt();
                    }

                    CustomPinInfo::setting(8)
                }
                ConfigNodes::HopSizeNode(node) => {
                    ui.end_row();
                    if egui::TextEdit::singleline(&mut node.hop_size)
                        .clip_text(false)
                        .desired_width(0.0)
                        .margin(ui.spacing().item_spacing)
                        .show(ui)
                        .response
                        .lost_focus()
                    {
                        node.hop_size.fmt();
                    }

                    CustomPinInfo::setting(8)
                }
            },
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

        ui.menu_button("config", |ui| {
            if ui.button("fft_size").clicked() {
                snarl.insert_node(
                    pos,
                    FlowNodes::ConfigNodes(ConfigNodes::FftSizeNode(FftSizeNode::default())),
                );
                ui.close_menu();
            }

            if ui.button("hop_size").clicked() {
                snarl.insert_node(
                    pos,
                    FlowNodes::ConfigNodes(ConfigNodes::HopSizeNode(HopSizeNode::default())),
                );
                ui.close_menu();
            }
        });
    }
}
