use crate::prelude::{egui::*, nodes::*, snarl::*, utils::*};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum FlowNodes {
    LayerNodes(LayerNodes),
    ConfigNodes(ConfigNodes),
    DataInspectorNode(DataInspectorNode),
    AbstractInputNode(AbstractInputNode),
    ExprNode(ExprNodes),
    FrameBufferNode(FrameBufferNode),
    FrequencyNodes(FrequencyNodes),
    FilterNodes(FilterNodes),
    IterNodes(IterNodes),
    LpcNodes(LpcNodes),
    OutputNodes(OutputNodes),
    UnknownNode(UnknownNode),
}

impl FlowNodes {
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
            FlowNodes::AbstractInputNode(node) => Box::new(node.to_info()),
            FlowNodes::DataInspectorNode(node) => match node {
                DataInspectorNode::DataPlotterNode(node) => Box::new(node.to_info()),
                DataInspectorNode::SchemaViewerNode(node) => Box::new(node.to_info()),
            },
            FlowNodes::ExprNode(expr_nodes) => Box::new(expr_nodes.to_info()),
            FlowNodes::FrameBufferNode(frame_buffer) => match frame_buffer {
                FrameBufferNode::FrameQueueNode(node) => Box::new(node.to_info()),
                FrameBufferNode::CycleBufferNode(node) => Box::new(node.to_info()),
            },
            FlowNodes::FrequencyNodes(frequency_nodes) => match frequency_nodes {
                FrequencyNodes::IFFTNode(node) => Box::new(node.to_info()),
                FrequencyNodes::FFTNode(node) => Box::new(node.to_info()),
            },
            FlowNodes::FilterNodes(filter_nodes) => match filter_nodes {
                FilterNodes::LifterNode(node) => Box::new(node.to_info()),
            },
            FlowNodes::IterNodes(iter_nodes) => match iter_nodes {
                IterNodes::EnumerateIterNode(node) => Box::new(node.to_info()),
            },
            FlowNodes::LpcNodes(lpc_nodes) => match lpc_nodes {
                LpcNodes::LpcNode(node) => Box::new(node.to_info()),
                LpcNodes::BurgNode(node) => Box::new(node.to_info()),
            },
            FlowNodes::UnknownNode(unknown_node) => Box::new(unknown_node.to_info()),
            FlowNodes::OutputNodes(output_nodes) => match output_nodes {
                OutputNodes::OutputNode(node) => Box::new(node.to_info()),
            },
        }
    }
}

pub struct FlowNodesViewer {
    running: bool,
}

impl FlowNodesViewer {
    pub fn new(running: bool) -> Self {
        Self { running }
    }

    pub fn running(&self) -> bool {
        self.running
    }
}

#[derive(Debug, Clone)]
pub struct FlowNodesViewerCtx {
    pub running: bool,
}

pub trait FlowNodesViewerTrait {
    fn show_input(
        &self,
        _ctx: &FlowNodesViewerCtx,
        _pin: &egui_snarl::InPin,
        _ui: &mut egui::Ui,
        _scale: f32,
        _snarl: &egui_snarl::Snarl<FlowNodes>,
    ) -> Box<dyn Fn(&mut Snarl<FlowNodes>, &mut egui::Ui) -> MyPinInfo> {
        todo!()
    }
}

