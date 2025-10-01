use std::{
    any::Any,
    collections::{HashMap, HashSet, hash_map::Entry},
    sync::LazyLock,
    sync::Mutex,
};

use crate::utils::pretty_type_name;
use bevy_asset::{Assets, Handle};
use bevy_egui::{EguiTextureHandle, EguiUserTextures};
use bevy_image::Image;
use bevy_reflect::DynamicTypePath;
use egui::{Vec2, load::SizedTexture};

use crate::{
    bevy_inspector::errors::{no_world_in_context, show_error},
    dropdown::DropDownBox,
    reflect_inspector::InspectorUi,
    restricted_world_view::RestrictedWorldView,
};

use super::InspectorPrimitive;

mod image_texture_conversion;

impl InspectorPrimitive for Handle<Image> {
    fn ui(
        &mut self,
        ui: &mut egui::Ui,
        _: &dyn Any,
        id: egui::Id,
        env: InspectorUi<'_, '_>,
    ) -> bool {
        let Some(world) = &mut env.context.world else {
            let immutable_self: &Handle<Image> = self;
            no_world_in_context(ui, immutable_self.reflect_short_type_path());
            return false;
        };

        update_and_show_image(self, world, ui);
        let (asset_server, images) =
            match world.get_two_resources_mut::<bevy_asset::AssetServer, Assets<Image>>() {
                (Ok(a), Ok(b)) => (a, b),
                (a, b) => {
                    if let Err(e) = a {
                        show_error(e, ui, &pretty_type_name::<bevy_asset::AssetServer>());
                    }
                    if let Err(e) = b {
                        show_error(e, ui, &pretty_type_name::<Assets<Image>>());
                    }
                    return false;
                }
            };

        // get all loaded image paths
        let mut image_paths = Vec::with_capacity(images.len());
        let mut handles = HashMap::new();
        for image in images.iter() {
            if let Some(image_path) = asset_server.get_path(image.0) {
                image_paths.push(image_path.to_string());
                handles.insert(image_path.to_string(), image.0);
            }
        }

        // first, get the typed search text from a stored egui data value
        let mut selected_path = None;
        let mut image_picker_search_text = String::from("");
        ui.data_mut(|data| {
            image_picker_search_text.clone_from(
                data.get_temp_mut_or_default::<String>(id.with("image_picker_search_text")),
            );
        });

        // build and show the dropdown
        let dropdown = DropDownBox::from_iter(
            image_paths.iter(),
            id.with("image_picker"),
            &mut image_picker_search_text,
            |ui, path| {
                let response = ui
                    .selectable_label(
                        self.path()
                            .is_some_and(|p| p.path().as_os_str().to_string_lossy().eq(path)),
                        path,
                    )
                    .on_hover_ui_at_pointer(|ui| {
                        if let Some(id) = handles.get(path) {
                            let s: Option<SizedTexture> =
                                ui.data(|d| d.get_temp(format!("image:{}", id).into()));
                            if let Some(id) = s {
                                ui.image(id);
                            }
                        }
                    });
                if response.clicked() {
                    selected_path = Some(path.to_string());
                }
                response
            },
        )
        .hint_text("Select image asset");
        ui.add_enabled(!image_paths.is_empty(), dropdown)
            .on_disabled_hover_text("No image assets are available");

        // update the typed search text
        ui.data_mut(|data| {
            *data.get_temp_mut_or_default::<String>(id.with("image_picker_search_text")) =
                image_picker_search_text;
        });

        // if the user selected an option, update the image handle
        if let Some(selected_path) = selected_path {
            *self = asset_server.load(selected_path);
        }

        false
    }

    fn ui_readonly(&self, ui: &mut egui::Ui, _: &dyn Any, _: egui::Id, env: InspectorUi<'_, '_>) {
        let Some(world) = &mut env.context.world else {
            no_world_in_context(ui, self.reflect_short_type_path());
            return;
        };

        update_and_show_image(self, world, ui);
    }
}

static SCALED_DOWN_TEXTURES: LazyLock<Mutex<ScaledDownTextures>> = LazyLock::new(Default::default);

fn update_and_show_image(
    image: &Handle<Image>,
    world: &mut RestrictedWorldView,
    ui: &mut egui::Ui,
) {
    let (mut egui_user_textures, mut images) =
        match world.get_two_resources_mut::<bevy_egui::EguiUserTextures, Assets<Image>>() {
            (Ok(a), Ok(b)) => (a, b),
            (a, b) => {
                if let Err(e) = a {
                    show_error(e, ui, &pretty_type_name::<bevy_egui::EguiContext>());
                }
                if let Err(e) = b {
                    show_error(e, ui, &pretty_type_name::<Assets<Image>>());
                }
                return;
            }
        };

    let mut scaled_down_textures = SCALED_DOWN_TEXTURES.lock().unwrap();

    // todo: read asset events to re-rescale images of they changed
    let rescaled = rescaled_image(
        image,
        &mut scaled_down_textures,
        &mut images,
        &mut egui_user_textures,
    );
    let (rescaled_handle, texture_id) = match rescaled {
        Some(it) => it,
        None => {
            ui.label("<texture>");
            return;
        }
    };

    let rescaled_image = images.get(&rescaled_handle).unwrap();
    ui.data_mut(|d| {
        d.insert_temp(
            format!("image:{}", image.id()).into(),
            SizedTexture {
                id: texture_id,
                size: Vec2::new(
                    rescaled_image.texture_descriptor.size.width as f32,
                    rescaled_image.texture_descriptor.size.height as f32,
                ),
            },
        )
    });
    show_image(rescaled_image, texture_id, ui);
}

fn show_image(
    image: &Image,
    texture_id: egui::TextureId,
    ui: &mut egui::Ui,
) -> Option<egui::Response> {
    let size = image.texture_descriptor.size;
    let size = egui::Vec2::new(size.width as f32, size.height as f32);

    let source = SizedTexture {
        id: texture_id,
        size,
    };

    if size.max_elem() >= 128.0 {
        let response = egui::CollapsingHeader::new("Texture").show(ui, |ui| ui.image(source));
        response.body_response
    } else {
        let response = ui.image(source);
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
    egui_usere_textures: &mut EguiUserTextures,
) -> Option<(Handle<Image>, egui::TextureId)> {
    let (texture, texture_id) = match scaled_down_textures.textures.entry(handle.clone()) {
        Entry::Occupied(handle) => {
            let handle: Handle<Image> = handle.get().clone();
            (
                handle.clone(),
                egui_usere_textures.add_image(EguiTextureHandle::Strong(handle)),
            )
        }
        Entry::Vacant(entry) => {
            if scaled_down_textures.rescaled_textures.contains(handle) {
                return None;
            }

            let original = textures.get(handle)?;

            let (image, is_srgb) = image_texture_conversion::try_into_dynamic(original)?;
            let resized = image.resize(
                RESCALE_TO_FIT.0,
                RESCALE_TO_FIT.1,
                image::imageops::FilterType::Triangle,
            );
            let resized = image_texture_conversion::from_dynamic(resized, is_srgb);

            let resized_handle = textures.add(resized);
            let weak = resized_handle.clone();
            let texture_id =
                egui_usere_textures.add_image(EguiTextureHandle::Strong(resized_handle.clone()));
            entry.insert(resized_handle);
            scaled_down_textures.rescaled_textures.insert(weak.clone());

            (weak, texture_id)
        }
    };

    Some((texture, texture_id))
}
