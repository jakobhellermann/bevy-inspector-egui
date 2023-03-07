//! General-purpose machinery for displaying [`Reflect`](bevy_reflect::Reflect) types using [`egui`]
//!
//! # Examples
//! **Basic usage**
//! ```rust
//! use bevy_reflect::{Reflect, TypeRegistry};
//! use bevy_inspector_egui::reflect_inspector::{ui_for_value, InspectorUi, Context};
//!
//! #[derive(Reflect)]
//! struct Data {
//!     value: f32,
//! }
//!
//! fn ui(data: &mut Data, ui: &mut egui::Ui, type_registry: &TypeRegistry) {
//!     let mut cx = Context::default(); // empty context, with no access to the bevy world
//!     let mut env = InspectorUi::new_no_short_circuit(type_registry, &mut cx); // no short circuiting, couldn't display `Handle<StandardMaterial>`
//!
//!     let _changed = env.ui_for_reflect(data, ui);
//!
//!     // alternatively, if you are using an empty `Context`:
//!     let _changed = ui_for_value(data, ui, type_registry);
//! }
//! ```
//!
//!
//! **Bevy specific usage**
//! ```rust
//! use bevy_reflect::{Reflect, TypeRegistry};
//! use bevy_inspector_egui::reflect_inspector::{InspectorUi, Context};
//!
//! use bevy_ecs::prelude::*;
//! use bevy_asset::Handle;
//! use bevy_pbr::StandardMaterial;
//!
//! #[derive(Reflect)]
//! struct Data {
//!     material: Handle<StandardMaterial>,
//! }
//!
//! fn ui(mut data: Mut<Data>, ui: &mut egui::Ui, world: &mut World, type_registry: &TypeRegistry) {
//!     let mut cx = Context {
//!         world: Some(world.into()),
//!     };
//!     let mut env = InspectorUi::for_bevy(type_registry, &mut cx);
//!
//!     // alternatively
//!     // use crate::bevy_inspector::short_circuit;
//!     // let mut env = InspectorUi::new(type_registry, &mut cx, Some(short_circuit::short_circuit), Some(short_circuit::short_circuit_readonly));
//!
//!     let changed = env.ui_for_reflect(data.bypass_change_detection(), ui);
//!     if changed {
//!         data.set_changed();
//!     }
//! }
//! ```

use crate::inspector_egui_impls::{iter_all_eq, InspectorEguiImpl};
use crate::inspector_options::{InspectorOptions, ReflectInspectorOptions, Target};
use crate::restricted_world_view::RestrictedWorldView;
use bevy_ecs::system::CommandQueue;
use bevy_reflect::{std_traits::ReflectDefault, DynamicStruct};
use bevy_reflect::{
    Array, DynamicEnum, DynamicTuple, DynamicVariant, Enum, EnumInfo, List, ListInfo, Map, Reflect,
    ReflectMut, ReflectRef, Struct, StructInfo, Tuple, TupleInfo, TupleStruct, TupleStructInfo,
    TypeInfo, TypeRegistry, ValueInfo, VariantInfo, VariantType,
};
use egui::Grid;
use std::any::{Any, TypeId};
use std::borrow::Cow;

pub(crate) mod errors;

/// Display the value without any [`Context`] or short circuiting behaviour.
/// This means that for example bevy's `Handle<StandardMaterial>` values cannot be displayed,
/// as they would need to have access to the `World`.
///
/// Use [`InspectorUi::new`] instead to provide context or use one of the methods in [`bevy_inspector`](crate::bevy_inspector).
pub fn ui_for_value(
    value: &mut dyn Reflect,
    ui: &mut egui::Ui,
    type_registry: &TypeRegistry,
) -> bool {
    InspectorUi::new_no_short_circuit(type_registry, &mut Context::default())
        .ui_for_reflect(value, ui)
}

/// Display the readonly value without any [`Context`] or short circuiting behaviour.
/// This means that for example bevy's `Handle<StandardMaterial>` values cannot be displayed,
/// as they would need to have access to the `World`.
///
/// Use [`InspectorUi::new`] instead to provide context or use one of the methods in [`bevy_inspector`](crate::bevy_inspector).
pub fn ui_for_value_readonly(value: &dyn Reflect, ui: &mut egui::Ui, type_registry: &TypeRegistry) {
    InspectorUi::new_no_short_circuit(type_registry, &mut Context::default())
        .ui_for_reflect_readonly(value, ui);
}

#[derive(Default)]
pub struct Context<'a> {
    pub world: Option<RestrictedWorldView<'a>>,
    pub queue: Option<&'a mut CommandQueue>,
}

