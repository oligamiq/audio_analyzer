use crate::prelude::nodes::*;
use audio_analyzer_core::prelude::*;

use serde::de;
use std::any::TypeId;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum LayerNodes {
    STFTLayer(STFTLayerNode),
    MelLayer(MelLayerNode),
    SpectrogramDensityLayer(SpectrogramDensityLayerNode),
}

impl LayerNodes {
    pub fn name(&self) -> &str {
        match self {
            LayerNodes::STFTLayer(_) => "STFTLayer",
            LayerNodes::MelLayer(_) => "MelLayer",
            LayerNodes::SpectrogramDensityLayer(_) => "SpectrogramDensityLayer",
        }
    }

    pub fn update(&mut self) {
        match self {
            LayerNodes::STFTLayer(node) => node.update(),
            LayerNodes::MelLayer(node) => node.update(),
            LayerNodes::SpectrogramDensityLayer(node) => node.update(),
        }
    }

    pub const fn inputs(&self) -> usize {
        // input is layer data and config
        match self {
            LayerNodes::STFTLayer(_) => STFTLayerNode::input(),
            LayerNodes::MelLayer(_) => MelLayerNode::input(),
            LayerNodes::SpectrogramDensityLayer(_) => SpectrogramDensityLayerNode::input(),
        }
    }

    pub const fn outputs(&self) -> usize {
        1
    }

    pub fn input_type_id(&self) -> TypeId {
        match self {
            LayerNodes::STFTLayer(_) => {
                TypeId::of::<<STFTLayerNode as InputAndOutputType>::Input>()
            }
            LayerNodes::MelLayer(_) => TypeId::of::<<MelLayerNode as InputAndOutputType>::Input>(),
            LayerNodes::SpectrogramDensityLayer(_) => {
                TypeId::of::<<SpectrogramDensityLayerNode as InputAndOutputType>::Input>()
            }
        }
    }

    pub fn output_type_id(&self) -> TypeId {
        match self {
            LayerNodes::STFTLayer(_) => {
                TypeId::of::<<STFTLayerNode as InputAndOutputType>::Output>()
            }
            LayerNodes::MelLayer(_) => TypeId::of::<<MelLayerNode as InputAndOutputType>::Output>(),
            LayerNodes::SpectrogramDensityLayer(_) => {
                TypeId::of::<<SpectrogramDensityLayerNode as InputAndOutputType>::Output>()
            }
        }
    }

    pub fn validate_connections(&self, to: &LayerNodes) -> bool {
        let input_type_id = self.output_type_id();
        let output_type_id = to.input_type_id();

        input_type_id == output_type_id
    }
}

pub trait InputAndOutputType {
    type Input: 'static;
    type Output: 'static;
}

#[derive(Debug, serde::Serialize)]
pub struct STFTLayerNode {
    pub fft_size: EditableOnText<usize>,
    pub hop_size: EditableOnText<usize>,

    #[serde(skip)]
    layer: ToSpectrogramLayer,
    #[serde(skip)]
    result: Option<Array1<Complex<f64>>>,
}

