use super::config::{ConfigNodes, NumberNode};
use super::layer::{LayerNodes, STFTLayerNode};
use super::pin_info::CustomPinInfo;
use super::raw_input::{FileInputNode, MicrophoneInputNode, RawInputNodes};
use super::NodeInfo;
use egui::Ui;
use egui_snarl::{
    ui::{AnyPins, PinInfo, SnarlViewer},
    InPin, NodeId, OutPin, Snarl,
};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum FlowNodes {
    LayerNodes(LayerNodes),
    ConfigNodes(ConfigNodes),
    RawInputNodes(RawInputNodes),
}

impl FlowNodes {
    fn name(&self) -> &str {
        match self {
            FlowNodes::LayerNodes(node) => node.name(),
            FlowNodes::ConfigNodes(node) => node.name(),
            FlowNodes::RawInputNodes(node) => node.name(),
        }
    }

    pub fn to_as_info(&self) -> Box<dyn NodeInfo> {
        match self {
            FlowNodes::LayerNodes(node) => match node {
                LayerNodes::STFTLayer(node) => Box::new(node.to_info()),
                LayerNodes::MelLayer(node) => Box::new(node.to_info()),
                LayerNodes::SpectrogramDensityLayer(node) => Box::new(node.to_info()),
            },
            FlowNodes::ConfigNodes(node) => match node {
                ConfigNodes::NumberNode(node) => Box::new(node.to_info()),
            },
            FlowNodes::RawInputNodes(node) => match node {
                RawInputNodes::MicrophoneInputNode(node) => Box::new(node.to_info()),
                RawInputNodes::FileInputNode(node) => Box::new(node.to_info()),
            },
        }
    }
}

pub struct FlowNodesViewer;

impl SnarlViewer<FlowNodes> for FlowNodesViewer {
    #[inline]
    fn connect(
        &mut self,
        from: &egui_snarl::OutPin,
        to: &egui_snarl::InPin,
        snarl: &mut Snarl<FlowNodes>,
    ) {
        let in_type = snarl[to.id.node].to_as_info().input_types()[to.id.input];

        let out_type = snarl[from.id.node].to_as_info().output_types()[from.id.output];

        if in_type != out_type {
            return;
        }

        for &remote in &to.remotes {
            snarl.disconnect(remote, to.id);
        }

        snarl.connect(from.id, to.id);
    }

    fn title(&mut self, node: &FlowNodes) -> String {
        node.name().to_string()
    }

    fn inputs(&mut self, node: &FlowNodes) -> usize {
        match node {
            FlowNodes::LayerNodes(node) => node.inputs(),
            FlowNodes::ConfigNodes(node) => node.inputs(),
            FlowNodes::RawInputNodes(raw_input_nodes) => raw_input_nodes.inputs(),
        }
    }