/// Function which will be executed for every field recursively, which can be used to skip regular traversal.
///
/// This can be used to recognize `Handle<T>` types and display them as their actual value instead.
/// Returning `None` means that no short circuiting is required, and `Some(changed)` means that the value was short-circuited
/// and changed if the boolean is true.
pub type ShortCircuitFn = fn(
    &mut InspectorUi<'_, '_>,
    value: &mut dyn Reflect,
    ui: &mut egui::Ui,
    id: egui::Id,
    options: &dyn Any,
) -> Option<bool>;
/// Function which will be executed for every field recursively, which can be used to skip regular traversal, `_readonly` variant
///
/// This can be used to recognize `Handle<T>` types and display them as their actual value instead.
/// Returning `None` means that no short circuiting is required, and `Some(changed)` means that the value was short-circuited
/// and changed if the boolean is true.
pub type ShortCircuitFnReadonly = fn(
    &mut InspectorUi<'_, '_>,
    value: &dyn Reflect,
    ui: &mut egui::Ui,
    id: egui::Id,
    options: &dyn Any,
) -> Option<()>;
/// Function which will be executed for every field recursively, which can be used to skip regular traversal, `_many` variant
///
/// This can be used to recognize `Handle<T>` types and display them as their actual value instead.
/// Returning `None` means that no short circuiting is required, and `Some(changed)` means that the value was short-circuited
/// and changed if the boolean is true.
pub type ShortCircuitFnMany = fn(
    &mut InspectorUi<'_, '_>,
    type_id: TypeId,
    type_name: &str,
    ui: &mut egui::Ui,
    id: egui::Id,
    options: &dyn Any,
    values: &mut [&mut dyn Reflect],
    projector: &dyn Fn(&mut dyn Reflect) -> &mut dyn Reflect,
) -> Option<bool>;

pub struct InspectorUi<'a, 'c> {
    /// Reference to the [`TypeRegistry`]
    pub type_registry: &'a TypeRegistry,
    /// [`Context`] with additional data that can be used to display values
    pub context: &'a mut Context<'c>,

    /// Function which will be executed for every field recursively, which can be used to skip regular traversal.
    /// This can be used to recognize `Handle<T>` types and display them as their actual value instead.
    pub short_circuit: ShortCircuitFn,
    /// Same as [`short_circuit`](InspectorUi::short_circuit), but for read only usage.
    pub short_circuit_readonly: ShortCircuitFnReadonly,
    pub short_circuit_many: ShortCircuitFnMany,
}

impl<'a, 'c> InspectorUi<'a, 'c> {
    pub fn new(
        type_registry: &'a TypeRegistry,
        context: &'a mut Context<'c>,
        short_circuit: Option<ShortCircuitFn>,
        short_circuit_readonly: Option<ShortCircuitFnReadonly>,
        short_circuit_many: Option<ShortCircuitFnMany>,
    ) -> Self {
        Self {
            type_registry,
            context,
            short_circuit: short_circuit.unwrap_or(|_, _, _, _, _| None),
            short_circuit_readonly: short_circuit_readonly.unwrap_or(|_, _, _, _, _| None),
            short_circuit_many: short_circuit_many.unwrap_or(|_, _, _, _, _, _, _, _| None),
        }
    }

    pub fn new_no_short_circuit(
        type_registry: &'a TypeRegistry,
        context: &'a mut Context<'c>,
    ) -> Self {
        InspectorUi::new(type_registry, context, None, None, None)
    }
}

impl InspectorUi<'_, '_> {
    /// Draws the inspector UI for the given value.
    pub fn ui_for_reflect(&mut self, value: &mut dyn Reflect, ui: &mut egui::Ui) -> bool {
        self.ui_for_reflect_with_options(value, ui, egui::Id::null(), &())
    }

    /// Draws the inspector UI for the given value in a read-only way.
    pub fn ui_for_reflect_readonly(&mut self, value: &dyn Reflect, ui: &mut egui::Ui) {
        self.ui_for_reflect_readonly_with_options(value, ui, egui::Id::null(), &());
    }

    /// Draws the inspector UI for the given value with some options.
    ///
    /// The options can be [`struct@InspectorOptions`] for structs or enums with nested options for their fields,
    /// or other structs like [`NumberOptions`](crate::inspector_options::std_options::NumberOptions) which are interpreted
    /// by leaf types like `f32` or `Vec3`,
    pub fn ui_for_reflect_with_options(
        &mut self,
        value: &mut dyn Reflect,
        ui: &mut egui::Ui,
        id: egui::Id,
        options: &dyn Any,
    ) -> bool {
        let mut options = options;
        if options.is::<()>() {
            if let Some(data) = self
                .type_registry
                .get_type_data::<ReflectInspectorOptions>(Any::type_id(value))
            {
                options = &data.0;
            }
        }

        if let Some(s) = self
            .type_registry
            .get_type_data::<InspectorEguiImpl>(Any::type_id(value))
        {
            return s.execute(value.as_any_mut(), ui, options, id, self.reborrow());
        }

        if let Some(changed) = (self.short_circuit)(self, value, ui, id, options) {
            return changed;
        }

        match value.reflect_mut() {
            ReflectMut::Struct(value) => self.ui_for_struct(value, ui, id, options),
            ReflectMut::TupleStruct(value) => self.ui_for_tuple_struct(value, ui, id, options),
            ReflectMut::Tuple(value) => self.ui_for_tuple(value, ui, id, options),
            ReflectMut::List(value) => self.ui_for_list(value, ui, id, options),
            ReflectMut::Array(value) => self.ui_for_array(value, ui, id, options),
            ReflectMut::Map(value) => self.ui_for_reflect_map(value, ui, id, options),
            ReflectMut::Enum(value) => self.ui_for_enum(value, ui, id, options),
            ReflectMut::Value(value) => self.ui_for_value(value, ui, id, options),
        }
    }

