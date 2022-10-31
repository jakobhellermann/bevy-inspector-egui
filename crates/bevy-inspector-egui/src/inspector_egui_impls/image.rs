use std::{
    any::Any,
    collections::{hash_map::Entry, HashMap, HashSet},
    sync::Mutex,
};

use bevy_asset::{Assets, Handle};
use bevy_egui::EguiContext;
use bevy_reflect::Reflect;
use bevy_render::texture::Image;
use once_cell::sync::Lazy;

use crate::{bevy_ecs_inspector::errors::no_world_in_context, egui_reflect_inspector::InspectorUi};

mod image_texture_conversion;

pub fn image_handle_ui(
    value: &mut dyn Any,
    ui: &mut egui::Ui,
    options: &dyn Any,
    env: InspectorUi<'_, '_>,
) -> bool {
    image_handle_ui_readonly(value, ui, options, env);
    false
}
pub fn image_handle_ui_readonly(
    value: &dyn Any,
    ui: &mut egui::Ui,
    _: &dyn Any,
    env: InspectorUi<'_, '_>,
) {
    let value = value.downcast_ref::<Handle<Image>>().unwrap();
    let Some(world) = &mut env.context.world else {
        no_world_in_context(ui, value.type_name());
        return;
    };
    let (mut egui_context, mut images) = match world
        .get_two_resources_mut::<bevy_egui::EguiContext, Assets<Image>>()
    {
        Ok(resources) => resources,
        Err(e) => return crate::bevy_ecs_inspector::errors::show_error(e, ui, env.type_registry),
    };

    let mut scaled_down_textures = SCALED_DOWN_TEXTURES.lock().unwrap();

    // todo: read asset events to re-rescale images of they changed
    let rescaled = rescaled_image(
        value,
        &mut scaled_down_textures,
        &mut images,
        &mut egui_context,
    );
    let (rescaled_handle, texture_id) = match rescaled {
        Some(it) => it,
        None => {
            ui.label("<texture>");
            return;
        }
    };

    let rescaled_image = images.get(&rescaled_handle).unwrap();
    show_image(rescaled_image, texture_id, ui);
}

static SCALED_DOWN_TEXTURES: Lazy<Mutex<ScaledDownTextures>> = Lazy::new(Default::default);

fn show_image(
    image: &Image,
    texture_id: egui::TextureId,
    ui: &mut egui::Ui,
) -> Option<egui::Response> {
    let size = image.texture_descriptor.size;
    let size = [size.width as f32, size.height as f32];

    let max = size[0].max(size[1]);
    if max >= 128.0 {
        let response =
            egui::CollapsingHeader::new("Texture").show(ui, |ui| ui.image(texture_id, size));
        response.body_response
    } else {
        let response = ui.image(texture_id, size);
        Some(response)
    }
}

#[derive(Default)]
struct ScaledDownTextures {
    textures: HashMap<Handle<Image>, Handle<Image>>,
    rescaled_textures: HashSet<Handle<Image>>,
}

const RESCALE_TO_FIT: (u32, u32) = (100, 100);

fn rescaled_image<'a>(
    handle: &Handle<Image>,
    scaled_down_textures: &'a mut ScaledDownTextures,
    textures: &mut Assets<Image>,
    egui_context: &mut EguiContext,
) -> Option<(Handle<Image>, egui::TextureId)> {
    let (texture, texture_id) = match scaled_down_textures.textures.entry(handle.clone()) {
        Entry::Occupied(handle) => {
            let handle: Handle<Image> = handle.get().clone();
            (handle.clone(), egui_context.add_image(handle))
        }
        Entry::Vacant(entry) => {
            if scaled_down_textures.rescaled_textures.contains(handle) {
                return None;
            }

            let original = textures.get(handle).unwrap();

            let (image, is_srgb) = image_texture_conversion::try_into_dynamic(original)?;
            let resized = image.resize(
                RESCALE_TO_FIT.0,
                RESCALE_TO_FIT.1,
                image::imageops::FilterType::Triangle,
            );
            let resized = image_texture_conversion::from_dynamic(resized, is_srgb);

            let resized_handle = textures.add(resized);
            let weak = resized_handle.clone_weak();
            let texture_id = egui_context.add_image(resized_handle.clone());
            entry.insert(resized_handle);
            scaled_down_textures.rescaled_textures.insert(weak.clone());

            (weak, texture_id)
        }
    };

    Some((texture, texture_id))
}
