use crate::options::{NumberAttributes, OptionAttributes, Vec2dAttributes};
use crate::{Context, Inspectable};
use bevy::asset::HandleId;
use bevy::{pbr::AmbientLight, prelude::*};
use bevy_egui::egui;
use egui::Grid;

impl_for_struct_delegate_fields!(
    Light:
    color,
    fov,
    depth with NumberAttributes::positive().speed(1.0),
    intensity with NumberAttributes::positive().speed(1.0),
    range with NumberAttributes::positive(),
);
impl_for_struct_delegate_fields!(ColorMaterial: color, texture);

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

    fn ui(&mut self, ui: &mut egui::Ui, _: Self::Attributes, context: &Context) {
        let mut min = Vec3::new(self.min_x, self.min_y, self.min_z);
        let mut max = Vec3::new(self.max_x, self.max_y, self.max_z);

        ui.vertical_centered(|ui| {
            egui::Grid::new(context.id()).show(ui, |ui| {
                ui.label("Min");
                min.ui(ui, Default::default(), &context.with_id(0));
                ui.end_row();
                ui.label("Max");
                max.ui(ui, Default::default(), &context.with_id(0));
                ui.end_row();
            });
        });

        self.min_x = min.x;
        self.min_y = min.y;
        self.min_z = min.z;
        self.max_x = max.x;
        self.max_y = max.y;
        self.max_z = max.z;
    }
}

//////// COMPONENTS ////////

impl Inspectable for Quat {
    type Attributes = NumberAttributes<[f32; 4]>;

    fn ui(&mut self, ui: &mut egui::Ui, options: Self::Attributes, context: &Context) {
        let options = options.map(|arr| Vec4::from(*arr));
        let mut vec4 = Vec4::from(*self);
        vec4.ui(ui, options, context);
        *self = vec4.into();
    }
}

impl Inspectable for Transform {
    type Attributes = ();

    fn ui(&mut self, ui: &mut bevy_egui::egui::Ui, _options: Self::Attributes, context: &Context) {
        ui.vertical_centered(|ui| {
            Grid::new(context.id()).show(ui, |ui| {
                ui.label("Translation");
                self.translation.ui(ui, Default::default(), context);
                ui.end_row();

                ui.label("Rotation");
                self.rotation.ui(ui, Default::default(), context);
                self.rotation = self.rotation.normalize();
                ui.end_row();

                ui.label("Scale");
                let scale_attributes = NumberAttributes {
                    min: Some(Vec3::splat(0.0)),
                    ..Default::default()
                };
                self.scale.ui(ui, scale_attributes, context);
                ui.end_row();
            });
        });
    }
}

impl Inspectable for GlobalTransform {
    type Attributes = <Transform as Inspectable>::Attributes;

    fn ui(&mut self, ui: &mut egui::Ui, options: Self::Attributes, context: &Context) {
        let global_transform = std::mem::take(self);

        let mut transform = Transform {
            translation: global_transform.translation,
            rotation: global_transform.rotation,
            scale: global_transform.scale,
        };

        transform.ui(ui, options, context);

        *self = GlobalTransform {
            translation: transform.translation,
            rotation: transform.rotation,
            scale: transform.scale,
        };
    }
}

impl Inspectable for Mat3 {
    type Attributes = ();

    fn ui(&mut self, ui: &mut egui::Ui, _: Self::Attributes, context: &Context) {
        ui.vertical(|ui| {
            self.x_axis.ui(ui, Default::default(), context);
            self.y_axis.ui(ui, Default::default(), context);
            self.z_axis.ui(ui, Default::default(), context);
        });
    }
}

impl Inspectable for Mat4 {
    type Attributes = ();

    fn ui(&mut self, ui: &mut egui::Ui, _: Self::Attributes, context: &Context) {
        ui.vertical(|ui| {
            self.x_axis.ui(ui, Default::default(), context);
            self.y_axis.ui(ui, Default::default(), context);
            self.z_axis.ui(ui, Default::default(), context);
            self.w_axis.ui(ui, Default::default(), context);
        });
    }
}

#[derive(Default, Debug, Clone)]
pub struct ColorAttributes {
    pub alpha: bool,
}

impl Inspectable for Color {
    type Attributes = ColorAttributes;

