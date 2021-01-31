pub mod ui;

use bevy_egui::egui;
use egui::{Color32, Label};

const ERROR_COLOR: Color32 = Color32::from_rgb(255, 0, 0);

pub(crate) fn error_label(ui: &mut egui::Ui, msg: impl Into<Label>) {
    ui.colored_label(ERROR_COLOR, msg);
}

pub(crate) fn short_name(type_name: &str) -> String {
    match type_name.find('<') {
        // no generics
        None => type_name.rsplit("::").next().unwrap_or(type_name).into(),
        // generics a::b::c<d>
        Some(angle_open) => {
            let angle_close = type_name.rfind('>').unwrap();

            let before_generics = &type_name[..angle_open];
            let after = &type_name[angle_close + 1..];
            let in_between = &type_name[angle_open + 1..angle_close];

            let before_generics = match before_generics.rfind("::") {
                None => before_generics,
                Some(i) => &before_generics[i + 2..],
            };

            let in_between = short_name(in_between);

            format!("{}<{}>{}", before_generics, in_between, after)
        }
    }
}

#[allow(unused)]
macro_rules! impl_for_simple_enum {
    ($name:ident with $($variant:ident),* ) => {
        impl $crate::Inspectable for $name {
            type Attributes = ();


            fn ui(&mut self, ui: &mut $crate::egui::Ui, _: Self::Attributes, _: &$crate::Context) {
                use $crate::egui;

                let id = ui.make_persistent_id(stringify!(#id));
                egui::combo_box(ui, id, format!("{:?}", self), |ui| {
                    $(
                        ui.selectable_value(self, $name::$variant, format!("{:?}", $name::$variant));
                    )*
                });
            }
        }
    }
}

macro_rules! impl_for_struct_delegate_fields {
    ($ty:ty: $($field:ident),+ $(,)?) => {
        impl $crate::Inspectable for $ty {
            type Attributes = ();

            fn ui(&mut self, ui: &mut $crate::egui::Ui, _: Self::Attributes, context: &$crate::Context) {
                let id = std::any::TypeId::of::<$ty>();
                ui.vertical_centered(|ui| {
                    $crate::egui::Grid::new(id).show(ui, |ui| {
                        $(
                            ui.label(stringify!($field));
                            self.$field.ui(ui, Default::default(), context);
                            ui.end_row();
                        )*
                    });
                });
            }
        }
    };
}

macro_rules! expect_context {
    ($ui:ident, $context:ident.$field:ident, $ty:literal) => {
        match $context.$field {
            Some(val) => val,
            None => {
                let msg = format!(
                    "'{}' needs exclusive access to the {}",
                    $ty,
                    stringify!($field)
                );
                return $crate::utils::error_label($ui, msg);
            }
        }
    };
}
macro_rules! expect_resource {
    ($ui:ident, $resources:ident, $method:ident $ty:ty) => {
        match $resources.$method::<$ty>() {
            Some(res) => res,
            None => {
                let msg = format!("No {} resource found", std::any::type_name::<$ty>());
                return $crate::utils::error_label($ui, msg);
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::short_name;

    #[test]
    fn shorten_name_basic() {
        assert_eq!(short_name("path::to::some::Type"), "Type".to_string());
    }
    #[test]
    fn shorten_name_generic() {
        assert_eq!(
            short_name("bevy::ecs::Handle<bevy::render::StandardMaterial>"),
            "Handle<StandardMaterial>".to_string()
        );
    }
    #[test]
    fn shorten_name_nested_generic() {
        assert_eq!(
            short_name("foo::bar::quux<qaax<p::t::b>>"),
            "quux<qaax<b>>".to_string()
        );
    }
}
