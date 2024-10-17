use egui_snarl::{ui::SnarlStyle, Snarl};

use crate::libs::nodes::FlowNodes;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize, Debug)]
#[serde(default)]
pub struct Config {
    pub snarl: Snarl<FlowNodes>,
    pub style: SnarlStyle,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            snarl: Snarl::new(),
            style: SnarlStyle::default(),
        }
    }
}

impl Config {
    pub fn from_ref(snarl: &Snarl<FlowNodes>, style: &SnarlStyle) -> Self {
        Self {
            snarl: serde_json::from_str(&serde_json::to_string(snarl).unwrap()).unwrap(),
            style: serde_json::from_str(&serde_json::to_string(style).unwrap()).unwrap(),
        }
    }
}