    fn outputs(&mut self, node: &FlowNodes) -> usize {
        match node {
            FlowNodes::LayerNodes(node) => node.outputs(),
            FlowNodes::ConfigNodes(node) => node.outputs(),
            FlowNodes::RawInputNodes(node) => node.outputs(),
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
            STFTCalc(NodeId),
        }

        let mut input_config = None;

        let info =
            match &snarl[pin.id.node] {
                FlowNodes::LayerNodes(layer_nodes) => match layer_nodes {
                    LayerNodes::STFTLayer(node) => match pin.id.input {
                        0 => {
                            let mut info = CustomPinInfo::setting(8);

                            input_config = Some(InputConfig::FftSize(None));

                            if let Some(pin) = pin.remotes.get(0) {
                                let remote = &snarl[pin.node];
                                if let FlowNodes::ConfigNodes(ConfigNodes::NumberNode(
                                    NumberNode { number, .. },
                                )) = remote
                                {
                                    ui.label(format!("fft_size: {}", number));

                                    input_config =
                                        Some(InputConfig::FftSize(Some(number.get() as usize)));

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
                                if let FlowNodes::ConfigNodes(ConfigNodes::NumberNode(
                                    NumberNode { number, .. },
                                )) = remote
                                {
                                    ui.label(format!("hop_size: {}", number));

                                    input_config =
                                        Some(InputConfig::HopSize(Some(number.get() as usize)));

                                    info = CustomPinInfo::lock();
                                }
                            }

                            info
                        }
                        2 => {
                            ui.label("raw stream");

                            if let Some(pin) = pin.remotes.get(0) {
                                input_config = Some(InputConfig::STFTCalc(pin.node));
                            }

                            PinInfo::circle().with_fill(egui::Color32::from_rgb(255, 0, 0))
                        }
                        _ => unreachable!(),
                    },
                    LayerNodes::MelLayer(_) => todo!(),
                    LayerNodes::SpectrogramDensityLayer(_) => todo!(),
                },
                FlowNodes::ConfigNodes(_) => unreachable!(),
                FlowNodes::RawInputNodes(raw_input_nodes) => match raw_input_nodes {
                    RawInputNodes::MicrophoneInputNode(_) => unreachable!(),
                    RawInputNodes::FileInputNode(_) => todo!(),
                },
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
            Some(InputConfig::STFTCalc(id)) => {
                let remote = &mut snarl[id];
                if let FlowNodes::RawInputNodes(node) = remote {
                    if let Some(data) = node.try_recv() {
                        if let FlowNodes::LayerNodes(LayerNodes::STFTLayer(node)) =
                            &mut snarl[pin.id.node]
                        {
                            if let Err(err) = node.calc(&data) {
                                log::error!("STFTLayerNode: {}", err);
                            }
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
                    PinInfo::circle().with_fill(egui::Color32::from_rgb(0, 0, 0))
                }
                LayerNodes::MelLayer(_) => todo!(),
                LayerNodes::SpectrogramDensityLayer(_) => todo!(),
            },
            FlowNodes::ConfigNodes(config_nodes) => match config_nodes {
                ConfigNodes::NumberNode(node) => {
                    ui.end_row();
                    if egui::TextEdit::singleline(&mut node.number)
                        .clip_text(false)
                        .desired_width(0.0)
                        .margin(ui.spacing().item_spacing)
                        .show(ui)
                        .response
                        .lost_focus()
                    {
                        node.number.fmt();
                    }

                    egui::TextEdit::singleline(&mut node.name)
                        .clip_text(false)
                        .desired_width(0.0)
                        .margin(ui.spacing().item_spacing)
                        .show(ui);

                    CustomPinInfo::setting(8)
                }
            },
            FlowNodes::RawInputNodes(raw_input_nodes) => match raw_input_nodes {
                RawInputNodes::MicrophoneInputNode(node) => {
                    ui.label("raw stream");
                    PinInfo::circle().with_fill(egui::Color32::from_rgb(255, 0, 0))
                }
                RawInputNodes::FileInputNode(_) => todo!(),
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
            if ui.button("number_node").clicked() {
                snarl.insert_node(
                    pos,
                    FlowNodes::ConfigNodes(ConfigNodes::NumberNode(NumberNode::default())),
                );
                ui.close_menu();
            }
        });

        ui.menu_button("raw_input", |ui| {
            if ui.button("MicrophoneInputNode").clicked() {
                snarl.insert_node(
                    pos,
                    FlowNodes::RawInputNodes(RawInputNodes::MicrophoneInputNode(
                        MicrophoneInputNode::default(),
                    )),
                );
                ui.close_menu();
            }

            if ui.button("FileInputNode").clicked() {
                snarl.insert_node(
                    pos,
                    FlowNodes::RawInputNodes(
                        RawInputNodes::FileInputNode(FileInputNode::default()),
                    ),
                );
                ui.close_menu();
            }
        });
    }

    fn has_node_menu(&mut self, _node: &FlowNodes) -> bool {
        true
    }

    fn show_node_menu(
        &mut self,
        node: NodeId,
        _inputs: &[InPin],
        _outputs: &[OutPin],
        ui: &mut Ui,
        _scale: f32,
        snarl: &mut Snarl<FlowNodes>,
    ) {
        ui.label("Node menu");
        if ui.button("Remove").clicked() {
            snarl.remove_node(node);
            ui.close_menu();
        }
        if ui.button("Duplicate").clicked() {
            let duplicate_node: FlowNodes =
                serde_json::from_str(&serde_json::to_string(&snarl[node]).unwrap()).unwrap();

            let mut now_pos = snarl.get_node_info(node).unwrap().pos;

            now_pos.x += 20.0;
            now_pos.y += 20.0;

            snarl.insert_node(now_pos, duplicate_node);

            ui.close_menu();
        }
    }

    fn has_dropped_wire_menu(&mut self, _src_pins: AnyPins, _snarl: &mut Snarl<FlowNodes>) -> bool {
        true
    }

    fn show_dropped_wire_menu(
        &mut self,
        pos: egui::Pos2,
        ui: &mut Ui,
        _scale: f32,
        src_pins: AnyPins,
        snarl: &mut Snarl<FlowNodes>,
    ) {
        // In this demo, we create a context-aware node graph menu, and connect a wire
        // dropped on the fly based on user input to a new node created.
        //
        // In your implementation, you may want to define specifications for each node's
        // pin inputs and outputs and compatibility to make this easier.

        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);

        ui.label("Add node");

        match src_pins {
            // inから伸ばしたとき
            AnyPins::In(pin) => {
                assert!(pin.len() == 1);

                let pin = &pin[0];

                let mut view_connect_in_node =
                    |node: &dyn NodeInfo, snarl: &mut Snarl<FlowNodes>| {
                        let input_id = pin.input;

                        let in_type = node.input_types()[input_id];

                        let all = super::NodeInfos::all();

                        for node in all {
                            let out_type = node.output_types();

                            if out_type.contains(&in_type) {
                                if ui.button(node.name()).clicked() {
                                    let mut pos = pos;
                                    pos.x -= 100.0;
                                    pos.y += 20.0;

                                    let node = node.flow_node();

                                    // Create new node.
                                    let new_node = snarl.insert_node(pos, node);

                                    let dst_pin = egui_snarl::OutPinId {
                                        node: new_node,
                                        output: out_type
                                            .iter()
                                            .position(|&x| x == in_type)
                                            .unwrap(),
                                    };

                                    snarl.connect(dst_pin, pin.clone());

                                    ui.close_menu();
                                }
                            }
                        }
                    };

                match &snarl[pin.node] {
                    FlowNodes::LayerNodes(layer_nodes) => match layer_nodes {
                        LayerNodes::STFTLayer(node) => {
                            view_connect_in_node(&node.to_info(), snarl);
                        }
                        LayerNodes::MelLayer(mel_layer_node) => {
                            view_connect_in_node(&mel_layer_node.to_info(), snarl);
                        }
                        LayerNodes::SpectrogramDensityLayer(spectrogram_density_layer_node) => {
                            view_connect_in_node(&spectrogram_density_layer_node.to_info(), snarl);
                        }
                    },
                    FlowNodes::ConfigNodes(config_nodes) => match config_nodes {
                        ConfigNodes::NumberNode(number_node) => {
                            view_connect_in_node(&number_node.to_info(), snarl);
                        }
                    },
                    FlowNodes::RawInputNodes(raw_input_nodes) => match raw_input_nodes {
                        RawInputNodes::MicrophoneInputNode(microphone_input_node) => {
                            view_connect_in_node(&microphone_input_node.to_info(), snarl);
                        }
                        RawInputNodes::FileInputNode(file_input_node) => {
                            view_connect_in_node(&file_input_node.to_info(), snarl);
                        }
                    },
                }
            }
            AnyPins::Out(pin) => {
                assert!(pin.len() == 1);

                let pin = &pin[0];

                let mut view_connect_out_node =
                    |node: &dyn NodeInfo, snarl: &mut Snarl<FlowNodes>| {
                        let output_id = pin.output;

                        let out_type = node.output_types()[output_id];

                        let all = super::NodeInfos::all();

                        for node in all {
                            let in_type = node.input_types();

                            if in_type.contains(&out_type) {
                                if ui.button(node.name()).clicked() {
                                    let mut pos = pos;
                                    pos.x += 100.0;
                                    pos.y += 20.0;

                                    let node = node.flow_node();

                                    // Create new node.
                                    let new_node = snarl.insert_node(pos, node);

                                    let src_pin = egui_snarl::InPinId {
                                        node: new_node,
                                        input: in_type.iter().position(|&x| x == out_type).unwrap(),
                                    };

                                    snarl.connect(pin.clone(), src_pin);

                                    ui.close_menu();
                                }
                            }
                        }
                    };

                match &snarl[pin.node] {
                    FlowNodes::LayerNodes(layer_nodes) => match layer_nodes {
                        LayerNodes::STFTLayer(node) => {
                            view_connect_out_node(&node.to_info(), snarl);
                        }
                        LayerNodes::MelLayer(mel_layer_node) => {
                            view_connect_out_node(&mel_layer_node.to_info(), snarl);
                        }
                        LayerNodes::SpectrogramDensityLayer(spectrogram_density_layer_node) => {
                            view_connect_out_node(&spectrogram_density_layer_node.to_info(), snarl);
                        }
                    },
                    FlowNodes::ConfigNodes(config_nodes) => match config_nodes {
                        ConfigNodes::NumberNode(number_node) => {
                            view_connect_out_node(&number_node.to_info(), snarl);
                        }
                    },
                    FlowNodes::RawInputNodes(raw_input_nodes) => match raw_input_nodes {
                        RawInputNodes::MicrophoneInputNode(microphone_input_node) => {
                            view_connect_out_node(&microphone_input_node.to_info(), snarl);
                        }
                        RawInputNodes::FileInputNode(file_input_node) => {
                            view_connect_out_node(&file_input_node.to_info(), snarl);
                        }
                    },
                }
            }
        }
    }
}
