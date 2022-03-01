use crate::options::{NumberAttributes, OptionAttributes, Vec2dAttributes};
use crate::{utils, Context, Inspectable};
use bevy::math::Vec4Swizzles;
use bevy::pbr::{Clusters, CubemapVisibleEntities, StandardMaterial, VisiblePointLights};
use bevy::render::mesh::VertexAttributeValues;
use bevy::render::primitives::{CubemapFrusta, Frustum, Plane};
use bevy::render::render_resource::{PrimitiveTopology, ShaderImport};
use bevy::render::view::{RenderLayers, VisibleEntities};
use bevy::sprite::Mesh2dHandle;
use bevy::text::Text2dSize;
use bevy::{
    asset::HandleId,
    pbr::DirectionalLight,
    render::{
        camera::{DepthCalculation, ScalingMode, WindowOrigin},
        mesh::Indices,
    },
};
use bevy::{pbr::AmbientLight, prelude::*};
use bevy_egui::egui::{self, RichText};
use egui::Grid;

impl_for_struct_delegate_fields!(
    PointLight:
    color,
    intensity with NumberAttributes::positive().with_speed(1.0),
    range with NumberAttributes::positive(),
    radius with NumberAttributes::positive(),
    shadows_enabled,
    shadow_depth_bias with NumberAttributes::positive(),
    shadow_normal_bias with NumberAttributes::positive(),
);

impl_for_struct_delegate_fields!(
    DirectionalLight: color,
    illuminance with NumberAttributes::positive(),
    shadows_enabled,
    shadow_projection,
    shadow_depth_bias with NumberAttributes::positive(),
    shadow_normal_bias with NumberAttributes::positive(),
);

impl_for_struct_delegate_fields!(ColorMaterial: color, texture);

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
impl_for_simple_enum!(
    ScalingMode: None,
    WindowSize,
    FixedVertical,
    FixedHorizontal
);
impl_for_simple_enum!(DepthCalculation: Distance, ZDifference);

impl_for_struct_delegate_fields!(Sprite:
    color,
    flip_x,
    flip_y,
    custom_size with OptionAttributes { replacement: Some(|| Vec2::splat(50.0)), deletable: true, inner: Vec2dAttributes::positive() }
);

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

//////// COMPONENTS ////////

impl Inspectable for Transform {
    type Attributes = ();

    fn ui(
        &mut self,
        ui: &mut bevy_egui::egui::Ui,
        _options: Self::Attributes,
        context: &mut Context,
    ) -> bool {
        let mut changed = false;
        ui.vertical_centered(|ui| {
            Grid::new(context.id()).show(ui, |ui| {
                ui.label("Translation");
                changed |= self.translation.ui(ui, Default::default(), context);
                ui.end_row();

                ui.label("Rotation");
                changed |= self.rotation.ui(ui, Default::default(), context);
                self.rotation = self.rotation.normalize();
                ui.end_row();

                ui.label("Scale");
                let scale_attributes = NumberAttributes {
                    min: Some(Vec3::splat(0.0)),
                    ..Default::default()
                };
                changed |= self.scale.ui(ui, scale_attributes, context);
                ui.end_row();
            });
        });
        changed
    }
}

impl Inspectable for GlobalTransform {
    type Attributes = <Transform as Inspectable>::Attributes;

    fn ui(&mut self, ui: &mut egui::Ui, options: Self::Attributes, context: &mut Context) -> bool {
        let global_transform = std::mem::take(self);

        let mut transform = Transform {
            translation: global_transform.translation,
            rotation: global_transform.rotation,
            scale: global_transform.scale,
        };

        let changed = transform.ui(ui, options, context);

        *self = GlobalTransform {
            translation: transform.translation,
            rotation: transform.rotation,
            scale: transform.scale,
        };

        changed
    }
}

impl Inspectable for Mat3 {
    type Attributes = ();

    fn ui(&mut self, ui: &mut egui::Ui, _: Self::Attributes, context: &mut Context) -> bool {
        let mut changed = false;
        ui.vertical(|ui| {
            changed |= self.x_axis.ui(ui, Default::default(), context);
            changed |= self.y_axis.ui(ui, Default::default(), context);
            changed |= self.z_axis.ui(ui, Default::default(), context);
        });
        changed
    }
}

impl Inspectable for Mat4 {
    type Attributes = ();

    fn ui(&mut self, ui: &mut egui::Ui, _: Self::Attributes, context: &mut Context) -> bool {
        let mut changed = false;
        ui.vertical(|ui| {
            changed |= self.x_axis.ui(ui, Default::default(), context);
            changed |= self.y_axis.ui(ui, Default::default(), context);
            changed |= self.z_axis.ui(ui, Default::default(), context);
            changed |= self.w_axis.ui(ui, Default::default(), context);
        });
        changed
    }
}

