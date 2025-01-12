use crate::prelude::nodes::*;

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct UnknownNode {
    pub name: String,
    pub input_num: usize,
    pub output_num: usize,
}

impl Default for UnknownNode {
    fn default() -> Self {
        Self {
            name: "Unknown".to_string(),
            input_num: 1,
            output_num: 1,
        }
    }
}

impl FlowNodesViewerTrait for UnknownNode {
    fn show_input(
        &self,
        _: &FlowNodesViewerCtx,
        _: &egui_snarl::InPin,
        _: &mut egui::Ui,
        _: f32,
        _: &egui_snarl::Snarl<FlowNodes>,
    ) -> Box<dyn Fn(&mut Snarl<FlowNodes>, &mut egui::Ui) -> MyPinInfo> {
        Box::new(move |_, _| CustomPinInfo::none_status())
    }
}

#[derive(Debug, Clone)]
pub struct UnknownNodeInfo {
    pub name: String,
    pub input_num: usize,
    pub output_num: usize,
}

impl From<UnknownNode> for UnknownNodeInfo {
    fn from(node: UnknownNode) -> Self {
        Self {
            name: node.name,
            input_num: node.input_num,
            output_num: node.output_num,
        }
    }
}

impl From<UnknownNodeInfo> for UnknownNode {
    fn from(info: UnknownNodeInfo) -> Self {
        Self {
            name: info.name,
            input_num: info.input_num,
            output_num: info.output_num,
        }
    }
}

impl NodeInfo for UnknownNodeInfo {
    fn name(&self) -> &str {
        "Unknown Node"
    }

    fn inputs(&self) -> usize {
        self.input_num
    }

    fn outputs(&self) -> usize {
        self.output_num
    }

    fn input_types(&self) -> Vec<NodeInfoTypes> {
        vec![NodeInfoTypes::AnyInput; self.input_num]
    }

    fn output_types(&self) -> Vec<NodeInfoTypes> {
        vec![NodeInfoTypes::AnyOutput; self.output_num]
    }

    fn flow_node(&self) -> super::editor::FlowNodes {
        FlowNodes::UnknownNode(UnknownNode::default())
    }
}

impl UnknownNode {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_result(&self) -> Option<NodeInfoTypesWithData> {
        None
    }
}

impl GraphNode for UnknownNode {
    type NodeInfoType = UnknownNodeInfo;

    fn to_info(&self) -> Self::NodeInfoType {
        UnknownNodeInfo::from(self.clone())
    }
}
