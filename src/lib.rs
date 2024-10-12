pub mod app;
pub mod command;
pub mod data;
pub mod layer;
pub mod mel_layer;
pub mod tui;
pub mod utils;

pub type Result<T> = color_eyre::Result<T>;