    /// Draws the inspector UI for the given value with some options in a read-only way.
    ///
    /// The options can be [`struct@InspectorOptions`] for structs or enums with nested options for their fields,
    /// or other structs like [`NumberOptions`](crate::inspector_options::std_options::NumberOptions) which are interpreted
    /// by leaf types like `f32` or `Vec3`,
    pub fn ui_for_reflect_readonly_with_options(
        &mut self,
        value: &dyn Reflect,
        ui: &mut egui::Ui,
        id: egui::Id,
        options: &dyn Any,
    ) {
        let mut options = options;
        if options.is::<()>() {
            if let Some(data) = self
                .type_registry
                .get_type_data::<ReflectInspectorOptions>(Any::type_id(value))
            {
                options = &data.0;
            }
        }

        if let Some(s) = self
            .type_registry
            .get_type_data::<InspectorEguiImpl>(Any::type_id(value))
        {
            s.execute_readonly(value.as_any(), ui, options, id, self.reborrow());
            return;
        }

        if let Some(()) = (self.short_circuit_readonly)(self, value, ui, id, options) {
            return;
        }

        match value.reflect_ref() {
            ReflectRef::Struct(value) => self.ui_for_struct_readonly(value, ui, id, options),
            ReflectRef::TupleStruct(value) => {
                self.ui_for_tuple_struct_readonly(value, ui, id, options)
            }
            ReflectRef::Tuple(value) => self.ui_for_tuple_readonly(value, ui, id, options),
            ReflectRef::List(value) => self.ui_for_list_readonly(value, ui, id, options),
            ReflectRef::Array(value) => self.ui_for_array_readonly(value, ui, id, options),
            ReflectRef::Map(value) => self.ui_for_reflect_map_readonly(value, ui, id, options),
            ReflectRef::Enum(value) => self.ui_for_enum_readonly(value, ui, id, options),
            ReflectRef::Value(value) => self.ui_for_value_readonly(value, ui, id, options),
        }
    }

    pub fn ui_for_reflect_many(
        &mut self,
        type_id: TypeId,
        name: &str,
        ui: &mut egui::Ui,
        id: egui::Id,
        values: &mut [&mut dyn Reflect],
        projector: &dyn Fn(&mut dyn Reflect) -> &mut dyn Reflect,
    ) -> bool {
        self.ui_for_reflect_many_with_options(type_id, name, ui, id, &(), values, projector)
    }

    pub fn ui_for_reflect_many_with_options(
        &mut self,
        type_id: TypeId,
        name: &str,
        ui: &mut egui::Ui,
        id: egui::Id,
        options: &dyn Any,
        values: &mut [&mut dyn Reflect],
        projector: &dyn Fn(&mut dyn Reflect) -> &mut dyn Reflect,
    ) -> bool {
        let Some(registration) = self.type_registry.get(type_id) else {
            errors::not_in_type_registry(ui, name);
            return false;
        };
        let info = registration.type_info();

        let mut options = options;
        if options.is::<()>() {
            if let Some(data) = self
                .type_registry
                .get_type_data::<ReflectInspectorOptions>(type_id)
            {
                options = &data.0;
            }
        }

        if let Some(s) = self
            .type_registry
            .get_type_data::<InspectorEguiImpl>(type_id)
        {
            return s.execute_many(ui, options, id, self.reborrow(), values, projector);
        }

        if let Some(changed) =
            (self.short_circuit_many)(self, type_id, name, ui, id, options, values, projector)
        {
            return changed;
        }

        match info {
            TypeInfo::Struct(info) => {
                self.ui_for_struct_many(info, ui, id, options, values, projector)
            }
            TypeInfo::TupleStruct(info) => {
                self.ui_for_tuple_struct_many(info, ui, id, options, values, projector)
            }
            TypeInfo::Tuple(info) => {
                self.ui_for_tuple_many(info, ui, id, options, values, projector)
            }
            TypeInfo::List(info) => self.ui_for_list_many(info, ui, id, options, values, projector),
            TypeInfo::Array(info) => {
                errors::no_multiedit(
                    ui,
                    &pretty_type_name::pretty_type_name_str(info.type_name()),
                );
                false
            }
            TypeInfo::Map(info) => {
                errors::no_multiedit(
                    ui,
                    &pretty_type_name::pretty_type_name_str(info.type_name()),
                );
                false
            }
            TypeInfo::Enum(info) => self.ui_for_enum_many(info, ui, id, options, values, projector),
            TypeInfo::Value(info) => self.ui_for_value_many(info, ui, id, options),
            TypeInfo::Dynamic(_) => {
                errors::no_multiedit(
                    ui,
                    &pretty_type_name::pretty_type_name_str(info.type_name()),
                );
                false
            }
        }
    }
}

impl InspectorUi<'_, '_> {
    fn ui_for_struct(
        &mut self,
        value: &mut dyn Struct,
        ui: &mut egui::Ui,
        id: egui::Id,
        options: &dyn Any,
    ) -> bool {
        let mut changed = false;
        Grid::new(id).show(ui, |ui| {
            for i in 0..value.field_len() {
                ui.label(value.name_at(i).unwrap());
                let field = value.field_at_mut(i).unwrap();
                changed |= self.ui_for_reflect_with_options(
                    field,
                    ui,
                    id.with(i),
                    inspector_options_struct_field(options, i),
                );
                ui.end_row();
            }
        });
        changed
    }

