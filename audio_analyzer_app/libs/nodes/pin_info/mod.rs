use egui_snarl::ui::{PinDrawer, PinShape};

use crate::prelude::{egui::*, snarl::*};

pub struct CustomPinInfo;

pub type MyPinInfo = PinInfo<MyDrawer>;

#[derive(Debug, Clone, Copy)]
pub enum MyDrawer {
    Lock,
    Setting { teeth: usize },
    Preset(PinShape),
}

impl From<PinShape> for MyDrawer {
    fn from(pin_shape: PinShape) -> Self {
        MyDrawer::Preset(pin_shape)
    }
}

impl PinDrawer for MyDrawer {
    fn draw(
        &self,
        painter: &egui::Painter,
        fill: egui::Color32,
        stroke: egui::Stroke,
        pos: Pos2,
        size: f32,
    ) {
        match self {
            MyDrawer::Lock => {
                let rect = Rect::from_center_size(pos, egui::vec2(size, size));
                let key_width = rect.width();
                let key_height = rect.height() * 0.7;

                // 丸い頭部分
                painter.circle_stroke(
                    Pos2::new(rect.center().x, rect.center().y - rect.height() / 4.),
                    rect.height() / 3.,
                    stroke,
                );

                // 鍵のシャフト部分（長方形）
                painter.rect(
                    egui::Rect::from_center_size(
                        Pos2::new(rect.center().x, rect.center().y + rect.height() / 4.),
                        Vec2::new(key_width, key_height),
                    ),
                    0.0,
                    fill,
                    stroke,
                );
            }
            MyDrawer::Setting { teeth } => {
                let teeth = *teeth;

                let rect = Rect::from_center_size(pos, egui::vec2(size, size));

                let inner_radius = rect.size().min_elem() * 0.6; // 中央の円のサイズ
                let outer_radius = rect.size().min_elem() * 0.8;
                let center = rect.center();

                let mut pointers = Vec::with_capacity(teeth * 3);

                for i in 0..8 {
                    let inner_angle = (i as f32 - 0.3) * std::f32::consts::PI * 2.0 / teeth as f32;
                    let inner_x = inner_radius * inner_angle.cos();
                    let inner_y = inner_radius * inner_angle.sin();
                    let inner_point = Pos2::new(center.x + inner_x, center.y + inner_y);

                    let angle = (i as f32 - 0.1) * (std::f32::consts::PI * 2.0 / teeth as f32);
                    let x = outer_radius * angle.cos();
                    let y = outer_radius * angle.sin();
                    let outer_point = Pos2::new(center.x + x, center.y + y);

                    let angle = (i as f32 + 0.1) * (std::f32::consts::PI * 2.0 / teeth as f32);
                    let x = outer_radius * angle.cos();
                    let y = outer_radius * angle.sin();
                    let outer_point2 = Pos2::new(center.x + x, center.y + y);

                    let angle = (i as f32 + 0.3) * (std::f32::consts::PI * 2.0 / teeth as f32);
                    let x = inner_radius * angle.cos();
                    let y = inner_radius * angle.sin();
                    let inner_point2 = Pos2::new(center.x + x, center.y + y);

                    pointers.push(inner_point);
                    pointers.push(outer_point);
                    pointers.push(outer_point2);
                    pointers.push(inner_point2);
                }

                let mut shape = PathShape::closed_line(pointers, stroke);
                shape.fill = fill;

                painter.add(Shape::Path(shape));
            }
            MyDrawer::Preset(pin_shape) => {
                pin_shape.draw(painter, fill, stroke, pos, size);
            }
        }
    }
}

impl CustomPinInfo {
    pub fn lock() -> PinInfo<MyDrawer> {
        PinInfo::shape(MyDrawer::Lock)
    }

    pub fn setting(teeth: usize) -> PinInfo<MyDrawer> {
        PinInfo::shape(MyDrawer::Setting { teeth })
    }

    pub fn ok_status() -> PinInfo<MyDrawer> {
        PinInfo::circle().with_fill(egui::Color32::from_rgb(0, 255, 0))
    }

    pub fn ng_status() -> PinInfo<MyDrawer> {
        PinInfo::circle().with_fill(egui::Color32::from_rgb(255, 0, 0))
    }

    pub fn none_status() -> PinInfo<MyDrawer> {
        PinInfo::circle().with_fill(egui::Color32::from_rgb(0, 0, 0))
    }
}
