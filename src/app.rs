#![allow(dead_code, unused_variables)]

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    // Example stuff:
    label: String,

    #[serde(skip)] // This how you opt-out of serialization of a field
    value: f32,

    picked_path: Option<String>,
    files: Vec<String>,
    file_to_remove: Option<String>,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            label: "Hello World!".to_owned(),
            value: 2.7,
            picked_path: None,
            files: Vec::new(),
            file_to_remove: None,
        }
    }
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for TemplateApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_dark_light_mode_buttons(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.heading("File Locker");

            ui.horizontal(|ui| {
                ui.label("Selected file: ");
                ui.text_edit_singleline(&mut self.label);
            });

            if ui.button("Open file…").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    let result = path.display().to_string();
                    self.picked_path = Some(result.clone());
                    if !self.files.contains(&result) {
                        self.files.push(result);
                    }
                }
            }

            // scrollable
            egui::ScrollArea::vertical().show(ui, |ui| {
                for file in &self.files {
                    ui.button(file.clone()).context_menu(|ui| {
                        if ui.button("Remove").clicked() {
                            self.file_to_remove = Some(file.clone());
                        }
                    });
                }
            });

            // modal on click
            // if ui.button("Modal").clicked() {
            //     egui::Window::new("Modal").show(&ui.ctx(), |ui| {
            //         ui.label("This is a modal window");
            //         if ui.button("Close").clicked() {
            //             // ui.ctx().close_window();
            //         }
            //     });
            // }

            // clearing file to remove
            if let Some(file_to_remove) = &self.file_to_remove {
                self.files.retain(|file| file != file_to_remove);
                self.file_to_remove = None;
            }
        });
    }
}