    fn ui_for_struct_readonly(
        &mut self,
        value: &dyn Struct,
        ui: &mut egui::Ui,
        id: egui::Id,
        options: &dyn Any,
    ) {
        Grid::new(id).show(ui, |ui| {
            for i in 0..value.field_len() {
                ui.label(value.name_at(i).unwrap());
                let field = value.field_at(i).unwrap();
                self.ui_for_reflect_readonly_with_options(
                    field,
                    ui,
                    id.with(i),
                    inspector_options_struct_field(options, i),
                );
                ui.end_row();
            }
        });
    }

    fn ui_for_struct_many(
        &mut self,
        info: &StructInfo,
        ui: &mut egui::Ui,
        id: egui::Id,
        options: &dyn Any,
        values: &mut [&mut dyn Reflect],
        projector: impl Fn(&mut dyn Reflect) -> &mut dyn Reflect,
    ) -> bool {
        let mut changed = false;
        Grid::new(id).show(ui, |ui| {
            for (i, field) in info.iter().enumerate() {
                ui.label(field.name());
                changed |= self.ui_for_reflect_many_with_options(
                    field.type_id(),
                    field.type_name(),
                    ui,
                    id.with(i),
                    inspector_options_struct_field(options, i),
                    values,
                    &|a| match projector(a).reflect_mut() {
                        ReflectMut::Struct(strukt) => strukt.field_at_mut(i).unwrap(),
                        _ => unreachable!(),
                    },
                );
                ui.end_row();
            }
        });
        changed
    }

    fn ui_for_tuple_struct(
        &mut self,
        value: &mut dyn TupleStruct,
        ui: &mut egui::Ui,
        id: egui::Id,
        options: &dyn Any,
    ) -> bool {
        maybe_grid(value.field_len(), ui, id, |ui, label| {
            (0..value.field_len())
                .map(|i| {
                    if label {
                        ui.label(i.to_string());
                    }
                    let field = value.field_mut(i).unwrap();
                    let changed = self.ui_for_reflect_with_options(
                        field,
                        ui,
                        id.with(i),
                        inspector_options_struct_field(options, i),
                    );
                    ui.end_row();
                    changed
                })
                .fold(false, or)
        })
    }

    fn ui_for_tuple_struct_readonly(
        &mut self,
        value: &dyn TupleStruct,
        ui: &mut egui::Ui,
        id: egui::Id,
        options: &dyn Any,
    ) {
        maybe_grid_readonly(value.field_len(), ui, id, |ui, label| {
            for i in 0..value.field_len() {
                if label {
                    ui.label(i.to_string());
                }
                let field = value.field(i).unwrap();
                self.ui_for_reflect_readonly_with_options(
                    field,
                    ui,
                    id.with(i),
                    inspector_options_struct_field(options, i),
                );
                ui.end_row();
            }
        })
    }

    fn ui_for_tuple_struct_many(
        &mut self,
        info: &TupleStructInfo,
        ui: &mut egui::Ui,
        id: egui::Id,
        options: &dyn Any,
        values: &mut [&mut dyn Reflect],
        projector: impl Fn(&mut dyn Reflect) -> &mut dyn Reflect,
    ) -> bool {
        maybe_grid(info.field_len(), ui, id, |ui, label| {
            info.iter()
                .enumerate()
                .map(|(i, field)| {
                    if label {
                        ui.label(i.to_string());
                    }
                    let changed = self.ui_for_reflect_many_with_options(
                        field.type_id(),
                        field.type_name(),
                        ui,
                        id.with(i),
                        inspector_options_struct_field(options, i),
                        values,
                        &|a| match projector(a).reflect_mut() {
                            ReflectMut::TupleStruct(strukt) => strukt.field_mut(i).unwrap(),
                            _ => unreachable!(),
                        },
                    );
                    ui.end_row();
                    changed
                })
                .fold(false, or)
        })
    }

    fn ui_for_tuple(
        &mut self,
        value: &mut dyn Tuple,
        ui: &mut egui::Ui,
        id: egui::Id,
        options: &dyn Any,
    ) -> bool {
        maybe_grid(value.field_len(), ui, id, |ui, label| {
            (0..value.field_len())
                .map(|i| {
                    if label {
                        ui.label(i.to_string());
                    }
                    let field = value.field_mut(i).unwrap();
                    let changed = self.ui_for_reflect_with_options(
                        field,
                        ui,
                        id.with(i),
                        inspector_options_struct_field(options, i),
                    );
                    ui.end_row();
                    changed
                })
                .fold(false, or)
        })
    }

    fn ui_for_tuple_readonly(
        &mut self,
        value: &dyn Tuple,
        ui: &mut egui::Ui,
        id: egui::Id,
        options: &dyn Any,
    ) {
        maybe_grid_readonly(value.field_len(), ui, id, |ui, label| {
            for i in 0..value.field_len() {
                if label {
                    ui.label(i.to_string());
                }
                let field = value.field(i).unwrap();
                self.ui_for_reflect_readonly_with_options(
                    field,
                    ui,
                    id.with(i),
                    inspector_options_struct_field(options, i),
                );
                ui.end_row();
            }
        });
    }

