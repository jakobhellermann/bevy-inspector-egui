use crate::{
    utils::{self, error_label_needs_world, image_texture_conversion},
    Context, Inspectable,
};
use bevy::utils::Entry;
use bevy::{
    asset::{Asset, HandleId},
    ecs::event::Events,
    prelude::*,
    render::texture::Image,
    utils::HashMap,
};
use bevy_egui::{
    egui::{self, Color32, RichText},
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

    fn ui(&mut self, ui: &mut egui::Ui, _: Self::Attributes, _: &mut Context) -> bool {
        ui.label("<handle id>");
        false
    }
}

impl<T: Asset + Inspectable> Inspectable for Handle<T> {
    type Attributes = T::Attributes;

    fn ui(&mut self, ui: &mut egui::Ui, options: Self::Attributes, context: &mut Context) -> bool {
        if self.id == HandleId::default::<T>() {
            ui.label("<default handle>");
            return false;
        }

        context.resource_scope(
            ui,
            "Handle<T>",
            |ui, context, mut assets: Mut<Assets<T>>| {
                let value = expect_handle!(ui, assets, get_mut self);
                value.ui(ui, options, context)
            },
        )
    }
}

#[derive(Default)]
struct ScaledDownTextures {
    textures: HashMap<Handle<Image>, Handle<Image>>,
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

impl Inspectable for Handle<Image> {
    type Attributes = TextureAttributes;

    fn ui(&mut self, ui: &mut egui::Ui, options: Self::Attributes, context: &mut Context) -> bool {
        let id = context.id();

        let world = match unsafe { context.world_mut() } {
            Some(world) => world,
            None => return error_label_needs_world(ui, "Handle<Image>"),
        };

        let _ = world.get_resource_or_insert_with(ScaledDownTextures::default);

        let world = world.cell();
        let mut textures = world.get_resource_mut::<Assets<Image>>().unwrap();
        let mut scaled_down_textures = world.get_resource_mut().unwrap();
        let mut egui_context = world.get_resource_mut::<bevy_egui::EguiContext>().unwrap();
        let asset_server = world.get_resource::<AssetServer>().unwrap();
        let file_events = world.get_resource::<Events<FileDragAndDrop>>().unwrap();

        if !textures.contains(&*self) {
            let response = utils::ui::drag_and_drop_target(ui);
            if response.hovered() {
                return utils::ui::replace_handle_if_dropped(self, &*file_events, &*asset_server);
            }
            return false;
        }

        let texture = match options.rescale {
            Some(_) => rescaled_image(
                self,
                &mut scaled_down_textures,
                &mut textures,
                &mut egui_context,
            ),
            None => Some((self.clone(), egui_context.add_image(self.clone_weak()))),
        };

        if let Some((texture, texture_id)) = texture {
            let texture = textures.get(texture).unwrap();
            let response = show_texture(texture, texture_id, ui, id);

            if response.map_or(false, |res| res.hovered()) {
                utils::ui::replace_handle_if_dropped(self, &*file_events, &*asset_server)
            } else {
                false
            }
        } else {
            ui.label("<texture>");
            false
        }
    }
}

fn rescaled_image<'a>(
    handle: &Handle<Image>,
    scaled_down_textures: &'a mut ScaledDownTextures,
    textures: &mut Assets<Image>,
    egui_context: &mut EguiContext,
) -> Option<(Handle<Image>, TextureId)> {
    let (texture, texture_id) = match scaled_down_textures.textures.entry(handle.clone()) {
        Entry::Occupied(handle) => {
            let handle: Handle<Image> = handle.get().clone();
            (handle.clone(), egui_context.add_image(handle))
        }
        Entry::Vacant(entry) => {
            let original = textures.get(handle).unwrap();

            let image = image_texture_conversion::texture_to_image(original)?;
            let resized = image.resize(50, 50, FilterType::Nearest);
            let resized = image_texture_conversion::image_to_texture(resized);

            let handle = textures.add(resized);
            let weak = handle.clone_weak();
            let texture_id = egui_context.add_image(handle.clone());
            entry.insert(handle);

            (weak, texture_id)
        }
    };

    Some((texture, texture_id))
}

impl Inspectable for Handle<Font> {
    type Attributes = ();

    fn ui(&mut self, ui: &mut egui::Ui, _: Self::Attributes, context: &mut Context) -> bool {
        let world = match context.world() {
            Some(world) => world,
            None => return error_label_needs_world(ui, "Handle<Font>"),
        };

        let asset_server = world.get_resource::<AssetServer>().unwrap();
        let file_events = world.get_resource::<Events<FileDragAndDrop>>().unwrap();

        let fonts = world.get_resource::<Assets<Font>>().unwrap();

        let label = if fonts.contains(self.id) {
            egui::Label::new("<font>")
        } else {
            egui::Label::new(RichText::new("No font").color(Color32::RED))
        };

        if utils::ui::drag_and_drop_target_label(ui, label).hovered() {
            utils::ui::replace_handle_if_dropped(self, file_events, asset_server)
        } else {
            false
        }
    }
}

fn show_texture(
    texture: &Image,
    texture_id: TextureId,
    ui: &mut egui::Ui,
    id: egui::Id,
) -> Option<egui::Response> {
    let size = texture.texture_descriptor.size;
    let size = [size.width as f32, size.height as f32];

    let max = size[0].max(size[1]);
    if max >= 128.0 {
        let response = egui::CollapsingHeader::new("Texture")
            .id_source(id)
            .show(ui, |ui| ui.image(texture_id, size));
        response.body_response
    } else {
        let response = ui.image(texture_id, size);
        Some(response)
    }
}
