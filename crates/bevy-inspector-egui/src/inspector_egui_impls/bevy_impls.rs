use bevy_color::{Color, Hsla, Hsva, LinearRgba, Srgba};
use bevy_ecs::entity::Entity;
use bevy_ecs::world::CommandQueue;
use bevy_ecs::world::World;
use egui::Color32;
use std::any::Any;

#[cfg(feature = "bevy_render")]
use ::{
    bevy_asset::Assets, bevy_asset::Handle, bevy_camera::visibility::RenderLayers, bevy_mesh::Mesh,
};

#[cfg(feature = "bevy_render")]
use crate::bevy_inspector::errors::{dead_asset_handle, show_error};
use crate::{
    bevy_inspector::errors::no_world_in_context,
    egui_utils,
    inspector_options::std_options::{EntityDisplay, EntityOptions},
    reflect_inspector::{Context, InspectorUi},
};

use super::InspectorPrimitive;

impl InspectorPrimitive for uuid::Uuid {
    fn ui(&mut self, ui: &mut egui::Ui, _: &dyn Any, _: egui::Id, _: InspectorUi<'_, '_>) -> bool {
        ui.label(self.to_string());
        false
    }
    fn ui_readonly(&self, ui: &mut egui::Ui, _: &dyn Any, _: egui::Id, _: InspectorUi<'_, '_>) {
        ui.label(self.to_string());
    }
}

impl InspectorPrimitive for Entity {
    fn ui(
        &mut self,
        ui: &mut egui::Ui,
        options: &dyn Any,
        id: egui::Id,
        mut env: InspectorUi<'_, '_>,
    ) -> bool {
        let entity = *self;

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
                    .id_salt(id)
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
                        if options.despawnable
                            && world.contains_entity(entity)
                            && let Some(queue) = queue
                            && egui_utils::label_button(ui, "✖ Despawn", egui::Color32::RED)
                        {
                            queue.push(move |world: &mut World| {
                                world.entity_mut(entity).despawn();
                            });
                        }
                    });
            }
        }
        false
    }

    fn ui_readonly(&self, ui: &mut egui::Ui, _: &dyn Any, _: egui::Id, _: InspectorUi<'_, '_>) {
        ui.label(format!("{self:?}"));
    }
}

#[cfg(feature = "bevy_render")]
impl InspectorPrimitive for Handle<Mesh> {
    fn ui(
        &mut self,
        ui: &mut egui::Ui,
        _: &dyn Any,
        _: egui::Id,
        env: InspectorUi<'_, '_>,
    ) -> bool {
        let handle = &*self;
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
            dead_asset_handle(ui, handle.id().untyped());
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

    fn ui_readonly(&self, ui: &mut egui::Ui, _: &dyn Any, _: egui::Id, env: InspectorUi<'_, '_>) {
        let Some(world) = &mut env.context.world else {
            no_world_in_context(ui, "Handle<Mesh>");
            return;
        };
        let meshes = match world.get_resource_mut::<Assets<Mesh>>() {
            Ok(meshes) => meshes,
            Err(error) => return show_error(error, ui, "Assets<Mesh>"),
        };
        let Some(mesh) = meshes.get(self) else {
            return dead_asset_handle(ui, self.id().untyped());
        };

        mesh_ui_inner(mesh, ui);
    }
}

#[cfg(feature = "bevy_render")]
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
                bevy_mesh::Indices::U16(vec) => vec.len(),
                bevy_mesh::Indices::U32(vec) => vec.len(),
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

impl InspectorPrimitive for Srgba {
    fn ui(&mut self, ui: &mut egui::Ui, _: &dyn Any, _: egui::Id, _: InspectorUi<'_, '_>) -> bool {
        let mut color = Color32::from_rgba_unmultiplied(
            (self.red * 255.) as u8,
            (self.green * 255.) as u8,
            (self.blue * 255.) as u8,
            (self.alpha * 255.) as u8,
        );
        if ui.color_edit_button_srgba(&mut color).changed() {
            let [r, g, b, a] = color.to_srgba_unmultiplied();
            self.red = r as f32 / 255.;
            self.green = g as f32 / 255.;
            self.blue = b as f32 / 255.;
            self.alpha = a as f32 / 255.;
            return true;
        }
        false
    }

    fn ui_readonly(
        &self,
        ui: &mut egui::Ui,
        options: &dyn Any,
        id: egui::Id,
        env: InspectorUi<'_, '_>,
    ) {
        let mut copy = *self;
        ui.add_enabled_ui(false, |ui| copy.ui(ui, options, id, env));
    }
}
impl InspectorPrimitive for LinearRgba {
    fn ui(&mut self, ui: &mut egui::Ui, _: &dyn Any, _: egui::Id, _: InspectorUi<'_, '_>) -> bool {
        let mut color = [self.red, self.green, self.blue, self.alpha];
        if ui
            .color_edit_button_rgba_premultiplied(&mut color)
            .changed()
        {
            self.red = color[0];
            self.green = color[1];
            self.blue = color[2];
            self.alpha = color[3];
            return true;
        }
        false
    }

    fn ui_readonly(
        &self,
        ui: &mut egui::Ui,
        options: &dyn Any,
        id: egui::Id,
        env: InspectorUi<'_, '_>,
    ) {
        let mut copy = *self;
        ui.add_enabled_ui(false, |ui| copy.ui(ui, options, id, env));
    }
}
impl InspectorPrimitive for Hsla {
    fn ui(&mut self, ui: &mut egui::Ui, _: &dyn Any, _: egui::Id, _: InspectorUi<'_, '_>) -> bool {
        let mut hsva =
            egui::ecolor::Hsva::new(self.hue, self.saturation, self.lightness, self.alpha);
        if ui.color_edit_button_hsva(&mut hsva).changed() {
            self.hue = hsva.h;
            self.saturation = hsva.s;
            self.lightness = hsva.v;
            self.alpha = hsva.a;
            return true;
        }
        false
    }

