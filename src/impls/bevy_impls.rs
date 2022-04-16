use crate::options::NumberAttributes;
use crate::{Context, Inspectable};
use bevy::ecs::system::Resource;
use bevy::prelude::*;

use bevy_egui::egui;
use egui::Grid;

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

impl Inspectable for Name {
    type Attributes = ();

    fn ui(&mut self, ui: &mut egui::Ui, _: Self::Attributes, _: &mut Context) -> bool {
        ui.label(self.as_str());
        false
    }
}

impl<'a, T: Inspectable> Inspectable for Mut<'a, T> {
    type Attributes = T::Attributes;

    fn ui(&mut self, ui: &mut egui::Ui, options: Self::Attributes, context: &mut Context) -> bool {
        (**self).ui(ui, options, context)
    }
}

impl<'a, T: Resource + Inspectable> Inspectable for ResMut<'a, T> {
    type Attributes = T::Attributes;

    fn ui(&mut self, ui: &mut egui::Ui, options: Self::Attributes, context: &mut Context) -> bool {
        (**self).ui(ui, options, context)
    }
}

impl<'a, T: Inspectable> Inspectable for NonSendMut<'a, T> {
    type Attributes = T::Attributes;

    fn ui(&mut self, ui: &mut egui::Ui, options: Self::Attributes, context: &mut Context) -> bool {
        (**self).ui(ui, options, context)
    }
}
