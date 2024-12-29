use crate::prelude::{snarl::*, utils::*};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize, Debug)]
#[serde(default)]
pub struct Config {
    pub snarl: Snarl<FlowNodes>,
    pub stop: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            snarl: Snarl::new(),
            stop: false,
        }
    }
}

impl Config {
    pub fn from_ref(snarl: &Snarl<FlowNodes>) -> Self {
        Self {
            snarl: snarl.serde_clone(),
            stop: false,
        }
    }

    pub const SNARL_STYLE: SnarlStyle = {
        let mut style = SnarlStyle::new();

        style.bg_pattern = Some(egui_snarl::ui::BackgroundPattern::Grid(
            egui_snarl::ui::Grid {
                spacing: egui::Vec2::new(50.0, 50.0),
                angle: 1.0,
            },
        ));

        style
    };
}
