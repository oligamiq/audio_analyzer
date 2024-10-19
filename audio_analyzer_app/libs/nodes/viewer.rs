use egui::{Pos2, Rect, Separator, UiBuilder, Vec2, Widget as _};
use egui_editable_num::EditableOnText;
use egui_plotter::EguiBackend;

use super::NodeInfoTypesWithData;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct DataPlotterNode {
    pub name: String,
    pub hold_data: Option<NodeInfoTypesWithData>,
    pub size: EditableOnText<usize>,
}

use plotters::prelude::*;

pub struct DataPlotterNodeInfo;

impl super::NodeInfo for DataPlotterNodeInfo {
    fn name(&self) -> &str {
        "DataPlotterNode"
    }

    fn inputs(&self) -> usize {
        1
    }

    fn outputs(&self) -> usize {
        0
    }

    fn input_types(&self) -> Vec<super::NodeInfoTypes> {
        vec![super::NodeInfoTypes::AnyInput]
    }

    fn output_types(&self) -> Vec<super::NodeInfoTypes> {
        vec![]
    }

    fn flow_node(&self) -> super::editor::FlowNodes {
        super::editor::FlowNodes::DataPlotterNode(DataPlotterNode::default())
    }
}

impl Default for DataPlotterNode {
    fn default() -> Self {
        Self::new("DataPlotterNode".to_string())
    }
}

impl DataPlotterNode {
    pub fn new(name: String) -> Self {
        Self {
            name,
            hold_data: None,
            size: EditableOnText::new(100),
        }
    }

    pub fn set_hold_data(&mut self, hold_data: NodeInfoTypesWithData) {
        self.hold_data = Some(hold_data);
    }

    pub fn get_hold_data(&self) -> Option<&NodeInfoTypesWithData> {
        self.hold_data.as_ref()
    }

    pub fn to_info(&self) -> DataPlotterNodeInfo {
        DataPlotterNodeInfo
    }

    pub fn show(&mut self, ui: &mut egui::Ui, is_new: bool, scale: f32) {
        let hold_data = self.hold_data.clone();

        if let Some(hold_data) = hold_data {
            match hold_data {
                NodeInfoTypesWithData::Number(number) => {
                    ui.label(format!("Number: {}", number));
                }
                NodeInfoTypesWithData::VecF32(vec_f32) => {
                    self.show_vec_f32(ui, vec_f32, scale);
                }
                NodeInfoTypesWithData::Array1TupleF64F64(array1_tuple_f64_f64) => {
                    ui.label(format!("Array1TupleF64F64: {:?}", array1_tuple_f64_f64));
                }
                NodeInfoTypesWithData::Array2F64(array2_f64) => {
                    ui.label(format!("Array2F64: {:?}", array2_f64));
                }
                NodeInfoTypesWithData::Array1ComplexF64(array1_complex_f64) => {
                    ui.label(format!("Array1ComplexF64: {:?}", array1_complex_f64));
                }
            }
        }
    }

    pub fn show_vec_f32(&mut self, ui: &mut egui::Ui, vec_f32: Vec<f32>, scale: f32) {
        self.plot(ui, scale, |ui, scale| {
            let root = EguiBackend::new(&ui).into_drawing_area();

            let mut chart = ChartBuilder::on(&root)
                .caption("vec<f32> stream", ("sans-serif", 5. * scale).into_font())
                .margin(10. * scale)
                .x_label_area_size((3. * scale) as i32)
                .y_label_area_size((3. * scale) as i32)
                .build_cartesian_2d(0f32..1f32, -0.1f32..1f32)
                .unwrap();

            chart
                .configure_mesh()
                .label_style(("sans-serif", 5. * scale).into_font())
                .draw()
                .unwrap();

            chart
                .draw_series(LineSeries::new(
                    vec_f32
                        .iter()
                        .enumerate()
                        .map(|(x, y)| (x as f32 / vec_f32.len() as f32, *y)),
                    &RED,
                ))
                .unwrap()
                .label("vec<f32> stream")
                .legend(|(x, y)| {
                    PathElement::new(vec![(x, y), (x + (2. * scale) as i32, y)], &RED)
                });

            chart
                .configure_series_labels()
                .legend_area_size((2. * scale) as i32)
                .label_font(("sans-serif", 5. * scale))
                .background_style(&WHITE.mix(0.8))
                .border_style(&BLACK)
                .draw()
                .unwrap();

            root.present().unwrap();
        });
    }

    pub fn plot<F: FnMut(&mut egui::Ui, f32)>(&mut self, ui: &mut egui::Ui, scale: f32, mut f: F) {
        ui.scope_builder(
            UiBuilder::new().max_rect({
                let mut rect = ui.available_rect_before_wrap();
                rect.max.x = rect.min.x + 100.0 * scale;
                rect.max.y = rect.min.y + 100.0 * scale;
                rect
            }),
            |ui| {
                ui.columns(1, |ui| {
                    let ui = &mut ui[0];

                    ui.label(self.name.clone());

                    Separator::default().horizontal().ui(ui);

                    ui.label("size: ");

                    if egui::TextEdit::singleline(&mut self.size)
                        .clip_text(false)
                        .show(ui)
                        .response
                        .lost_focus()
                    {
                        self.size.fmt();
                    }

                    ui.separator();
                });

                ui.separator();
            },
        );

        let scale = scale * self.size.get() as f32 / 100.0;

        ui.set_min_size(Vec2::new(100.0 * scale, 100.0 * scale));

        ui.scope_builder(
            UiBuilder::new().max_rect({
                let mut rect = ui.available_rect_before_wrap();
                rect.max.x = rect.min.x + 100.0 * scale;
                rect.max.y = rect.min.y + 100.0 * scale;
                rect
            }),
            |new_ui| {
                f(new_ui, scale);
            },
        );
    }
}
