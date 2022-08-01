use bevy::prelude::*;
use bevy::render::primitives::{CubemapFrusta, Frustum, Plane};
use bevy::render::render_resource::ShaderImport;
use bevy::render::view::RenderLayers;
use bevy::render::{
    camera::{DepthCalculation, ScalingMode, WindowOrigin},
    mesh::{Indices, PrimitiveTopology},
    view::VisibleEntities,
};
use bevy_egui::egui::{self, Grid, RichText};

use crate::{utils, Context, Inspectable};

use super::NumberAttributes;

impl_for_struct_delegate_fields!(
    OrthographicProjection:
    left,
    right,
    bottom,
    top,
    near,
    far,
    window_origin,
    scaling_mode,
    scale with NumberAttributes::positive(),
    depth_calculation
);

impl_for_simple_enum!(
    PrimitiveTopology: PointList,
    LineList,
    LineStrip,
    TriangleList,
    TriangleStrip
);

impl_for_simple_enum!(WindowOrigin: Center, BottomLeft);

impl Inspectable for ScalingMode {
    type Attributes = ();
    fn ui(&mut self, ui: &mut egui::Ui, _: Self::Attributes, context: &mut Context) -> bool {
        let mut changed = false;
        crate::egui::ComboBox::from_id_source(context.id())
            .selected_text(format!("{self:?}"))
            .show_ui(ui, |ui| {
                if ui
                    .selectable_label(
                        matches!(self, ScalingMode::None),
                        format!("{:?}", ScalingMode::None),
                    )
                    .clicked()
                {
                    *self = ScalingMode::None;
                    changed = true;
                }
                if ui
                    .selectable_label(
                        matches!(self, ScalingMode::WindowSize),
                        format!("{:?}", ScalingMode::WindowSize),
                    )
                    .clicked()
                {
                    *self = ScalingMode::WindowSize;
                    changed = true;
                }
                if ui
                    .selectable_label(
                        matches!(self, ScalingMode::Auto { .. }),
                        format!(
                            "{:?}",
                            ScalingMode::Auto {
                                min_width: 10.0,
                                min_height: 10.0
                            }
                        ),
                    )
                    .clicked()
                {
                    *self = ScalingMode::Auto {
                        min_width: 10.0,
                        min_height: 10.0,
                    };
                    changed = true;
                }
                if ui
                    .selectable_label(
                        matches!(self, ScalingMode::FixedVertical(_)),
                        format!("{:?}", ScalingMode::FixedVertical(10.0)),
                    )
                    .clicked()
                {
                    *self = ScalingMode::FixedVertical(10.0);
                    changed = true;
                }
                if ui
                    .selectable_label(
                        matches!(self, ScalingMode::FixedHorizontal(_)),
                        format!("{:?}", ScalingMode::FixedHorizontal(10.0)),
                    )
                    .clicked()
                {
                    *self = ScalingMode::FixedHorizontal(10.0);
                    changed = true;
                }
            });
        changed
    }
}
impl_for_simple_enum!(DepthCalculation: Distance, ZDifference);

//////// SHAPES ////////

impl_for_struct_delegate_fields!(shape::Cube: size);
impl_for_struct_delegate_fields!(shape::Quad: size, flip);
impl_for_struct_delegate_fields!(shape::Plane: size);
impl_for_struct_delegate_fields!(
    shape::Capsule: radius,
    rings,
    depth,
    latitudes,
    longitudes,
    uv_profile
);
impl_for_simple_enum!(shape::CapsuleUvProfile: Aspect, Uniform, Fixed);
impl_for_struct_delegate_fields!(shape::Icosphere: radius, subdivisions);
impl_for_struct_delegate_fields!(
    shape::Torus: radius,
    ring_radius,
    subdivisions_segments,
    subdivisions_sides
);

impl Inspectable for shape::Box {
    type Attributes = ();

    fn ui(&mut self, ui: &mut egui::Ui, _: Self::Attributes, context: &mut Context) -> bool {
        let mut changed = false;

        let mut min = Vec3::new(self.min_x, self.min_y, self.min_z);
        let mut max = Vec3::new(self.max_x, self.max_y, self.max_z);

        ui.vertical_centered(|ui| {
            egui::Grid::new(context.id()).show(ui, |ui| {
                ui.label("Min");
                changed |= min.ui(ui, Default::default(), &mut context.with_id(0));
                ui.end_row();
                ui.label("Max");
                changed |= max.ui(ui, Default::default(), &mut context.with_id(0));
                ui.end_row();
            });
        });

        self.min_x = min.x;
        self.min_y = min.y;
        self.min_z = min.z;
        self.max_x = max.x;
        self.max_y = max.y;
        self.max_z = max.z;

        changed
    }
}

impl Inspectable for Mesh {
    type Attributes = ();

    fn ui(&mut self, ui: &mut egui::Ui, _: Self::Attributes, context: &mut Context) -> bool {
        Grid::new(context.id()).show(ui, |ui| {
            ui.label("Primitive Topology");
            let _ = ui.button(format!("{:?}", self.primitive_topology()));

            let attributes = &[
                Mesh::ATTRIBUTE_POSITION,
                Mesh::ATTRIBUTE_COLOR,
                Mesh::ATTRIBUTE_UV_0,
                Mesh::ATTRIBUTE_NORMAL,
                Mesh::ATTRIBUTE_TANGENT,
                Mesh::ATTRIBUTE_COLOR,
                Mesh::ATTRIBUTE_JOINT_INDEX,
                Mesh::ATTRIBUTE_JOINT_WEIGHT,
            ];
            ui.end_row();

            ui.label("Vertices");
            ui.label(self.count_vertices().to_string());
            ui.end_row();

            if let Some(indices) = self.indices() {
                ui.label("Indices");
                let len = match indices {
                    Indices::U16(vec) => vec.len(),
                    Indices::U32(vec) => vec.len(),
                };
                ui.label(len.to_string());
                ui.end_row();
            }

            ui.label("Vertex Attributes");
            ui.collapsing("Attributes", |ui| {
                ui.vertical(|ui| {
                    for attribute in attributes {
                        if self.attribute(attribute.id).is_some() {
                            ui.label(attribute.name);
                        }
                    }
                });
            });
        });

        false
    }
}