impl FlowNodesViewerTrait for STFTLayerNode {
    fn show_input(
        &self,
        pin: &InPin,
        ui: &mut egui::Ui,
        _scale: f32,
        snarl: &Snarl<FlowNodes>,
    ) -> Box<dyn Fn(&mut Snarl<FlowNodes>, &mut egui::Ui) -> PinInfo> {
        let pin_id = pin.id;

        match pin.id.input {
            0 => {
                if let Some(out_pin) = pin.remotes.get(0) {
                    if let FlowNodes::ConfigNodes(ConfigNodes::NumberNode(NumberNode {
                        number,
                        ..
                    })) = &snarl[out_pin.node]
                    {
                        ui.label(format!("fft_size: {}", number));

                        let fft_size = number.get();

                        return Box::new(move |snarl: &mut Snarl<FlowNodes>, _: &mut egui::Ui| {
                            extract_node!(
                                &mut snarl[pin_id.node],
                                FlowNodes::LayerNodes(LayerNodes::STFTLayer(node)) => {
                                    if node.fft_size.set(fft_size as usize) {
                                        node.update();
                                    }
                                }
                            );

                            CustomPinInfo::lock()
                        });
                    }
                }

                return Box::new(move |snarl: &mut Snarl<FlowNodes>, ui: &mut egui::Ui| {
                    let node = extract_node!(
                        &mut snarl[pin_id.node],
                        FlowNodes::LayerNodes(LayerNodes::STFTLayer(node)) => node
                    );

                    config_ui!(@fmt, node, ui, fft_size);

                    CustomPinInfo::setting(8)
                });
            }
            1 => {
                if let Some(out_pin) = pin.remotes.get(0) {
                    if let FlowNodes::ConfigNodes(ConfigNodes::NumberNode(NumberNode {
                        number,
                        ..
                    })) = &snarl[out_pin.node]
                    {
                        ui.label(format!("hop_size: {}", number));

                        let hop_size = number.get();

                        return Box::new(move |snarl: &mut Snarl<FlowNodes>, _: &mut egui::Ui| {
                            extract_node!(
                                &mut snarl[pin_id.node],
                                FlowNodes::LayerNodes(LayerNodes::STFTLayer(node)) => {
                                    if node.hop_size.set(hop_size as usize) {
                                        node.update();
                                    }
                                }
                            );

                            CustomPinInfo::lock()
                        });
                    }
                }

                return Box::new(move |snarl: &mut Snarl<FlowNodes>, ui: &mut egui::Ui| {
                    let node = extract_node!(
                        &mut snarl[pin_id.node],
                        FlowNodes::LayerNodes(LayerNodes::STFTLayer(node)) => node
                    );

                    config_ui!(@fmt, node, ui, hop_size);

                    CustomPinInfo::setting(8)
                });
            }
            2 => {
                ui.label("raw stream");

                if let Some(out_pin) = pin.remotes.get(0) {
                    if let FlowNodes::RawInputNodes(node) = &snarl[out_pin.node] {
                        if let Some(data) = node.get() {
                            return Box::new(move |snarl: &mut Snarl<FlowNodes>, _| {
                                extract_node!(
                                    &mut snarl[pin_id.node],
                                    FlowNodes::LayerNodes(LayerNodes::STFTLayer(node)) => {
                                        if let Err(err) = node.calc(&data) {
                                            log::error!("STFTLayerNode: {}", err);
                                        }
                                    }
                                );

                                CustomPinInfo::lock()
                            });
                        }
                    }
                }

                return Box::new(|_, _| {
                    PinInfo::circle().with_fill(egui::Color32::from_rgb(255, 0, 0))
                });
            }
            _ => unreachable!(),
        }
    }
}

pub struct STFTLayerNodeInfo;

impl NodeInfo for STFTLayerNodeInfo {
    fn name(&self) -> &str {
        "STFTLayer"
    }

    fn inputs(&self) -> usize {
        STFTLayerNode::input()
    }

    fn outputs(&self) -> usize {
        1
    }

    fn input_types(&self) -> Vec<NodeInfoTypes> {
        vec![
            NodeInfoTypes::Number,
            NodeInfoTypes::Number,
            NodeInfoTypes::VecF32,
        ]
    }

    fn output_types(&self) -> Vec<NodeInfoTypes> {
        vec![NodeInfoTypes::Array1ComplexF64]
    }

    fn flow_node(&self) -> super::editor::FlowNodes {
        super::editor::FlowNodes::LayerNodes(LayerNodes::STFTLayer(Default::default()))
    }
}

impl<'a> serde::Deserialize<'a> for STFTLayerNode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'a>,
    {
        #[derive(serde::Deserialize)]
        struct STFTLayerNodeHelper {
            fft_size: EditableOnText<usize>,
            hop_size: EditableOnText<usize>,
        }

        let helper = STFTLayerNodeHelper::deserialize(deserializer)?;

        Ok(STFTLayerNode::new(
            helper.fft_size.get(),
            helper.hop_size.get(),
        ))
    }
}

impl Default for STFTLayerNode {
    fn default() -> Self {
        let config = FftConfig::default();

        Self::new(config.fft_size, config.hop_size)
    }
}

