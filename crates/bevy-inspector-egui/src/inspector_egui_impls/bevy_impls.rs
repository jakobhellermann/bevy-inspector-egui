use bevy_asset::{Assets, Handle};
use bevy_ecs::{entity::Entity, system::CommandQueue};
use bevy_render::mesh::Mesh;
use bevy_render::{color::Color, view::RenderLayers};
use egui::{ecolor::Hsva, Color32};
use std::any::Any;

use crate::{
    bevy_inspector::errors::{dead_asset_handle, no_world_in_context, show_error},
    egui_utils,
    inspector_options::std_options::{EntityDisplay, EntityOptions},
    many_ui,
    reflect_inspector::{Context, InspectorUi},
};

pub fn entity_ui(
    value: &mut dyn Any,
    ui: &mut egui::Ui,
    options: &dyn Any,
    id: egui::Id,
    mut env: InspectorUi<'_, '_>,
) -> bool {
    let entity = *value.downcast_ref::<Entity>().unwrap();
    let options = options
        .downcast_ref::<EntityOptions>()
        .cloned()
        .unwrap_or_default();

    match options.display {
        EntityDisplay::Id => {
            ui.label(format!("{entity:?}"));
        }
        EntityDisplay::Components => {
            let Context {
                world: Some(world),
                queue,
            } = &mut env.context
            else {
                no_world_in_context(ui, "Entity");
                return false;
            };

            let entity_name =
                crate::utils::guess_entity_name::guess_entity_name_restricted(world, entity);
            egui::CollapsingHeader::new(entity_name)
                .id_source(id)
                .show(ui, |ui| {
                    let _queue = CommandQueue::default();
                    crate::bevy_inspector::ui_for_entity_components(
                        world,
                        queue.as_deref_mut(),
                        entity,
                        ui,
                        id,
                        env.type_registry,
                    );
                    if options.despawnable && world.contains_entity(entity) {
                        if let Some(queue) = queue {
                            if egui_utils::label_button(ui, "✖ Despawn", egui::Color32::RED) {
                                queue.push(bevy_ecs::system::Despawn { entity });
                            }
                        }
                    }
                });
        }
    }

    false
}
pub fn entity_ui_readonly(
    value: &dyn Any,
    ui: &mut egui::Ui,
    _: &dyn Any,
    _: egui::Id,
    _: InspectorUi<'_, '_>,
) {
    let entity = value.downcast_ref::<Entity>().unwrap();
    ui.label(format!("{entity:?}"));
}

pub fn mesh_ui(
    value: &mut dyn Any,
    ui: &mut egui::Ui,
    _: &dyn Any,
    _: egui::Id,
    env: InspectorUi<'_, '_>,
) -> bool {
    let handle = value.downcast_ref::<Handle<Mesh>>().unwrap();

    let Some(world) = &mut env.context.world else {
        no_world_in_context(ui, "Handle<Mesh>");
        return false;
    };
    let mut meshes = match world.get_resource_mut::<Assets<Mesh>>() {
        Ok(meshes) => meshes,
        Err(error) => {
            show_error(error, ui, "Assets<Mesh>");
            return false;
        }
    };
    let Some(mesh) = meshes.get_mut(handle) else {
        dead_asset_handle(ui, handle.into());
        return false;
    };

    mesh_ui_inner(mesh, ui);

    ui.add_enabled_ui(mesh.indices().is_some(), |ui| {
        if ui.button("Duplicate vertices").clicked() {
            mesh.duplicate_vertices();
        }
    });
    ui.add_enabled_ui(mesh.indices().is_none(), |ui| {
        if ui.button("Compute flat normals").clicked() {
            mesh.compute_flat_normals();
        }
    });
    if ui.button("Generate tangents").clicked() {
        let _ = mesh.generate_tangents();
    }

    false
}

pub fn mesh_ui_readonly(
    value: &dyn Any,
    ui: &mut egui::Ui,
    _: &dyn Any,
    _: egui::Id,
    env: InspectorUi<'_, '_>,
) {
    let handle = value.downcast_ref::<Handle<Mesh>>().unwrap();

    let Some(world) = &mut env.context.world else {
        no_world_in_context(ui, "Handle<Mesh>");
        return;
    };
    let meshes = match world.get_resource_mut::<Assets<Mesh>>() {
        Ok(meshes) => meshes,
        Err(error) => return show_error(error, ui, "Assets<Mesh>"),
    };
    let Some(mesh) = meshes.get(handle) else {
        return dead_asset_handle(ui, handle.into());
    };

    mesh_ui_inner(mesh, ui);
}

