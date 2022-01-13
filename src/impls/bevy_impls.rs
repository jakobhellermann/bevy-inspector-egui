use crate::options::{NumberAttributes, OptionAttributes, Vec2dAttributes};
use crate::{Context, Inspectable};
use bevy::math::Vec4Swizzles;
use bevy::pbr::{CubemapVisibleEntities, StandardMaterial};
use bevy::render::primitives::{CubemapFrusta, Frustum, Plane};
use bevy::render::render_resource::PrimitiveTopology;
use bevy::render::view::VisibleEntities;
use bevy::{
    asset::HandleId,
    pbr::DirectionalLight,
    render::{
        camera::{DepthCalculation, ScalingMode, WindowOrigin},
        mesh::Indices,
    },
};
use bevy::{pbr::AmbientLight, prelude::*};
use bevy_egui::egui;
use egui::Grid;

impl_for_struct_delegate_fields!(
    PointLight:
    color,
    intensity with NumberAttributes::positive().speed(1.0),
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

impl_for_struct_delegate_fields!(
    OrthographicProjection:
    left with NumberAttributes::positive(),
    right with NumberAttributes::positive(),
    bottom with NumberAttributes::positive(),
    top with NumberAttributes::positive(),
    near with NumberAttributes::positive(),
    far with NumberAttributes::positive(),
    window_origin,
    scaling_mode,
    scale with NumberAttributes::positive(),
    depth_calculation
);

// impl_for_struct_delegate_fields!(ColorMaterial: color, texture);
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
        let brightness_attributes = NumberAttributes::positive().speed(0.01);

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
                    changed |= self.perceptual_roughness.ui(ui, NumberAttributes::between(0.089, 1.0).speed(0.01), context);
                    ui.end_row();
                    
                    ui.label("metallic");
                    changed |= self.metallic.ui(ui, NumberAttributes::normalized().speed(0.01), context);
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
                    for &attribute in attributes {
                        if self.attribute(attribute).is_some() {
                            ui.label(attribute);
                        }
                    }
                });
            });
        });

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

impl<'a, T: Inspectable> Inspectable for Mut<'a, T> {
    type Attributes = T::Attributes;

    fn ui(&mut self, ui: &mut egui::Ui, options: Self::Attributes, context: &mut Context) -> bool {
        (**self).ui(ui, options, context)
    }
}
