use std::collections::HashMap;
use std::hash::Hash;

use bevy::prelude::App;

use crate::{egui, Context};
use crate::{utils, Inspectable};

impl<K, V> Inspectable for HashMap<K, V>
where
    K: Inspectable + Clone + Eq + Hash + Default,
    V: Inspectable + Default,
{
    type Attributes = (
        <K as Inspectable>::Attributes,
        <V as Inspectable>::Attributes,
    );

    fn ui(&mut self, ui: &mut egui::Ui, options: Self::Attributes, context: &mut Context) -> bool {
        let mut changed = false;

        ui.vertical(|ui| {
            let mut to_delete = None;
            let mut to_update = Vec::new();

            let len = self.len();
            for (i, (key, val)) in self.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    if utils::ui::label_button(ui, "✖", egui::Color32::RED) {
                        to_delete = Some(key.clone());
                    }

                    let mut k = key.clone();
                    if k.ui(ui, options.0.clone(), &mut context.with_id(i as u64)) {
                        to_update.push((key.clone(), k));
                    }

                    changed |= val.ui(ui, options.1.clone(), &mut context.with_id(i as u64));
                });

                if i != len - 1 {
                    ui.separator();
                }
            }

            ui.vertical_centered_justified(|ui| {
                if ui.button("+").clicked() {
                    self.insert(K::default(), V::default());
                    changed = true;
                }
            });

            for (old_key, new_key) in to_update.drain(..) {
                if let Some(val) = self.remove(&old_key) {
                    self.insert(new_key, val);
                    changed = true;
                }
            }

            if let Some(key) = to_delete {
                if self.remove(&key).is_some() {
                    changed = true;
                }
            }
        });

        changed
    }

    fn setup(app: &mut App) {
        K::setup(app);
        V::setup(app);
    }
}
