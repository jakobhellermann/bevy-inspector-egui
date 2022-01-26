#[macro_export]
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
            changed |= grid!(@field ui context field; $(by $ui_fn)? $(with $attrs)?);
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

#[macro_export]
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

#[macro_export]
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

#[macro_export]
macro_rules! inspectable {
    (grid $name:ident $ty:ty as $wrapper:ty: $($tt:tt)*) => {
        fn $name(val: &mut $wrapper, ui: &mut egui::Ui, context: &mut Context<'_>) -> bool {
            grid!(ui context &mut val.0; $($tt)*)
        }
    };
    (flags $name:ident $ty:ty as $wrapper:ty: $($tt:tt)*) => {
        fn $name(val: &mut $wrapper, ui: &mut egui::Ui, context: &mut Context<'_>) -> bool {
            flags!(ui context &mut val.0; $ty: $($tt)*)
        }
    };
    (enum $name:ident $ty:ty as $wrapper:ty: $($tt:tt)*) => {
        fn $name(val: &mut $wrapper, ui: &mut egui::Ui, context: &mut Context<'_>) -> bool {
            simple_enum!(ui context &mut val.0; $ty: $($tt)*)
        }
    };
    (defer $name:ident $ty:ty as $wrapper:ty: $val:tt) => {
        fn $name(val: &mut $wrapper, ui: &mut egui::Ui, context: &mut Context<'_>) -> bool {
            val.0.$val.ui(ui, Default::default(), context)
        }
    }
}
