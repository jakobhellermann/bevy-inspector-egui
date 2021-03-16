use crate::{utils, Context, Inspectable};
use bevy::{
    app::Events,
    asset::{Asset, HandleId},
    prelude::*,
    render::texture::Texture,
};
use bevy_egui::egui;
use egui::TextureId;

macro_rules! expect_handle {
    ($ui:ident, $assets:ident, $method:ident $asset:ident) => {
        match $assets.$method($asset.clone()) {
            Some(val) => val,
            None => {
                return utils::error_label($ui, format!("No value for handle {:?}", $asset));
            }
        }
    };
}

impl<T: Asset + Inspectable> Inspectable for Handle<T> {
    type Attributes = T::Attributes;

    fn ui(&mut self, ui: &mut egui::Ui, options: Self::Attributes, context: &Context) {
        if self.id == HandleId::default::<T>() {
            ui.label("<default handle>");
            return;
        }

        let world = expect_world!(ui, context, "Handle<T>");
        let mut assets = world.get_resource_mut::<Assets<T>>().unwrap();

        let value = expect_handle!(ui, assets, get_mut self);

        value.ui(ui, options, context);
    }
}

impl Inspectable for Handle<Texture> {
    type Attributes = ();

    fn ui(&mut self, ui: &mut egui::Ui, _: Self::Attributes, context: &Context) {
        let world = expect_world!(ui, context, "Handle<Texture>");
        let asset_server = world.get_resource::<AssetServer>().unwrap();
        let textures = world.get_resource::<Assets<Texture>>().unwrap();
        let file_events = world.get_resource::<Events<FileDragAndDrop>>().unwrap();

        let texture = textures.get(self.clone());

        let response = match texture {
            Some(texture) => show_texture(self, texture, ui, context),
            None => Some(utils::ui::drag_and_drop_target(ui)),
        };

        utils::ui::replace_handle_if_dropped(self, response, &*file_events, &*asset_server);
    }
}

fn show_texture(
    handle: &Handle<Texture>,
    texture: &Texture,
    ui: &mut egui::Ui,
    context: &Context,
) -> Option<egui::Response> {
    let size = texture.size;
    let size = [size.width as f32, size.height as f32];

    let id = id_of_handle(handle);
    let texture_id = TextureId::User(id);

    let max = size[0].max(size[1]);
    if max >= 256.0 {
        let response = egui::CollapsingHeader::new("Texture")
            .id_source(context.id())
            .show(ui, |ui| ui.image(texture_id, size));
        response.body_response
    } else {
        let response = ui.image(texture_id, size);
        Some(response)
    }
}

pub(crate) fn id_of_handle(handle: &Handle<Texture>) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = bevy::utils::AHasher::default();
    handle.hash(&mut hasher);
    hasher.finish()
}

impl Inspectable for HandleId {
    type Attributes = ();

    fn ui(&mut self, ui: &mut egui::Ui, _: Self::Attributes, _: &Context) {
        ui.label("<handle id>");
    }
}