impl InputAndOutputType for STFTLayerNode {
    type Input = Vec<f32>;
    type Output = Array1<Complex<f64>>;
}

impl STFTLayerNode {
    pub fn new(fft_size: usize, hop_size: usize) -> Self {
        let layer = ToSpectrogramLayer::new(FftConfig::new(fft_size, hop_size));

        Self {
            fft_size: EditableOnText::new(fft_size),
            hop_size: EditableOnText::new(hop_size),
            layer,
            result: None,
        }
    }

    pub fn calc(&mut self, data: &Vec<f32>) -> crate::Result<()> {
        self.result = self.layer.through_inner(data)?.pop();

        Ok(())
    }

    pub fn update(&mut self) {
        self.layer =
            ToSpectrogramLayer::new(FftConfig::new(self.fft_size.get(), self.hop_size.get()));
    }

    pub fn get_result(&self) -> Option<Array1<Complex<f64>>> {
        self.result.clone()
    }

    // input_num is layer data and config num
    pub const fn input() -> usize {
        1 + 2
    }

    pub fn to_info(&self) -> STFTLayerNodeInfo {
        STFTLayerNodeInfo
    }
}

#[derive(Debug, serde::Serialize)]
pub struct MelLayerNode {
    pub fft_size: EditableOnText<usize>,
    pub hop_size: EditableOnText<usize>,
    pub n_mels: EditableOnText<usize>,
    pub sample_rate: EditableOnText<f64>,

    #[serde(skip)]
    layer: ToMelSpectrogramLayer,

    #[serde(skip)]
    result: Option<Array2<f64>>,
}

pub struct MelLayerNodeInfo;

impl NodeInfo for MelLayerNodeInfo {
    fn name(&self) -> &str {
        "MelLayer"
    }

    fn inputs(&self) -> usize {
        MelLayerNode::input()
    }

    fn outputs(&self) -> usize {
        1
    }

    fn input_types(&self) -> Vec<NodeInfoTypes> {
        vec![
            NodeInfoTypes::Number,
            NodeInfoTypes::Number,
            NodeInfoTypes::Array1ComplexF64,
        ]
    }

    fn output_types(&self) -> Vec<NodeInfoTypes> {
        vec![NodeInfoTypes::Array2F64]
    }

    fn flow_node(&self) -> super::editor::FlowNodes {
        super::editor::FlowNodes::LayerNodes(LayerNodes::MelLayer(Default::default()))
    }
}

impl<'a> serde::Deserialize<'a> for MelLayerNode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'a>,
    {
        #[derive(serde::Deserialize)]
        struct MelLayerNodeHelper {
            fft_size: EditableOnText<usize>,
            hop_size: EditableOnText<usize>,
            n_mels: EditableOnText<usize>,
            sample_rate: EditableOnText<f64>,
        }

        let helper = MelLayerNodeHelper::deserialize(deserializer)?;

        Ok(MelLayerNode::new(
            helper.fft_size.get(),
            helper.hop_size.get(),
            helper.n_mels.get(),
            helper.sample_rate.get(),
        ))
    }
}

impl Default for MelLayerNode {
    fn default() -> Self {
        Self::new(400, 160, 128, 44100.0)
    }
}

impl InputAndOutputType for MelLayerNode {
    type Input = Array1<Complex<f64>>;
    type Output = Array2<f64>;
}

impl MelLayerNode {
    pub fn new(fft_size: usize, hop_size: usize, n_mels: usize, sample_rate: f64) -> Self {
        let layer =
            ToMelSpectrogramLayer::new(MelConfig::new(fft_size, hop_size, n_mels, sample_rate));

        Self {
            fft_size: EditableOnText::new(fft_size),
            hop_size: EditableOnText::new(hop_size),
            n_mels: EditableOnText::new(n_mels),
            sample_rate: EditableOnText::new(sample_rate),
            layer,
            result: None,
        }
    }

    pub fn calc(&mut self, data: Array1<Complex<f64>>) -> crate::Result<()> {
        self.result = self.layer.through_inner(&data)?;

        Ok(())
    }

