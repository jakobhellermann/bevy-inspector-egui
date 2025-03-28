//! General-purpose machinery for displaying [`Reflect`] types using [`egui`]
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
//! use bevy_ecs::world::CommandQueue;
//! use bevy_asset::Handle;
//! use bevy_pbr::StandardMaterial;
//!
//! #[derive(Reflect)]
//! struct Data {
//!     material: Handle<StandardMaterial>,
//! }
//!
//! fn ui(mut data: Mut<Data>, ui: &mut egui::Ui, world: &mut World, type_registry: &TypeRegistry) {
//!     let mut queue = CommandQueue::default();
//!     let mut cx = Context {
//!         world: Some(world.into()),
//!         queue: Some(&mut queue),
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
//!
//!     queue.apply(world);
//! }
//! ```

use crate::egui_utils::show_docs;
use crate::inspector_egui_impls::{iter_all_eq, InspectorEguiImpl};
use crate::inspector_options::{InspectorOptions, ReflectInspectorOptions, Target};
use crate::restricted_world_view::RestrictedWorldView;
use crate::{
    egui_utils::{add_button, down_button, remove_button, up_button},
    utils::pretty_type_name_str,
};
use bevy_ecs::world::CommandQueue;
use bevy_reflect::{std_traits::ReflectDefault, DynamicStruct};
use bevy_reflect::{
    Array, DynamicEnum, DynamicTuple, DynamicVariant, Enum, EnumInfo, List, ListInfo, Map, Reflect,
    ReflectMut, ReflectRef, Struct, StructInfo, Tuple, TupleInfo, TupleStruct, TupleStructInfo,
    TypeInfo, TypeRegistry, VariantInfo, VariantType,
};
use bevy_reflect::{OpaqueInfo, PartialReflect, Set, SetInfo};
use egui::{Grid, WidgetText};
use std::borrow::Cow;
use std::{
    any::{Any, TypeId},
    borrow::Borrow,
};

pub(crate) mod errors;

pub trait ProjectorReflect: Fn(&mut dyn PartialReflect) -> &mut dyn PartialReflect {}

impl<T> ProjectorReflect for T where T: Fn(&mut dyn PartialReflect) -> &mut dyn PartialReflect {}

/// Display the value without any [`Context`] or short circuiting behaviour.
///
/// This means that for example bevy's `Handle<StandardMaterial>` values cannot be displayed,
/// as they would need to have access to the `World`.
///
/// Use [`InspectorUi::new`] instead to provide context or use one of the methods in [`bevy_inspector`](crate::bevy_inspector).
pub fn ui_for_value(
    value: &mut dyn PartialReflect,
    ui: &mut egui::Ui,
    type_registry: &TypeRegistry,
) -> bool {
    InspectorUi::new_no_short_circuit(type_registry, &mut Context::default())
        .ui_for_reflect(value, ui)
}