    fn ui_for_tuple_many(
        &mut self,
        info: &TupleInfo,
        ui: &mut egui::Ui,
        id: egui::Id,
        options: &dyn Any,
        values: &mut [&mut dyn Reflect],
        projector: impl Fn(&mut dyn Reflect) -> &mut dyn Reflect,
    ) -> bool {
        maybe_grid(info.field_len(), ui, id, |ui, label| {
            info.iter()
                .enumerate()
                .map(|(i, field)| {
                    if label {
                        ui.label(i.to_string());
                    }
                    let changed = self.ui_for_reflect_many_with_options(
                        field.type_id(),
                        field.type_name(),
                        ui,
                        id.with(i),
                        inspector_options_struct_field(options, i),
                        values,
                        &|a| match projector(a).reflect_mut() {
                            ReflectMut::Tuple(strukt) => strukt.field_mut(i).unwrap(),
                            _ => unreachable!(),
                        },
                    );
                    ui.end_row();
                    changed
                })
                .fold(false, or)
        })
    }

    fn ui_for_list(
        &mut self,
        list: &mut dyn List,
        ui: &mut egui::Ui,
        id: egui::Id,
        options: &dyn Any,
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
                    changed |= self.ui_for_reflect_with_options(val, ui, id.with(i), options);
                });

                if i != len - 1 {
                    ui.separator();
                }
            }

            let TypeInfo::List(info) = list.get_type_info() else { return };
            let error_id = id.with("error");

            ui.vertical_centered_justified(|ui| {
                if ui.button("+").clicked() {
                    let default = self.get_default_value_for(info.item_type_id()).or_else(|| {
                        let last = len.checked_sub(1)?;
                        Some(Reflect::clone_value(list.get(last)?))
                    });

                    if let Some(new_value) = default {
                        list.push(new_value);
                    } else {
                        ui.data_mut(|data| data.insert_temp::<bool>(error_id, true));
                    }

                    changed = true;
                }
            });
            let error = ui.data_mut(|data| *data.get_temp_mut_or_default::<bool>(error_id));
            if error {
                errors::no_default_value(ui, info.item_type_name());
            }
            if ui.input(|input| input.pointer.any_down()) {
                ui.data_mut(|data| data.insert_temp::<bool>(error_id, false));
            }

            /*if let Some(_) = to_delete {
                changed = true;
            }*/
        });

        changed
    }

    fn ui_for_list_readonly(
        &mut self,
        list: &dyn List,
        ui: &mut egui::Ui,
        id: egui::Id,
        options: &dyn Any,
    ) {
        ui.vertical(|ui| {
            let len = list.len();
            for i in 0..len {
                let val = list.get(i).unwrap();
                ui.horizontal(|ui| {
                    self.ui_for_reflect_readonly_with_options(val, ui, id.with(i), options)
                });

                if i != len - 1 {
                    ui.separator();
                }
            }
        });
    }

    fn ui_for_list_many(
        &mut self,
        info: &ListInfo,
        ui: &mut egui::Ui,
        id: egui::Id,
        options: &dyn Any,
        values: &mut [&mut dyn Reflect],
        projector: impl Fn(&mut dyn Reflect) -> &mut dyn Reflect,
    ) -> bool {
        let mut changed = false;

        let add_button = |ui: &mut egui::Ui, values: &mut [&mut dyn Reflect]| {
            ui.vertical_centered_justified(|ui| {
                if ui.button("+").clicked() {
                    for list in values.iter_mut() {
                        let list = match projector(*list).reflect_mut() {
                            ReflectMut::List(list) => list,
                            _ => unreachable!(),
                        };
                        let last_element = list.get(list.len() - 1).unwrap().clone_value();
                        list.push(last_element);
                    }
                    true
                } else {
                    false
                }
            })
            .inner
        };

        let same_len =
            iter_all_eq(
                values
                    .iter_mut()
                    .map(|value| match projector(*value).reflect_mut() {
                        ReflectMut::List(l) => l.len(),
                        _ => unreachable!(),
                    }),
            );

        match same_len {
            Some(len) => {
                ui.vertical(|ui| {
                    // let mut to_delete = None;

                    for i in 0..len {
                        let mut items_at_i: Vec<&mut dyn Reflect> = values
                            .iter_mut()
                            .map(|value| match projector(*value).reflect_mut() {
                                ReflectMut::List(list) => list.get_mut(i).unwrap(),
                                _ => unreachable!(),
                            })
                            .collect();

                        ui.horizontal(|ui| {
                            changed |= self.ui_for_reflect_many_with_options(
                                info.item_type_id(),
                                info.item_type_name(),
                                ui,
                                id.with(i),
                                options,
                                items_at_i.as_mut_slice(),
                                &|a| a,
                            );

                            /*if utils::ui::label_button(ui, "✖", egui::Color32::RED) {
                                to_delete = Some(i);
                            }*/
                        });

                        if i != len - 1 {
                            ui.separator();
                        }
                    }

                    if len > 0 {
                        add_button(ui, values);
                    }

                    /*if let Some(_) = to_delete {
                        changed = true;
                    }*/
                });
            }
            None => {
                ui.label("lists have different sizes, cannot multiedit");
            }
        }

        changed
    }

    fn ui_for_reflect_map(
        &mut self,
        map: &mut dyn Map,
        ui: &mut egui::Ui,
        id: egui::Id,
        _options: &dyn Any,
    ) -> bool {
        let changed = false;
        egui::Grid::new(id).show(ui, |ui| {
            for (i, (key, value)) in map.iter().enumerate() {
                self.ui_for_reflect_readonly_with_options(key, ui, id.with(i), &());
                // TODO: iterate over values mutably
                self.ui_for_reflect_readonly_with_options(value, ui, id.with(i), &());
                ui.end_row();
            }
        });

        changed
    }

    fn ui_for_reflect_map_readonly(
        &mut self,
        map: &dyn Map,
        ui: &mut egui::Ui,
        id: egui::Id,
        _options: &dyn Any,
    ) {
        egui::Grid::new(id).show(ui, |ui| {
            for (i, (key, value)) in map.iter().enumerate() {
                self.ui_for_reflect_readonly_with_options(key, ui, id.with(i), &());
                self.ui_for_reflect_readonly_with_options(value, ui, id.with(i), &());
                ui.end_row();
            }
        });
    }

    fn ui_for_array(
        &mut self,
        array: &mut dyn Array,
        ui: &mut egui::Ui,
        id: egui::Id,
        options: &dyn Any,
    ) -> bool {
        let mut changed = false;

        ui.vertical(|ui| {
            let len = array.len();
            for i in 0..len {
                let val = array.get_mut(i).unwrap();
                ui.horizontal(|ui| {
                    changed |= self.ui_for_reflect_with_options(val, ui, id.with(i), options);
                });

                if i != len - 1 {
                    ui.separator();
                }
            }
        });

        changed
    }

    fn ui_for_array_readonly(
        &mut self,
        array: &dyn Array,
        ui: &mut egui::Ui,
        id: egui::Id,
        options: &dyn Any,
    ) {
        ui.vertical(|ui| {
            let len = array.len();
            for i in 0..len {
                let val = array.get(i).unwrap();
                ui.horizontal(|ui| {
                    self.ui_for_reflect_readonly_with_options(val, ui, id.with(i), options);
                });

                if i != len - 1 {
                    ui.separator();
                }
            }
        });
    }

    fn ui_for_enum(
        &mut self,
        value: &mut dyn Enum,
        ui: &mut egui::Ui,
        id: egui::Id,
        options: &dyn Any,
    ) -> bool {
        let type_info = value.get_type_info();
        let type_info = match type_info {
            TypeInfo::Enum(info) => info,
            _ => unreachable!("invalid reflect impl: type info mismatch"),
        };

        let mut changed = false;

        ui.vertical(|ui| {
            let changed_variant =
                self.ui_for_enum_variant_select(id, ui, value.variant_index(), type_info);
            if let Some((_new_variant, dynamic_enum)) = changed_variant {
                changed = true;
                value.apply(&dynamic_enum);
            }
            let variant_index = value.variant_index();

            let always_show_label = matches!(value.variant_type(), VariantType::Struct);
            changed |=
                maybe_grid_label_if(value.field_len(), ui, id, always_show_label, |ui, label| {
                    (0..value.field_len())
                        .map(|i| {
                            if label {
                                if let Some(name) = value.name_at(i) {
                                    ui.label(name);
                                } else {
                                    ui.label(i.to_string());
                                }
                            }
                            let field_value = value
                                .field_at_mut(i)
                                .expect("invalid reflect impl: field len");
                            let changed = self.ui_for_reflect_with_options(
                                field_value,
                                ui,
                                id.with(i),
                                inspector_options_enum_variant_field(options, variant_index, i),
                            );
                            ui.end_row();
                            changed
                        })
                        .fold(false, or)
                });
        });

        changed
    }

    fn ui_for_enum_many(
        &mut self,
        info: &EnumInfo,
        ui: &mut egui::Ui,
        id: egui::Id,
        options: &dyn Any,
        values: &mut [&mut dyn Reflect],
        projector: impl Fn(&mut dyn Reflect) -> &mut dyn Reflect,
    ) -> bool {
        let mut changed = false;

        let same_variant =
            iter_all_eq(
                values
                    .iter_mut()
                    .map(|value| match projector(*value).reflect_mut() {
                        ReflectMut::Enum(info) => info.variant_index(),
                        _ => unreachable!(),
                    }),
            );

        if let Some(variant_index) = same_variant {
            let mut variant = info.variant_at(variant_index).unwrap();

            ui.vertical(|ui| {
                let variant_changed = self.ui_for_enum_variant_select(id, ui, variant_index, info);
                if let Some((new_variant_idx, dynamic_enum)) = variant_changed {
                    changed = true;
                    variant = info.variant_at(new_variant_idx).unwrap();

                    for value in values.iter_mut() {
                        let value = projector(*value);
                        value.apply(&dynamic_enum);
                    }
                }

                let field_len = match variant {
                    VariantInfo::Struct(info) => info.field_len(),
                    VariantInfo::Tuple(info) => info.field_len(),
                    VariantInfo::Unit(_) => 0,
                };

                let always_show_label = matches!(variant, VariantInfo::Struct(_));
                changed |=
                    maybe_grid_label_if(field_len, ui, id, always_show_label, |ui, label| {
                        let handle = |(field_index, field_name, field_type_id, field_type_name)| {
                            if label {
                                ui.label(field_name);
                            }

                            let mut variants_across: Vec<&mut dyn Reflect> = values
                                .iter_mut()
                                .map(|value| match projector(*value).reflect_mut() {
                                    ReflectMut::Enum(value) => {
                                        value.field_at_mut(field_index).unwrap()
                                    }
                                    _ => unreachable!(),
                                })
                                .collect();

                            self.ui_for_reflect_many_with_options(
                                field_type_id,
                                field_type_name,
                                ui,
                                id.with(field_index),
                                inspector_options_enum_variant_field(
                                    options,
                                    variant_index,
                                    field_index,
                                ),
                                variants_across.as_mut_slice(),
                                &|a| a,
                            );

                            ui.end_row();

                            false
                        };

                        match variant {
                            VariantInfo::Struct(info) => info
                                .iter()
                                .enumerate()
                                .map(|(i, field)| {
                                    (
                                        i,
                                        Cow::Borrowed(field.name()),
                                        field.type_id(),
                                        field.type_name(),
                                    )
                                })
                                .map(handle)
                                .fold(false, or),
                            VariantInfo::Tuple(info) => info
                                .iter()
                                .enumerate()
                                .map(|(i, field)| {
                                    (
                                        i,
                                        Cow::Owned(i.to_string()),
                                        field.type_id(),
                                        field.type_name(),
                                    )
                                })
                                .map(handle)
                                .fold(false, or),
                            VariantInfo::Unit(_) => false,
                        }
                    });
            });
        } else {
            ui.label("enums have different selected variants, cannot multiedit");
        }

        changed
    }

    fn ui_for_enum_variant_select(
        &mut self,
        id: egui::Id,
        ui: &mut egui::Ui,
        active_variant_idx: usize,
        info: &bevy_reflect::EnumInfo,
    ) -> Option<(usize, DynamicEnum)> {
        let mut changed_variant = None;

        ui.horizontal(|ui| {
            let mut unconstructable_variants = Vec::new();
            egui::ComboBox::new(id.with("select"), "")
                .selected_text(info.variant_names()[active_variant_idx])
                .show_ui(ui, |ui| {
                    for (i, variant) in info.iter().enumerate() {
                        let variant_name = variant.name();
                        let is_active_variant = i == active_variant_idx;

                        let variant_is_constructable =
                            is_variant_constructable(self.type_registry, variant);
                        if !variant_is_constructable && !is_active_variant {
                            unconstructable_variants.push(variant_name);
                        }
                        ui.add_enabled_ui(variant_is_constructable, |ui| {
                            if ui
                                .selectable_label(is_active_variant, variant_name)
                                .clicked()
                            {
                                if let Ok(dynamic_enum) =
                                    self.construct_default_variant(variant, ui, info.type_name())
                                {
                                    changed_variant = Some((i, dynamic_enum));
                                };
                            }
                        });
                    }

                    false
                });
            if !unconstructable_variants.is_empty() {
                errors::unconstructable_variants(ui, info.type_name(), &unconstructable_variants);
            }
        });

        changed_variant
    }

    fn ui_for_enum_readonly(
        &mut self,
        value: &dyn Enum,
        ui: &mut egui::Ui,
        id: egui::Id,
        options: &dyn Any,
    ) {
        ui.vertical(|ui| {
            let active_variant = value.variant_name();
            ui.add_enabled_ui(false, |ui| {
                egui::ComboBox::new(id, "")
                    .selected_text(active_variant)
                    .show_ui(ui, |_| {})
            });

            let always_show_label = matches!(value.variant_type(), VariantType::Struct);
            maybe_grid_readonly_label_if(
                value.field_len(),
                ui,
                id,
                always_show_label,
                |ui, label| {
                    for i in 0..value.field_len() {
                        if label {
                            if let Some(name) = value.name_at(i) {
                                ui.label(name);
                            } else {
                                ui.label(i.to_string());
                            }
                        }
                        let field_value =
                            value.field_at(i).expect("invalid reflect impl: field len");
                        self.ui_for_reflect_readonly_with_options(
                            field_value,
                            ui,
                            id.with(i),
                            inspector_options_enum_variant_field(options, value.variant_index(), i),
                        );
                        ui.end_row();
                    }
                },
            );
        });
    }

    fn ui_for_value(
        &mut self,
        value: &mut dyn Reflect,
        ui: &mut egui::Ui,
        _id: egui::Id,
        _options: &dyn Any,
    ) -> bool {
        errors::reflect_value_no_impl(ui, value.type_name());
        false
    }

    fn ui_for_value_readonly(
        &mut self,
        value: &dyn Reflect,
        ui: &mut egui::Ui,
        _id: egui::Id,
        _options: &dyn Any,
    ) {
        errors::reflect_value_no_impl(ui, value.type_name());
    }

    fn ui_for_value_many(
        &mut self,
        info: &ValueInfo,
        ui: &mut egui::Ui,
        _id: egui::Id,
        _options: &dyn Any,
    ) -> bool {
        errors::reflect_value_no_impl(ui, info.type_name());
        false
    }
}

