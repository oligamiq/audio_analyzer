use std::sync::Arc;

use rustfft::{Fft, FftPlanner};

use crate::prelude::nodes::*;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum FrequencyNodes {
    IFFTNode(IFFTNode),
    FFTNode(FFTNode),
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct IFFTNode {
    #[serde(skip)]
    pub(crate) fft_size: usize,
    #[serde(skip)]
    calculated: Vec<Complex<f64>>,
    #[serde(skip)]
    scratch_buf: Vec<Complex<f64>>,
    #[serde(skip)]
    fft: Arc<dyn Fft<f64>>,
}

impl Default for IFFTNode {
    fn default() -> Self {
        let fft_size = 400;
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_inverse(fft_size);
        let scratch_buf = vec![Complex::new(0.0, 0.0); fft_size];
        let calculated = vec![Complex::new(0.0, 0.0); fft_size];

        Self {
            fft_size,
            calculated,
            scratch_buf,
            fft,
        }
    }
}

impl core::fmt::Debug for IFFTNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IFFTNode")
            .field("calculated", &self.calculated)
            .finish()
    }
}

impl FlowNodesViewerTrait for IFFTNode {
    fn show_input(
        &self,
        ctx: &FlowNodesViewerCtx,
        pin: &egui_snarl::InPin,
        ui: &mut egui::Ui,
        _scale: f32,
        snarl: &egui_snarl::Snarl<FlowNodes>,
    ) -> Box<dyn Fn(&mut Snarl<FlowNodes>, &mut egui::Ui) -> MyPinInfo> {
        let pin_id = pin.id;

        match pin_id.input {
            0 => {
                ui.label("input");

                if !ctx.running {
                    return Box::new(|_, _| CustomPinInfo::none_status());
                }

                if let Some(out_pin) = pin.remotes.get(0) {
                    let data = snarl[out_pin.node].to_node_info_types_with_data(out_pin.output);

                    if let Some(NodeInfoTypesWithData::Array1ComplexF64(data)) = data {
                        return Box::new(move |snarl, _ui| {
                            extract_node!(
                                &mut snarl[pin_id.node],
                                FlowNodes::FrequencyNodes(FrequencyNodes::IFFTNode(node)) => {
                                    node.through_inner(&data.to_vec());
                                }
                            );

                            CustomPinInfo::ok_status()
                        });
                    }
                }

                return Box::new(|_, _| CustomPinInfo::ng_status());
            }
            _ => unreachable!(),
        }
    }
}

impl IFFTNode {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_result(&self) -> Option<Array1<Complex<f64>>> {
        Some(Array1::from(self.calculated.clone()))
    }

    pub fn through_inner(&mut self, input: &[Complex<f64>]) {
        if self.fft_size != input.len() {
            let fft_size = input.len();

            self.fft_size = fft_size;
            self.update();
        }

        let fft = &self.fft;
        let scratch_buf = &mut self.scratch_buf;
        let calculated = &mut self.calculated;

        // IFFT
        calculated.copy_from_slice(input);
        fft.process_with_scratch(calculated, scratch_buf);
    }
}

pub struct IFFTNodeInfo;

impl NodeInfo for IFFTNodeInfo {
    fn name(&self) -> &'static str {
        "IFFT"
    }

    fn inputs(&self) -> usize {
        1
    }

    fn outputs(&self) -> usize {
        1
    }

    fn input_types(&self) -> Vec<NodeInfoTypes> {
        vec![NodeInfoTypes::Array1ComplexF64]
    }

    fn output_types(&self) -> Vec<NodeInfoTypes> {
        vec![NodeInfoTypes::Array1ComplexF64]
    }

    fn flow_node(&self) -> FlowNodes {
        FlowNodes::FrequencyNodes(FrequencyNodes::IFFTNode(IFFTNode::new()))
    }
}

impl GraphNode for IFFTNode {
    type NodeInfoType = IFFTNodeInfo;

    fn to_info(&self) -> Self::NodeInfoType {
        IFFTNodeInfo
    }