/// Display the readonly value without any [`Context`] or short circuiting behaviour.
///
/// This means that for example bevy's `Handle<StandardMaterial>` values cannot be displayed,
/// as they would need to have access to the `World`.
///
/// Use [`InspectorUi::new`] instead to provide context or use one of the methods in [`bevy_inspector`](crate::bevy_inspector).
pub fn ui_for_value_readonly(
    value: &dyn PartialReflect,
    ui: &mut egui::Ui,
    type_registry: &TypeRegistry,
) {
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
    value: &mut dyn PartialReflect,
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
    value: &dyn PartialReflect,
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
    values: &mut [&mut dyn PartialReflect],
    projector: &dyn ProjectorReflect,
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
    pub fn ui_for_reflect(&mut self, value: &mut dyn PartialReflect, ui: &mut egui::Ui) -> bool {
        self.ui_for_reflect_with_options(value, ui, egui::Id::NULL, &())
    }

    /// Draws the inspector UI for the given value in a read-only way.
    pub fn ui_for_reflect_readonly(&mut self, value: &dyn PartialReflect, ui: &mut egui::Ui) {
        self.ui_for_reflect_readonly_with_options(value, ui, egui::Id::NULL, &());
    }

    /// Draws the inspector UI for the given value with some options.
    ///
    /// The options can be [`struct@InspectorOptions`] for structs or enums with nested options for their fields,
    /// or other structs like [`NumberOptions`](crate::inspector_options::std_options::NumberOptions) which are interpreted
    /// by leaf types like `f32` or `Vec3`,
    pub fn ui_for_reflect_with_options(
        &mut self,
        value: &mut dyn PartialReflect,
        ui: &mut egui::Ui,
        id: egui::Id,
        options: &dyn Any,
    ) -> bool {
        let mut options = options;
        if options.is::<()>() {
            if let Some(data) = value.try_as_reflect().and_then(|val| {
                self.type_registry
                    .get_type_data::<ReflectInspectorOptions>(val.type_id())
            }) {
                options = &data.0;
            }
        }

        if let Some(reflected) = value.try_as_reflect_mut() {
            if let Some(s) = self
                .type_registry
                .get_type_data::<InspectorEguiImpl>(reflected.reflect_type_info().type_id())
            {
                if let Some(value) = value.try_as_reflect_mut() {
                    return s.execute(value.as_any_mut(), ui, options, id, self.reborrow());
                }
            }
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
            ReflectMut::Opaque(value) => self.ui_for_value(value, ui, id, options),
            ReflectMut::Set(value) => self.ui_for_set(value, ui, id, options),
            #[allow(unreachable_patterns)]
            _ => {
                ui.label("unsupported");
                false
            }
        }
    }

    /// Draws the inspector UI for the given value with some options in a read-only way.
    ///
    /// The options can be [`struct@InspectorOptions`] for structs or enums with nested options for their fields,
    /// or other structs like [`NumberOptions`](crate::inspector_options::std_options::NumberOptions) which are interpreted
    /// by leaf types like `f32` or `Vec3`,
    pub fn ui_for_reflect_readonly_with_options(
        &mut self,
        value: &dyn PartialReflect,
        ui: &mut egui::Ui,
        id: egui::Id,
        options: &dyn Any,
    ) {
        let mut options = options;
        if options.is::<()>() {
            if let Some(value_reflect) = value.try_as_reflect() {
                if let Some(data) = self
                    .type_registry
                    .get_type_data::<ReflectInspectorOptions>(value_reflect.type_id())
                {
                    options = &data.0;
                }
            }
        }

        if let Some(value_reflect) = value.try_as_reflect() {
            if let Some(s) = self
                .type_registry
                .get_type_data::<InspectorEguiImpl>(value_reflect.type_id())
            {
                if let Some(value) = value.try_as_reflect() {
                    s.execute_readonly(value.as_any(), ui, options, id, self.reborrow());
                    return;
                }
            }
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
            ReflectRef::Opaque(value) => self.ui_for_value_readonly(value, ui, id, options),
            ReflectRef::Set(value) => self.ui_for_set_readonly(value, ui, id, options),
            #[allow(unreachable_patterns)]
            _ => {
                ui.label("unsupported");
            }
        }
    }

    pub fn ui_for_reflect_many(
        &mut self,
        type_id: TypeId,
        name: &str,
        ui: &mut egui::Ui,
        id: egui::Id,
        values: &mut [&mut dyn PartialReflect],
        projector: &dyn ProjectorReflect,
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
        values: &mut [&mut dyn PartialReflect],
        projector: &dyn ProjectorReflect,
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
                errors::no_multiedit(ui, &pretty_type_name_str(info.type_path()));
                false
            }
            TypeInfo::Map(info) => {
                errors::no_multiedit(ui, &pretty_type_name_str(info.type_path()));
                false
            }
            TypeInfo::Enum(info) => self.ui_for_enum_many(info, ui, id, options, values, projector),
            TypeInfo::Opaque(info) => self.ui_for_value_many(info, ui, id, options),
            TypeInfo::Set(info) => self.ui_for_set_many(info, ui, id, options, values, projector),
        }
    }
}

enum ListOp {
    AddElement(usize),
    RemoveElement(usize),
    MoveElementUp(usize),
    MoveElementDown(usize),
}

enum SetOp {
    RemoveElement(Box<dyn PartialReflect>),
    AddElement(Box<dyn PartialReflect>),
}

fn ui_for_empty_collection(ui: &mut egui::Ui, label: impl Into<WidgetText>) -> bool {
    let mut add = false;
    ui.vertical_centered(|ui| {
        ui.label(label);
        if add_button(ui).on_hover_text("Add element").clicked() {
            add = true;
        }
    });
    add
}

fn ui_for_empty_list(ui: &mut egui::Ui) -> bool {
    ui_for_empty_collection(ui, "(Empty List)")
}

