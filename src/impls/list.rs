use bevy::prelude::AppBuilder;

use crate::Inspectable;
use crate::{egui, Context};

impl<T> Inspectable for Vec<T>
where
    T: Inspectable + Default,
{
    type Attributes = <T as Inspectable>::Attributes;

    fn ui(&mut self, ui: &mut egui::Ui, options: Self::Attributes, context: &Context) {
        ui.vertical(|ui| {
            let mut to_delete = None;

            for (i, val) in self.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    ui.label(i.to_string());
                    val.ui(ui, options.clone(), &context.with_id(i as u64));
                    if ui.button("-").clicked() {
                        to_delete = Some(i);
                    }
                });
            }

            ui.vertical_centered_justified(|ui| {
                if ui.button("+").clicked() {
                    self.push(T::default());
                }
            });

            if let Some(i) = to_delete {
                self.remove(i);
            }
        });
    }

    fn setup(app: &mut AppBuilder) {
        T::setup(app);
    }
}

impl<T: Inspectable, const N: usize> Inspectable for [T; N] {
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

    fn setup(app: &mut AppBuilder) {
        T::setup(app);
    }
}

macro_rules! impl_for_tuple {
    ( $($ty:ident : $i:tt),* ) => {
        #[allow(unused_variables, non_snake_case)]
        impl<$($ty: Inspectable),*> Inspectable for ($($ty,)*) {
            type Attributes = ($(<$ty as Inspectable>::Attributes,)*);

            fn ui(&mut self, ui: &mut egui::Ui, options: Self::Attributes, context: &Context) {
                ui.horizontal(|ui| {
                    let ($($ty,)*) = options;

                    ui.label("(");
                    $(self.$i.ui(ui, $ty, &context.with_id($i));)*
                    ui.label(")");
                });
            }

            fn setup(app: &mut AppBuilder) {
                $($ty::setup(app);)*
            }
        }
    };
}

impl_for_tuple!();
impl_for_tuple!(A:0);
impl_for_tuple!(A:0, B:1);
impl_for_tuple!(A:0, B:1, C:2);
impl_for_tuple!(A:0, B:1, C:2, D:3);