impl FlowNodesViewer {
    fn show_input(
        &mut self,
        pin: &egui_snarl::InPin,
        ui: &mut egui::Ui,
        scale: f32,
        snarl: &egui_snarl::Snarl<FlowNodes>,
    ) -> Box<dyn Fn(&mut Snarl<FlowNodes>, &mut egui::Ui) -> MyPinInfo> {
        let ctx = FlowNodesViewerCtx {
            running: self.running,
        };
        match &snarl[pin.id.node] {
            FlowNodes::LayerNodes(layer_nodes) => match layer_nodes {
                LayerNodes::STFTLayer(node) => node.show_input(&ctx, pin, ui, scale, snarl),
                LayerNodes::MelLayer(node) => node.show_input(&ctx, pin, ui, scale, snarl),
                LayerNodes::SpectrogramDensityLayer(node) => {
                    node.show_input(&ctx, pin, ui, scale, snarl)
                }
            },
            FlowNodes::ConfigNodes(_) => unreachable!(),
            FlowNodes::AbstractInputNode(node) => node.show_input(&ctx, pin, ui, scale, snarl),
            FlowNodes::DataInspectorNode(node) => match node {
                DataInspectorNode::DataPlotterNode(node) => {
                    node.show_input(&ctx, pin, ui, scale, snarl)
                }
                DataInspectorNode::SchemaViewerNode(node) => {
                    node.show_input(&ctx, pin, ui, scale, snarl)
                }
            },
            FlowNodes::ExprNode(node) => node.show_input(&ctx, pin, ui, scale, snarl),
            FlowNodes::FrameBufferNode(frame_buffer) => match frame_buffer {
                FrameBufferNode::FrameQueueNode(node) => {
                    node.show_input(&ctx, pin, ui, scale, snarl)
                }
                FrameBufferNode::CycleBufferNode(node) => {
                    node.show_input(&ctx, pin, ui, scale, snarl)
                }
            },
            FlowNodes::FrequencyNodes(frequency_nodes) => match frequency_nodes {
                FrequencyNodes::IFFTNode(node) => node.show_input(&ctx, pin, ui, scale, snarl),
                FrequencyNodes::FFTNode(node) => node.show_input(&ctx, pin, ui, scale, snarl),
            },
            FlowNodes::FilterNodes(filter_nodes) => match filter_nodes {
                FilterNodes::LifterNode(node) => node.show_input(&ctx, pin, ui, scale, snarl),
            },
            FlowNodes::IterNodes(iter_nodes) => match iter_nodes {
                IterNodes::EnumerateIterNode(node) => node.show_input(&ctx, pin, ui, scale, snarl),
            },
            FlowNodes::LpcNodes(lpc_nodes) => match lpc_nodes {
                LpcNodes::LpcNode(node) => node.show_input(&ctx, pin, ui, scale, snarl),
                LpcNodes::BurgNode(node) => node.show_input(&ctx, pin, ui, scale, snarl),
            },
            FlowNodes::UnknownNode(unknown_node) => {
                unknown_node.show_input(&ctx, pin, ui, scale, snarl)
            }
            FlowNodes::OutputNodes(output_nodes) => match output_nodes {
                OutputNodes::OutputNode(node) => node.show_input(&ctx, pin, ui, scale, snarl),
            },
        }
    }
}

impl SnarlViewer<FlowNodes> for FlowNodesViewer {
    type Drawer = MyDrawer;

    #[inline]
    fn connect(
        &mut self,
        from: &egui_snarl::OutPin,
        to: &egui_snarl::InPin,
        snarl: &mut Snarl<FlowNodes>,
    ) {
        let in_type = snarl[to.id.node].to_as_info().input_types()[to.id.input];

        let out_type = snarl[from.id.node].to_as_info().output_types()[from.id.output];

        if !in_type.eq(&out_type) {
            return;
        }

        // expr is only allowed to connect to multiple inputs
        if !matches!(snarl[to.id.node], FlowNodes::ExprNode(_)) {
            for &remote in &to.remotes {
                snarl.disconnect(remote, to.id);
            }
        }

        snarl.connect(from.id, to.id);
    }

    fn title(&mut self, node: &FlowNodes) -> String {
        node.to_as_info().name().to_string()
    }

    fn show_header(
        &mut self,
        node: NodeId,
        _inputs: &[InPin],
        _outputs: &[OutPin],
        ui: &mut Ui,
        _scale: f32,
        snarl: &mut Snarl<FlowNodes>,
    ) {
        if matches!(snarl[node], FlowNodes::UnknownNode(_)) {
            ui.label(
                Into::<egui::WidgetText>::into("UnknownNode")
                    .color(egui::Color32::from_rgb(255, 0, 0)),
            );
        } else {
            ui.label(self.title(&snarl[node]));
        }
    }

    fn inputs(&mut self, node: &FlowNodes) -> usize {
        node.to_as_info().inputs()
    }

    fn outputs(&mut self, node: &FlowNodes) -> usize {
        node.to_as_info().outputs()
    }

    // inputを表示する
    fn show_input(
        &mut self,
        pin: &egui_snarl::InPin,
        ui: &mut egui::Ui,
        scale: f32,
        snarl: &mut egui_snarl::Snarl<FlowNodes>,
    ) -> MyPinInfo {
        self.show_input(pin, ui, scale, snarl)(snarl, ui)
    }

