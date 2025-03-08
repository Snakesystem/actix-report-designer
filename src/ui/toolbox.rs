use crate::ui::report_designer::ReportDesigner;

pub fn toolbox_panel(ctx: &egui::Context, designer: &mut ReportDesigner) {
    egui::SidePanel::left("toolbox").show(ctx, |ui| {
        ui.heading("Toolbox");
        if ui.button("Text").clicked() {
            designer.elements.push("Text".to_string());
        }
        if ui.button("Image").clicked() {
            designer.elements.push("Image".to_string());
        }
        if ui.button("Table").clicked() {
            designer.elements.push("Table".to_string());
        }
    });
}