    fn ui_readonly(
        &self,
        ui: &mut egui::Ui,
        options: &dyn Any,
        id: egui::Id,
        env: InspectorUi<'_, '_>,
    ) {
        let mut copy = *self;
        ui.add_enabled_ui(false, |ui| copy.ui(ui, options, id, env));
    }
}
impl InspectorPrimitive for Hsva {
    fn ui(&mut self, ui: &mut egui::Ui, _: &dyn Any, _: egui::Id, _: InspectorUi<'_, '_>) -> bool {
        let mut hsva = egui::ecolor::Hsva::new(self.hue, self.saturation, self.value, self.alpha);
        if ui.color_edit_button_hsva(&mut hsva).changed() {
            self.hue = hsva.h;
            self.saturation = hsva.s;
            self.value = hsva.v;
            self.alpha = hsva.a;
            return true;
        }
        false
    }

    fn ui_readonly(
        &self,
        ui: &mut egui::Ui,
        options: &dyn Any,
        id: egui::Id,
        env: InspectorUi<'_, '_>,
    ) {
        let mut copy = *self;
        ui.add_enabled_ui(false, |ui| copy.ui(ui, options, id, env));
    }
}

impl InspectorPrimitive for Color {
    fn ui(
        &mut self,
        ui: &mut egui::Ui,
        options: &dyn Any,
        id: egui::Id,
        env: InspectorUi<'_, '_>,
    ) -> bool {
        match self {
            Color::Srgba(color) => color.ui(ui, options, id, env),
            Color::LinearRgba(color) => color.ui(ui, options, id, env),
            Color::Hsla(color) => color.ui(ui, options, id, env),
            Color::Hsva(color) => color.ui(ui, options, id, env),
            Color::Hwba(_)
            | Color::Lcha(_)
            | Color::Laba(_)
            | Color::Oklaba(_)
            | Color::Oklcha(_)
            | Color::Xyza(_) => {
                ui.label(format!(
                    "Colorspace of {self:?} is not supported yet. PRs welcome"
                ));
                false
            }
        }
    }

    fn ui_readonly(
        &self,
        ui: &mut egui::Ui,
        options: &dyn Any,
        id: egui::Id,
        env: InspectorUi<'_, '_>,
    ) {
        let mut copy = *self;
        ui.add_enabled_ui(false, |ui| copy.ui(ui, options, id, env));
    }
}

#[cfg(feature = "bevy_render")]
impl InspectorPrimitive for RenderLayers {
    fn ui(&mut self, ui: &mut egui::Ui, _: &dyn Any, id: egui::Id, _: InspectorUi<'_, '_>) -> bool {
        let mut new_value = None;
        egui::Grid::new(id).num_columns(2).show(ui, |ui| {
            for layer in self.iter() {
                let mut layer_copy = layer;
                if ui.add(egui::DragValue::new(&mut layer_copy)).changed() {
                    new_value = Some(self.clone().without(layer).with(layer_copy));
                }

                if ui.button("-").clicked() {
                    new_value = Some(self.clone().without(layer));
                }
                ui.end_row();
            }
        });

        ui.horizontal(|ui| {
            if ui.button("Add").clicked() {
                let new_layer = self.iter().last().map_or(0, |last| last + 1);
                new_value = Some(self.clone().with(new_layer));
            }
        });

        if let Some(new_value) = new_value {
            *self = new_value;
            true
        } else {
            false
        }
    }

    fn ui_readonly(&self, ui: &mut egui::Ui, _: &dyn Any, _: egui::Id, _: InspectorUi<'_, '_>) {
        for layer in self.iter() {
            ui.label(format!("- {layer}"));
        }
    }
}

#[cfg(feature = "bevy_gizmos")]
impl InspectorPrimitive for bevy_gizmos::config::GizmoConfigStore {
    fn ui(
        &mut self,
        ui: &mut egui::Ui,
        _: &dyn Any,
        id: egui::Id,
        mut env: InspectorUi<'_, '_>,
    ) -> bool {
        for (ty, group, value) in self.iter_mut() {
            use egui::CollapsingHeader;

            let name = env
                .type_registry
                .get(*ty)
                .map(|x| x.type_info().ty().short_path())
                .unwrap_or("<unknown gizmo group>");
            CollapsingHeader::new(name)
                .id_salt(id.with(ty))
                .show(ui, |ui| {
                    env.ui_for_reflect(group, ui);
                    ui.separator();
                    env.ui_for_reflect_with_options(value, ui, egui::Id::new("data"), &());
                });
        }

        false
    }
    fn ui_readonly(
        &self,
        ui: &mut egui::Ui,
        _: &dyn Any,
        id: egui::Id,
        mut env: InspectorUi<'_, '_>,
    ) {
        for (ty, group, value) in self.iter() {
            use egui::CollapsingHeader;

            let name = env
                .type_registry
                .get(*ty)
                .map(|x| x.type_info().ty().short_path())
                .unwrap_or("<unknown gizmo group>");
            CollapsingHeader::new(name)
                .id_salt(id.with(ty))
                .show(ui, |ui| {
                    env.ui_for_reflect_readonly(group, ui);
                    ui.separator();
                    env.ui_for_reflect_readonly_with_options(value, ui, egui::Id::new("data"), &());
                });
        }
    }
}
