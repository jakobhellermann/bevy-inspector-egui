macro_rules! impl_for_simple_enum {
    ($name:ty: $($variant:ident),* ) => {
        impl $crate::Inspectable for $name {
            type Attributes = ();

            fn ui(&mut self, ui: &mut $crate::egui::Ui, _: Self::Attributes, context: &mut $crate::Context) -> bool {
                let mut changed = false;
                crate::egui::ComboBox::from_id_source(context.id())
                    .selected_text(format!("{:?}", self))
                    .show_ui(ui, |ui| {
                    $(
                        if ui.selectable_label(matches!(self, <$name>::$variant), format!("{:?}", <$name>::$variant)).clicked() {
                            *self = <$name>::$variant;
                            changed = true;
                        }
                    )*
                });
                changed
            }
        }
    }
}

macro_rules! display_struct_delegate_fields {
    ($val:ident $ui:ident $context:ident $ty:ty: $($field:ident $(inline $dummy:ident)? $(with $attrs:expr)? ),* $(,)?) => {{
        let ui = $ui;
        let context = $context;
        let val = $val;
        let mut changed = false;
        ui.vertical_centered(|ui| {
            $crate::egui::Grid::new(context.id()).show(ui, |ui| {
                let mut i = 0;
                $(

                    let mut show_label = true;
                    $(
                        show_label = false;
                        let $dummy = ();
                    )?

                    if show_label {
                        ui.label(stringify!($field));
                    }

                    let mut attrs = Default::default();
                    $(attrs = $attrs;)?
                    changed |= val.$field.ui(ui, attrs, &mut context.with_id(i));
                    ui.end_row();
                    i += 1;
                )*
            });
        });
        changed
    }}
}

macro_rules! impl_for_struct_delegate_fields {
    ($ty:ty: $($field:ident $(inline $dummy:ident)? $(with $attrs:expr)? ),* $(,)?) => {
        #[allow(unused)]
        impl $crate::Inspectable for $ty {
            type Attributes = ();

            fn ui(&mut self, ui: &mut $crate::egui::Ui, _: Self::Attributes, context: &mut $crate::Context) -> bool {
                display_struct_delegate_fields! { self ui context $ty: $($field $(inline $dummy)? $(with $attrs)? ),* }
            }
        }
    };
}

#[allow(unused)]
macro_rules! impl_for_bitflags {
    ($ty:ty: $($field:ident),* $(,)?) => {
        impl Inspectable for $ty {
            type Attributes = ();

            fn ui(
                &mut self,
                ui: &mut egui::Ui,
                _: Self::Attributes,
                _: &mut Context,
            ) -> bool {
                let mut changed = false;

                ui.vertical(|ui| {
                    $(
                    let mut value = self.contains(<$ty>::$field);
                    let has_changed = ui.checkbox(&mut value, stringify!($field)).changed();
                    if has_changed {
                        self.set(<$ty>::$field, value);
                        changed = true;
                    }
                    )*
                });

                changed
            }
        }
    };
}

#[allow(unused)]
macro_rules! impl_defer_to {
    ($ty:ty: $inner:tt ) => {
        impl $crate::Inspectable for $ty {
            type Attributes = ();

            fn ui(
                &mut self,
                ui: &mut egui::Ui,
                (): Self::Attributes,
                context: &mut Context,
            ) -> bool {
                self.$inner.ui(ui, Default::default(), context)
            }
        }
    };
}
