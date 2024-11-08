#![allow(unused_imports)]

pub mod nodes {
    pub(crate) use crate::libs::nodes::{
        config::{ConfigNodes, NumberNode, NumberNodeInfo},
        editor::{FlowNodes, FlowNodesViewer, FlowNodesViewerTrait},
        expr::{ExprNodeInfo, ExprNodes},
        frame_queue::{CycleBuffer, CycleBufferInfo, FrameBuffer, FrameQueue, FrameQueueInfo},
        layer::{
            LayerNodes, MelLayerNode, MelLayerNodeInfo, STFTLayerNode, STFTLayerNodeInfo,
            SpectrogramDensityLayerNode, SpectrogramDensityLayerNodeInfo,
        },
        raw_input::{
            FileInputNode, FileInputNodeInfo, MicrophoneInputNode, MicrophoneInputNodeInfo,
            RawInputNodes,
        },
        utils::{config_ui, extract_node},
        viewer::{DataPlotterNode, DataPlotterNodeInfo},
        GraphNode, NodeInfo, NodeInfoTypes, NodeInfoTypesWithData,
    };
    pub use egui_editable_num::EditableOnText;
    pub use ndarray::{Array1, Array2};
    pub use num_complex::Complex;

    pub use super::snarl::*;
}

pub mod snarl {
    pub(crate) use crate::libs::nodes::{editor::FlowNodes, pin_info::CustomPinInfo};
    pub use egui_snarl::{
        ui::{AnyPins, PinInfo, SnarlStyle, SnarlViewer},
        InPin, NodeId, OutPin, Snarl,
    };
}

pub mod egui {
    pub use egui::{
        epaint::PathShape, Pos2, Rect, Separator, Shape, Ui, UiBuilder, Vec2, Widget as _,
    };
}

pub mod ui {
    pub use crate::libs::widget::{UiWidget, View};
}

pub mod utils {
    pub use crate::libs::{
        nodes::SerdeClone, separate_window_widget::SeparateWindowWidget,
        utils::log::LogViewerWidget,
    };
}
