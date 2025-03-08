use tokio::runtime::Runtime;
use crate::ui::canvas::canvas_panel;
use crate::ui::toolbox::toolbox_panel;
use crate::ui::ui_properties::properties_panel;
use eframe::egui;
use std::sync::{Arc, Mutex};

pub struct ReportDesigner {
    pub elements: Vec<String>,
    pub db_status: Arc<Mutex<String>>,  // Gunakan Arc<Mutex<T>> agar bisa diakses dari async
    pub rt: Runtime,
}

impl Default for ReportDesigner {
    fn default() -> Self {
        Self { 
            elements: vec![],
            db_status: Arc::new(Mutex::new("Not Connected".to_string())),
            rt: Runtime::new().expect("Failed to create Tokio runtime"),
        }
    }
}

impl ReportDesigner {
    pub fn get_db_status(&self) -> String {
        self.db_status.lock().unwrap().clone()
    }

    pub fn set_db_status(&self, status: String) {
        let mut db_status = self.db_status.lock().unwrap();
        *db_status = status;
    }
}

impl eframe::App for ReportDesigner {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        toolbox_panel(ctx, self);
        canvas_panel(ctx);
        properties_panel(ctx, self);
    }
}