    fn show_output(
        &mut self,
        pin: &egui_snarl::OutPin,
        ui: &mut egui::Ui,
        _scale: f32,
        snarl: &mut egui_snarl::Snarl<FlowNodes>,
    ) -> MyPinInfo {
        match &mut snarl[pin.id.node] {
            FlowNodes::LayerNodes(layer_nodes) => match layer_nodes {
                LayerNodes::STFTLayer(_) => {
                    ui.label("output STFTLayer");
                }
                LayerNodes::MelLayer(_) => {
                    ui.label("output MelLayer");
                }
                LayerNodes::SpectrogramDensityLayer(_) => {
                    ui.label("output SpectrogramDensityLayer");
                }
            },
            FlowNodes::ConfigNodes(config_nodes) => match config_nodes {
                ConfigNodes::NumberNode(node) => {
                    ui.end_row();

                    config_ui!(@fmt, node, ui, number);

                    config_ui!(node, ui, name);

                    return CustomPinInfo::setting(8);
                }
            },
            FlowNodes::AbstractInputNode(node) => match pin.id.output {
                0 => {
                    ui.label("raw stream");

                    node.update();
                }
                1 => {
                    ui.label("sample rate");
                }
                _ => unreachable!(),
            },
            FlowNodes::DataInspectorNode(_) => {
                ui.label(format!("shape.{:?}", pin.id.output));
            }
            FlowNodes::ExprNode(_) => {}
            FlowNodes::FrameBufferNode(frame_buffer) => match frame_buffer {
                FrameBufferNode::FrameQueueNode(_) => {
                    ui.label("FrameQueue");
                }
                FrameBufferNode::CycleBufferNode(_) => {
                    ui.label("CycleBuffer");
                }
            },
            FlowNodes::FrequencyNodes(frequency_nodes) => match frequency_nodes {
                FrequencyNodes::IFFTNode(_) => {
                    ui.label("IFFTNode");
                }
                FrequencyNodes::FFTNode(_) => {
                    ui.label("FFTNode");
                }
            },
            FlowNodes::FilterNodes(filter_nodes) => match filter_nodes {
                FilterNodes::LifterNode(_) => {
                    ui.label("LifterNode");
                }
            },
            FlowNodes::IterNodes(iter_nodes) => match iter_nodes {
                IterNodes::EnumerateIterNode(_) => {
                    ui.label("EnumerateIterNode");
                }
            },
            FlowNodes::LpcNodes(lpc_nodes) => match lpc_nodes {
                LpcNodes::LpcNode(_) => {
                    ui.label("LpcNode");
                }
                LpcNodes::BurgNode(_) => {
                    ui.label("BurgNode");
                }
            },
            FlowNodes::UnknownNode(_) => {
                return CustomPinInfo::none_status();
            }
            FlowNodes::OutputNodes(output_nodes) => match output_nodes {
                OutputNodes::OutputNode(_) => {
                    ui.label("OutputNode");
                }
            },
        }
        if !self.running {
            return CustomPinInfo::none_status();
        }

        CustomPinInfo::ok_status()
    }

    fn has_body(&mut self, node: &FlowNodes) -> bool {
        matches!(node, FlowNodes::UnknownNode(_))
    }