fn mesh_ui_inner(mesh: &Mesh, ui: &mut egui::Ui) {
    egui::Grid::new("mesh").show(ui, |ui| {
        ui.label("primitive_topology");
        ui.label(format!("{:?}", mesh.primitive_topology()));
        ui.end_row();

        ui.label("Vertices");
        ui.label(mesh.count_vertices().to_string());
        ui.end_row();

        if let Some(indices) = mesh.indices() {
            ui.label("Indices");
            let len = match indices {
                bevy_render::mesh::Indices::U16(vec) => vec.len(),
                bevy_render::mesh::Indices::U32(vec) => vec.len(),
            };
            ui.label(len.to_string());
            ui.end_row();
        }

        ui.label("Vertex Attributes");

        let builtin_attributes = &[
            Mesh::ATTRIBUTE_POSITION,
            Mesh::ATTRIBUTE_COLOR,
            Mesh::ATTRIBUTE_UV_0,
            Mesh::ATTRIBUTE_NORMAL,
            Mesh::ATTRIBUTE_TANGENT,
            Mesh::ATTRIBUTE_COLOR,
            Mesh::ATTRIBUTE_JOINT_INDEX,
            Mesh::ATTRIBUTE_JOINT_WEIGHT,
        ];

        ui.vertical(|ui| {
            for attribute in builtin_attributes {
                if mesh.attribute(attribute.id).is_some() {
                    ui.label(attribute.name);
                }
            }
        });
    });
}

pub fn color_ui(
    value: &mut dyn Any,
    ui: &mut egui::Ui,
    _: &dyn Any,
    _: egui::Id,
    _: InspectorUi<'_, '_>,
) -> bool {
    let value = value.downcast_mut::<Color>().unwrap();

    color_ui_inner(value, ui)
}

pub fn color_ui_readonly(
    value: &dyn Any,
    ui: &mut egui::Ui,
    _: &dyn Any,
    _: egui::Id,
    _: InspectorUi<'_, '_>,
) {
    let value = value.downcast_ref::<Color>().unwrap();

    ui.add_enabled_ui(false, |ui| {
        let mut color = *value;
        color_ui_inner(&mut color, ui);
    });
}

many_ui!(color_ui_many color_ui Color);

fn color_ui_inner(value: &mut Color, ui: &mut egui::Ui) -> bool {
    match value {
        Color::Rgba {
            red,
            green,
            blue,
            alpha,
        } => {
            let mut color = Color32::from_rgba_premultiplied(
                (*red * 255.) as u8,
                (*green * 255.) as u8,
                (*blue * 255.) as u8,
                (*alpha * 255.) as u8,
            );
            if ui.color_edit_button_srgba(&mut color).changed() {
                let [r, g, b, a] = color.to_array();
                *red = r as f32 / 255.;
                *green = g as f32 / 255.;
                *blue = b as f32 / 255.;
                *alpha = a as f32 / 255.;
                return true;
            }
        }
        Color::RgbaLinear {
            red,
            green,
            blue,
            alpha,
        } => {
            let mut color = [*red, *green, *blue, *alpha];
            if ui
                .color_edit_button_rgba_premultiplied(&mut color)
                .changed()
            {
                *red = color[0];
                *green = color[1];
                *blue = color[2];
                *alpha = color[3];
                return true;
            }
        }
        Color::Hsla {
            hue,
            saturation,
            lightness,
            alpha,
        } => {
            let mut hsva = Hsva::new(*hue, *saturation, *lightness, *alpha);
            if ui.color_edit_button_hsva(&mut hsva).changed() {
                *hue = hsva.h;
                *saturation = hsva.s;
                *lightness = hsva.v;
                *alpha = hsva.a;
                return true;
            }
        }
        Color::Lcha { .. } => {
            let [hue, saturation, lightness, alpha] = value.as_hsla_f32();
            let mut hsva = Hsva::new(hue, saturation, lightness, alpha);
            if ui.color_edit_button_hsva(&mut hsva).changed() {
                *value = Color::hsla(hue, saturation, lightness, alpha).as_lcha();
                return true;
            }
        }
    }
    false
}

pub fn render_layers_ui(
    value: &mut dyn Any,
    ui: &mut egui::Ui,
    _: &dyn Any,
    id: egui::Id,
    _: InspectorUi<'_, '_>,
) -> bool {
    let value = value.downcast_mut::<RenderLayers>().unwrap();

    let mut new_value = None;
    egui::Grid::new(id).num_columns(2).show(ui, |ui| {
        for layer in value.iter() {
            let mut layer_copy = layer;
            if ui
                .add(
                    egui::DragValue::new(&mut layer_copy)
                        .clamp_range(0..=RenderLayers::TOTAL_LAYERS - 1),
                )
                .changed()
            {
                new_value = Some(value.without(layer).with(layer_copy));
            }

            if ui.button("-").clicked() {
                new_value = Some(value.without(layer));
            }
            ui.end_row();
        }
    });

    ui.horizontal(|ui| {
        if ui.button("Add").clicked() {
            let new_layer = value.iter().last().map_or(0, |last| last + 1);
            new_value = Some(value.with(new_layer));
        }
    });

    if let Some(new_value) = new_value {
        *value = new_value;
        true
    } else {
        false
    }
}

pub fn render_layers_ui_readonly(
    value: &dyn Any,
    ui: &mut egui::Ui,
    _: &dyn Any,
    _: egui::Id,
    _: InspectorUi<'_, '_>,
) {
    let value = value.downcast_ref::<RenderLayers>().unwrap();

    for layer in value.iter() {
        ui.label(format!("- {layer}"));
    }
}

many_ui!(render_layers_ui_many render_layers_ui RenderLayers);
