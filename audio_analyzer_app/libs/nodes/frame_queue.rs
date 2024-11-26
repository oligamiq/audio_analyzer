use ndarray::{concatenate, prelude::*};

use crate::prelude::nodes::*;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum FrameBufferNode {
    FrameQueueNode(FrameQueueNode),
    CycleBufferNode(CycleBufferNode),
}

impl FrameBufferNode {
    pub fn name(&self) -> &str {
        match self {
            Self::FrameQueueNode(_) => "FrameQueueNode",
            Self::CycleBufferNode(_) => "CycleBufferNode",
        }
    }

    pub fn inputs(&self) -> usize {
        match self {
            Self::FrameQueueNode(_) => FrameQueueNodeInfo.inputs(),
            Self::CycleBufferNode(_) => CycleBufferNodeInfo.inputs(),
        }
    }

    pub fn outputs(&self) -> usize {
        match self {
            Self::FrameQueueNode(_) => FrameQueueNodeInfo.outputs(),
            Self::CycleBufferNode(_) => CycleBufferNodeInfo.outputs(),
        }
    }
}

/// FrameQueueNode
/// This node holds a queue of frames.
/// FIFO
///
/// if the queue is full, the oldest frame is removed
/// and the new frame is added to the end of the queue
///
/// hold n frames
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct FrameQueueNode {
    queue: NodeInfoTypesWithData,

    pub len: EditableOnText<usize>,
}

impl FrameQueueNode {
    pub fn get_queue(&self) -> &NodeInfoTypesWithData {
        &self.queue
    }
}

pub struct FrameQueueNodeInfo;

impl NodeInfo for FrameQueueNodeInfo {
    fn name(&self) -> &str {
        "FrameQueueNode"
    }

    fn inputs(&self) -> usize {
        1
    }

    fn outputs(&self) -> usize {
        1
    }

    fn input_types(&self) -> Vec<NodeInfoTypes> {
        vec![NodeInfoTypes::AnyInput]
    }

    fn output_types(&self) -> Vec<NodeInfoTypes> {
        vec![NodeInfoTypes::AnyOutput]
    }

    fn flow_node(&self) -> super::editor::FlowNodes {
        super::editor::FlowNodes::FrameBufferNode(FrameBufferNode::FrameQueueNode(
            Default::default(),
        ))
    }
}

impl Default for FrameQueueNode {
    fn default() -> Self {
        Self {
            queue: NodeInfoTypesWithData::Array1F64(ndarray::Array1::zeros(0)),
            len: EditableOnText::new(16),
        }
    }
}

impl GraphNode for FrameQueueNode {
    type NodeInfoType = FrameQueueNodeInfo;

    fn to_info(&self) -> Self::NodeInfoType {
        FrameQueueNodeInfo
    }
}

impl FlowNodesViewerTrait for FrameQueueNode {
    fn show_input(
        &self,
        pin: &egui_snarl::InPin,
        _ui: &mut egui::Ui,
        _scale: f32,
        snarl: &egui_snarl::Snarl<FlowNodes>,
    ) -> Box<dyn Fn(&mut Snarl<FlowNodes>, &mut egui::Ui) -> PinInfo> {
        assert!(pin.id.input == 0);

        let pin_id = pin.id;

        let data = pin
            .remotes
            .get(0)
            .map(|out_pin| snarl[out_pin.node].to_node_info_types_with_data(out_pin.output))
            .flatten();

        return Box::new(move |snarl: &mut Snarl<FlowNodes>, ui: &mut egui::Ui| {
            if let FlowNodes::FrameBufferNode(FrameBufferNode::FrameQueueNode(node)) =
                &mut snarl[pin_id.node]
            {
                config_ui!(node, ui, len);

                if let Some(data) = data.clone() {
                    match data {
                        NodeInfoTypesWithData::Number(f) => match node.queue {
                            NodeInfoTypesWithData::Array1F64(ref mut queue) => {
                                if queue.len() >= node.len.get() {
                                    queue.remove_index(Axis(0), 0);
                                }

                                queue.push(Axis(0), arr0(f).view()).unwrap();
                            }
                            _ => {
                                node.queue = NodeInfoTypesWithData::Array1F64(arr1(&[f]));
                            }
                        },
                        NodeInfoTypesWithData::Array1F64(data) => match node.queue {
                            NodeInfoTypesWithData::Array2F64(ref mut queue) => {
                                if queue.len() >= node.len.get() {
                                    queue.remove_index(Axis(0), 0);
                                }

                                queue.push(Axis(0), data.view()).unwrap();
                            }
                            _ => {
                                node.queue = NodeInfoTypesWithData::Array2F64(
                                    ndarray::Array2::from_shape_vec((1, data.len()), data.to_vec())
                                        .unwrap(),
                                );
                            }
                        },
                        _ => {
                            static mut WARNED: bool = false;

                            if !unsafe { WARNED } {
                                log::warn!("FrameQueueNode: Unsupported data type: {:?}", data);
                                unsafe { WARNED = true };
                            }
                        }
                    }
                }
            }

            CustomPinInfo::none_status()
        });
    }
}

