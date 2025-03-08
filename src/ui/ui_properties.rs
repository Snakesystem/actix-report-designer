use std::sync::Arc;
use crate::ui::report_designer::ReportDesigner;

pub fn properties_panel(ctx: &egui::Context, designer: &mut ReportDesigner) {
    egui::SidePanel::right("properties").show(ctx, |ui| {

        for element in &designer.elements {
            ui.label(element);
        }
        
        if ui.button("Connect to Database").clicked() {
            let status = "Connecting...".to_string();
            designer.set_db_status(status.clone());

            // Gunakan Arc<Mutex<T>> agar bisa diubah di async task
            let db_status = Arc::clone(&designer.db_status);
            let rt = designer.rt.handle().clone();

            rt.spawn(async move {
                let response = reqwest::get("http://localhost:8000/connect").await;
                let new_status = match response {
                    Ok(resp) if resp.status().is_success() => "Connected".to_string(),
                    Ok(_) => "Connection Failed (Invalid Response)".to_string(),
                    Err(_) => "Connection Failed".to_string(),
                };

                // Update status di thread utama
                let mut db_status = db_status.lock().unwrap();
                *db_status = new_status;
            });
        }

        // Pastikan status terbaru diambil dari Arc<Mutex<T>>
        let current_status = designer.db_status.lock().unwrap().clone();
        ui.label(format!("Database Status: {}", current_status));

        ui.heading("Properties");
        ui.label("(Select an element to edit properties)");
    });
}