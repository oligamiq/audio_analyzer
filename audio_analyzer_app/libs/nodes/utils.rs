use crate::prelude::nodes::*;

impl FlowNodes {
    pub fn to_node_info_types_with_data(&self, pin: usize) -> Option<NodeInfoTypesWithData> {
        match self {
            FlowNodes::LayerNodes(layer_nodes) => match layer_nodes {
                LayerNodes::STFTLayer(stft_layer_node) => Some(
                    NodeInfoTypesWithData::Array1ComplexF64(stft_layer_node.get_result()?),
                ),
                // LayerNodes::MelLayer(mel_layer_node) => Some(NodeInfoTypesWithData::Array2F64(
                //     mel_layer_node.get_result()?,
                // )),
                LayerNodes::MelLayer(to_mel_spectrogram_layer_node) => Some(
                    NodeInfoTypesWithData::Array1F64(to_mel_spectrogram_layer_node.get_result()?),
                ),
                LayerNodes::SpectrogramDensityLayer(spectrogram_density_layer_node) => {
                    Some(NodeInfoTypesWithData::Array1TupleF64F64(
                        spectrogram_density_layer_node.get_result()?,
                    ))
                }
            },
            FlowNodes::ConfigNodes(config_nodes) => match config_nodes {
                ConfigNodes::NumberNode(number_node) => {
                    Some(NodeInfoTypesWithData::Number(number_node.number.get()))
                }
            },
            FlowNodes::DataInspectorNode(data_inspector_node) => match data_inspector_node {
                DataInspectorNode::DataPlotterNode(_) => None,
                DataInspectorNode::SchemaViewerNode(schema_viewer_node) => {
                    let shape = schema_viewer_node.get_shape();
                    match pin {
                        0 => shape
                            .map(|x| x.get(0).cloned())
                            .flatten()
                            .map(|x| NodeInfoTypesWithData::Number(x)),
                        1 => shape
                            .map(|x| x.get(1).cloned())
                            .flatten()
                            .map(|x| NodeInfoTypesWithData::Number(x)),
                        _ => unreachable!(),
                    }
                }
            },
            FlowNodes::AbstractInputNode(raw_input_nodes) => {
                match pin {
                    // raw stream
                    0 => Some(NodeInfoTypesWithData::Array1F64(raw_input_nodes.get()?)),

                    // sample rate
                    1 => Some(NodeInfoTypesWithData::Number(
                        raw_input_nodes.get_sample_rate() as f64,
                    )),

                    _ => unreachable!(),
                }
            }
            FlowNodes::ExprNode(expr_nodes) => expr_nodes.calculated.clone(),
            FlowNodes::FrameBufferNode(frame_buffer) => match frame_buffer {
                FrameBufferNode::FrameQueueNode(frame_queue) => {
                    Some(frame_queue.get_queue().clone())
                }
                FrameBufferNode::CycleBufferNode(cycle_buffer) => {
                    Some(cycle_buffer.get_queue().clone())
                }
            },
            FlowNodes::FrequencyNodes(frequency_nodes) => match frequency_nodes {
                FrequencyNodes::IFFTNode(idct_node) => Some(
                    NodeInfoTypesWithData::Array1ComplexF64(idct_node.get_result()?),
                ),
                FrequencyNodes::FFTNode(fft_node) => Some(NodeInfoTypesWithData::Array1ComplexF64(
                    fft_node.get_result()?,
                )),
            },
            FlowNodes::FilterNodes(filter_nodes) => match filter_nodes {
                FilterNodes::LifterNode(lifter_node) => {
                    Some(NodeInfoTypesWithData::Array1F64(lifter_node.get_result()?))
                }
            },
            FlowNodes::IterNodes(iter_nodes) => match iter_nodes {
                IterNodes::EnumerateIterNode(enumerate_iter_node) => Some(
                    NodeInfoTypesWithData::Array1F64(enumerate_iter_node.get_result()?),
                ),
            },
            FlowNodes::LpcNodes(lpc_nodes) => match lpc_nodes {
                LpcNodes::LpcNode(lpc_node) => {
                    Some(NodeInfoTypesWithData::Array1F64(lpc_node.get_result()?))
                }
            },
        }
    }
}

/// config_ui!(node, ui, expr);
macro_rules! config_ui {
    (@fmt, $node:ident, $ui:ident, $config:ident) => {
        $ui.label(stringify!($config));
        let response = egui::TextEdit::singleline(&mut $node.$config)
            .clip_text(false)
            .desired_width(0.0)
            .margin($ui.spacing().item_spacing)
            .show($ui)
            .response;

        if response.lost_focus() {
            $node.$config.fmt();
            $node.update();
        } else if response.changed() {
            if $node.$config.try_update() {
                $node.update();
            }
        }
    };

    ($node:ident, $ui:ident, $config:ident) => {
        $ui.label(stringify!($config));
        let response = egui::TextEdit::singleline(&mut $node.$config)
            .clip_text(false)
            .desired_width(0.0)
            .margin($ui.spacing().item_spacing)
            .show($ui)
            .response;

        if response.lost_focus() {
            $node.update();
        } else if response.changed() {
            $node.update();
        }
    };
}

pub(crate) use config_ui;

macro_rules! extract_node {
    ($expr:expr, $pattern:pat => $result:expr) => {
        if let $pattern = $expr {
            $result
        } else {
            unreachable!()
        }
    };
}

pub(crate) use extract_node;
