use egui_tracing::EventCollector;

use crate::prelude::ui::*;

pub struct LogViewerWidget {
    collector: EventCollector,
}

impl View for LogViewerWidget {
    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);
        ui.add(egui_tracing::Logs::new(self.collector.clone()));
    }
}

impl UiWidget for LogViewerWidget {
    fn name<'a>(&'a self) -> &'a str {
        "Log Viewer"
    }
}

impl LogViewerWidget {
    pub fn new(collector: EventCollector) -> Self {
        Self { collector }
    }
}