#[derive(Default, Debug, Clone)]
pub struct ColorAttributes {
    pub alpha: bool,
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

//////// RESOURCES ////////

impl Inspectable for AmbientLight {
    type Attributes = <Color as Inspectable>::Attributes;

    fn ui(&mut self, ui: &mut egui::Ui, options: Self::Attributes, context: &mut Context) -> bool {
        let brightness_attributes = NumberAttributes::positive().with_speed(0.01);

        self.color.ui(ui, options, context);
        self.brightness.ui(ui, brightness_attributes, context)
    }
}
impl Inspectable for ClearColor {
    type Attributes = <Color as Inspectable>::Attributes;

    fn ui(&mut self, ui: &mut egui::Ui, options: Self::Attributes, context: &mut Context) -> bool {
        self.0.ui(ui, options, context)
    }
}

////// OTHER //////

impl_for_struct_delegate_fields!(bevy::sprite::Rect:
    min with Vec2dAttributes::integer(),
    max with Vec2dAttributes::integer(),
);
impl_for_struct_delegate_fields!(TextureAtlasSprite: color, index, flip_x, flip_y);

impl_for_struct_delegate_fields!(Text2dSize: size);

impl Inspectable for TextureAtlas {
    type Attributes = ();

    fn ui(&mut self, ui: &mut egui::Ui, _: Self::Attributes, context: &mut Context) -> bool {
        let mut changed = false;
        egui::Grid::new(context.id()).show(ui, |ui| {
            ui.label("texture");
            changed |= self
                .texture
                .ui(ui, Default::default(), &mut context.with_id(0));
            ui.end_row();

            ui.label("size");
            changed |= self.size.ui(ui, Vec2dAttributes::integer(), context);
            ui.end_row();

            ui.label("textures");
            ui.collapsing("Sections", |ui| {
                changed |= self
                    .textures
                    .ui(ui, Default::default(), &mut context.with_id(2));
            });
        });
        changed
    }
}

#[rustfmt::skip]
impl Inspectable for StandardMaterial {
    type Attributes = ();

