use std::collections::hash_map::Entry;

use crate::{
    utils::{self, image_texture_conversion},
    Context, Inspectable,
};
use bevy::{
    app::Events,
    asset::{Asset, HandleId},
    prelude::*,
    render::texture::Texture,
    utils::HashMap,
};
use bevy_egui::{
    egui::{self, Color32},
    EguiContext,
};
use egui::TextureId;
pub use image::imageops::FilterType;

macro_rules! expect_handle {
    ($ui:ident, $assets:ident, $method:ident $asset:ident) => {
        match $assets.$method($asset.clone()) {
            Some(val) => val,
            None => {
                utils::error_label($ui, format!("No value for handle {:?}", $asset));
                return false;
            }
        }
    };
}

impl Inspectable for HandleId {
    type Attributes = ();

    fn ui(
        &mut self,
        ui: &mut egui::Ui,
        _: Self::Attributes,
        _: &Context,
    ) -> bool {
        ui.label("<handle id>");
        false
    }
}

impl<T: Asset + Inspectable> Inspectable for Handle<T> {
    type Attributes = T::Attributes;

    fn ui(
        &mut self,
        ui: &mut egui::Ui,
        options: Self::Attributes,
        context: &Context,
    ) -> bool {
        if self.id == HandleId::default::<T>() {
            ui.label("<default handle>");
            return false;
        }

        let world = expect_world!(ui, context, "Handle<T>");
        let mut assets = world.get_resource_mut::<Assets<T>>().unwrap();

        let value = expect_handle!(ui, assets, get_mut self);

        value.ui(ui, options, context)
    }
}

#[derive(Default)]
struct ScaledDownTextures {
    textures: HashMap<Handle<Texture>, Handle<Texture>>,
}

#[derive(Clone)]
pub struct TextureAttributes {
    /// If true, display a rescaled version of the image which fits in the
    /// size specified by the `Vec2`.
    pub rescale: Option<(Vec2, FilterType)>,
}

impl Default for TextureAttributes {
    fn default() -> Self {
        TextureAttributes {
            rescale: Some((Vec2::new(60., 60.), FilterType::Triangle)),
        }
    }
}

impl Inspectable for Handle<Texture> {
    type Attributes = TextureAttributes;

    fn ui(
        &mut self,
        ui: &mut egui::Ui,
        options: Self::Attributes,
        context: &Context,
    ) -> bool {
        let world = expect_world!(ui, context, "Handle<Texture>");
        let _ = world.get_resource_or_insert_with(ScaledDownTextures::default);

        let world = world.cell();
        let asset_server = world.get_resource::<AssetServer>().unwrap();
        let mut textures = world.get_resource_mut::<Assets<Texture>>().unwrap();
        let file_events = world.get_resource::<Events<FileDragAndDrop>>().unwrap();
        let mut scaled_down_textures = world.get_resource_mut().unwrap();
        let mut egui_context = world.get_resource_mut::<bevy_egui::EguiContext>().unwrap();

        if !textures.contains(&*self) {
            let response = utils::ui::drag_and_drop_target(ui);
            if response.hovered() {
                return utils::ui::replace_handle_if_dropped(self, &*file_events, &*asset_server);
            }
            return false;
        }

        let (texture, id) = match options.rescale {
            Some(_) => rescaled_image(
                &self,
                &mut scaled_down_textures,
                &mut textures,
                &mut egui_context,
            ),
            None => (self.clone(), TextureId::User(id_of_handle(self))),
        };

        let texture = textures.get(texture).unwrap();
        let response = show_texture(texture, id, ui, context);

        if response.map_or(false, |res| res.hovered()) {
            utils::ui::replace_handle_if_dropped(self, &*file_events, &*asset_server)
        } else {
            false
        }
    }
}

fn rescaled_image<'a>(
    handle: &Handle<Texture>,
    scaled_down_textures: &'a mut ScaledDownTextures,
    textures: &mut Assets<Texture>,
    egui_context: &mut EguiContext,
) -> (Handle<Texture>, TextureId) {
    let id = id_of_handle(handle);
    let texture = match scaled_down_textures.textures.entry(handle.clone()) {
        Entry::Occupied(handle) => handle.get().clone(),
        Entry::Vacant(entry) => {
            let original = textures.get(handle).unwrap();

            let image = image_texture_conversion::texture_to_image(&original).unwrap();
            let resized = image.resize(50, 50, FilterType::Nearest);
            let resized = image_texture_conversion::image_to_texture(resized);

            let handle = textures.add(resized);
            let weak = handle.clone_weak();
            egui_context.set_egui_texture(id, handle.clone());
            entry.insert(handle);

            weak
        }
    };

    let id = TextureId::User(id);

    (texture, id)
}

impl Inspectable for Handle<Font> {
    type Attributes = ();

    fn ui(
        &mut self,
        ui: &mut egui::Ui,
        _: Self::Attributes,
        context: &Context,
    ) -> bool {
        let world = expect_world!(ui, context, "Handle<Texture>");
        let asset_server = world.get_resource::<AssetServer>().unwrap();
        let file_events = world.get_resource::<Events<FileDragAndDrop>>().unwrap();

        let fonts = world.get_resource::<Assets<Font>>().unwrap();

        let label = if fonts.contains(self.id) {
            egui::Label::new("<font>")
        } else {
            egui::Label::new("No font").text_color(Color32::RED)
        };

        if utils::ui::drag_and_drop_target_label(ui, label).hovered() {
            utils::ui::replace_handle_if_dropped(self, file_events, asset_server)
        } else {
            false
        }
    }
}

fn show_texture(
    texture: &Texture,
    texture_id: TextureId,
    ui: &mut egui::Ui,
    context: &Context,
) -> Option<egui::Response> {
    let size = texture.size;
    let size = [size.width as f32, size.height as f32];

    let max = size[0].max(size[1]);
    if max >= 128.0 {
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
    let mut hasher = std::collections::hash_map::DefaultHasher::default();
    handle.hash(&mut hasher);
    hasher.finish()
}