fn ui_for_list_controls(ui: &mut egui::Ui, index: usize, len: usize) -> Option<ListOp> {
    use ListOp::*;
    let mut op = None;
    ui.horizontal_top(|ui| {
        if add_button(ui).on_hover_text("Add element").clicked() {
            op = Some(AddElement(index));
        }
        if remove_button(ui).on_hover_text("Remove element").clicked() {
            op = Some(RemoveElement(index));
        }
        let up_enabled = index > 0;
        ui.add_enabled_ui(up_enabled, |ui| {
            if up_button(ui).on_hover_text("Move element up").clicked() {
                op = Some(MoveElementUp(index));
            }
        });
        let down_enabled = len.checked_sub(1).map(|l| index < l).unwrap_or(false);
        ui.add_enabled_ui(down_enabled, |ui| {
            if down_button(ui).on_hover_text("Move element down").clicked() {
                op = Some(MoveElementDown(index));
            }
        });
    });
    op
}

fn ui_for_empty_set(ui: &mut egui::Ui) {
    ui.vertical_centered(|ui| ui.label("(Empty Set)"));
}

struct MapDraftElement {
    key: Box<dyn PartialReflect>,
    value: Box<dyn PartialReflect>,
}
impl Clone for MapDraftElement {
    fn clone(&self) -> Self {
        Self {
            key: self.key.to_dynamic(),
            value: self.value.to_dynamic(),
        }
    }
}

struct SetDraftElement(Box<dyn PartialReflect>);

