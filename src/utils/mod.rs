pub mod ui;

use bevy_egui::egui;
use egui::{Color32, Label};

const ERROR_COLOR: Color32 = Color32::from_rgb(255, 0, 0);

pub(crate) fn error_label(ui: &mut egui::Ui, msg: impl Into<Label>) {
    ui.colored_label(ERROR_COLOR, msg);
}

pub(crate) fn short_name(type_name: &str) -> String {
    // handle tuple structs separately
    if let Some(inner) = type_name
        .strip_prefix('(')
        .and_then(|name| name.strip_suffix(')'))
    {
        let ty = inner
            .split(", ")
            .map(short_name)
            .collect::<Vec<_>>()
            .join(", ");
        return format!("({})", ty);
    }

    match find_group_in_string(type_name, '<', '>') {
        Ok((before, in_between, after)) => {
            let before = before.rsplit("::").next().unwrap_or(before);
            format!("{}<{}>{}", before, short_name(in_between), after)
        }
        Err(_) => type_name.rsplit("::").next().unwrap_or(type_name).into(),
    }
}

fn find_group_in_string<'a>(
    input: &'a str,
    left_terminator: char,
    right_terminator: char,
) -> Result<(&'a str, &'a str, &'a str), &'a str> {
    match input.find(left_terminator) {
        None => Err(input),
        Some(start) => {
            let end = input.rfind(right_terminator).unwrap();

            let before = &input[..start];
            let after = &input[end + 1..];
            let in_between = &input[start + 1..end];

            Ok((before, in_between, after))
        }
    }
}

#[allow(unused)]
macro_rules! impl_for_simple_enum {
    ($name:ty: $($variant:ident),* ) => {
        impl $crate::Inspectable for $name {
            type Attributes = ();


            fn ui(&mut self, ui: &mut $crate::egui::Ui, _: Self::Attributes, context: &$crate::Context) {
                use $crate::egui;

                egui::combo_box(ui, context.id(), format!("{:?}", self), |ui| {
                    $(
                        ui.selectable_value(self, <$name>::$variant, format!("{:?}", <$name>::$variant));
                    )*
                });
            }
        }
    }
}

macro_rules! impl_for_struct_delegate_fields {
    ($ty:ty: $($field:ident $(with $attrs:expr)? ),+ $(,)?) => {
        #[allow(unused)]
        impl $crate::Inspectable for $ty {
            type Attributes = ();

            fn ui(&mut self, ui: &mut $crate::egui::Ui, _: Self::Attributes, context: &$crate::Context) {
                ui.vertical_centered(|ui| {
                    $crate::egui::Grid::new(context.id()).show(ui, |ui| {
                        let mut i = 0;
                        $(
                            ui.label(stringify!($field));
                            let mut attrs = Default::default();
                            $(attrs = $attrs;)?
                            self.$field.ui(ui, attrs, &context.with_id(i));
                            ui.end_row();
                            i += 1;
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

    #[test]
    fn tuple() {
        assert_eq!(short_name("(x::a, x::b)"), "(a, b)".to_string());
    }

    #[test]
    fn complex_name() {
        assert_eq!(
            short_name("bevy_inspector_egui::world_inspector::impls::InspectorQuery<(bevy_ecs::core::filter::With<bevy_ui::node::Node>, bevy_ecs::core::filter::Without<bevy_transform::components::parent::Parent>)>"),
            "InspectorQuery<(With<Node>, Without<Parent>)>".to_string());
    }
}
