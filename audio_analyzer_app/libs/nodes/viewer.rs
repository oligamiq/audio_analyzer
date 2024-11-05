use crate::prelude::{egui::*, nodes::*};
use egui_plotter::EguiBackend;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct DataPlotterNode {
    pub name: String,
    pub size: EditableOnText<usize>,
    chart_pitch: f32,
    chart_yaw: f32,
    chart_scale: f32,
    chart_pitch_vel: f32,
    chart_yaw_vel: f32,

    #[serde(skip)]
    pub hold_data: Option<NodeInfoTypesWithData>,
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
            chart_pitch: 0.3,
            chart_yaw: 0.9,
            chart_scale: 0.9,
            chart_pitch_vel: 0.0,
            chart_yaw_vel: 0.0,
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

    pub fn show(&mut self, ui: &mut egui::Ui, _is_new: bool, scale: f32) {
        let hold_data = self.hold_data.clone();

        if let Some(hold_data) = hold_data {
            match hold_data {
                NodeInfoTypesWithData::Number(number) => {
                    ui.label(format!("Number: {}", number));
                }
                NodeInfoTypesWithData::VecF32(vec_f32) => {
                    self.show_vec_f32(ui, vec_f32, scale);
                }
                NodeInfoTypesWithData::Array1F64(array1_f64) => {
                    // todo show_array1_f64

                    self.show_vec_f32(ui, array1_f64.iter().map(|x| *x as f32).collect(), scale);
                }
                NodeInfoTypesWithData::Array1TupleF64F64(array1_tuple_f64_f64) => {
                    self.show_array1_tuple_f64_f64(ui, array1_tuple_f64_f64, scale);
                }
                NodeInfoTypesWithData::Array2F64(array2_f64) => {
                    ui.label(format!("Array2F64: {:?}", array2_f64));
                }
                NodeInfoTypesWithData::Array1ComplexF64(array1_complex_f64) => {
                    self.show_array1_complex_f64(ui, array1_complex_f64, scale);
                }
            }
        }
    }

    pub fn show_vec_f32(&mut self, ui: &mut egui::Ui, vec_f32: Vec<f32>, scale: f32) {
        self.plot(ui, scale, |ui, scale, _| {
            let root = EguiBackend::new(&ui).into_drawing_area();

            let mut chart = ChartBuilder::on(&root)
                .caption("vec<f32> stream", ("sans-serif", 5. * scale).into_font())
                .margin(10. * scale)
                .x_label_area_size((3. * scale) as i32)
                .y_label_area_size((3. * scale) as i32)
                .build_cartesian_2d(-0.1f32..1.1f32, -0.1f32..1.1f32)
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

            // chart
            //     .configure_series_labels()
            //     .legend_area_size((2. * scale) as i32)
            //     .label_font(("sans-serif", 5. * scale))
            //     .background_style(&WHITE.mix(0.8))
            //     .border_style(&BLACK)
            //     .draw()
            //     .unwrap();

            root.present().unwrap();
        });
    }

    pub fn show_array1_tuple_f64_f64(
        &mut self,
        ui: &mut egui::Ui,
        array1_tuple_f64_f64: Array1<(f64, f64)>,
        scale: f32,
    ) {
        self.plot(ui, scale, |ui, scale, _| {
            let root = EguiBackend::new(&ui).into_drawing_area();

            let mut chart = ChartBuilder::on(&root)
                .caption(
                    "Array1TupleF64F64 stream",
                    ("sans-serif", 5. * scale).into_font(),
                )
                .margin(10. * scale)
                .x_label_area_size((3. * scale) as i32)
                .y_label_area_size((3. * scale) as i32)
                .build_cartesian_2d(-0.1f64..1.1f64, -0.1f64..1.1f64)
                .unwrap();

            chart
                .configure_mesh()
                .label_style(("sans-serif", 5. * scale).into_font())
                .draw()
                .unwrap();

            chart
                .draw_series(LineSeries::new(
                    array1_tuple_f64_f64
                        .iter()
                        .enumerate()
                        .map(|(x, y)| (x as f64 / array1_tuple_f64_f64.len() as f64, y.0)),
                    &RED,
                ))
                .unwrap()
                // .label("Array1<(F64, F64)> stream")
                .legend(|(x, y)| {
                    PathElement::new(vec![(x, y), (x + (2. * scale) as i32, y)], &RED)
                });

            chart
                .draw_series(LineSeries::new(
                    array1_tuple_f64_f64
                        .iter()
                        .enumerate()
                        .map(|(x, y)| (x as f64 / array1_tuple_f64_f64.len() as f64, y.1)),
                    &BLUE,
                ))
                .unwrap()
                // .label("Array1<(F64, F64)> stream")
                .legend(|(x, y)| {
                    PathElement::new(vec![(x, y), (x + (2. * scale) as i32, y)], &BLUE)
                });

            // chart
            //     .configure_series_labels()
            //     .legend_area_size((2. * scale) as i32)
            //     .label_font(("sans-serif", 5. * scale))
            //     .background_style(&WHITE.mix(0.8))
            //     .border_style(&BLACK)
            //     .draw()
            //     .unwrap();

            root.present().unwrap();
        });
    }

    pub fn show_array1_complex_f64(
        &mut self,
        ui: &mut egui::Ui,
        complex_array: Array1<num_complex::Complex<f64>>,
        scale: f32,
    ) {
        // グラフにプロットするためのデータに変換しますの
        // x = 時間軸、y = 周波数軸（仮定）、z = 振幅（複素数の絶対値）
        let points: Vec<_> = complex_array
            .iter()
            .enumerate()
            .map(|(i, c)| {
                let time = i as f64; // 時間軸 (例としてインデックスを使用)
                let frequency = (i % 5) as f64; // 周波数軸 (例として周波数を5に分割)
                let amplitude = c.norm(); // 複素数の振幅（絶対値）
                (time, frequency, amplitude)
            })
            .collect();

        // let x_max = points.iter().map(|(x, _, _)| *x).fold(0.0, f64::max);
        // let y_max = points.iter().map(|(_, y, _)| *y).fold(0.0, f64::max);
        // let z_max = points.iter().map(|(_, _, z)| *z).fold(0.0, f64::max);

        // log::trace!("x_max: {}, y_max: {}, z_max: {}", x_max, y_max, z_max);

        self.plot(ui, scale, |ui, scale, sl| {
            const MOVE_SCALE: f32 = 0.01;
            const SCROLL_SCALE: f32 = 0.001;

            // First, get mouse data
            if ui.rect_contains_pointer(ui.max_rect()) {
                let (pitch_delta, yaw_delta, scale_delta) = ui.input(|input| {
                    let pointer = &input.pointer;
                    let delta = pointer.delta();

                    let (pitch_delta, yaw_delta) = match pointer.primary_down() {
                        true => (delta.y * MOVE_SCALE, -delta.x * MOVE_SCALE),
                        false => (sl.chart_pitch_vel, sl.chart_yaw_vel),
                    };

                    let scale_delta = input.raw_scroll_delta.y * SCROLL_SCALE;

                    (pitch_delta, yaw_delta, scale_delta)
                });

                sl.chart_pitch_vel = pitch_delta;
                sl.chart_yaw_vel = yaw_delta;

                sl.chart_pitch += sl.chart_pitch_vel;
                sl.chart_yaw += sl.chart_yaw_vel;
                sl.chart_scale += scale_delta;
            }

            let root = EguiBackend::new(&ui).into_drawing_area();

            let mut chart = ChartBuilder::on(&root)
                .caption(
                    "Array1<Complex<F64>> stream",
                    ("sans-serif", 5. * scale).into_font(),
                )
                .margin(10. * scale)
                .x_label_area_size((3. * scale) as i32)
                .y_label_area_size((3. * scale) as i32)
                .build_cartesian_3d(-10f64..1010f64, -0.1f64..5.1f64, -0.1f64..1.1f64)
                .unwrap();

            chart.with_projection(|mut pb| {
                pb.yaw = sl.chart_yaw as f64;
                pb.pitch = sl.chart_pitch as f64;
                pb.scale = sl.chart_scale as f64;
                pb.into_matrix()
            });

            chart
                .configure_axes()
                .light_grid_style(&WHITE.mix(0.8))
                .max_light_lines(3)
                .label_style(("sans-serif", 5. * scale).into_font())
                .draw()
                .unwrap();

            chart
                .draw_series(
                    points
                        .iter()
                        .map(|(x, y, z)| Circle::new((*x, *y, *z), 2, &RED)),
                )
                .unwrap()
                .label("Array1<Complex<F64>> stream")
                .legend(|(x, y)| {
                    Rectangle::new(
                        [(x, y), (x + (2. * scale) as i32, y)],
                        BLUE.mix(0.5).filled(),
                    )
                });

            root.present().unwrap();
        });
    }

    pub fn plot<F: FnMut(&mut egui::Ui, f32, &mut DataPlotterNode)>(
        &mut self,
        ui: &mut egui::Ui,
        scale: f32,
        mut f: F,
    ) {
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
                f(new_ui, scale, self);
            },
        );
    }
}

