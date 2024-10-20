use super::{config::ConfigNodes, editor::FlowNodes, layer::LayerNodes, NodeInfoTypesWithData};

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
        }
    }
}
