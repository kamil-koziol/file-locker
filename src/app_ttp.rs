#![allow(dead_code, unused_variables)]

use egui::Color32;
use rsa::{
    pkcs1::{EncodeRsaPrivateKey, EncodeRsaPublicKey},
    pkcs8::LineEnding,
    Pkcs1v15Encrypt, RsaPublicKey,
};
use serde::{Deserialize, Serialize};

use crate::crypto::{keypair::KEY_SIZE, Keypair, Verifier};
use std::fs;

#[derive(Deserialize, Serialize, Default)]
#[serde(default)]
pub struct TrustedThirdPartyApp {}

impl TrustedThirdPartyApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}
impl TrustedThirdPartyApp {
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

impl eframe::App for TrustedThirdPartyApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.top_panel(ctx, _frame);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Trusted Third Party");

            if ui.button("Generate").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    let result = path.display().to_string();

                    let kp = Keypair::generate(KEY_SIZE);
                    kp.write_to_dir_u(&result).unwrap();
                }
            }
        });
    }
}
