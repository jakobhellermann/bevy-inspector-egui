use bevy::{
    asset::HandleId,
    pbr::{
        AmbientLight, Clusters, CubemapVisibleEntities, DirectionalLight, PointLight,
        StandardMaterial, VisiblePointLights,
    },
    prelude::{Color, Handle, Image},
};
use bevy_egui::egui;

use crate::{Context, Inspectable};

use super::{NumberAttributes, OptionAttributes};

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

impl Inspectable for AmbientLight {
    type Attributes = <Color as Inspectable>::Attributes;

    fn ui(&mut self, ui: &mut egui::Ui, options: Self::Attributes, context: &mut Context) -> bool {
        let brightness_attributes = NumberAttributes::positive().with_speed(0.01);

        self.color.ui(ui, options, context);
        self.brightness.ui(ui, brightness_attributes, context)
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

impl Inspectable for VisiblePointLights {
    type Attributes = ();

    fn ui(&mut self, ui: &mut egui::Ui, _: Self::Attributes, _: &mut Context) -> bool {
        let len = self.len();
        let entity = match len {
            1 => "light",
            _ => "lights",
        };
        ui.label(format!("{} visible point {}", len, entity));
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

impl Inspectable for Clusters {
    type Attributes = ();

    fn ui(&mut self, _: &mut egui::Ui, _: Self::Attributes, _: &mut Context) -> bool {
        false
    }
}
