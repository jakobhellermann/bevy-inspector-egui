use bevy::{
    app::{Events, ManualEventReader},
    asset::{Asset, AssetPath},
    prelude::*,
};
use bevy_egui::egui::{self, Response, RichText, Widget};

pub fn drag_and_drop_target(ui: &mut egui::Ui) -> Response {
    drag_and_drop_target_label(ui, egui::Label::new("Drag file here"))
}
pub fn drag_and_drop_target_label(ui: &mut egui::Ui, label: egui::Label) -> Response {
    let frame = egui::containers::Frame::dark_canvas(ui.style());
    frame.show(ui, |ui| label.ui(ui)).inner
}

pub fn replace_handle_if_dropped<T: Asset>(
    handle: &mut Handle<T>,
    events: &Events<FileDragAndDrop>,
    asset_server: &AssetServer,
) -> bool {
    let mut reader = ManualEventReader::default();
    let drag_and_drop_event = reader.iter(events).next_back();
    if let Some(FileDragAndDrop::DroppedFile { path_buf, .. }) = &drag_and_drop_event {
        let asset_path = AssetPath::new_ref(path_buf, None);
        let new_handle: Handle<T> = asset_server.load(asset_path);

        *handle = new_handle;
        return true;
    }

    false
}

pub fn label_button(ui: &mut egui::Ui, text: &str, text_color: egui::Color32) -> bool {
    ui.add(egui::Button::new(RichText::new(text).color(text_color)).frame(false))
        .clicked()
}
