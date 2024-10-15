use egui_tracing::EventCollector;

use crate::libs::widget::{UiWidget, View};

pub struct LogViewerWidget {
    collector: EventCollector,
}

impl View for LogViewerWidget {
    fn ui(&mut self, ui: &mut egui::Ui) {
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
