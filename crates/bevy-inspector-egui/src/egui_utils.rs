use egui::FontId;

pub fn layout_job(text: &[(FontId, &str)]) -> egui::epaint::text::LayoutJob {
    let mut job = egui::epaint::text::LayoutJob::default();
    for (font_id, text) in text {
        job.append(
            text,
            0.0,
            egui::TextFormat {
                font_id: font_id.clone(),
                ..Default::default()
            },
        );
    }
    job
}

pub fn label_button(ui: &mut egui::Ui, text: &str, text_color: egui::Color32) -> bool {
    ui.add(egui::Button::new(egui::RichText::new(text).color(text_color)).frame(false))
        .clicked()
}