/// CycleBufferNode
/// This node holds a buffer of frames.
/// FIFO
/// if the buffer is full, the oldest frame is removed
/// and the new frame is added to the end of the buffer
///
/// to extend the buffer, buffer size must be self.len
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct CycleBufferNode {
    buffer: NodeInfoTypesWithData,

    pub len: EditableOnText<usize>,
}

impl CycleBufferNode {
    pub fn get_queue(&self) -> &NodeInfoTypesWithData {
        &self.buffer
    }
}

pub struct CycleBufferNodeInfo;

impl NodeInfo for CycleBufferNodeInfo {
    fn name(&self) -> &str {
        "CycleBufferNode"
    }

    fn inputs(&self) -> usize {
        1
    }

    fn outputs(&self) -> usize {
        1
    }

    fn input_types(&self) -> Vec<NodeInfoTypes> {
        vec![NodeInfoTypes::AnyInput]
    }

    fn output_types(&self) -> Vec<NodeInfoTypes> {
        vec![NodeInfoTypes::AnyOutput]
    }

    fn flow_node(&self) -> super::editor::FlowNodes {
        super::editor::FlowNodes::FrameBufferNode(FrameBufferNode::CycleBufferNode(
            Default::default(),
        ))
    }
}

impl Default for CycleBufferNode {
    fn default() -> Self {
        Self {
            buffer: NodeInfoTypesWithData::Array1F64(ndarray::Array1::zeros(0)),
            len: EditableOnText::new(16),
        }
    }
}

impl GraphNode for CycleBufferNode {
    type NodeInfoType = CycleBufferNodeInfo;

    fn to_info(&self) -> Self::NodeInfoType {
        CycleBufferNodeInfo
    }
}

