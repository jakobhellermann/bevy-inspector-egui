use bevy::{
    asset::HandleId,
    pbr::{
        AlphaMode, AmbientLight, Clusters, CubemapVisibleEntities, DirectionalLight, PointLight,
        StandardMaterial, VisiblePointLights,
    },
    prelude::{Color, Handle, Image},
    render::render_resource::Face,
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

                    ui.label("cull_mode");
                    egui::ComboBox::from_id_source("cull_mode")
                        .selected_text(format!("{:?}", self.cull_mode))
                        .show_ui(ui, |ui| {
                            changed |= ui.selectable_value(&mut self.cull_mode, None, "None").changed();
                            changed |= ui.selectable_value(&mut self.cull_mode, Some(Face::Front), "Front").changed();
                            changed |= ui.selectable_value(&mut self.cull_mode, Some(Face::Back), "Back").changed();
                        });
                    ui.end_row();

                    ui.label("flip_normal_map_y");
                    changed |= self.flip_normal_map_y.ui(ui, Default::default(), context);
                    ui.end_row();

                    ui.label("double_sided");
                    changed |= self.double_sided.ui(ui, Default::default(), context);
                    ui.end_row();

                    ui.label("alpha_mode");
                    egui::ComboBox::from_id_source("alpha_mode")
                        .selected_text(format!("{:?}", self.alpha_mode))
                        .show_ui(ui, |ui| {
                            changed |= ui.selectable_value(&mut self.alpha_mode, AlphaMode::Blend, "Blend").changed();
                            let alpha_mask = match self.alpha_mode {
                                AlphaMode::Mask(m) => m,
                                _ => 0.0
                            };
                            changed |= ui.selectable_value(&mut self.alpha_mode, AlphaMode::Mask(alpha_mask), "Mask").changed();
                            changed |= ui.selectable_value(&mut self.alpha_mode, AlphaMode::Opaque, "Opaque").changed();
                        });
                    ui.end_row();

                    if let AlphaMode::Mask(ref mut alpha_mask) = self.alpha_mode {
                        ui.label("alpha_mask");
                        changed |= alpha_mask.ui(ui, NumberAttributes::positive(), context);
                        ui.end_row();
                    }
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