impl Inspectable for VisibleEntities {
    type Attributes = ();

    fn ui(&mut self, ui: &mut egui::Ui, _: Self::Attributes, _: &mut Context) -> bool {
        let len = self.entities.len();
        let entity = match len {
            1 => "entity",
            _ => "entities",
        };
        ui.label(format!("{} visible {}", self.entities.len(), entity));
        false
    }
}

#[derive(Debug, Clone)]
pub struct ColorAttributes {
    pub alpha: bool,
}

impl Default for ColorAttributes {
    fn default() -> Self {
        Self { alpha: true }
    }
}

impl Inspectable for Color {
    type Attributes = ColorAttributes;

    fn ui(&mut self, ui: &mut egui::Ui, options: Self::Attributes, _: &mut Context) -> bool {
        let old: [f32; 4] = (*self).into();

        if options.alpha {
            let mut color = egui::color::Color32::from_rgba_premultiplied(
                (old[0] * u8::MAX as f32) as u8,
                (old[1] * u8::MAX as f32) as u8,
                (old[2] * u8::MAX as f32) as u8,
                (old[3] * u8::MAX as f32) as u8,
            );
            let changed = ui.color_edit_button_srgba(&mut color).changed();
            let [r, g, b, a] = color.to_array();
            *self = Color::rgba_u8(r, g, b, a);

            changed
        } else {
            let mut color = [old[0], old[1], old[2]];
            let changed = ui.color_edit_button_rgb(&mut color).changed();
            let [r, g, b] = color;
            *self = Color::rgba(r, g, b, old[3]);

            changed
        }
    }
}

impl Inspectable for CubemapFrusta {
    type Attributes = ();

    fn ui(&mut self, ui: &mut egui::Ui, _: Self::Attributes, cx: &mut Context) -> bool {
        for frustrum in self.iter_mut() {
            frustrum.ui(ui, (), cx);
        }
        false
    }
}

impl Inspectable for Frustum {
    type Attributes = ();

    fn ui(&mut self, ui: &mut egui::Ui, _: Self::Attributes, cx: &mut Context) -> bool {
        for plane in self.planes.iter_mut() {
            plane.ui(ui, (), cx);
        }
        false
    }
}

impl Inspectable for Plane {
    type Attributes = ();

    fn ui(&mut self, ui: &mut egui::Ui, _: Self::Attributes, context: &mut Context) -> bool {
        let mut changed = false;
        ui.vertical_centered(|ui| {
            Grid::new(context.id()).show(ui, |ui| {
                ui.label("Normal");
                let mut normal = self.normal();
                changed |= normal.ui(ui, Default::default(), context);
                ui.end_row();

                ui.label("Distance");
                let mut distance = self.d();
                changed |= distance.ui(ui, Default::default(), context);
                ui.end_row();

                *self = Self::new(normal.extend(distance));
            });
        });
        changed
    }
}

impl Inspectable for RenderLayers {
    type Attributes = ();

    fn ui(&mut self, ui: &mut egui::Ui, _: Self::Attributes, context: &mut Context) -> bool {
        let mut to_delete = None;

        let mut changed = false;
        for layer in self.iter() {
            ui.horizontal(|ui| {
                if utils::ui::label_button(ui, "✖", egui::Color32::RED) {
                    to_delete = Some(layer);
                }
                ui.label(layer.to_string());
            });
        }

        ui.horizontal(|ui| {
            let id = context.id();
            let mut new_layer: u8 = *ui
                .memory()
                .data
                .get_temp_mut_or_insert_with(id, || self.iter().next().map_or(0, |val| val + 1));

            if utils::ui::label_button(ui, "+", egui::Color32::GREEN) {
                *self = self.with(new_layer);
                changed = true;
            }

            if new_layer.ui(
                ui,
                NumberAttributes::default().with_max(RenderLayers::TOTAL_LAYERS as u8 - 1),
                context,
            ) {
                ui.memory().data.insert_temp(id, new_layer);
            }
        });

        if let Some(to_remove) = to_delete {
            *self = self.without(to_remove);
            changed = true;
        }

        changed
    }
}

impl Inspectable for Shader {
    type Attributes = ();

    fn ui(&mut self, ui: &mut egui::Ui, (): Self::Attributes, _: &mut Context) -> bool {
        if let Some(import) = self.import_path() {
            ui.label(RichText::new(shader_import_to_string(import)).color(egui::Color32::WHITE));
        } else {
            ui.label(RichText::new("<no import path>").color(egui::Color32::WHITE));
        }

        let imports = self.imports();
        if imports.len() > 0 {
            ui.label("Imports:");
            for import in imports {
                ui.label(format!("- {}", shader_import_to_string(import)));
            }
        }

        false
    }
}

fn shader_import_to_string(import: &ShaderImport) -> String {
    match import {
        ShaderImport::AssetPath(path) => {
            format!("\"{}\"", path)
        }
        ShaderImport::Custom(custom) => custom.clone(),
    }
}