    fn ui(&mut self, ui: &mut egui::Ui, _: Self::Attributes, context: &mut Context) -> bool {
        let mut changed = false;
        ui.vertical_centered(|ui| {
            egui::Grid::new(context.id()).show(ui, |ui| {
                egui::Grid::new("grid").show(ui, |ui| {
                    ui.label("base_color");
                    changed |= self.base_color.ui(ui, Default::default(), context);
                    ui.end_row();

                    ui.label("emissive");
                    changed |= self.emissive.ui(ui, Default::default(), context);
                    ui.end_row();

                    ui.label("perceptual_roughness");
                    changed |= self.perceptual_roughness.ui(ui, NumberAttributes::between(0.089, 1.0).with_speed(0.01), context);
                    ui.end_row();

                    ui.label("metallic");
                    changed |= self.metallic.ui(ui, NumberAttributes::normalized().with_speed(0.01), context);
                    ui.end_row();

                    ui.label("reflectance");
                    changed |= self.reflectance.ui(ui, NumberAttributes::positive(), context);
                    ui.end_row();

                    ui.label("unlit");
                    changed |= self.unlit.ui(ui, Default::default(), context);
                    ui.end_row();
                });
            });

            ui.collapsing("Textures", |ui| {
                egui::Grid::new("Textures").show(ui, |ui| {
                    let texture_option_attributes = OptionAttributes { replacement: Some(|| Handle::weak(HandleId::random::<Image>())), ..Default::default() };

                    ui.label("base_color");
                    changed |= self.base_color_texture.ui(ui, texture_option_attributes.clone(), &mut context.with_id(0));
                    ui.end_row();

                     ui.label("normal_map");
                     changed |= self.normal_map_texture.ui(ui, texture_option_attributes.clone(), &mut context.with_id(1));
                     ui.end_row();

                    ui.label("metallic_roughness");
                    changed |= self.metallic_roughness_texture.ui(ui, texture_option_attributes.clone(), &mut context.with_id(2));
                    ui.end_row();

                    ui.label("emmissive");
                    changed |= self.emissive_texture.ui(ui, texture_option_attributes.clone(), &mut context.with_id(3));
                    ui.end_row();

                    ui.label("occlusion texture");
                    changed |= self.occlusion_texture.ui(ui, texture_option_attributes, &mut context.with_id(4));
                    ui.end_row();
                });
            });
        });
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

impl Inspectable for Mesh2dHandle {
    type Attributes = ();

    fn ui(&mut self, ui: &mut egui::Ui, _: Self::Attributes, context: &mut Context) -> bool {
        // Get the mesh from the handle
        if let Some(mesh) = context
            .world()
            .map(|world| world.get_resource::<Assets<Mesh>>())
            .flatten()
            .map(|meshes| meshes.get(&self.0))
            .flatten()
        {
            // Get 2D mesh attributes
            let indices = mesh.indices();
            let vertices = mesh.attribute(Mesh::ATTRIBUTE_POSITION);
            let colors = mesh.attribute(Mesh::ATTRIBUTE_COLOR);

            if let Some(((indices, vertices), colors)) = indices.zip(vertices).zip(colors) {
                // Convert the mesh into colored triangles
                if let Indices::U32(indices) = indices {
                    if let VertexAttributeValues::Float32x3(vertices) = vertices {
                        if let VertexAttributeValues::Float32x4(colors) = colors {
                            // Convert the mesh data into plot data
                            let vertices_and_colors = indices
                                .iter()
                                .map(|index| {
                                    let pos = vertices[*index as usize];
                                    let color = colors[*index as usize];
                                    (
                                        // Convert the bevy position to an egui position, discarding the Z value
                                        egui::plot::Value::new(pos[0], pos[1]),
                                        // Convert the bevy color to an egui color
                                        egui::Color32::from_rgba_unmultiplied(
                                            (color[0] * 255.0) as u8,
                                            (color[1] * 255.0) as u8,
                                            (color[2] * 255.0) as u8,
                                            (color[3] * 255.0) as u8,
                                        ),
                                    )
                                })
                                .collect::<Vec<_>>();

                            // Draw a grid with all the triangles
                            Grid::new(context.id()).show(ui, |ui| {
                                let plot = egui::plot::Plot::new("triangles")
                                    .legend(egui::plot::Legend::default())
                                    .data_aspect(0.8)
                                    .min_size(egui::Vec2::new(250.0, 250.0))
                                    .show_x(true)
                                    .show_y(true);

                                plot.show(ui, |plot_ui| {
                                    vertices_and_colors.chunks_exact(3).for_each(|triangle| {
                                        plot_ui.polygon(
                                            egui::plot::Polygon::new(
                                                egui::plot::Values::from_values_iter(
                                                    triangle
                                                        .iter()
                                                        .map(|vertex_and_color| vertex_and_color.0),
                                                ),
                                            )
                                            // Add thicker strokes and reduce the fill
                                            // transparency
                                            .highlight(true)
                                            .color(triangle[0].1)
                                            // Use the color as the name so everything
                                            // with the same color will be grouped
                                            .name(
                                                format!(
                                                    "#{:02X?}{:02X?}{:02X?}{:02X?}",
                                                    triangle[0].1.r(),
                                                    triangle[0].1.g(),
                                                    triangle[0].1.b(),
                                                    triangle[0].1.a()
                                                ),
                                            ),
                                        );
                                    });
                                });
                            });
                        }
                    }
                }
            }
        }

        false
    }
}

impl Inspectable for Name {
    type Attributes = ();

    fn ui(&mut self, ui: &mut egui::Ui, _: Self::Attributes, _: &mut Context) -> bool {
        ui.label(self.as_str());
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

impl Inspectable for VisiblePointLights {
    type Attributes = ();

    fn ui(&mut self, ui: &mut egui::Ui, _: Self::Attributes, _: &mut Context) -> bool {
        let len = self.len();
        let entity = match len {
            1 => "light",
            _ => "lights",
        };
        ui.label(format!("{} visible point {}", self.entities.len(), entity));
        false
    }
}

impl Inspectable for CubemapVisibleEntities {
    type Attributes = ();

    fn ui(&mut self, ui: &mut egui::Ui, _: Self::Attributes, cx: &mut Context) -> bool {
        for visible in self.iter_mut() {
            visible.ui(ui, (), cx);
        }
        false
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
                let mut normal = self.normal_d.xyz();
                changed |= normal.ui(ui, Default::default(), context);
                ui.end_row();

                ui.label("Distance");
                let mut distance = self.normal_d.w;
                changed |= distance.ui(ui, Default::default(), context);
                ui.end_row();

                self.normal_d = normal.extend(distance);
            });
        });
        changed
    }
}

impl Inspectable for Clusters {
    type Attributes = ();

    fn ui(&mut self, _: &mut egui::Ui, _: Self::Attributes, _: &mut Context) -> bool {
        false
    }
}

impl<'a, T: Inspectable> Inspectable for Mut<'a, T> {
    type Attributes = T::Attributes;

    fn ui(&mut self, ui: &mut egui::Ui, options: Self::Attributes, context: &mut Context) -> bool {
        (**self).ui(ui, options, context)
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
