use crate::Inspectable;
use crate::{egui, Context};

impl<T> Inspectable for Vec<T>
where
    T: Inspectable + Default,
    T::Attributes: Clone,
{
    type Attributes = <T as Inspectable>::Attributes;

    fn ui(&mut self, ui: &mut egui::Ui, options: Self::Attributes, context: &Context) {
        ui.vertical(|ui| {
            let mut to_delete = None;

            for (i, val) in self.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    ui.label(i.to_string());
                    val.ui(ui, options.clone(), &context.with_id(i as u64));
                    if ui.button("-").clicked {
                        to_delete = Some(i);
                    }
                });
            }

            ui.vertical_centered_justified(|ui| {
                if ui.button("+").clicked {
                    self.push(T::default());
                }
            });

            if let Some(i) = to_delete {
                self.remove(i);
            }
        });
    }
}

#[cfg(feature = "nightly")]
impl<T: Inspectable, const N: usize> Inspectable for [T; N]
where
    T::Attributes: Clone,
{
    type Attributes = <T as Inspectable>::Attributes;

    fn ui(&mut self, ui: &mut egui::Ui, options: Self::Attributes, context: &Context) {
        ui.vertical(|ui| {
            for (i, val) in self.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    ui.label(i.to_string());
                    val.ui(ui, options.clone(), &context.with_id(i as u64));
                });
            }
        });
    }
}
