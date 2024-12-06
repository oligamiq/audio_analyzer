use std::sync::Arc;

#[allow(unused_imports)]
use crate::prelude::{nodes::*, snarl::*, utils::*};
use egui::mutex::Mutex;
use egui_editable_num::picker;
use egui_tracing::tracing::collector;
use log::{info, trace};

use super::config::Config;

pub struct App {
    collector: egui_tracing::EventCollector,
    // streamer: Streamer,
    config: Config,
    reloader: Arc<Mutex<Option<Vec<u8>>>>,
}

impl App {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>, collector: collector::EventCollector) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // cc.egui_ctx.tessellation_options_mut(|tess_options| {
        //     tess_options.feathering = false;
        // });

        // cc.egui_ctx.set_visuals(Visuals::light());

        cc.egui_ctx.set_theme(egui::Theme::Light);

        info!("Initialized app");

        // let streamer = new_stream();

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            let config = match eframe::get_value::<Config>(storage, eframe::APP_KEY) {
                Some(config) => {
                    info!("Loaded app state: {:?}", config);
                    config
                }
                None => {
                    info!("failed to load app state");
                    Config::default()
                }
            };

            return Self {
                collector,
                config: config,
                reloader: Arc::new(Mutex::new(None)),
            };
        }

        Self {
            collector,
            // streamer,
            config: Config::default(),
            reloader: Arc::new(Mutex::new(None)),
        }
    }
}

impl eframe::App for App {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(
            storage,
            eframe::APP_KEY,
            &self.config,
        );

        trace!("Saved app state");
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.reloader.lock().is_some() {
            log::info!("Reloading config");

            let mut reloader = self.reloader.lock();

            log::info!("Reloader: {:?}", reloader);

            if let Some(config) = reloader.take() {
                log::info!("taken config: {:?}", config);

                if let Ok(config) = serde_json::from_slice(&config) {
                    log::info!("parsed config: {:?}", config);

                    self.config = config;
                }
            }
        }

        // self.streamer.apply();

        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                ui.menu_button("File", |ui| {
                    if !is_web {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                        ui.add_space(16.0);
                    }

                    if ui.button("📂 Open text file").clicked() {
                        log::info!("Open text file");

                        let reloader = self.reloader.clone();

                        picker::open_file(move |file| match std::str::from_utf8(&file) {
                            Ok(config) => {
                                log::info!("Loaded file: {}", config);

                                let mut reloader = reloader.lock();
                                *reloader = Some(file);
                            }
                            Err(e) => {
                                trace!("Failed to load file: {}", e);
                            }
                        });
                    }

                    if ui.button("💾 Save text file").clicked() {
                        log::info!("Save text file");

                        picker::save_file(
                            serde_json::to_string(&self.config)
                                .unwrap()
                                .as_bytes()
                                .to_vec(),
                        );
                    }
                });

                egui::widgets::global_theme_preference_buttons(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's

            ui.heading("Audio Analyzer");

            // ui.horizontal(|ui| {
            //     ui.label("Write something: ");
            //     ui.text_edit_singleline(&mut self.label);
            // });

            // ui.add(egui::Slider::new(&mut self.value, 0.0..=10.0).text("value"));
            // if ui.button("Increment").clicked() {
            //     self.value += 1.0;
            //     trace!("Incremented value to {}", self.value);
            // }

            ui.separator();

            ui.add(egui::github_link_file!(
                "https://github.com/oligamiq/audio_analyzer_core/tree/main/",
                "Source code."
            ));

            self.config.snarl
                .show(&mut FlowNodesViewer, &self.config.style, "snarl", ui);

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                powered_by_egui_and_eframe(ui);
                egui::warn_if_debug_build(ui);
            });
        });

        let mut separate_window_widget =
            SeparateWindowWidget::new([400.0, 300.0], LogViewerWidget::new(self.collector.clone()));

        separate_window_widget.show(ctx);

        ctx.request_repaint();
    }
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });
}
