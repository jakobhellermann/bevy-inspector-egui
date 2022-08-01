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
/// # use bevy_inspector_egui::{Inspectable, widgets::ReflectedUI};
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

    fn ui(&mut self, ui: &mut egui::Ui, _: Self::Attributes, context: &mut Context) -> bool {
        ui_for_reflect(&mut self.0, ui, context)
    }
}

/// Draws the inspector UI for the given value.
///
/// This function gets used for the implementation of [`Inspectable`](crate::Inspectable)
/// for [`ReflectedUI`](ReflectedUI).
pub fn ui_for_reflect(value: &mut dyn Reflect, ui: &mut egui::Ui, context: &mut Context) -> bool {
    if let Some((cx, world)) = context.take_world() {
        if world.contains_resource::<InspectableRegistry>() {
            let changed =
                world.resource_scope(|world, inspectable_registry: Mut<InspectableRegistry>| {
                    let mut context = cx.with_world(world);
                    ui_for_reflect_with_registry(
                        value,
                        ui,
                        &mut context,
                        Some(&inspectable_registry),
                    )
                });
            return changed;
        }
        context.world = Some(world);
    }

    ui_for_reflect_with_registry(value, ui, context, None)
}

/// Same as `ui_for_reflect` but explicitly passes the `InspectableRegistry` instead of retrieving
/// it from the `Context`.
pub fn ui_for_reflect_with_registry(
    value: &mut dyn Reflect,
    ui: &mut egui::Ui,
    context: &mut Context,
    inspectable_registry: Option<&InspectableRegistry>,
) -> bool {
    if let Some(inspectable_registry) = inspectable_registry {
        if let Ok(changed) = inspectable_registry.try_execute(value.as_any_mut(), ui, context) {
            return changed;
        }
    }

    match value.reflect_mut() {
        bevy::reflect::ReflectMut::Struct(s) => {
            ui_for_reflect_struct(s, ui, context, inspectable_registry)
        }
        bevy::reflect::ReflectMut::TupleStruct(value) => {
            ui_for_tuple_struct(value, ui, context, inspectable_registry)
        }
        bevy::reflect::ReflectMut::Tuple(value) => {
            ui_for_tuple(value, ui, context, inspectable_registry)
        }
        bevy::reflect::ReflectMut::List(value) => {
            ui_for_list(value, ui, context, inspectable_registry)
        }
        bevy::reflect::ReflectMut::Map(value) => ui_for_map(value, ui),
        bevy::reflect::ReflectMut::Value(value) => {
            ui_for_reflect_value(value, ui, inspectable_registry)
        }
        bevy::reflect::ReflectMut::Array(_) => todo!(),
    }
}

fn ui_for_reflect_struct(
    value: &mut dyn Struct,
    ui: &mut egui::Ui,
    context: &mut Context,
    inspectable_registry: Option<&InspectableRegistry>,
) -> bool {
    let mut changed = false;
    ui.vertical_centered(|ui| {
        let grid = Grid::new(value.type_id());
        grid.show(ui, |ui| {
            for i in 0..value.field_len() {
                match value.name_at(i) {
                    Some(name) => ui.label(name),
                    None => ui.label("<missing>"),
                };
                if let Some(field) = value.field_at_mut(i) {
                    changed |= ui_for_reflect_with_registry(
                        field,
                        ui,
                        &mut context.with_id(i as u64),
                        inspectable_registry,
                    );
                } else {
                    ui.label("<missing>");
                }
                ui.end_row();
            }
        });
    });
    changed
}

fn ui_for_tuple_struct(
    value: &mut dyn TupleStruct,
    ui: &mut egui::Ui,
    context: &mut Context,
    inspectable_registry: Option<&InspectableRegistry>,
) -> bool {
    let mut changed = false;
    let grid = Grid::new(value.type_id());
    grid.show(ui, |ui| {
        for i in 0..value.field_len() {
            ui.label(i.to_string());
            if let Some(field) = value.field_mut(i) {
                changed |= ui_for_reflect_with_registry(
                    field,
                    ui,
                    &mut context.with_id(i as u64),
                    inspectable_registry,
                );
            } else {
                ui.label("<missing>");
            }
            ui.end_row();
        }
    });
    changed
}

fn ui_for_tuple(
    value: &mut dyn Tuple,
    ui: &mut egui::Ui,
    context: &mut Context,
    inspectable_registry: Option<&InspectableRegistry>,
) -> bool {
    let mut changed = false;
    let grid = Grid::new(value.type_id());
    grid.show(ui, |ui| {
        for i in 0..value.field_len() {
            ui.label(i.to_string());
            if let Some(field) = value.field_mut(i) {
                changed |= ui_for_reflect_with_registry(
                    field,
                    ui,
                    &mut context.with_id(i as u64),
                    inspectable_registry,
                );
            } else {
                ui.label("<missing>");
            }
            ui.end_row();
        }
    });
    changed
}

fn ui_for_list(
    list: &mut dyn List,
    ui: &mut egui::Ui,
    context: &mut Context,
    inspectable_registry: Option<&InspectableRegistry>,
) -> bool {
    let mut changed = false;

    ui.vertical(|ui| {
        // let mut to_delete = None;

        let len = list.len();
        for i in 0..len {
            let val = list.get_mut(i).unwrap();
            ui.horizontal(|ui| {
                /*if utils::ui::label_button(ui, "✖", egui::Color32::RED) {
                    to_delete = Some(i);
                }*/
                changed |= ui_for_reflect_with_registry(
                    val,
                    ui,
                    &mut context.with_id(i as u64),
                    inspectable_registry,
                );
            });

            if i != len - 1 {
                ui.separator();
            }
        }

        if len > 0 {
            ui.vertical_centered_justified(|ui| {
                if ui.button("+").clicked() {
                    let last_element = list.get(len - 1).unwrap().clone_value();
                    list.push(last_element);

                    changed = true;
                }
            });
        }

        /*if let Some(_) = to_delete {
            changed = true;
        }*/
    });

    changed
}

fn ui_for_map(_value: &mut dyn Map, ui: &mut egui::Ui) -> bool {
    ui.label("Map not yet implemented");
    false
}

fn ui_for_reflect_value(
    value: &mut dyn Reflect,
    ui: &mut egui::Ui,
    inspectable_registry: Option<&InspectableRegistry>,
) -> bool {
    if inspectable_registry.is_some() {
        ui.label(format!(
            "{} is `#[reflect_value()]`, but not registered on the `InspectableRegistry`.",
            value.type_name()
        ));
    } else {
        ui.label(format!(
            "{} (InspectableRegistry not accessible)",
            value.type_name()
        ));
    }

    false
}
