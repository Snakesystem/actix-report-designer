use eframe::egui;
use reqwest::blocking::Client;
use std::sync::{Arc, Mutex};
use crate::AppState; // Import AppState agar bisa mengubah halaman

pub struct LoginPage {
    email: String,
    password: String,
    error_message: Arc<Mutex<Option<String>>>,
    app_state: Arc<Mutex<AppState>>, // Tambahkan referensi ke state aplikasi
}

impl LoginPage {
    pub fn new(app_state: Arc<Mutex<AppState>>) -> Self {
        Self {
            email: "".to_string(),
            password: "".to_string(),
            error_message: Arc::new(Mutex::new(None)),
            app_state,
        }
    }

    fn login(&mut self) {
        let client = Client::new();
        let url = "http://127.0.0.1:8001/services/auth/login";
        let body = serde_json::json!({
            "email": self.email,
            "password": self.password
        });

        let res = client.post(url)
            .json(&body)
            .send();

        match res {
            Ok(response) => {
                if response.status().is_success() {
                    *self.error_message.lock().unwrap() = Some("Login successful".to_string());
                    
                    // Ubah state ke ReportDesigner jika login sukses
                    let mut state = self.app_state.lock().unwrap();
                    *state = AppState::ReportDesigner;
                } else {
                    *self.error_message.lock().unwrap() = Some("Login failed".to_string());
                }
            },
            Err(_) => {
                *self.error_message.lock().unwrap() = Some("Server error".to_string());
            }
        }
    }
}

impl eframe::App for LoginPage {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Login");
            ui.label("Email:");
            ui.text_edit_singleline(&mut self.email);
            ui.label("Password:");
            ui.text_edit_singleline(&mut self.password);

            if let Some(err) = &*self.error_message.lock().unwrap() {
                ui.label(err);
            }

            if ui.button("Login").clicked() {
                self.login();
            }
        });
    }
}
