use std::{collections::VecDeque, sync::Arc};

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
    queue: Arc<std::sync::Mutex<VecDeque<reqwest_client::lib::Msg>>>,
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
                    // Config::default()

                    Config::deserialize(include_str!("./audio_analyzer_config.json"))
                        .expect("failed to load default config")
                }
            };

            return Self {
                collector,
                config: config,
                reloader: Arc::new(Mutex::new(None)),
                queue: Arc::new(std::sync::Mutex::new(VecDeque::new())),
            };
        }

        Self {
            collector,
            // streamer,
            config: Config::default(),
            reloader: Arc::new(Mutex::new(None)),
            queue: Arc::new(std::sync::Mutex::new(VecDeque::new())),
        }
    }
}

impl eframe::App for App {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, &self.config);

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

                let str = match String::from_utf8(config) {
                    Ok(str) => str,
                    Err(e) => {
                        log::info!("failed to parse config: {:?}", e);
                        return;
                    }
                };

                match Config::deserialize(&str) {
                    Ok(config) => {
                        log::info!("parsed config: {:?}", config);

                        self.config = config;
                    }
                    Err(e) => {
                        log::info!("failed to parse config: {:?}", e);
                    }
                }
            }
        }

        if let Ok(mut q) = self.queue.clone().try_lock() {
            if let Some(msg) = q.pop_front() {
                assert_eq!(msg, reqwest_client::lib::Msg::CompileStart);

                log::info!("Compile start");
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

                        #[cfg(not(target_arch = "wasm32"))]
                        {
                            picker::save_file(
                                serde_json::to_string(&self.config)
                                    .unwrap()
                                    .as_bytes()
                                    .to_vec(),
                            );
                        }
                        #[cfg(target_arch = "wasm32")]
                        {
                            filedl_on_web::file_dl(
                                serde_json::to_string(&self.config).unwrap().as_bytes(),
                                "audio_analyzer_config.json",
                            );
                        }
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

            ui.add(egui::Checkbox::new(&mut self.config.stop, "stop"));

            if ui.button("check").clicked() {
                log::info!("check");

                use std::panic;

                let snarl_copy_str = serde_json::to_string(&self.config.snarl).unwrap();
                match panic::catch_unwind(move || {
                    let snarl_copy = serde_json::from_str(&snarl_copy_str).unwrap();
                    crate::libs::gen_code::analysis::analysis(&snarl_copy)
                }) {
                    Ok(Ok(_)) => {
                        log::info!("Analysis successful");
                    }
                    Ok(Err(e)) => {
                        log::info!("Analysis failed: {:?}", e);
                    }
                    Err(panic_info) => {
                        log::error!("Analysis panicked: {:?}", panic_info);
                    }
                }

                let code = "fn main() { println!(\"Hello, world!\"); }".to_string();

                log::info!("Running code: {}", code);

                assert!(reqwest_client::run_code(code, self.queue.clone()).is_ok());
            }

            ui.separator();

            ui.add(egui::github_link_file!(
                "https://github.com/oligamiq/audio_analyzer_core/tree/main/",
                "Source code."
            ));

            self.config.snarl.show(
                &mut FlowNodesViewer::new(!self.config.stop),
                &Config::SNARL_STYLE,
                "snarl",
                ui,
            );

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