    fn ui(&mut self, ui: &mut egui::Ui, options: Self::Attributes, _: &Context) {
        let old: [f32; 4] = (*self).into();

        if options.alpha {
            let mut color = egui::color::Color32::from_rgba_premultiplied(
                (old[0] * u8::MAX as f32) as u8,
                (old[1] * u8::MAX as f32) as u8,
                (old[2] * u8::MAX as f32) as u8,
                (old[3] * u8::MAX as f32) as u8,
            );
            ui.color_edit_button_srgba(&mut color);
            let [r, g, b, a] = color.to_array();
            *self = Color::rgba_u8(r, g, b, a);
        } else {
            let mut color = [old[0], old[1], old[2]];
            ui.color_edit_button_rgb(&mut color);
            let [r, g, b] = color;
            *self = Color::rgba(r, g, b, old[3]);
        }
    }
}

//////// RESOURCES ////////

impl Inspectable for AmbientLight {
    type Attributes = <Color as Inspectable>::Attributes;

    fn ui(&mut self, ui: &mut egui::Ui, options: Self::Attributes, context: &Context) {
        let brightness_attributes = NumberAttributes::positive().speed(0.01);

        self.color.ui(ui, options, context);
        self.brightness.ui(ui, brightness_attributes, context);
    }
}
impl Inspectable for ClearColor {
    type Attributes = <Color as Inspectable>::Attributes;

    fn ui(&mut self, ui: &mut egui::Ui, options: Self::Attributes, context: &Context) {
        self.0.ui(ui, options, context);
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

    fn ui(&mut self, ui: &mut egui::Ui, _: Self::Attributes, context: &Context) {
        egui::Grid::new(context.id()).show(ui, |ui| {
            ui.label("texture");
            self.texture.ui(ui, Default::default(), &context.with_id(0));
            ui.end_row();

            ui.label("size");
            self.size.ui(ui, Vec2dAttributes::integer(), context);
            ui.end_row();

            ui.label("textures");
            ui.collapsing("Sections", |ui| {
                self.textures
                    .ui(ui, Default::default(), &context.with_id(2));
            });
        });
    }
}

#[rustfmt::skip]
impl Inspectable for StandardMaterial {
    type Attributes = ();

    fn ui(&mut self, ui: &mut egui::Ui, _: Self::Attributes, context: &Context) {
        ui.vertical_centered(|ui| {
            egui::Grid::new(context.id()).show(ui, |ui| {
                ui.label("base_color");
                self.base_color.ui(ui, Default::default(), context);
                ui.end_row();

                ui.label("emissive");
                self.emissive.ui(ui, Default::default(), context);
                ui.end_row();

                ui.label("roughness");
                self.roughness.ui(ui, NumberAttributes::between(0.089, 1.0).speed(0.01), context);
                ui.end_row();

                ui.label("metallic");
                self.metallic.ui(ui, NumberAttributes::normalized().speed(0.01), context);
                ui.end_row();

                ui.label("reflectance");
                self.reflectance.ui(ui, NumberAttributes::positive(), context);
                ui.end_row();

                ui.label("unlit");
                self.unlit.ui(ui, Default::default(), context);
                ui.end_row();

                ui.label("Textures");
                ui.collapsing("Textures", |ui| {
                        egui::Grid::new("Textures").show(ui, |ui| {
                        let texture_option_attributes = OptionAttributes { replacement: Some(|| Handle::weak(HandleId::random::<Texture>())), ..Default::default() };

                        ui.label("base_color");
                        self.base_color_texture.ui(ui, texture_option_attributes.clone(), &context.with_id(0));
                        ui.end_row();

                        ui.label("normal_map");
                        self.normal_map.ui(ui, texture_option_attributes.clone(), &context.with_id(0));
                        ui.end_row();

                        ui.label("metallic_roughness");
                        self.metallic_roughness_texture.ui(ui, texture_option_attributes.clone(), &context.with_id(1));
                        ui.end_row();

                        ui.label("emmissive");
                        self.emissive_texture.ui(ui, texture_option_attributes.clone(), &context.with_id(2));
                        ui.end_row();

                        ui.label("occlusion texture");
                        self.occlusion_texture.ui(ui, texture_option_attributes.clone(), &context.with_id(3));
                        ui.end_row();
                    });
                });
                ui.end_row();
            });
        });
    }
}
