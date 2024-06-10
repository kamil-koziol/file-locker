#![allow(dead_code, unused_variables)]

use crate::crypto::Keypair;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    files: Vec<String>,
    file_to_remove: Option<String>,

    #[serde(skip)]
    keypair: Option<Keypair>,

    keypair_path: Option<String>,
    pin: Option<String>,

    #[serde(skip)]
    pin_textfield: String,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            files: Vec::new(),
            file_to_remove: None,
            keypair: None,
            keypair_path: None,
            pin: None,
            pin_textfield: String::from(""),
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
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                ui.add_space(16.0);

                egui::widgets::global_dark_light_mode_buttons(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("File Locker");

            if self.pin.is_none() {
                ui.heading("Welcome, enter pin to continue");
                ui.text_edit_singleline(&mut self.pin_textfield);
                if ui.button("Enter").clicked() {
                    self.pin = Some(self.pin_textfield.clone());
                    // TODO: Hash
                }
                return;
            }

            if self.keypair.is_none() {
                ui.heading("Continue by generating or loading keypair");
                if ui.button("Generate").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        let result = path.display().to_string();

                        let keypair = Keypair::generate(4096);
                        keypair
                            .write_to_dir(&result, &self.pin.clone().unwrap())
                            .unwrap();

                        self.keypair = Some(keypair);
                        self.keypair_path = Some(result.clone());
                    }
                }

                if ui.button("Load").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        let result = path.display().to_string();
                        self.keypair_path = Some(result.clone());

                        match Keypair::load_from_dir(&result, &self.pin.clone().unwrap()) {
                            Ok(kp) => self.keypair = Some(kp),
                            Err(_) => println!("Invalid pin!"),
                        }
                    }
                }
                return;
            }

            if ui.button("Add file").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    let result = path.display().to_string();
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

                        if let Some(kp) = &self.keypair {
                            if ui.button("Sign").clicked() {}
                        }
                    });
                }
            });

            // clearing file to remove
            if let Some(file_to_remove) = &self.file_to_remove {
                self.files.retain(|file| file != file_to_remove);
                self.file_to_remove = None;
            }
        });
    }
}
