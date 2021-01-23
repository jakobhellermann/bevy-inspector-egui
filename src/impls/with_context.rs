use crate::{utils, Context, Inspectable};
use bevy::{
    asset::{Asset, HandleId},
    prelude::*,
};
use bevy_egui::egui;

impl<T: Asset + Inspectable> Inspectable for Handle<T> {
    type Attributes = T::Attributes;

    fn ui(&mut self, ui: &mut egui::Ui, options: Self::Attributes, context: &Context) {
        if self.id == HandleId::default::<T>() {
            ui.label("<default handle>");
            return;
        }

        let resources = match context.resources {
            Some(resources) => resources,
            None => {
                let msg = "Handle<T> needs to get run via InspectorPlugin::thread_local";
                return utils::error_label(ui, msg);
            }
        };

        let mut assets = match resources.get_mut::<Assets<T>>() {
            Some(assets) => assets,
            None => {
                let msg = format!("No Assets resource for {}", std::any::type_name::<T>());
                return utils::error_label(ui, msg);
            }
        };
        let value = match assets.get_mut(self.clone()) {
            Some(val) => val,
            None => {
                return utils::error_label(ui, format!("No value for handle {:?}", self));
            }
        };

        value.ui(ui, options, context);
    }
}
