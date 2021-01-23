use crate::{Context, InspectableWithContext};
use bevy::{
    asset::{Asset, HandleId},
    prelude::*,
};
use bevy_egui::egui;

impl<T: Asset + InspectableWithContext> InspectableWithContext for Handle<T> {
    type Attributes = T::Attributes;

    fn ui_with_context(&mut self, ui: &mut egui::Ui, options: Self::Attributes, context: &Context) {
        if self.id == HandleId::default::<T>() {
            ui.label("<default handle>");
            return;
        }

        let mut assets = match context.resources.get_mut::<Assets<T>>() {
            Some(assets) => assets,
            None => {
                let msg = format!("No Assets resource for {}", std::any::type_name::<T>());
                ui.label(msg);
                return;
            }
        };
        let value = match assets.get_mut(self.clone()) {
            Some(val) => val,
            None => {
                let msg = format!("No value for handle {:?}", self);
                ui.label(msg);
                return;
            }
        };

        value.ui_with_context(ui, options, context);
    }
}
