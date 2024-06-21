// https://github.com/plotters-rs/plotters/blob/16714cffdabcda17c241394d4178d97489f1f711/plotters/examples/console.rs

use std::fmt::Debug;
use std::io::Write as _;
use std::sync::RwLock;

use anyhow::Result;
use crossterm::{queue, QueueableCommand};
use fundsp::Num;
use plotters::prelude::*;
use plotters::style::text_anchor::{HPos, VPos};
use plotters_backend::{
    BackendColor, BackendStyle, BackendTextStyle, DrawingBackend, DrawingErrorKind,
};

#[derive(Copy, Clone)]
enum PixelState {
    Empty,
    HLine(Option<BackendColor>),
    VLine(Option<BackendColor>),
    Cross(Option<BackendColor>),
    Pixel(BackendColor),
    Text(char),
    Circle(bool),
}

impl Debug for PixelState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PixelState::Empty => write!(f, " "),
            PixelState::HLine(_) => write!(f, "─"),
            PixelState::VLine(_) => write!(f, "│"),
            PixelState::Cross(_) => write!(f, "┼"),
            PixelState::Pixel(_) => write!(f, "█"),
            PixelState::Text(chr) => write!(f, "{}", chr),
            PixelState::Circle(filled) => write!(f, "{}", if *filled { "●" } else { "○" }),
        }
    }
}

impl PixelState {
    fn add_color(a: Option<BackendColor>, b: Option<BackendColor>) -> Option<BackendColor> {
        match (a, b) {
            (Some(a), Some(b)) => Some(BackendColor {
                alpha: (a.alpha + b.alpha) / 2.,
                rgb: (
                    ((a.rgb.0 as u32 + b.rgb.0 as u32) / 2) as u8,
                    ((a.rgb.1 as u32 + b.rgb.1 as u32) / 2) as u8,
                    ((a.rgb.2 as u32 + b.rgb.2 as u32) / 2) as u8,
                ),
            }),
            (Some(a), None) => Some(a),
            (None, Some(a)) => Some(a),
            (None, None) => None,
        }
    }

    pub fn update(&mut self, new: PixelState) {
        let n = match (*self, new) {
            (PixelState::HLine(a), PixelState::VLine(b)) => {
                PixelState::Cross(Self::add_color(a, b))
            }
            (PixelState::VLine(a), PixelState::HLine(b)) => {
                PixelState::Cross(Self::add_color(a, b))
            }

            (
                PixelState::Cross(_) | PixelState::HLine(_) | PixelState::VLine(_),
                PixelState::Empty | PixelState::Pixel(_),
            ) => *self,

            (PixelState::Cross(a), PixelState::HLine(b) | PixelState::VLine(b)) => {
                PixelState::Cross(Self::add_color(a, b))
            }

            // others
            _ => new,
        };

        *self = n;
    }

    pub(crate) fn get_color(color: Option<BackendColor>) -> Option<crossterm::style::Color> {
        match color {
            Some(c) => Some(crossterm::style::Color::Rgb {
                r: c.rgb.0,
                g: c.rgb.1,
                b: c.rgb.2,
            }),
            None => None,
        }
    }
}

pub struct TextDrawingBackend {
    buff: Vec<Vec<PixelState>>,
    size: (u32, u32),
}

impl TextDrawingBackend {
    pub fn new() -> Self {
        let (x, y) = crossterm::terminal::size().unwrap();
        let (x, y) = (x as u32, y as u32);

        // let x = (x / (4 * 3)).floor() * 4 * 3;
        // let y = (y / (4 * 3)).floor() * 4 * 3;

        // 3:4に調整する
        let x = (x / 4) * 3;
        let y = (y / 3) * 4;

        Self {
            buff: vec![vec![PixelState::Empty; x as usize]; y as usize],
            size: (x, y),
        }
    }
}

impl DrawingBackend for TextDrawingBackend {
    type ErrorType = std::io::Error;

    fn get_size(&self) -> (u32, u32) {
        self.size
    }

    fn ensure_prepared(&mut self) -> Result<(), DrawingErrorKind<std::io::Error>> {
        Ok(())
    }