impl Clone for SetDraftElement {
    fn clone(&self) -> Self {
        Self(self.0.to_dynamic())
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
        let Some(TypeInfo::Struct(type_info)) = value.get_represented_type_info() else {
            return false;
        };

        let mut changed = false;
        Grid::new(id).show(ui, |ui| {
            for i in 0..value.field_len() {
                let field_info = type_info.field_at(i).unwrap();

                let _response = ui.label(field_info.name());
                #[cfg(feature = "documentation")]
                show_docs(_response, field_info.docs());

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
        let Some(TypeInfo::Struct(type_info)) = value.get_represented_type_info() else {
            return;
        };

        Grid::new(id).show(ui, |ui| {
            for i in 0..value.field_len() {
                let field_info = type_info.field_at(i).unwrap();

                let _response = ui.label(field_info.name());
                #[cfg(feature = "documentation")]
                show_docs(_response, field_info.docs());

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
        values: &mut [&mut dyn PartialReflect],
        projector: impl ProjectorReflect,
    ) -> bool {
        let mut changed = false;
        Grid::new(id).show(ui, |ui| {
            for (i, field) in info.iter().enumerate() {
                let _response = ui.label(field.name());
                #[cfg(feature = "documentation")]
                show_docs(_response, field.docs());

                changed |= self.ui_for_reflect_many_with_options(
                    field.type_id(),
                    field.type_path(),
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
        values: &mut [&mut dyn PartialReflect],
        projector: impl ProjectorReflect,
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
                        field.type_path(),
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
        values: &mut [&mut dyn PartialReflect],
        projector: impl ProjectorReflect,
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
                        field.type_path(),
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

    /// Mutate one or more lists based on a [`ListOp`], generated by some user interaction.
    fn respond_to_list_op<'a>(
        &mut self,
        ui: &mut egui::Ui,
        id: egui::Id,
        lists: impl Iterator<Item = &'a mut dyn List>,
        op: ListOp,
    ) -> bool {
        use ListOp::*;
        let mut changed = false;
        let error_id = id.with("error");

        for list in lists {
            let Some(TypeInfo::List(info)) = list.get_represented_type_info() else {
                continue;
            };
            match op {
                AddElement(i) => {
                    let default = self
                        .get_default_value_for(info.item_ty().id())
                        .map(|def| def.into_partial_reflect())
                        .or_else(|| list.get(i).map(|v| v.to_dynamic()));
                    if let Some(new_value) = default {
                        list.insert(i, new_value);
                    } else {
                        ui.data_mut(|data| data.insert_temp::<bool>(error_id, true));
                    }
                    changed = true;
                }
                RemoveElement(i) => {
                    list.remove(i);
                    changed = true;
                }
                MoveElementUp(i) => {
                    if let Some(prev_idx) = i.checked_sub(1) {
                        // Clone this element and insert it at its index - 1.
                        if let Some(element) = list.get(i) {
                            let clone = element.to_dynamic();
                            list.insert(prev_idx, clone);
                        }
                        // Remove the original, now at its index + 1.
                        list.remove(i + 1);
                        changed = true;
                    }
                }
                MoveElementDown(i) => {
                    // Clone the next element and insert it at this index.
                    if let Some(next_element) = list.get(i + 1) {
                        let next_clone = next_element.to_dynamic();
                        list.insert(i, next_clone);
                    }
                    // Remove the original, now at i + 2.
                    list.remove(i + 2);
                    changed = true;
                }
            }
        }
        changed
    }

    fn ui_for_list(
        &mut self,
        list: &mut dyn List,
        ui: &mut egui::Ui,
        id: egui::Id,
        options: &dyn Any,
    ) -> bool {
        use ListOp::*;
        let mut changed = false;

        ui.vertical(|ui| {
            let mut op = None;
            let len = list.len();
            if len == 0 && ui_for_empty_list(ui) {
                op = Some(AddElement(0))
            }
            for i in 0..len {
                egui::Grid::new((id, i)).show(ui, |ui| {
                    ui.label(i.to_string());
                    let val = list.get_mut(i).unwrap();
                    ui.horizontal_top(|ui| {
                        changed |= self.ui_for_reflect_with_options(val, ui, id.with(i), options);
                    });
                    ui.end_row();

                    let item_op = ui_for_list_controls(ui, i, len);
                    if item_op.is_some() {
                        op = item_op;
                    }
                });

                if i != len - 1 {
                    ui.separator();
                }
            }

            let Some(TypeInfo::List(info)) = list.get_represented_type_info() else {
                return;
            };
            let error_id = id.with("error");

            // Respond to control interaction
            if let Some(op) = op {
                let lists = std::iter::once(list);
                changed |= self.respond_to_list_op(ui, id, lists, op);
            }

            let error = ui.data_mut(|data| *data.get_temp_mut_or_default::<bool>(error_id));
            if error {
                errors::no_default_value(ui, info.type_path());
            }
            if ui.input(|input| input.pointer.any_down()) {
                ui.data_mut(|data| data.insert_temp::<bool>(error_id, false));
            }
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
                ui.horizontal_top(|ui| {
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
        values: &mut [&mut dyn PartialReflect],
        projector: impl ProjectorReflect,
    ) -> bool {
        use ListOp::*;
        let mut changed = false;

        let same_len =
            iter_all_eq(
                values
                    .iter_mut()
                    .map(|value| match projector(*value).reflect_mut() {
                        ReflectMut::List(l) => l.len(),
                        _ => unreachable!(),
                    }),
            );

        let Some(len) = same_len else {
            ui.label("lists have different sizes, cannot multiedit");
            return changed;
        };

        ui.vertical(|ui| {
            let mut op = None;

            if len == 0 && ui_for_empty_list(ui) {
                op = Some(AddElement(0));
            }

            for i in 0..len {
                let mut items_at_i: Vec<&mut dyn PartialReflect> = values
                    .iter_mut()
                    .map(|value| match projector(*value).reflect_mut() {
                        ReflectMut::List(list) => list.get_mut(i).unwrap(),
                        _ => unreachable!(),
                    })
                    .collect();

                egui::Grid::new((id, i)).show(ui, |ui| {
                    ui.label(i.to_string());
                    ui.horizontal_top(|ui| {
                        changed |= self.ui_for_reflect_many_with_options(
                            info.item_ty().id(),
                            info.type_path(),
                            ui,
                            id.with(i),
                            options,
                            items_at_i.as_mut_slice(),
                            &|a| a,
                        );
                    });
                    ui.end_row();
                    let item_op = ui_for_list_controls(ui, i, len);
                    if item_op.is_some() {
                        op = item_op;
                    }
                });

                if i != len - 1 {
                    ui.separator();
                }
            }

            let error_id = id.with("error");
            let error = ui.data_mut(|data| *data.get_temp_mut_or_default::<bool>(error_id));
            if error {
                errors::no_default_value(ui, info.type_path());
            }
            if ui.input(|input| input.pointer.any_down()) {
                ui.data_mut(|data| data.insert_temp::<bool>(error_id, false));
            }
            if let Some(op) = op {
                let lists = values
                    .iter_mut()
                    .map(|l| match projector(*l).reflect_mut() {
                        ReflectMut::List(list) => list,
                        _ => unreachable!(),
                    });
                changed |= self.respond_to_list_op(ui, id, lists, op);
            }
        });

        changed
    }

    fn ui_for_reflect_map(
        &mut self,
        map: &mut dyn Map,
        ui: &mut egui::Ui,
        id: egui::Id,
        _options: &dyn Any,
    ) -> bool {
        let mut changed = false;
        let map_draft_id = id.with("map_draft");
        if map.is_empty() {
            ui.label("(Empty Map)");
            ui.end_row();
        }
        let draft_clone = ui.data_mut(|data| {
            data.get_temp_mut_or_default::<Option<MapDraftElement>>(map_draft_id)
                .to_owned()
        });
        let mut to_delete: Option<usize> = None;

        egui::Grid::new(id).show(ui, |ui| {
            for i in 0..map.len() {
                if let Some((key, value)) = map.get_at_mut(i) {
                    self.ui_for_reflect_readonly_with_options(key, ui, id.with(i), &());
                    changed |= self.ui_for_reflect_with_options(value, ui, id.with(i), &());
                    if remove_button(ui).on_hover_text("Remove element").clicked() {
                        to_delete = Some(i);
                    }
                    ui.end_row();
                }
            }
            ui.separator();
            ui.end_row();
            ui.label("New element");
            match draft_clone {
                None => {
                    // If no draft element exists, show a button to create one.
                    if add_button(ui).clicked() {
                        // Insert a temporary 'draft' key-value pair into UI state.
                        if let Some(TypeInfo::Map(map_info)) = map.get_represented_type_info() {
                            let op = Option::zip(
                                self.get_default_value_for(map_info.key_ty().id()),
                                self.get_default_value_for(map_info.value_ty().id()),
                            )
                            .map(|(k, v)| MapDraftElement {
                                key: k.into_partial_reflect(),
                                value: v.into_partial_reflect(),
                            });
                            if op.is_some() {
                                ui.data_mut(|data| data.insert_temp(map_draft_id, op));
                            }
                        }
                    }
                    ui.end_row();
                }
                Some(MapDraftElement {
                    key: mut k,
                    value: mut v,
                }) => {
                    ui.end_row();
                    // Show controls for editing our draft element.
                    let key_changed = self.ui_for_reflect_with_options(k.as_mut(), ui, id, &());
                    let value_changed = self.ui_for_reflect_with_options(v.as_mut(), ui, id, &());
                    // If the clone changed, update the data in UI state.
                    if key_changed || value_changed {
                        let next_draft = MapDraftElement { key: k, value: v };
                        ui.data_mut(|data| data.insert_temp(map_draft_id, Some(next_draft)));
                    }
                    // Show controls to insert the draft into the map, or remove it.
                    if ui.button("Insert").clicked() {
                        let draft = ui
                            .data_mut(|data| data.get_temp::<Option<MapDraftElement>>(map_draft_id))
                            .flatten();
                        if let Some(draft) = draft {
                            map.insert_boxed(draft.key, draft.value);
                            ui.data_mut(|data| data.remove_by_type::<Option<MapDraftElement>>());
                        }
                        changed = true;
                    }
                    if ui.button("Cancel").clicked() {
                        ui.data_mut(|data| data.remove_by_type::<Option<MapDraftElement>>());
                        changed = true;
                    }
                    ui.end_row();
                }
            }
        });

        if let Some(index) = to_delete {
            // Can't have both an immutable borrow of the map's key,
            // and mutably borrow the map to delete the element.
            let cloned_key = map.get_at(index).map(|(key, _)| key.to_dynamic());
            if let Some(key) = cloned_key {
                map.remove(key.as_ref());
            }
        }

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

    /// Mutate one or more lists based on a [`SetOp`], generated by some user interaction.
    fn respond_to_sets_op<'a>(
        &mut self,
        sets: impl Iterator<Item = &'a mut dyn Set>,
        op: SetOp,
    ) -> bool {
        let mut changed = false;

        for set in sets {
            changed |= self.respond_to_set_op(set, &op);
        }
        changed
    }
    fn respond_to_set_op<'a>(&mut self, set: &'a mut dyn Set, op: &SetOp) -> bool {
        use SetOp::*;
        match &op {
            AddElement(new_value) => {
                set.insert_boxed(new_value.to_dynamic());
            }
            RemoveElement(val) => {
                set.remove(&**val);
            }
        }
        true
    }

    fn ui_for_set(
        &mut self,
        set: &mut dyn Set,
        ui: &mut egui::Ui,
        id: egui::Id,
        options: &dyn Any,
    ) -> bool {
        use SetOp::*;
        let mut changed = false;

        ui.vertical(|ui| {
            let mut op = None;

            let len = set.len();
            if len == 0 {
                ui_for_empty_set(ui);
            }

            for (i, val) in set.iter().enumerate() {
                egui::Grid::new((id, i)).show(ui, |ui| {
                    ui.horizontal_top(|ui| {
                        self.ui_for_reflect_readonly_with_options(val, ui, id.with(i), options);
                    });
                    ui.horizontal_top(|ui| {
                        if remove_button(ui).on_hover_text("Remove element").clicked() {
                            let copy = val.to_dynamic();
                            op = Some(RemoveElement(copy));
                        }
                    });
                    ui.end_row();
                });

                if i != len - 1 {
                    ui.separator();
                }
            }
            let Some(TypeInfo::Set(set_info)) = set.get_represented_type_info() else {
                return;
            };
            let value_type = set_info.value_ty();
            let (new_op, new_changed) =
                self.ui_to_insert_set_element_with_options(value_type, ui, id, options);
            if new_op.is_some() {
                op = new_op;
            }
            changed |= new_changed;

            ui.end_row();

            let error_id = id.with("error");

            // Respond to control interaction
            if let Some(op) = op {
                changed |= self.respond_to_set_op(set, &op);
            }

            let error = ui.data_mut(|data| *data.get_temp_mut_or_default::<bool>(error_id));
            if error {
                errors::no_default_value(ui, set_info.type_path());
            }
            if ui.input(|input| input.pointer.any_down()) {
                ui.data_mut(|data| data.insert_temp::<bool>(error_id, false));
            }
        });

        changed
    }

    #[must_use]
    fn ui_to_insert_set_element_with_options(
        &mut self,
        value_type: bevy_reflect::Type,
        ui: &mut egui::Ui,
        id: egui::Id,
        options: &dyn Any,
    ) -> (Option<SetOp>, bool) {
        let mut changed = false;
        let mut op = None;
        ui.vertical(|ui| {
            ui.label("New element");
            let set_draft_id = id.with("set_draft");
            let draft_clone = ui.data_mut(|data| {
                data.get_temp_mut_or_default::<Option<SetDraftElement>>(set_draft_id)
                    .to_owned()
            });
            ui.end_row();
            match draft_clone {
                None => {
                    // If no draft element exists, show a button to create one.
                    if add_button(ui).clicked() {
                        // Insert a temporary 'draft' value into UI state, once inserted, we cannot modify it.
                        let maybe_default = self
                            .get_default_value_for(value_type.id())
                            .map(|v| SetDraftElement(v.into_partial_reflect()));
                        if maybe_default.is_some() {
                            ui.data_mut(|data| data.insert_temp(set_draft_id, maybe_default));
                        }
                    }
                    ui.end_row();
                }
                Some(SetDraftElement(mut v)) => {
                    ui.end_row();
                    // Show controls for editing our draft element.
                    // FIXME: is the id passed here correct?
                    let value_changed =
                        self.ui_for_reflect_with_options(v.as_mut(), ui, id, options);
                    // If the clone changed, update the data in UI state.
                    if value_changed {
                        let next_draft = SetDraftElement(v);
                        ui.data_mut(|data| data.insert_temp(set_draft_id, Some(next_draft)));
                    }
                    // Show controls to insert the draft into the set, or remove it.
                    if ui.button("Insert").clicked() {
                        let draft = ui
                            .data_mut(|data| data.get_temp::<Option<SetDraftElement>>(set_draft_id))
                            .flatten();
                        if let Some(draft) = draft {
                            op = Some(SetOp::AddElement(draft.0));
                            ui.data_mut(|data| data.remove_by_type::<Option<SetDraftElement>>());
                        }
                        changed = true;
                    }
                    if ui.button("Cancel").clicked() {
                        ui.data_mut(|data| data.remove_by_type::<Option<SetDraftElement>>());
                        changed = true;
                    }
                    ui.end_row();
                }
            }
        });

        (op, changed)
    }

    fn ui_for_set_readonly(
        &mut self,
        set: &dyn Set,
        ui: &mut egui::Ui,
        id: egui::Id,
        options: &dyn Any,
    ) {
        let len = set.len();
        ui.vertical(|ui| {
            for (i, val) in set.iter().enumerate() {
                ui.horizontal_top(|ui| {
                    self.ui_for_reflect_readonly_with_options(val, ui, id.with(i), options)
                });

                if i != len - 1 {
                    ui.separator();
                }
            }
        });
    }

    fn ui_for_set_many(
        &mut self,
        info: &SetInfo,
        ui: &mut egui::Ui,
        id: egui::Id,
        options: &dyn Any,
        values: &mut [&mut dyn PartialReflect],
        projector: impl ProjectorReflect,
    ) -> bool {
        use SetOp::*;
        let mut changed = false;

        let same_len =
            iter_all_eq(
                values
                    .iter_mut()
                    .map(|value| match projector(*value).reflect_mut() {
                        ReflectMut::List(l) => l.len(),
                        _ => unreachable!(),
                    }),
            );

        let Some(len) = same_len else {
            ui.label("lists have different sizes, cannot multiedit");
            return changed;
        };

        ui.vertical(|ui| {
            let mut op = None;

            if len == 0 {
                ui_for_empty_set(ui)
            }

            let set0 = match projector(values[0]).reflect_mut() {
                ReflectMut::Set(set) => set,
                _ => unreachable!(),
            };
            let Some(TypeInfo::Set(set_info)) = set0.get_represented_type_info() else {
                return;
            };
            let value_type = set_info.value_ty();
            let reflected_values: Vec<Box<dyn PartialReflect>> =
                set0.iter().map(|v| v.to_dynamic()).collect();

            for (i, value_to_check) in reflected_values.iter().enumerate() {
                let value_type_id = (**value_to_check).type_id();
                egui::Grid::new((value_type_id, i)).show(ui, |ui| {
                    // Do all sets contain this value ?
                    if len == 1
                        || values[1..].iter_mut().all(|set_to_compare| {
                            let set_to_compare = match projector(*set_to_compare).reflect_mut() {
                                ReflectMut::Set(set) => set,
                                _ => unreachable!(),
                            };
                            set_to_compare.iter().any(|value| {
                                value.reflect_partial_eq(value_to_check.borrow()) == Some(true)
                            })
                        })
                    {
                        // All sets contain this value: Show value
                        ui.horizontal_top(|ui| {
                            self.ui_for_reflect_readonly_with_options(
                                value_to_check.borrow(),
                                ui,
                                // FIXME: is the id passed here correct?
                                id.with(i),
                                options,
                            );
                        });
                        ui.horizontal_top(|ui| {
                            if remove_button(ui).on_hover_text("Remove element").clicked() {
                                let copy = value_to_check.to_dynamic();
                                op = Some(RemoveElement(copy));
                            }
                        });
                    } else {
                        ui.label("Different values");
                    }

                    ui.end_row();
                });
                if i != len - 1 {
                    ui.separator();
                }
            }
            let (op, new_changed) =
                self.ui_to_insert_set_element_with_options(value_type, ui, id, options);
            changed |= new_changed;

            ui.end_row();

            let error_id = id.with("error");
            let error = ui.data_mut(|data| *data.get_temp_mut_or_default::<bool>(error_id));
            if error {
                errors::no_default_value(ui, info.type_path());
            }
            if ui.input(|input| input.pointer.any_down()) {
                ui.data_mut(|data| data.insert_temp::<bool>(error_id, false));
            }
            if let Some(op) = op {
                let sets = values
                    .iter_mut()
                    .map(|l| match projector(*l).reflect_mut() {
                        ReflectMut::Set(list) => list,
                        _ => unreachable!(),
                    });
                changed |= self.respond_to_sets_op(sets, op);
            }
        });

        changed
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
                ui.horizontal_top(|ui| {
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
                ui.horizontal_top(|ui| {
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
        let Some(type_info) = value.get_represented_type_info() else {
            ui.label("Unrepresentable");
            return false;
        };
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
                                #[cfg(feature = "documentation")]
                                let field_docs = type_info.variant_at(variant_index).and_then(
                                    |info| match info {
                                        VariantInfo::Struct(info) => info.field_at(i)?.docs(),
                                        _ => None,
                                    },
                                );

                                let _response = if let Some(name) = value.name_at(i) {
                                    ui.label(name)
                                } else {
                                    ui.label(i.to_string())
                                };
                                #[cfg(feature = "documentation")]
                                show_docs(_response, field_docs);
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
        values: &mut [&mut dyn PartialReflect],
        projector: &dyn ProjectorReflect,
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

                            let mut variants_across: Vec<&mut dyn PartialReflect> = values
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
                                        field.type_path(),
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
                                        field.type_path(),
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

        ui.horizontal_top(|ui| {
            egui::ComboBox::new(id.with("select"), "")
                .selected_text(info.variant_names()[active_variant_idx])
                .show_ui(ui, |ui| {
                    for (i, variant) in info.iter().enumerate() {
                        let variant_name = variant.name();
                        let is_active_variant = i == active_variant_idx;

                        let variant_is_constructable =
                            variant_constructable(self.type_registry, variant);

                        ui.add_enabled_ui(variant_is_constructable.is_ok(), |ui| {
                            let mut variant_label_response =
                                ui.selectable_label(is_active_variant, variant_name);

                            if let Err(fields) = variant_is_constructable {
                                variant_label_response = variant_label_response
                                    .on_disabled_hover_ui(|ui| {
                                        errors::unconstructable_variant(
                                            ui,
                                            info.type_path(),
                                            variant_name,
                                            &fields,
                                        );
                                    });
                            }

                            /*let res = variant_label_response.on_hover_ui(|ui| {
                                if !unconstructable_variants.is_empty() {
                                    errors::unconstructable_variants(
                                        ui,
                                        info.type_name(),
                                        &unconstructable_variants,
                                    );
                                }
                            });*/

                            if variant_label_response.clicked() {
                                if let Ok(dynamic_enum) =
                                    self.construct_default_variant(variant, ui)
                                {
                                    changed_variant = Some((i, dynamic_enum));
                                };
                            }
                        });
                    }

                    false
                });
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
        value: &mut dyn PartialReflect,
        ui: &mut egui::Ui,
        _id: egui::Id,
        _options: &dyn Any,
    ) -> bool {
        errors::reflect_value_no_impl(ui, value.reflect_short_type_path());
        false
    }

    fn ui_for_value_readonly(
        &mut self,
        value: &dyn PartialReflect,
        ui: &mut egui::Ui,
        _id: egui::Id,
        _options: &dyn Any,
    ) {
        errors::reflect_value_no_impl(ui, value.reflect_short_type_path());
    }

    fn ui_for_value_many(
        &mut self,
        info: &OpaqueInfo,
        ui: &mut egui::Ui,
        _id: egui::Id,
        _options: &dyn Any,
    ) -> bool {
        errors::reflect_value_no_impl(ui, info.type_path());
        false
    }
}

impl<'a, 'c> InspectorUi<'a, 'c> {
    pub fn reborrow<'s>(&'s mut self) -> InspectorUi<'s, 'c> {
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
    ) -> Result<DynamicEnum, ()> {
        let dynamic_variant = match variant {
            VariantInfo::Struct(struct_info) => {
                let mut dynamic_struct = DynamicStruct::default();
                for field in struct_info.iter() {
                    let field_default_value = match self.get_default_value_for(field.type_id()) {
                        Some(value) => value,
                        None => {
                            errors::no_default_value(ui, field.type_path());
                            return Err(());
                        }
                    };
                    dynamic_struct.insert_boxed(field.name(), field_default_value.to_dynamic());
                }
                DynamicVariant::Struct(dynamic_struct)
            }
            VariantInfo::Tuple(tuple_info) => {
                let mut dynamic_tuple = DynamicTuple::default();
                for field in tuple_info.iter() {
                    let field_default_value = match self.get_default_value_for(field.type_id()) {
                        Some(value) => value,
                        None => {
                            errors::no_default_value(ui, field.type_path());
                            return Err(());
                        }
                    };
                    dynamic_tuple.insert_boxed(field_default_value.to_dynamic());
                }
                DynamicVariant::Tuple(dynamic_tuple)
            }
            VariantInfo::Unit(_) => DynamicVariant::Unit,
        };
        let dynamic_enum = DynamicEnum::new(variant.name(), dynamic_variant);
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

fn variant_constructable<'a>(
    type_registry: &TypeRegistry,
    variant: &'a VariantInfo,
) -> Result<(), Vec<&'a str>> {
    let type_id_is_constructable = |type_id: TypeId| {
        type_registry
            .get_type_data::<ReflectDefault>(type_id)
            .is_some()
    };

    let unconstructable_fields: Vec<&'a str> = match variant {
        VariantInfo::Struct(variant) => variant
            .iter()
            .filter_map(|field| {
                (!type_id_is_constructable(field.type_id())).then_some(field.type_path())
            })
            .collect(),
        VariantInfo::Tuple(variant) => variant
            .iter()
            .filter_map(|field| {
                (!type_id_is_constructable(field.type_id())).then_some(field.type_path())
            })
            .collect(),
        VariantInfo::Unit(_) => return Ok(()),
    };

    if unconstructable_fields.is_empty() {
        Ok(())
    } else {
        Err(unconstructable_fields)
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
