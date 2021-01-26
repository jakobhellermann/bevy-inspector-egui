use std::ops::{Deref, DerefMut};

use bevy::prelude::*;
use bevy::reflect::{List, Map, Tuple};
use bevy_egui::egui;
use egui::Grid;

use crate::{Context, Inspectable, InspectableRegistry};

/// Wrapper type for displaying inspector UI based on the types [`Reflect`](bevy::reflect::Reflect) implementation.
///
/// Say you wanted to display a type defined in another crate in the inspector, and that type implements `Reflect`.
/// ```rust
/// # use bevy::prelude::*;
/// #[derive(Reflect, Default)]
/// struct SomeComponent {
///     a: f32,
///     b: Vec2,
/// }
/// ```
///
/// Using the `ReflectedUI` wrapper type, you can include it in your inspector
/// and edit the fields like you would expect:
///
/// ```rust
/// # use bevy::prelude::*;
/// # use bevy_inspector_egui::{Inspectable, reflect::ReflectedUI};
/// # #[derive(Reflect, Default)] struct SomeComponent;
/// #[derive(Inspectable, Default)]
/// struct Data {
///     component: ReflectedUI<SomeComponent>,
///     // it also works for bevy's types
///     timer: ReflectedUI<Timer>,
/// }
/// ```
#[derive(Debug, Default)]
pub struct ReflectedUI<T>(T);
impl<T> ReflectedUI<T> {
    #[allow(missing_docs)]
    pub fn new(val: T) -> Self {
        ReflectedUI(val)
    }
}

impl<T> Deref for ReflectedUI<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T> DerefMut for ReflectedUI<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: Reflect> Inspectable for ReflectedUI<T> {
    type Attributes = ();

    fn ui(&mut self, ui: &mut egui::Ui, _: Self::Attributes, context: &Context) {
        ui_for_reflect(&mut self.0, ui, context);
    }
}

macro_rules! try_downcast_ui {
    ($value:ident $ui:ident $context:ident => $ty:ty) => {
        if let Some(v) = $value.downcast_mut::<$ty>() {
            <$ty as Inspectable>::ui(v, $ui, <$ty as Inspectable>::Attributes::default(), $context);
            return;
        }
    };

    ( $value:ident $ui:ident $context:ident => $( $ty:ty ),+ $(,)? ) => {
        $(try_downcast_ui!($value $ui $context => $ty);)*
    };
}

/// Draws the inspector UI for the given value.
///
/// This function gets used for the implementation of [`Inspectable`](crate::Inspectable)
/// for [`ReflectedUI`](ReflectedUI).
pub fn ui_for_reflect(value: &mut dyn Reflect, ui: &mut egui::Ui, context: &Context) {
    if let Some(resources) = context.resources {
        if let Some(inspect_registry) = resources.get::<InspectableRegistry>() {
            if inspect_registry.try_execute(value, ui, resources) {
                return;
            }
        }
    }

    try_downcast_ui!(value ui context => Color);
    try_downcast_ui!(value ui context => Handle<Texture>, Handle<StandardMaterial>);

    match value.reflect_mut() {
        bevy::reflect::ReflectMut::Struct(s) => ui_for_reflect_struct(s, ui, context),
        bevy::reflect::ReflectMut::TupleStruct(value) => ui_for_tuple_struct(value, ui, context),
        bevy::reflect::ReflectMut::Tuple(value) => ui_for_tuple(value, ui, context),
        bevy::reflect::ReflectMut::List(value) => ui_for_list(value, ui),
        bevy::reflect::ReflectMut::Map(value) => ui_for_map(value, ui),
        bevy::reflect::ReflectMut::Value(value) => ui_for_reflect_value(value, ui, context),
    }
}

fn ui_for_reflect_struct(value: &mut dyn Struct, ui: &mut egui::Ui, context: &Context) {
    ui.vertical_centered(|ui| {
        let grid = Grid::new(value.type_id());
        grid.show(ui, |ui| {
            for i in 0..value.field_len() {
                match value.name_at(i) {
                    Some(name) => ui.label(name),
                    None => ui.label("<missing>"),
                };
                if let Some(field) = value.field_at_mut(i) {
                    ui_for_reflect(field, ui, context);
                } else {
                    ui.label("<missing>");
                }
                ui.end_row();
            }
        });
    });
}

fn ui_for_tuple_struct(value: &mut dyn TupleStruct, ui: &mut egui::Ui, context: &Context) {
    let grid = Grid::new(value.type_id());
    grid.show(ui, |ui| {
        for i in 0..value.field_len() {
            ui.label(i.to_string());
            if let Some(field) = value.field_mut(i) {
                ui_for_reflect(field, ui, context);
            } else {
                ui.label("<missing>");
            }
            ui.end_row();
        }
    });
}

fn ui_for_tuple(value: &mut dyn Tuple, ui: &mut egui::Ui, context: &Context) {
    let grid = Grid::new(value.type_id());
    grid.show(ui, |ui| {
        for i in 0..value.field_len() {
            ui.label(i.to_string());
            if let Some(field) = value.field_mut(i) {
                ui_for_reflect(field, ui, context);
            } else {
                ui.label("<missing>");
            }
            ui.end_row();
        }
    });
}

fn ui_for_list(_value: &mut dyn List, ui: &mut egui::Ui) {
    ui.label("List not yet implemented");
}

fn ui_for_map(_value: &mut dyn Map, ui: &mut egui::Ui) {
    ui.label("Map not yet implemented");
}

fn ui_for_reflect_value(value: &mut dyn Reflect, ui: &mut egui::Ui, context: &Context) {
    try_downcast_ui!(
        value ui context =>
        f32, f64, u8, u16, u32, u64, i8, i16, i32, i64,
        String, bool,
        Vec2, Vec3, Vec4, Mat3, Mat4,
    );

    try_downcast_ui!(value ui context => Option<String>);

    ui.label(format!("Not implemented: {}", value.type_name()));
}