impl<'a, 'c> InspectorUi<'a, 'c> {
    fn reborrow<'s>(&'s mut self) -> InspectorUi<'s, 'c> {
        InspectorUi {
            type_registry: self.type_registry,
            context: self.context,
            short_circuit: self.short_circuit,
            short_circuit_readonly: self.short_circuit_readonly,
            short_circuit_many: self.short_circuit_many,
        }
    }

    fn get_default_value_for(&mut self, type_id: TypeId) -> Option<Box<dyn Reflect>> {
        if let Some(reflect_default) = self.type_registry.get_type_data::<ReflectDefault>(type_id) {
            return Some(reflect_default.default());
        }

        None
    }

    fn construct_default_variant(
        &mut self,
        variant: &VariantInfo,
        ui: &mut egui::Ui,
        enum_type_name: &'static str,
    ) -> Result<DynamicEnum, ()> {
        let dynamic_variant = match variant {
            VariantInfo::Struct(struct_info) => {
                let mut dynamic_struct = DynamicStruct::default();
                for field in struct_info.iter() {
                    let field_default_value = match self.get_default_value_for(field.type_id()) {
                        Some(value) => value,
                        None => {
                            errors::no_default_value(ui, field.type_name());
                            return Err(());
                        }
                    };
                    dynamic_struct.insert_boxed(field.name(), field_default_value);
                }
                DynamicVariant::Struct(dynamic_struct)
            }
            VariantInfo::Tuple(tuple_info) => {
                let mut dynamic_tuple = DynamicTuple::default();
                for field in tuple_info.iter() {
                    let field_default_value = match self.get_default_value_for(field.type_id()) {
                        Some(value) => value,
                        None => {
                            errors::no_default_value(ui, field.type_name());
                            return Err(());
                        }
                    };
                    dynamic_tuple.insert_boxed(field_default_value);
                }
                DynamicVariant::Tuple(dynamic_tuple)
            }
            VariantInfo::Unit(_) => DynamicVariant::Unit,
        };
        let dynamic_enum = DynamicEnum::new(enum_type_name, variant.name(), dynamic_variant);
        Ok(dynamic_enum)
    }
}

