#![allow(dead_code, unused_variables)]

use egui::Color32;
use rsa::Pkcs1v15Encrypt;
use serde::{Deserialize, Serialize};

use crate::crypto::{keypair::KEY_SIZE, Keypair, Signer, Verifier};
use std::{fs, path::Path};

use sysinfo::Disks;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(Deserialize, Serialize, Default)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    files: Vec<AppFile>,
    file_to_remove: Option<String>,

    #[serde(skip)]
    keypair: Option<Keypair>,

    keypair_path: Option<String>,

    #[serde(skip)]
    pin: Option<String>,

    #[serde(skip)]
    pin_textfield: String,
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

impl TemplateApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}
impl TemplateApp {
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

    fn enter_pin(&mut self, ui: &mut egui::Ui) {
        ui.heading("Welcome, enter pin to continue");
        ui.text_edit_singleline(&mut self.pin_textfield);
        if ui.button("Enter").clicked() {
            self.pin = Some(self.pin_textfield.clone());

            if let Some(keypair_path) = &self.keypair_path {
                let keypair_path = Path::new(&keypair_path);
                if !keypair_path.exists() {
                    return;
                }

                let keypair = Keypair::load_from_dir(
                    keypair_path.to_str().unwrap(),
                    &self.pin.clone().unwrap(),
                );

                if let Ok(keypair) = keypair {
                    self.keypair = Some(keypair);
                }
            }
        }
    }

    fn keypair_load(&mut self, ui: &mut egui::Ui) {
        let disks = Disks::new_with_refreshed_list();
        for disk in disks.list() {
            if !disk.is_removable() {
                continue;
            }

            if ui.button(disk.name().to_str().unwrap()).clicked() {
                let mount_point = disk.mount_point();
                let search_path = mount_point.join(".filelocker");
                if search_path.exists() && search_path.is_dir() {
                    match Keypair::load_from_dir(
                        search_path.as_path().to_str().unwrap(),
                        &self.pin.clone().unwrap(),
                    ) {
                        Ok(kp) => self.keypair = Some(kp),
                        Err(_) => {
                            self.pin = None;
                        }
                    }
                } else {
                    fs::create_dir(&search_path).unwrap();

                    let filelocker_path = search_path.to_str().unwrap();
                    let keypair = Keypair::generate(KEY_SIZE);
                    keypair
                        .write_to_dir(filelocker_path, &self.pin.clone().unwrap())
                        .unwrap();

                    self.keypair = Some(keypair);
                    self.keypair_path = Some(String::from(filelocker_path.clone()));
                }
            }
        }
    }
}

impl eframe::App for TemplateApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.top_panel(ctx, _frame);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("File Locker");

            if self.pin.is_none() {
                self.enter_pin(ui);
                return;
            }

            if self.keypair.is_none() {
                ui.heading("Select removable drive from a list");
                self.keypair_load(ui);
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
                                if let Some(kp) = &self.keypair {
                                    let mut rng = rand::thread_rng();
                                    let data = fs::read(&file.path).unwrap();
                                    let enc_data = kp
                                        .public_key
                                        .encrypt(&mut rng, Pkcs1v15Encrypt, &data)
                                        .expect("failed to encrypt");

                                    file.encrypted = true;

                                    let _ = fs::write(&file.path, enc_data);
                                }
                            }

                            if ui.button("Decrypt").clicked() {
                                if let Some(kp) = &self.keypair {
                                    let ciphertext = fs::read(&file.path).unwrap();
                                    let dec_data = kp
                                        .private_key
                                        .decrypt(Pkcs1v15Encrypt, &ciphertext)
                                        .expect("Failed to decrypt");

                                    file.encrypted = false;
                                    let _ = fs::write(&file.path, dec_data);
                                }
                            }

                            if let Some(kp) = &self.keypair {
                                if ui.button("Sign").clicked() {
                                    let signer = Signer::new(&kp.private_key);
                                    let signature = signer.sign_file(&file.path).unwrap();
                                    let signature_path = String::from(&file.path) + ".sig.xml";
                                    signature.save_to_file(&signature_path).unwrap();
                                    file.signature = Some(signature_path);
                                    file.verified = true;
                                    ui.close_menu();
                                }

                                if let Some(sig) = &file.signature {
                                    if ui.button("Open signature").clicked() {
                                        let _ = open::that(sig);
                                        ui.close_menu();
                                    }
                                }

                                if ui.button("Verify").clicked() {
                                    if let Some(kp) = &self.keypair {
                                        if let Some(path) = rfd::FileDialog::new().pick_file() {
                                            let v = Verifier::new(&kp.public_key);
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
