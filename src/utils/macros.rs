macro_rules! impl_for_simple_enum {
    ($name:ty: $($variant:ident),* ) => {
        impl $crate::Inspectable for $name {
            type Attributes = ();

            fn ui(&mut self, ui: &mut $crate::egui::Ui, _: Self::Attributes, context: &$crate::Context) -> bool {
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

macro_rules! impl_for_struct_delegate_fields {
    ($ty:ty: $($field:ident $(with $attrs:expr)? ),+ $(,)?) => {
        #[allow(unused)]
        impl $crate::Inspectable for $ty {
            type Attributes = ();

            fn ui(&mut self, ui: &mut $crate::egui::Ui, _: Self::Attributes, context: &$crate::Context) -> bool {
                let mut changed = false;

                ui.vertical_centered(|ui| {
                    $crate::egui::Grid::new(context.id()).show(ui, |ui| {
                        let mut i = 0;
                        $(
                            ui.label(stringify!($field));
                            let mut attrs = Default::default();
                            $(attrs = $attrs;)?
                            changed |= self.$field.ui(ui, attrs, &context.with_id(i));
                            ui.end_row();
                            i += 1;
                        )*
                    });
                });

                changed
            }
        }
    };
}