impl FlowNodesViewerTrait for CycleBufferNode {
    fn show_input(
        &self,
        pin: &egui_snarl::InPin,
        _ui: &mut egui::Ui,
        _scale: f32,
        snarl: &egui_snarl::Snarl<FlowNodes>,
    ) -> Box<dyn Fn(&mut Snarl<FlowNodes>, &mut egui::Ui) -> PinInfo> {
        assert!(pin.id.input == 0);

        let pin_id = pin.id;

        let data = pin
            .remotes
            .get(0)
            .map(|out_pin| snarl[out_pin.node].to_node_info_types_with_data(out_pin.output))
            .flatten();

        return Box::new(move |snarl: &mut Snarl<FlowNodes>, ui: &mut egui::Ui| {
            if let FlowNodes::FrameBufferNode(FrameBufferNode::CycleBufferNode(node)) =
                &mut snarl[pin_id.node]
            {
                config_ui!(node, ui, len);

                if let Some(data) = data.clone() {
                    match data {
                        NodeInfoTypesWithData::Array1F64(data) => match node.buffer {
                            NodeInfoTypesWithData::Array1F64(ref mut buffer) => {
                                let extended =
                                    concatenate(Axis(0), &[buffer.view(), data.view()]).unwrap();

                                let new_len = extended.len();

                                if new_len > node.len.get() {
                                    let diff = new_len - node.len.get();
                                    let new_buffer = extended.slice(s![diff..]);
                                    *buffer = new_buffer.to_owned();

                                    assert!(new_buffer.len() == node.len.get());
                                } else {
                                    *buffer = extended;
                                }
                            }
                            _ => {
                                let extended = data;

                                let new_len = extended.len();

                                if new_len > node.len.get() {
                                    let diff = new_len - node.len.get();
                                    let new_buffer = extended.slice(s![diff..]);
                                    node.buffer =
                                        NodeInfoTypesWithData::Array1F64(new_buffer.to_owned());

                                    assert!(new_buffer.len() == node.len.get());
                                } else {
                                    node.buffer = NodeInfoTypesWithData::Array1F64(extended);
                                }
                            }
                        },
                        NodeInfoTypesWithData::Array1ComplexF64(data) => match node.buffer {
                            NodeInfoTypesWithData::Array1ComplexF64(ref mut buffer) => {
                                let extended =
                                    concatenate(Axis(0), &[buffer.view(), data.view()]).unwrap();

                                let new_len = extended.len();

                                if new_len > node.len.get() {
                                    let diff = new_len - node.len.get();
                                    let new_buffer = extended.slice(s![diff..]);
                                    *buffer = new_buffer.to_owned();

                                    assert!(new_buffer.len() == node.len.get());
                                } else {
                                    *buffer = extended;
                                }
                            }
                            _ => {
                                let extended = data;

                                let new_len = extended.len();

                                if new_len > node.len.get() {
                                    let diff = new_len - node.len.get();
                                    let new_buffer = extended.slice(s![diff..]);
                                    node.buffer = NodeInfoTypesWithData::Array1ComplexF64(
                                        new_buffer.to_owned(),
                                    );

                                    assert!(new_buffer.len() == node.len.get());
                                } else {
                                    node.buffer = NodeInfoTypesWithData::Array1ComplexF64(extended);
                                }
                            }
                        },
                        NodeInfoTypesWithData::Array1TupleF64F64(data) => match node.buffer {
                            NodeInfoTypesWithData::Array1TupleF64F64(ref mut buffer) => {
                                let extended =
                                    concatenate(Axis(0), &[buffer.view(), data.view()]).unwrap();

                                let new_len = extended.len();

                                if new_len > node.len.get() {
                                    let diff = new_len - node.len.get();
                                    let new_buffer = extended.slice(s![diff..]);
                                    *buffer = new_buffer.to_owned();

                                    assert!(new_buffer.len() == node.len.get());
                                } else {
                                    *buffer = extended;
                                }
                            }
                            _ => {
                                let extended = data;

                                let new_len = extended.len();

                                if new_len > node.len.get() {
                                    let diff = new_len - node.len.get();
                                    let new_buffer = extended.slice(s![diff..]);
                                    node.buffer = NodeInfoTypesWithData::Array1TupleF64F64(
                                        new_buffer.to_owned(),
                                    );

                                    assert!(new_buffer.len() == node.len.get());
                                } else {
                                    node.buffer =
                                        NodeInfoTypesWithData::Array1TupleF64F64(extended);
                                }
                            }
                        },
                        NodeInfoTypesWithData::Array2F64(data) => match node.buffer {
                            NodeInfoTypesWithData::Array2F64(ref mut buffer) => {
                                let extended =
                                    concatenate(Axis(0), &[buffer.view(), data.view()]).unwrap();

                                let new_len = extended.len();

                                if new_len > node.len.get() {
                                    let diff = new_len - node.len.get();
                                    let new_buffer = extended.slice(s![diff.., ..]);
                                    *buffer = new_buffer.to_owned();

                                    assert!(new_buffer.len() == node.len.get());
                                } else {
                                    *buffer = extended;
                                }
                            }
                            _ => {
                                let extended = data;

                                let new_len = extended.len();

                                if new_len > node.len.get() {
                                    let diff = new_len - node.len.get();
                                    let new_buffer = extended.slice(s![diff.., ..]);
                                    node.buffer =
                                        NodeInfoTypesWithData::Array2F64(new_buffer.to_owned());

                                    assert!(new_buffer.len() == node.len.get());
                                } else {
                                    node.buffer = NodeInfoTypesWithData::Array2F64(extended);
                                }
                            }
                        },
                        _ => {
                            static mut WARNED: bool = false;

                            if !unsafe { WARNED } {
                                log::warn!("CycleBufferNode: Unsupported data type: {:?}", data);
                                unsafe { WARNED = true };
                            }
                        }
                    }
                }
            }

            CustomPinInfo::none_status()
        });
    }
}