impl FlowNodesViewerTrait for DataPlotterNode {
    fn show_input(
        &self,
        pin: &egui_snarl::InPin,
        _: &mut egui::Ui,
        scale: f32,
        snarl: &egui_snarl::Snarl<FlowNodes>,
    ) -> Box<dyn Fn(&mut Snarl<FlowNodes>, &mut egui::Ui) -> PinInfo> {
        let pin_id = pin.id;

        if let Some(out_pin) = pin.remotes.get(0) {
            let remote = &snarl[out_pin.node];

            let data = remote.to_node_info_types_with_data(out_pin.output);

            return Box::new(move |snarl: &mut Snarl<FlowNodes>, ui: &mut egui::Ui| {
                extract_node!(
                    &mut snarl[pin_id.node],
                    FlowNodes::DataPlotterNode(node) => {
                        if let Some(data) = &data {
                            node.set_hold_data(data.clone());
                            node.show(ui, true, scale);

                            return PinInfo::circle()
                                .with_fill(egui::Color32::from_rgb(0, 255, 0));
                        } else {
                            node.show(ui, false, scale);
                        }
                    }
                );

                PinInfo::circle().with_fill(egui::Color32::from_rgb(0, 0, 255))
            });
        }

        return Box::new(|_, _| PinInfo::circle().with_fill(egui::Color32::from_rgb(0, 0, 255)));
    }
}