    fn present(&mut self) -> Result<(), DrawingErrorKind<std::io::Error>> {
        let mut stdout = std::io::stdout();

        for row in &self.buff {
            for pixel in row {
                match pixel {
                    PixelState::Empty => {
                        stdout.queue(crossterm::style::Print(" ")).unwrap();
                    }
                    PixelState::HLine(c) => {
                        if let Some(c) = PixelState::get_color(*c) {
                            stdout.queue(crossterm::style::SetForegroundColor(c)).unwrap();
                            stdout.queue(crossterm::style::Print("─")).unwrap();
                            stdout.queue(crossterm::style::ResetColor).unwrap();
                        } else {
                            stdout.queue(crossterm::style::Print("─")).unwrap();
                        }
                    }
                    PixelState::VLine(c) => {
                        if let Some(c) = PixelState::get_color(*c) {
                            stdout.queue(crossterm::style::SetForegroundColor(c)).unwrap();
                            stdout.queue(crossterm::style::Print("│")).unwrap();
                            stdout.queue(crossterm::style::ResetColor).unwrap();
                        } else {
                            stdout.queue(crossterm::style::Print("│")).unwrap();
                        }
                    }
                    PixelState::Cross(c) => {
                        if let Some(c) = PixelState::get_color(*c) {
                            stdout.queue(crossterm::style::SetForegroundColor(c)).unwrap();
                            stdout.queue(crossterm::style::Print("┼")).unwrap();
                            stdout.queue(crossterm::style::ResetColor).unwrap();
                        } else {
                            stdout.queue(crossterm::style::Print("┼")).unwrap();
                        }
                    }
                    PixelState::Pixel(color) => {
                        stdout
                            .queue(crossterm::style::SetForegroundColor(
                                PixelState::get_color(Some(*color)).unwrap(),
                            ))
                            .unwrap();
                        stdout.queue(crossterm::style::Print("█")).unwrap();
                        stdout.queue(crossterm::style::ResetColor).unwrap();
                    }
                    PixelState::Text(chr) => {
                        stdout.queue(crossterm::style::Print(chr)).unwrap();
                    }
                    PixelState::Circle(filled) => {
                        stdout
                            .queue(crossterm::style::Print(if *filled { "●" } else { "○" }))
                            .unwrap();
                    }
                }
            }
            stdout.queue(crossterm::style::Print("\n")).unwrap();
        }

        stdout.flush().unwrap();

        Ok(())
    }

    fn draw_pixel(
        &mut self,
        pos: (i32, i32),
        color: BackendColor,
    ) -> Result<(), DrawingErrorKind<std::io::Error>> {
        self.buff[pos.1 as usize][pos.0 as usize].update(PixelState::Pixel(color));

        Ok(())
    }

    fn draw_line<S: BackendStyle>(
        &mut self,
        from: (i32, i32),
        to: (i32, i32),
        style: &S,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        // dbg!(from);
        // dbg!(to);

        let color = {
            let color = style.color();
            if color.alpha < 0.3 {
                None
            } else {
                Some(color)
            }
        };

        if from.0 == to.0 {
            let x = from.0;
            let y0 = Ord::min(from.1, to.1);
            let y1 = Ord::max(from.1, to.1);
            for y in y0..y1 {
                self.buff[y as usize][x as usize].update(PixelState::VLine(color))
            }
            return Ok(());
        }

        if from.1 == to.1 {
            let y = from.1;
            let x0 = Ord::min(from.0, to.0);
            let x1 = Ord::max(from.0, to.0);
            for x in x0..x1 {
                self.buff[y as usize][x as usize].update(PixelState::HLine(color));
            }
            return Ok(());
        }

        plotters_backend::rasterizer::draw_line(self, from, to, style)
    }

    fn estimate_text_size<S: BackendTextStyle>(
        &self,
        text: &str,
        _: &S,
    ) -> Result<(u32, u32), DrawingErrorKind<Self::ErrorType>> {
        Ok((text.len() as u32, 1))
    }

    fn draw_text<S: BackendTextStyle>(
        &mut self,
        text: &str,
        style: &S,
        pos: (i32, i32),
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        let (width, height) = self.estimate_text_size(text, style)?;
        let (width, height) = (width as i32, height as i32);
        let dx = match style.anchor().h_pos {
            HPos::Left => 0,
            HPos::Right => -width,
            HPos::Center => -width / 2,
        };
        let dy = match style.anchor().v_pos {
            VPos::Top => 0,
            VPos::Center => -height / 2,
            VPos::Bottom => -height,
        };
        // let offset = (pos.1 + dy).max(0) * 100 + (pos.0 + dx).max(0);
        // for (idx, chr) in (offset..).zip(text.chars()) {
        //     self.0[idx as usize].update(PixelState::Text(chr));
        // }
        for (idx, chr) in (0..).zip(text.chars()) {
            if idx >= width {
                break;
            }
            self.buff[(pos.1 + dy) as usize][(pos.0 + dx + idx) as usize]
                .update(PixelState::Text(chr));
        }
        Ok(())
    }
}
