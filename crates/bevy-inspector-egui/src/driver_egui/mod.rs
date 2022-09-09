use crate::egui_utils::layout_job;
use crate::options::{InspectorOptions, ReflectInspectorOptions, Target};
use bevy_ecs::prelude::World;
use bevy_reflect::{std_traits::ReflectDefault, DynamicStruct};
use bevy_reflect::{
    Array, DynamicEnum, DynamicTuple, DynamicVariant, Enum, List, Map, Reflect, Struct, Tuple,
    TupleStruct, TypeInfo, TypeRegistry, VariantInfo,
};
use egui::{FontId, Grid};
use std::any::{Any, TypeId};
use std::borrow::Cow;

mod inspector_egui_overrides;

pub use inspector_egui_overrides::InspectorEguiOverrides;

pub fn ui_for_reflect<'a>(
    type_registry: &'a TypeRegistry,
    egui_overrides: &'a InspectorEguiOverrides,
    context: &'a mut Context<'a>,
    value: &mut dyn Reflect,
    ui: &mut egui::Ui,
    options: &dyn Any,
) {
    InspectorUi::new(type_registry, egui_overrides, context).ui_for_reflect_with_options(
        value,
        ui,
        egui::Id::null(),
        options,
    );
}

pub fn split_world_permission<'a>(
    world: &'a mut World,
    except_resource: Option<TypeId>,
) -> (NoResourceRefsWorld<'a>, OnlyResourceAccessWorld<'a>) {
    (
        NoResourceRefsWorld {
            world,
            except_resource,
        },
        OnlyResourceAccessWorld {
            world,
            except_resource,
        },
    )
}

pub struct NoResourceRefsWorld<'a> {
    world: &'a World,
    except_resource: Option<TypeId>,
}
impl<'a> NoResourceRefsWorld<'a> {
    /// # Safety
    /// Any usages of the world must not keep resources alive around calls having access to the [`OnlyResourceAccessWorld`], except for the resource with the type id returned by `except`.
    pub unsafe fn get(&self) -> &World {
        self.world
    }

    pub fn except_resource(&self) -> Option<TypeId> {
        self.except_resource
    }
}
pub struct OnlyResourceAccessWorld<'a> {
    world: &'a World,
    except_resource: Option<TypeId>,
}
impl<'a> OnlyResourceAccessWorld<'a> {
    /// # Safety
    /// The returned world must only be used to access resources (possibly mutably), but it may not access the resource with the type id returned by `except`.
    pub unsafe fn get(&self) -> &World {
        self.world
    }

    pub fn except_resource(&self) -> Option<TypeId> {
        self.except_resource
    }
}

pub struct Context<'a> {
    pub world: Option<OnlyResourceAccessWorld<'a>>,
}

pub struct InspectorUi<'a, 'c> {
    type_registry: &'a TypeRegistry,
    egui_overrides: &'a InspectorEguiOverrides,
    context: &'a mut Context<'c>,
}

impl<'a, 'c> InspectorUi<'a, 'c> {
    pub fn new(
        type_registry: &'a TypeRegistry,
        egui_overrides: &'a InspectorEguiOverrides,
        context: &'a mut Context<'c>,
    ) -> Self {
        Self {
            type_registry,
            egui_overrides,
            context,
        }
    }

    /// Draws the inspector UI for the given value.
    pub fn ui_for_reflect(
        &mut self,
        value: &mut dyn Reflect,
        ui: &mut egui::Ui,
        id: egui::Id,
    ) -> bool {
        self.ui_for_reflect_with_options(value, ui, id, &())
    }

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

        if let Some(changed) = self.egui_overrides.try_execute_mut(
            value.as_any_mut(),
            ui,
            options,
            self.context,
            self.type_registry,
        ) {
            return changed;
        }

