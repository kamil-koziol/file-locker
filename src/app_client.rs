#![allow(dead_code, unused_variables)]

use egui::Color32;
use rsa::{Pkcs1v15Encrypt, RsaPublicKey};
use serde::{Deserialize, Serialize};

use crate::crypto::{Keypair, Verifier};
use std::fs;

#[derive(Deserialize, Serialize, Default)]
#[serde(default)]
pub struct ClientApp {
    files: Vec<AppFile>,
    file_to_remove: Option<String>,
    keypair_path: Option<String>,

    #[serde(skip)]
    public_key: Option<RsaPublicKey>,
}

#[derive(Default, Serialize, Deserialize)]
pub struct AppFile {
    path: String,
    signature: Option<String>,
    encrypted: bool,
    verified: bool,
}

impl AppFile {
    pub fn new(path: &str) -> AppFile {
        Self {
            path: String::from(path),
            ..Self::default()
        }
    }
}

impl ClientApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}
impl ClientApp {
    fn top_panel(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
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
    }
}

impl eframe::App for ClientApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.top_panel(ctx, _frame);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("File Locker Client");

            if self.public_key.is_none() {
                if ui.button("Load public key").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_file() {
                        let public_key_path = path.display().to_string();
                        let pk = Keypair::load_public_from_file(&public_key_path).unwrap();
                        self.public_key = Some(pk);
                    }
                }
                return;
            }

            if ui.button("Add file").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    let result = path.display().to_string();
                    self.files.push(AppFile::new(&result));
                }
            }

            // scrollable
            egui::ScrollArea::vertical().show(ui, |ui| {
                for file in &mut self.files {
                    ui.horizontal(|ui| {
                        ui.button(file.path.clone()).context_menu(|ui| {
                            if ui.button("Remove").clicked() {
                                self.file_to_remove = Some(file.path.clone());
                                ui.close_menu();
                            }

                            if ui.button("Encrypt").clicked() {
                                if let Some(kp) = &self.public_key {
                                    let mut rng = rand::thread_rng();
                                    let data = fs::read(&file.path).unwrap();
                                    let enc_data = kp
                                        .encrypt(&mut rng, Pkcs1v15Encrypt, &data)
                                        .expect("failed to encrypt");

                                    file.encrypted = true;

                                    let _ = fs::write(&file.path, enc_data);
                                }
                            }

                            if let Some(kp) = &self.public_key {
                                if let Some(sig) = &file.signature {
                                    if ui.button("Open signature").clicked() {
                                        let _ = open::that(sig);
                                        ui.close_menu();
                                    }
                                }

                                if ui.button("Verify").clicked() {
                                    if let Some(kp) = &self.public_key {
                                        if let Some(path) = rfd::FileDialog::new().pick_file() {
                                            let v = Verifier::new(kp);
                                            let p = path.to_str().unwrap();
                                            match v.verify_file(&file.path, &p.into()) {
                                                Ok(result) => {
                                                    file.verified = result;
                                                }
                                                Err(_) => file.verified = false,
                                            }
                                        }
                                        ui.close_menu();
                                    }
                                }
                            }
                        });

                        // File statuses

                        if file.verified {
                            ui.colored_label(Color32::GREEN, "OK");
                        } else {
                            ui.colored_label(Color32::RED, "NOT VERIFIED");
                        }

                        if file.encrypted {
                            ui.colored_label(Color32::LIGHT_BLUE, "ENCRYPTED");
                        }
                    });
                }
            });

            // clearing file to remove
            if let Some(file_to_remove) = &self.file_to_remove {
                self.files.retain(|file| &file.path != file_to_remove);
                self.file_to_remove = None;
            }
        });
    }
}
