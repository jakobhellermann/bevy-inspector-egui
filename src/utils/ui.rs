use bevy::{
    app::ManualEventReader,
    asset::{Asset, AssetPath},
    prelude::*,
};
use bevy_egui::egui::{self, Response};

pub fn drag_and_drop_target(ui: &mut egui::Ui) -> Response {
    let frame = egui::containers::Frame::dark_canvas(&ui.style());
    frame.show(ui, |ui| ui.label("Drag file here"))
}

pub fn replace_handle_if_dropped<T: Asset>(
    handle: &mut Handle<T>,
    response: Option<Response>,
    events: &Events<FileDragAndDrop>,
    asset_server: &AssetServer,
) {
    let drag_and_drop_event = ManualEventReader::default().iter(events).next_back();
    if let Some(FileDragAndDrop::DroppedFile { path_buf, .. }) = &drag_and_drop_event {
        if response.map_or(false, |response| response.hovered) {
            let asset_path = AssetPath::new_ref(path_buf, None);
            let new_handle: Handle<T> = asset_server.load(asset_path);

            *handle = new_handle;
        }
    }
}