#[must_use]
fn maybe_grid(
    i: usize,
    ui: &mut egui::Ui,
    id: egui::Id,
    mut f: impl FnMut(&mut egui::Ui, bool) -> bool,
) -> bool {
    match i {
        0 => false,
        1 => f(ui, false),
        _ => Grid::new(id).show(ui, |ui| f(ui, true)).inner,
    }
}
#[must_use]
fn maybe_grid_label_if(
    i: usize,
    ui: &mut egui::Ui,
    id: egui::Id,
    always_show_label: bool,
    mut f: impl FnMut(&mut egui::Ui, bool) -> bool,
) -> bool {
    match i {
        0 => false,
        1 if !always_show_label => f(ui, false),
        _ => Grid::new(id).show(ui, |ui| f(ui, true)).inner,
    }
}

fn maybe_grid_readonly(
    i: usize,
    ui: &mut egui::Ui,
    id: egui::Id,
    mut f: impl FnMut(&mut egui::Ui, bool),
) {
    match i {
        0 => {}
        1 => f(ui, false),
        _ => {
            Grid::new(id).show(ui, |ui| f(ui, true));
        }
    }
}
fn maybe_grid_readonly_label_if(
    i: usize,
    ui: &mut egui::Ui,
    id: egui::Id,
    always_show_label: bool,
    mut f: impl FnMut(&mut egui::Ui, bool),
) {
    match i {
        0 => {}
        1 if !always_show_label => f(ui, false),
        _ => {
            Grid::new(id).show(ui, |ui| f(ui, true));
        }
    }
}

