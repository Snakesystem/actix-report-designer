// Prevent console window in addition to egui window in Windows release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use backend::service::start_server;
use tokio::runtime::Runtime;
use std::{error::Error, sync::{Arc, Mutex}};
use ui::{login::LoginPage, report_designer::ReportDesigner};

mod backend {
    pub mod service;
    pub mod context {
        pub mod connection;
        pub mod crypto;
        pub mod jwt_session;
    }
    pub mod services {
        pub mod generic_services;
    }
}

mod ui {
    pub mod canvas;
    pub mod report_designer;
    pub mod toolbox;
    pub mod ui_properties;
    pub mod login;
}

/// Enum untuk menentukan state aplikasi
#[derive(Clone, PartialEq)]
pub enum AppState {
    Login,
    ReportDesigner,
}

/// Struktur utama aplikasi
pub struct App {
    app_state: Arc<Mutex<AppState>>,
    login_page: LoginPage,
    report_designer: ReportDesigner,
}

impl App {
    fn new() -> Self {
        let app_state = Arc::new(Mutex::new(AppState::Login));

        Self {
            login_page: LoginPage::new(app_state.clone()),
            report_designer: ReportDesigner::default(),
            app_state,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        match *self.app_state.lock().unwrap() {
            AppState::Login => self.login_page.update(ctx, frame),
            AppState::ReportDesigner => self.report_designer.update(ctx, frame),
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let rt = Runtime::new().unwrap();

    // Menjalankan backend di thread terpisah
    rt.spawn_blocking(|| {
        if let Err(e) = tokio::runtime::Runtime::new().unwrap().block_on(start_server()) {
            eprintln!("Failed to start backend: {}", e);
        }
    });

    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Report Designer",
        options,
        Box::new(|_cc| Ok::<Box<dyn eframe::App>, Box<dyn std::error::Error + Send + Sync>>(Box::new(App::new())))
    )?;
    
    
    Ok(())
}