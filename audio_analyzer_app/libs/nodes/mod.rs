use egui_snarl::ui::{PinInfo, SnarlViewer};
use ndarray::{Array1, Array2};
use num_complex::Complex;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub enum LayerNodes {
    STFTLayer(Vec<f32>),
    MelLayer(Array1<Complex<f64>>),
    SpectrogramDensityLayer(Array2<f64>),
}

impl LayerNodes {
    fn name(&self) -> &str {
        match self {
            LayerNodes::STFTLayer(_) => "STFTLayer",
            LayerNodes::MelLayer(_) => "MelLayer",
            LayerNodes::SpectrogramDensityLayer(_) => "SpectrogramDensityLayer",
        }
    }
}

pub struct LayerNodesViewer;

impl SnarlViewer<LayerNodes> for LayerNodesViewer {
    fn title(&mut self, node: &LayerNodes) -> String {
        match node {
            LayerNodes::STFTLayer(_) => "STFTLayer".to_string(),
            LayerNodes::MelLayer(_) => "MelLayer".to_string(),
            LayerNodes::SpectrogramDensityLayer(_) => "SpectrogramDensityLayer".to_string(),
        }
    }

    fn outputs(&mut self, node: &LayerNodes) -> usize {
        match node {
            LayerNodes::STFTLayer(_) => 1,
            LayerNodes::MelLayer(_) => 1,
            LayerNodes::SpectrogramDensityLayer(_) => 1,
        }
    }

    fn inputs(&mut self, node: &LayerNodes) -> usize {
        match node {
            LayerNodes::STFTLayer(_) => 1,
            LayerNodes::MelLayer(_) => 1,
            LayerNodes::SpectrogramDensityLayer(_) => 1,
        }
    }

    fn show_input(
        &mut self,
        pin: &egui_snarl::InPin,
        ui: &mut egui::Ui,
        scale: f32,
        snarl: &mut egui_snarl::Snarl<LayerNodes>,
    ) -> egui_snarl::ui::PinInfo {
        match &snarl[pin.id.node] {
            LayerNodes::STFTLayer(vec) => {
                assert_eq!(pin.id.input, 0, "STFTLayer has only one input pin");

                match &*pin.remotes {
                    [] => {
                        ui.label("none");
                        PinInfo::star().with_fill(egui::Color32::from_rgb(0, 0, 0))
                    }
                    [remote] => match &snarl[remote.node] {
                        LayerNodes::STFTLayer(vec) => todo!(),
                        LayerNodes::MelLayer(array_base) => todo!(),
                        LayerNodes::SpectrogramDensityLayer(array_base) => todo!(),
                    },
                    _ => todo!(),
                }
            }
            LayerNodes::MelLayer(array_base) => todo!(),
            LayerNodes::SpectrogramDensityLayer(array_base) => todo!(),
        }
    }

    fn show_output(
        &mut self,
        pin: &egui_snarl::OutPin,
        ui: &mut egui::Ui,
        scale: f32,
        snarl: &mut egui_snarl::Snarl<LayerNodes>,
    ) -> egui_snarl::ui::PinInfo {
        match &snarl[pin.id.node] {
            LayerNodes::STFTLayer(vec) => {
                assert_eq!(pin.id.output, 0, "STFTLayer has only one output pin");

                match &*pin.remotes {
                    [] => {
                        ui.label("none");
                        PinInfo::star().with_fill(egui::Color32::from_rgb(0, 0, 0))
                    }
                    [remote] => match &snarl[remote.node] {
                        LayerNodes::STFTLayer(vec) => todo!(),
                        LayerNodes::MelLayer(array_base) => todo!(),
                        LayerNodes::SpectrogramDensityLayer(array_base) => todo!(),
                    },
                    _ => todo!(),
                }
            }
            LayerNodes::MelLayer(array_base) => todo!(),
            LayerNodes::SpectrogramDensityLayer(array_base) => todo!(),
        }
    }
}
