use crate::prelude::nodes::*;

impl FlowNodes {
    pub fn to_node_info_types_with_data(&self) -> Option<NodeInfoTypesWithData> {
        match self {
            FlowNodes::LayerNodes(layer_nodes) => match layer_nodes {
                LayerNodes::STFTLayer(stft_layer_node) => Some(
                    NodeInfoTypesWithData::Array1ComplexF64(stft_layer_node.get_result()?),
                ),
                LayerNodes::MelLayer(mel_layer_node) => Some(NodeInfoTypesWithData::Array2F64(
                    mel_layer_node.get_result()?,
                )),
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
            FlowNodes::DataPlotterNode(_) => None,
            FlowNodes::RawInputNodes(raw_input_nodes) => {
                Some(NodeInfoTypesWithData::VecF32(raw_input_nodes.get()?))
            }
            FlowNodes::ExprNode(expr_nodes) => expr_nodes.calculated.clone(),
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
