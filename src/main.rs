// Prevent console window in addition to egui window in Windows release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use backend::service::start_server;
use egui_report_designer::connect_with_host_port_username_password;
use tokio::runtime::Runtime;
use ui::report_designer::ReportDesigner;
use std::error::Error;

mod backend {
    pub mod service;
}

mod ui {
    pub mod canvas;
    pub mod report_designer;
    pub mod toolbox;
    pub mod ui_properties;
}

fn main() -> Result<(), Box<dyn Error>> {
    let rt = Runtime::new().unwrap();
    rt.spawn_blocking(|| {
        if let Err(e) = tokio::runtime::Runtime::new().unwrap().block_on(start_server()) {
            eprintln!("Failed to start backend: {}", e);
        }
    });
        
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Report Designer",
        options,
        Box::new(|_cc| Ok::<Box<dyn eframe::App>, _>(Box::new(ReportDesigner::default()))),
    )?;
    Ok(())
}

#[tokio::test]
async fn connect_to_sql_server_using_host_port_username_password() {
    let result = connect_with_host_port_username_password().await;
    assert_eq!(result.is_ok(), true);
}