fn is_variant_constructable(type_registry: &TypeRegistry, variant: &VariantInfo) -> bool {
    let type_id_is_constructable = |type_id: TypeId| {
        type_registry
            .get_type_data::<ReflectDefault>(type_id)
            .is_some()
    };

    match variant {
        VariantInfo::Struct(variant) => variant
            .iter()
            .map(|field| field.type_id())
            .all(type_id_is_constructable),
        VariantInfo::Tuple(variant) => variant
            .iter()
            .map(|field| field.type_id())
            .all(type_id_is_constructable),
        VariantInfo::Unit(_) => true,
    }
}

fn inspector_options_struct_field(options: &dyn Any, field: usize) -> &dyn Any {
    options
        .downcast_ref::<InspectorOptions>()
        .and_then(|options| options.get(Target::Field(field)))
        .unwrap_or(&())
}

fn inspector_options_enum_variant_field<'a>(
    options: &'a dyn Any,
    variant_index: usize,
    field_index: usize,
) -> &'a dyn Any {
    options
        .downcast_ref::<InspectorOptions>()
        .and_then(|options| {
            options.get(Target::VariantField {
                variant_index,
                field_index,
            })
        })
        .unwrap_or(&())
}

fn or(a: bool, b: bool) -> bool {
    a || b
}
