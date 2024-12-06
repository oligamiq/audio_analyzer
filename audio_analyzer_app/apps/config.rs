use crate::prelude::{snarl::*, utils::*};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize, Debug)]
#[serde(default)]
pub struct Config {
    pub snarl: Snarl<FlowNodes>,
    pub style: SnarlStyle,
    pub stop: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            snarl: Snarl::new(),
            style: SnarlStyle::default(),
            stop: false,
        }
    }
}

impl Config {
    pub fn from_ref(snarl: &Snarl<FlowNodes>, style: &SnarlStyle) -> Self {
        Self {
            snarl: snarl.serde_clone(),
            style: style.serde_clone(),
            stop: false,
        }
    }
}