    /// Renders the node's body.
    #[inline]
    fn show_body(
        &mut self,
        node: NodeId,
        _inputs: &[InPin],
        _outputs: &[OutPin],
        ui: &mut Ui,
        _scale: f32,
        snarl: &mut Snarl<FlowNodes>,
    ) {
        match &mut snarl[node] {
            FlowNodes::UnknownNode(node) => {
                ui.label(
                    Into::<egui::WidgetText>::into(format!("unknown node: {}", node.name))
                        .color(egui::Color32::from_rgb(255, 0, 0)),
                );
            }
            _ => {}
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

        ui.menu_button("layer", |ui| {
            if ui.button("STFTLayer").clicked() {
                snarl.insert_node(pos, STFTLayerNodeInfo.flow_node());
                ui.close_menu();
            }

            if ui.button("MelLayer").clicked() {
                snarl.insert_node(pos, MelLayerNodeInfo.flow_node());
                ui.close_menu();
            }

            if ui.button("SpectrogramDensityLayer").clicked() {
                snarl.insert_node(pos, SpectrogramDensityLayerNodeInfo.flow_node());
                ui.close_menu();
            }
        });

        ui.menu_button("config", |ui| {
            if ui.button("number_node").clicked() {
                snarl.insert_node(pos, NumberNodeInfo.flow_node());
                ui.close_menu();
            }
        });

        if ui.button("AbstractInputNode").clicked() {
            snarl.insert_node(pos, AbstractInputNodeInfo.flow_node());
            ui.close_menu();
        }

        ui.menu_button("inspector", |ui| {
            if ui.button("DataPlotterNode").clicked() {
                snarl.insert_node(pos, DataPlotterNodeInfo.flow_node());
                ui.close_menu();
            }

            if ui.button("SchemaViewerNode").clicked() {
                snarl.insert_node(pos, SchemaViewerNodeInfo.flow_node());
                ui.close_menu();
            }
        });

        ui.menu_button("expr", |ui| {
            if ui.button("ExprNode").clicked() {
                snarl.insert_node(pos, ExprNodeInfo.flow_node());
                ui.close_menu();
            }
        });

        ui.menu_button("frame_buffer", |ui| {
            if ui.button("FrameQueue").clicked() {
                snarl.insert_node(pos, FrameQueueNodeInfo.flow_node());
                ui.close_menu();
            }

            if ui.button("CycleBuffer").clicked() {
                snarl.insert_node(pos, CycleBufferNodeInfo.flow_node());
                ui.close_menu();
            }
        });

        ui.menu_button("frequency", |ui| {
            if ui.button("IFFT").clicked() {
                snarl.insert_node(pos, IFFTNodeInfo.flow_node());
                ui.close_menu();
            }

            if ui.button("FFT").clicked() {
                snarl.insert_node(pos, FFTNodeInfo.flow_node());
                ui.close_menu();
            }
        });

        ui.menu_button("filter", |ui| {
            if ui.button("Lifter").clicked() {
                snarl.insert_node(pos, LifterNodeInfo.flow_node());
                ui.close_menu();
            }
        });

        ui.menu_button("iter", |ui| {
            if ui.button("EnumerateIterNode").clicked() {
                snarl.insert_node(pos, EnumerateIterNodeInfo.flow_node());
                ui.close_menu();
            }
        });

        ui.menu_button("lpc", |ui| {
            if ui.button("LpcNode").clicked() {
                snarl.insert_node(pos, LpcNodeInfo.flow_node());
                ui.close_menu();
            }

            if ui.button("BurgNode").clicked() {
                snarl.insert_node(pos, BurgNodeInfo.flow_node());
                ui.close_menu();
            }
        });

        ui.menu_button("output", |ui| {
            if ui.button("OutputNode").clicked() {
                snarl.insert_node(pos, OutputNodeInfo.flow_node());
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
            let duplicate_node: FlowNodes = snarl[node].serde_clone();

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

                            if in_type.contains_in(&out_type) {
                                if ui.button(node.name()).clicked() {
                                    let mut pos = pos;
                                    pos.x -= 100.0;
                                    pos.y += 20.0;

                                    let node = node.flow_node();

                                    // Create new node.
                                    let new_node = snarl.insert_node(pos, node);

                                    let dst_pin = egui_snarl::OutPinId {
                                        node: new_node,
                                        output: in_type.positions_in(&out_type)[0],
                                    };

                                    snarl.connect(dst_pin, pin.clone());

                                    ui.close_menu();
                                }
                            }
                        }
                    };

                let node = &snarl[pin.node];
                let as_info = node.to_as_info();

                view_connect_in_node(as_info.as_ref(), snarl);
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

                            if out_type.contains_out(&in_type) {
                                if ui.button(node.name()).clicked() {
                                    let mut pos = pos;
                                    pos.x += 100.0;
                                    pos.y += 20.0;

                                    let node = node.flow_node();

                                    // Create new node.
                                    let new_node = snarl.insert_node(pos, node);

                                    let src_pin = egui_snarl::InPinId {
                                        node: new_node,
                                        input: out_type.positions_out(&in_type)[0],
                                    };

                                    snarl.connect(pin.clone(), src_pin);

                                    ui.close_menu();
                                }
                            }
                        }
                    };

                let node = &snarl[pin.node];
                let as_info = node.to_as_info();

                view_connect_out_node(as_info.as_ref(), snarl);
            }
        }
    }
}