    fn update(&mut self) {
        // adjust fft size

        let Self {
            fft_size,
            calculated,
            scratch_buf,
            fft,
        } = self;

        *calculated = vec![Complex::new(0.0, 0.0); *fft_size];
        *scratch_buf = vec![Complex::new(0.0, 0.0); *fft_size];
        *fft = FftPlanner::new().plan_fft_inverse(*fft_size);
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct FFTNode {
    #[serde(skip)]
    pub(crate) fft_size: usize,
    #[serde(skip)]
    calculated: Vec<Complex<f64>>,
    #[serde(skip)]
    scratch_buf: Vec<Complex<f64>>,
    #[serde(skip)]
    fft: Arc<dyn Fft<f64>>,
}

impl Default for FFTNode {
    fn default() -> Self {
        let fft_size = 400;
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(fft_size);
        let scratch_buf = vec![Complex::new(0.0, 0.0); fft_size];
        let calculated = vec![Complex::new(0.0, 0.0); fft_size];

        Self {
            fft_size,
            calculated,
            scratch_buf,
            fft,
        }
    }
}

impl core::fmt::Debug for FFTNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FFTNode")
            .field("calculated", &self.calculated)
            .finish()
    }
}

impl FlowNodesViewerTrait for FFTNode {
    fn show_input(
        &self,
        ctx: &FlowNodesViewerCtx,
        pin: &egui_snarl::InPin,
        ui: &mut egui::Ui,
        _scale: f32,
        snarl: &egui_snarl::Snarl<FlowNodes>,
    ) -> Box<dyn Fn(&mut Snarl<FlowNodes>, &mut egui::Ui) -> MyPinInfo> {
        let pin_id = pin.id;

        match pin_id.input {
            0 => {
                ui.label("input");

                if !ctx.running {
                    return Box::new(|_, _| CustomPinInfo::none_status());
                }

                if let Some(out_pin) = pin.remotes.get(0) {
                    let data = snarl[out_pin.node].to_node_info_types_with_data(out_pin.output);

                    if let Some(NodeInfoTypesWithData::Array1ComplexF64(data)) = data {
                        return Box::new(move |snarl, _ui| {
                            extract_node!(
                                &mut snarl[pin_id.node],
                                FlowNodes::FrequencyNodes(FrequencyNodes::FFTNode(node)) => {
                                    node.through_inner(&data.to_vec());
                                }
                            );

                            CustomPinInfo::ok_status()
                        });
                    }
                }

                return Box::new(|_, _| CustomPinInfo::ng_status());
            }
            _ => unreachable!(),
        }
    }
}

impl FFTNode {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_result(&self) -> Option<Array1<Complex<f64>>> {
        Some(Array1::from(self.calculated.clone()))
    }

    pub fn through_inner(&mut self, input: &[Complex<f64>]) {
        if self.fft_size != input.len() {
            let fft_size = input.len();

            self.fft_size = fft_size;
            self.update();
        }

        let fft = &self.fft;
        let scratch_buf = &mut self.scratch_buf;
        let calculated = &mut self.calculated;

        // FFT
        calculated.copy_from_slice(input);
        fft.process_with_scratch(calculated, scratch_buf);
    }
}

pub struct FFTNodeInfo;

impl NodeInfo for FFTNodeInfo {
    fn name(&self) -> &'static str {
        "FFT"
    }

    fn inputs(&self) -> usize {
        1
    }

    fn outputs(&self) -> usize {
        1
    }

    fn input_types(&self) -> Vec<NodeInfoTypes> {
        vec![NodeInfoTypes::Array1ComplexF64]
    }

    fn output_types(&self) -> Vec<NodeInfoTypes> {
        vec![NodeInfoTypes::Array1ComplexF64]
    }

    fn flow_node(&self) -> FlowNodes {
        FlowNodes::FrequencyNodes(FrequencyNodes::FFTNode(FFTNode::new()))
    }
}

impl GraphNode for FFTNode {
    type NodeInfoType = FFTNodeInfo;

    fn to_info(&self) -> Self::NodeInfoType {
        FFTNodeInfo
    }

    fn update(&mut self) {
        // adjust fft size

        let Self {
            fft_size,
            calculated,
            scratch_buf,
            fft,
        } = self;

        *calculated = vec![Complex::new(0.0, 0.0); *fft_size];
        *scratch_buf = vec![Complex::new(0.0, 0.0); *fft_size];
        *fft = FftPlanner::new().plan_fft_forward(*fft_size);
    }
}