        match value.reflect_mut() {
            bevy_reflect::ReflectMut::Struct(value) => self.ui_for_struct(value, ui, id, options),
            bevy_reflect::ReflectMut::TupleStruct(value) => {
                self.ui_for_tuple_struct(value, ui, id, options)
            }
            bevy_reflect::ReflectMut::Tuple(value) => self.ui_for_tuple(value, ui, id, options),
            bevy_reflect::ReflectMut::List(value) => self.ui_for_list(value, ui, id, options),
            bevy_reflect::ReflectMut::Array(value) => self.ui_for_array(value, ui, id, options),
            bevy_reflect::ReflectMut::Map(value) => self.ui_for_reflect_map(value, ui, id, options),
            bevy_reflect::ReflectMut::Enum(value) => self.ui_for_enum(value, ui, id, options),
            bevy_reflect::ReflectMut::Value(value) => self.ui_for_value(value, ui, id, options),
        }
    }

    fn ui_for_struct(
        &mut self,
        value: &mut dyn Struct,
        ui: &mut egui::Ui,
        id: egui::Id,
        options: &dyn Any,
    ) -> bool {
        if value.field_len() == 0 {
            return false;
        };

        let mut changed = false;
        ui.vertical_centered(|ui| {
            let grid = Grid::new(id);
            grid.show(ui, |ui| {
                for i in 0..value.field_len() {
                    match value.name_at(i) {
                        Some(name) => ui.label(name),
                        None => ui.label("<missing>"),
                    };
                    if let Some(field) = value.field_at_mut(i) {
                        changed |= self.ui_for_reflect_with_options(
                            field,
                            ui,
                            id.with(i),
                            inspector_options_struct_field(options, i),
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
        &mut self,
        value: &mut dyn TupleStruct,
        ui: &mut egui::Ui,
        id: egui::Id,
        options: &dyn Any,
    ) -> bool {
        if value.field_len() == 0 {
            return false;
        };

        let mut changed = false;
        let grid = Grid::new(id);
        grid.show(ui, |ui| {
            for i in 0..value.field_len() {
                ui.label(i.to_string());
                if let Some(field) = value.field_mut(i) {
                    changed |= self.ui_for_reflect_with_options(
                        field,
                        ui,
                        id.with(i),
                        inspector_options_struct_field(options, i),
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
        &mut self,
        value: &mut dyn Tuple,
        ui: &mut egui::Ui,
        id: egui::Id,
        options: &dyn Any,
    ) -> bool {
        if value.field_len() == 0 {
            return false;
        };

        let mut changed = false;
        let grid = Grid::new(id);
        grid.show(ui, |ui| {
            for i in 0..value.field_len() {
                ui.label(i.to_string());
                if let Some(field) = value.field_mut(i) {
                    changed |= self.ui_for_reflect_with_options(
                        field,
                        ui,
                        id.with(i),
                        inspector_options_struct_field(options, i),
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

    fn ui_for_reflect_map(
        &mut self,
        _value: &mut dyn Map,
        ui: &mut egui::Ui,
        _id: egui::Id,
        _options: &dyn Any,
    ) -> bool {
        ui.label("Map not yet implemented");
        false
    }

    fn ui_for_array(
        &mut self,
        _value: &mut dyn Array,
        ui: &mut egui::Ui,
        _id: egui::Id,
        _options: &dyn Any,
    ) -> bool {
        ui.label("Array not yet implemented");
        false
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
            let (variant_changed, active_variant) =
                self.ui_for_enum_variant_select(id, value, ui, type_info);
            changed |= variant_changed;

            egui::Grid::new(id.with("fields")).show(ui, |ui| {
                for i in 0..value.field_len() {
                    if let Some(name) = value.name_at(i) {
                        ui.label(name);
                    };
                    let field_value = value
                        .field_at_mut(i)
                        .expect("invalid reflect impl: field len");
                    changed |= self.ui_for_reflect_with_options(
                        field_value,
                        ui,
                        id.with(i),
                        inspector_options_enum_variant_field(options, active_variant.clone(), i),
                    );
                    ui.end_row();
                }
            });
        });

        changed
    }

    fn ui_for_enum_variant_select(
        &mut self,
        id: egui::Id,
        value: &mut dyn Enum,
        ui: &mut egui::Ui,
        type_info: &bevy_reflect::EnumInfo,
    ) -> (bool, Cow<'static, str>) {
        let mut active_variant = None;
        let changed = ui
            .horizontal(|ui| {
                let mut unconstructable_variants = Vec::new();
                let response = egui::ComboBox::new(id.with("select"), "")
                    .selected_text(value.variant_name())
                    .show_ui(ui, |ui| {
                        for variant in type_info.iter() {
                            let variant_name = variant.name().as_ref();
                            let is_active_variant = variant_name == value.variant_name();

                            if is_active_variant {
                                active_variant = Some(variant.name().clone())
                            }

                            let variant_is_constructable =
                                is_variant_constructable(self.type_registry, variant);
                            if !variant_is_constructable {
                                unconstructable_variants.push(variant_name);
                            }
                            ui.add_enabled_ui(variant_is_constructable, |ui| {
                                if ui
                                    .selectable_label(is_active_variant, variant_name)
                                    .clicked()
                                {
                                    if let Ok(dynamic_enum) = construct_default_variant(
                                        self.type_registry,
                                        variant,
                                        ui,
                                        value,
                                    ) {
                                        value.apply(&dynamic_enum);
                                    };
                                }
                            });
                        }

                        false
                    });
                if !unconstructable_variants.is_empty() {
                    error_message_unconstructable_variants(
                        ui,
                        value.type_name(),
                        &unconstructable_variants,
                    );
                    return false;
                }
                response.inner.unwrap_or(false)
            })
            .inner;

        (
            changed,
            active_variant.unwrap_or_else(|| Cow::Owned(value.variant_name().to_owned())),
        )
    }

    fn ui_for_value(
        &mut self,
        value: &mut dyn Reflect,
        ui: &mut egui::Ui,
        _id: egui::Id,
        _options: &dyn Any,
    ) -> bool {
        error_message_reflect_value_no_override(ui, value.type_name());
        false
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
    variant: Cow<'static, str>,
    field: usize,
) -> &'a dyn Any {
    options
        .downcast_ref::<InspectorOptions>()
        .and_then(|options| options.get(Target::VariantField(variant, field)))
        .unwrap_or(&())
}

fn get_default_value_for(
    type_registry: &TypeRegistry,
    type_id: TypeId,
) -> Option<Box<dyn Reflect>> {
    let reflect_default = type_registry.get_type_data::<ReflectDefault>(type_id)?;
    Some(reflect_default.default())
}

fn construct_default_variant(
    type_registry: &TypeRegistry,
    variant: &VariantInfo,
    ui: &mut egui::Ui,
    value: &dyn Enum,
) -> Result<DynamicEnum, ()> {
    let dynamic_variant = match variant {
        VariantInfo::Struct(struct_info) => {
            let mut dynamic_struct = DynamicStruct::default();
            for field in struct_info.iter() {
                let field_default_value =
                    match get_default_value_for(type_registry, field.type_id()) {
                        Some(value) => value,
                        None => {
                            error_message_no_default_value(ui, field.type_name());
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
                let field_default_value =
                    match get_default_value_for(type_registry, field.type_id()) {
                        Some(value) => value,
                        None => {
                            error_message_no_default_value(ui, field.type_name());
                            return Err(());
                        }
                    };
                dynamic_tuple.insert_boxed(field_default_value);
            }
            DynamicVariant::Tuple(dynamic_tuple)
        }
        VariantInfo::Unit(_) => DynamicVariant::Unit,
    };
    let dynamic_enum = DynamicEnum::new(value.type_name(), variant.name(), dynamic_variant);
    Ok(dynamic_enum)
}

fn error_message_reflect_value_no_override(ui: &mut egui::Ui, type_name: &str) {
    let job = layout_job(&[
        (FontId::monospace(14.0), type_name),
        (FontId::default(), " is "),
        (FontId::monospace(14.0), "#[reflect_value]"),
        (FontId::default(), ", but not registered on the "),
        (FontId::monospace(14.0), "InspectorEguiOverrides"),
        (FontId::default(), "."),
    ]);

    ui.label(job);
}
fn error_message_no_default_value(ui: &mut egui::Ui, type_name: &str) {
    let job = layout_job(&[
        (FontId::monospace(14.0), type_name),
        (FontId::default(), " has no "),
        (FontId::monospace(14.0), "ReflectDefault"),
        (
            FontId::default(),
            " type data, so no value of it can be constructed.",
        ),
    ]);

    ui.label(job);
}
fn error_message_unconstructable_variants(
    ui: &mut egui::Ui,
    type_name: &str,
    unconstructable_variants: &[&str],
) {
    let mut vec = Vec::with_capacity(2 + unconstructable_variants.len() * 2 + 3);
    vec.extend([
        (FontId::monospace(14.0), type_name),
        (FontId::default(), " has unconstructable variants: "),
    ]);
    vec.extend(unconstructable_variants.iter().flat_map(|variant| {
        [
            (FontId::monospace(14.0), *variant),
            (FontId::default(), ", "),
        ]
    }));
    vec.extend([
        (FontId::default(), "\nyou should register "),
        (FontId::monospace(14.0), "ReflectDefault"),
        (FontId::default(), " for all fields."),
    ]);
    let job = layout_job(&vec);

    ui.label(job);
}
