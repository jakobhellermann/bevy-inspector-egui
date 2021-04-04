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
        impl<$($ty: Inspectable + 'static),*> Inspectable for ($($ty,)*) {
            type Attributes = ($(<$ty as Inspectable>::Attributes,)*);

            fn ui(&mut self, ui: &mut egui::Ui, options: Self::Attributes, context: &Context) {
                #[allow(unused_mut)]
                let mut inline = true;
                $(inline &= should_display_inline::<$ty>();)*


                if inline {
                    ui.horizontal(|ui| {
                        let ($($ty,)*) = options;
                        ui.label("(");
                        $(self.$i.ui(ui, $ty, &context.with_id($i));)*
                        ui.label(")");
                    });
                } else {
                    let ($($ty,)*) = options;

                    ui.vertical(|ui| {
                        $(
                            if $i != 0 {
                                ui.separator();
                            }
                            self.$i.ui(ui, $ty, &context.with_id($i));
                        )*
                    });
                }

            }

            fn setup(app: &mut AppBuilder) {
                $($ty::setup(app);)*
            }
        }
    };
}

macro_rules! matches_ty {
    ($ty:ty, $($types:ty)|+) => {{
        let type_id = std::any::TypeId::of::<$ty>();
        $(
            type_id == std::any::TypeId::of::<$types>()
        )||*
    }};
}
fn should_display_inline<T: 'static>() -> bool {
    matches_ty!(
        T,
        i8 | i16 | i32 | i64 | isize | u8 | u16 | u32 | u64 | usize | char | bool | String | &'static str | f32 | f64 | &'static str
    )
}

impl_for_tuple!();
impl_for_tuple!(A:0);
impl_for_tuple!(A:0, B:1);
impl_for_tuple!(A:0, B:1, C:2);
impl_for_tuple!(A:0, B:1, C:2, D:3);