    pub fn update(&mut self) {
        self.layer = ToMelSpectrogramLayer::new(MelConfig::new(
            self.fft_size.get(),
            self.hop_size.get(),
            self.n_mels.get(),
            self.sample_rate.get(),
        ));
    }

    pub fn get_result(&self) -> Option<Array2<f64>> {
        self.result.clone()
    }

    // input_num is layer data and config num
    pub const fn input() -> usize {
        1 + 4
    }

    pub fn to_info(&self) -> MelLayerNodeInfo {
        MelLayerNodeInfo
    }
}

#[derive(Debug, serde::Serialize)]
pub struct SpectrogramDensityLayerNode {
    pub sample_rate: EditableOnText<f64>,
    pub time_range: EditableOnText<usize>,
    pub n_mels: EditableOnText<usize>,

    #[serde(skip)]
    layer: ToPowerSpectralDensityLayer,

    #[serde(skip)]
    result: Option<Array1<(f64, f64)>>,
}

pub struct SpectrogramDensityLayerNodeInfo;

impl NodeInfo for SpectrogramDensityLayerNodeInfo {
    fn name(&self) -> &str {
        "SpectrogramDensityLayer"
    }

    fn inputs(&self) -> usize {
        SpectrogramDensityLayerNode::input()
    }

    fn outputs(&self) -> usize {
        1
    }

    fn input_types(&self) -> Vec<NodeInfoTypes> {
        vec![
            NodeInfoTypes::Number,
            NodeInfoTypes::Number,
            NodeInfoTypes::Number,
            NodeInfoTypes::Array2F64,
        ]
    }

    fn output_types(&self) -> Vec<NodeInfoTypes> {
        vec![NodeInfoTypes::Array1TupleF64F64]
    }

    fn flow_node(&self) -> super::editor::FlowNodes {
        super::editor::FlowNodes::LayerNodes(
            LayerNodes::SpectrogramDensityLayer(Default::default()),
        )
    }
}

impl<'a> serde::Deserialize<'a> for SpectrogramDensityLayerNode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'a>,
    {
        #[derive(serde::Deserialize)]
        struct SpectrogramDensityLayerNodeHelper {
            sample_rate: f64,
            time_range: usize,
            n_mels: usize,
        }

        let helper = SpectrogramDensityLayerNodeHelper::deserialize(deserializer)?;

        Ok(SpectrogramDensityLayerNode::new(
            helper.sample_rate,
            helper.time_range,
            helper.n_mels,
        ))
    }
}

impl Default for SpectrogramDensityLayerNode {
    fn default() -> Self {
        Self::new(44100.0, 20, 128)
    }
}

impl InputAndOutputType for SpectrogramDensityLayerNode {
    type Input = Array2<f64>;
    type Output = Array1<(f64, f64)>;
}

impl SpectrogramDensityLayerNode {
    pub fn new(sample_rate: f64, time_range: usize, n_mels: usize) -> Self {
        let layer = ToPowerSpectralDensityLayer::new(ToPowerSpectralDensityLayerConfig {
            sample_rate: sample_rate.into(),
            time_range: time_range,
            n_mels: n_mels,
        });

        Self {
            sample_rate: EditableOnText::new(sample_rate),
            time_range: EditableOnText::new(time_range),
            n_mels: EditableOnText::new(n_mels),
            layer,
            result: None,
        }
    }

    pub fn calc(&mut self, data: Array2<f64>) -> crate::Result<()> {
        self.result = self.layer.through_inner(&data)?;

        Ok(())
    }

    pub fn update(&mut self) {
        self.layer = ToPowerSpectralDensityLayer::new(ToPowerSpectralDensityLayerConfig {
            sample_rate: self.sample_rate.get().into(),
            time_range: self.time_range.get(),
            n_mels: self.n_mels.get(),
        });
    }

    pub fn get_result(&self) -> Option<Array1<(f64, f64)>> {
        self.result.clone()
    }

    // input_num is layer data and config num
    pub const fn input() -> usize {
        1 + 3
    }

    pub fn to_info(&self) -> SpectrogramDensityLayerNodeInfo {
        SpectrogramDensityLayerNodeInfo
    }
}
