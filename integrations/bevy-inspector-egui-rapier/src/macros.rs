#![allow(unused)]

macro_rules! grid {
    ($ui:ident $context:ident $val:expr; $($field:ident $(by $ui_fn:ident)? $(with $attrs:expr)?),*) => {{
        let val = $val;
        let mut changed = false;

        #[allow(unused_mut, unused_assignments)]
        egui::Grid::new("collision groups").show($ui, |ui| {
            let mut i = 0;
            $(
            ui.label(stringify!($field));
            let field = &mut val.$field;
            let context = &mut $context.with_id(i);
            changed |= $crate::macros::grid!(@field ui context field; $(by $ui_fn)? $(with $attrs)?);
            ui.end_row();
            i += 1;
            )*
        });

        changed
    }};

    (@field $ui:ident $context:ident $field:ident; by $ui_fn:ident) => {
        $ui_fn($field, $ui, $context)
    };

    (@field $ui:ident $context:ident $field:ident; $(with $attrs:expr)?) => {{
        let mut attrs = Default::default();
        $(attrs = $attrs;)?
        $field.ui($ui, attrs, $context)
    }};
}

macro_rules! flags {
    ($ui:ident $context:ident $val:expr; $ty:ty: $($flag:ident)|*) => {{
        let val = $val;
        let _ = $context;
        let mut changed = true;
        $(
            #[allow(non_snake_case)]
            let mut $flag = val.contains(<$ty>::$flag);
            if $ui.checkbox(&mut $flag, stringify!($flag)).clicked() {
                val.toggle(<$ty>::$flag);
                changed = true;
            }
        )*
        changed
    }};
}

macro_rules! simple_enum {
    ($ui:ident $context:ident $val:expr; $ty:ty : $($variant:ident)|*) => {{
        let val = $val;
        let mut changed = false;

        egui::ComboBox::from_id_source($context.id())
            .selected_text(format!("{val:?}"))
            .show_ui($ui, |ui| {
                $(changed |= ui.selectable_value(val, <$ty>::$variant, stringify!($variant)).changed();)*
            });

        changed
    }};
}

macro_rules! enum_one {
    ($ui:ident $context:ident $val:ident; $ty:ident: $($variant:ident {$attributes:expr})|*) => {{
        let selected = match $val {
            $($ty::$variant(_) => stringify!($variant),)*
        };
        ComboBox::from_id_source($context.id())
            .selected_text(selected)
            .show_ui($ui, |ui| {
                $(if ui
                    .selectable_label(matches!($val, $ty::$variant(_)), stringify!($variant))
                    .clicked()
                {
                    *$val = $ty::$variant(Default::default());
                })*
            });
        match $val {
            $($ty::$variant(val) => val.ui($ui, $attributes, $context),)*
        }
    }};
}

macro_rules! inspectable {
    (grid $name:ident $ty:ty: $($tt:tt)*) => {
        fn $name(val: &mut $ty, ui: &mut egui::Ui, context: &mut Context<'_>) -> bool {
            $crate::macros::grid!(ui context val; $($tt)*)
        }
    };
    (flags $name:ident $ty:ty: $($tt:tt)*) => {
        fn $name(val: &mut $ty, ui: &mut egui::Ui, context: &mut Context<'_>) -> bool {
            $crate::macros::flags!(ui context val; $ty: $($tt)*)
        }
    };
    (enum $name:ident $ty:ty: $($tt:tt)*) => {
        fn $name(val: &mut $ty, ui: &mut egui::Ui, context: &mut Context<'_>) -> bool {
            $crate::macros::simple_enum!(ui context val; $ty: $($tt)*)
        }
    };
    (enum_one $name:ident $ty:ident: $($tt:tt)*) => {
        fn $name(val: &mut $ty, ui: &mut egui::Ui, context: &mut Context<'_>) -> bool {
            $crate::macros::enum_one!(ui context val; $ty: $($tt)*)
        }
    };
    (defer $name:ident $ty:ty: $val:tt) => {
        fn $name(val: &mut $ty, ui: &mut egui::Ui, context: &mut Context<'_>) -> bool {
            val.$val.ui(ui, Default::default(), context)
        }
    }
}

pub(crate) use {enum_one, flags, grid, inspectable, simple_enum};
