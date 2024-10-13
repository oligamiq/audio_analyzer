use std::sync::{atomic::AtomicBool, Arc};

use egui::mutex::Mutex;

use super::widget::{UiWidget, View};

pub struct SeparateWindowWidget<W: UiWidget + View> {
    title: String,
    initial_size: [f32; 2],
    show: Arc<AtomicBool>,
    widget: Arc<Mutex<W>>,
}

impl<W: UiWidget + View + Send + 'static> SeparateWindowWidget<W> {
    pub fn new(initial_size: [f32; 2], widget: W) -> Self {
        SeparateWindowWidget {
            title: widget.name().to_string(),
            initial_size,
            show: Arc::new(AtomicBool::new(true)),
            widget: Arc::new(Mutex::new(widget)),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn show(&mut self, ctx: &egui::Context) {
        if self.show.load(std::sync::atomic::Ordering::Relaxed) {
            let widget = Arc::clone(&self.widget);
            let show = Arc::clone(&self.show);

            ctx.show_viewport_deferred(
                egui::ViewportId::from_hash_of(&self.title),
                egui::ViewportBuilder::default()
                    .with_title(&self.title)
                    .with_inner_size(self.initial_size),
                move |ctx, _class| {
                    let widget = widget.clone();

                    egui::CentralPanel::default().show(ctx, |ui| {
                        let mut widget = widget.lock();
                        widget.ui(ui);
                    });

                    if ctx.input(|i| i.viewport().close_requested()) {
                        // Tell parent viewport that we should not show next frame:
                        show.store(false, std::sync::atomic::Ordering::Relaxed);
                    }
                },
            );
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn show(&mut self, ctx: &egui::Context) {
        if self.show {
            egui::Window::new(self.title.clone())
                .default_size(self.initial_size)
                .show(ctx, |ui| {
                    self.widget.ui(ui);
                });
        }
    }
}
