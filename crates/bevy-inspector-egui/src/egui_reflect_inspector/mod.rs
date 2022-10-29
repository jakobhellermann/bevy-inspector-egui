use crate::inspector_egui_impls::InspectorEguiImpl;
use crate::inspector_options::{InspectorOptions, ReflectInspectorOptions, Target};
use crate::split_world_permission::OnlyResourceAccessWorld;
use bevy_reflect::{std_traits::ReflectDefault, DynamicStruct};
use bevy_reflect::{
    Array, DynamicEnum, DynamicTuple, DynamicVariant, Enum, List, Map, Reflect, Struct, Tuple,
    TupleStruct, TypeInfo, TypeRegistry, VariantInfo,
};
use egui::Grid;
use std::any::{Any, TypeId};
use std::borrow::Cow;

mod errors;

pub fn ui_for_reflect<'a>(
    type_registry: &'a TypeRegistry,
    context: &'a mut Context<'a>,
    short_circuit: Option<ShortCircuitFn>,
    short_circuit_readonly: Option<ShortCircuitFnReadonly>,
    value: &mut dyn Reflect,
    ui: &mut egui::Ui,
    options: &dyn Any,
) {
    InspectorUi::new(
        type_registry,
        context,
        short_circuit,
        short_circuit_readonly,
    )
    .ui_for_reflect_with_options(value, ui, egui::Id::null(), options);
}

pub struct Context<'a> {
    pub world: Option<OnlyResourceAccessWorld<'a>>,
}

type ShortCircuitFn = fn(
    &mut InspectorUi<'_, '_>,
    value: &mut dyn Reflect,
    ui: &mut egui::Ui,
    id: egui::Id,
    options: &dyn Any,
) -> Option<bool>;
type ShortCircuitFnReadonly = fn(
    &mut InspectorUi<'_, '_>,
    value: &dyn Reflect,
    ui: &mut egui::Ui,
    id: egui::Id,
    options: &dyn Any,
) -> Option<()>;

pub struct InspectorUi<'a, 'c> {
    pub type_registry: &'a TypeRegistry,
    pub context: &'a mut Context<'c>,

    pub short_circuit: ShortCircuitFn,
    pub short_circuit_readonly: ShortCircuitFnReadonly,
}

impl<'a, 'c> InspectorUi<'a, 'c> {
    pub fn new(
        type_registry: &'a TypeRegistry,
        context: &'a mut Context<'c>,
        short_circuit: Option<ShortCircuitFn>,
        short_circuit_readonly: Option<ShortCircuitFnReadonly>,
    ) -> Self {
        Self {
            type_registry,
            context,
            short_circuit: short_circuit.unwrap_or(|_, _, _, _, _| None),
            short_circuit_readonly: short_circuit_readonly.unwrap_or(|_, _, _, _, _| None),
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

    pub fn ui_for_reflect_ref(&mut self, value: &dyn Reflect, ui: &mut egui::Ui, id: egui::Id) {
        self.ui_for_reflect_ref_with_options(value, ui, id, &());
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

        if let Some(s) = self
            .type_registry
            .get_type_data::<InspectorEguiImpl>(Any::type_id(value))
        {
            return s.execute(value.as_any_mut(), ui, options, self.reborrow());
        }

        if let Some(changed) = (self.short_circuit)(self, value, ui, id, options) {
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

    pub fn ui_for_reflect_ref_with_options(
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
            s.execute_readonly(value.as_any(), ui, options, self.reborrow());
            return;
        }

        if let Some(()) = (self.short_circuit_readonly)(self, value, ui, id, options) {
            return;
        }

        match value.reflect_ref() {
            bevy_reflect::ReflectRef::Struct(value) => {
                self.ui_for_struct_ref(value, ui, id, options)
            }
            bevy_reflect::ReflectRef::TupleStruct(value) => {
                self.ui_for_tuple_struct_ref(value, ui, id, options)
            }
            bevy_reflect::ReflectRef::Tuple(value) => self.ui_for_tuple_ref(value, ui, id, options),
            bevy_reflect::ReflectRef::List(value) => self.ui_for_list_ref(value, ui, id, options),
            bevy_reflect::ReflectRef::Array(value) => self.ui_for_array_ref(value, ui, id, options),
            bevy_reflect::ReflectRef::Map(value) => {
                self.ui_for_reflect_map_ref(value, ui, id, options)
            }
            bevy_reflect::ReflectRef::Enum(value) => self.ui_for_enum_ref(value, ui, id, options),
            bevy_reflect::ReflectRef::Value(value) => self.ui_for_value_ref(value, ui, id, options),
        }
    }

    fn ui_for_struct(
        &mut self,
        value: &mut dyn Struct,
        ui: &mut egui::Ui,
        id: egui::Id,
        options: &dyn Any,
    ) -> bool {
        maybe_grid(value.field_len(), ui, id, |ui, label| {
            (0..value.field_len())
                .map(|i| {
                    if label {
                        ui.label(value.name_at(i).unwrap());
                    }
                    let field = value.field_at_mut(i).unwrap();
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

    fn ui_for_struct_ref(
        &mut self,
        value: &dyn Struct,
        ui: &mut egui::Ui,
        id: egui::Id,
        options: &dyn Any,
    ) {
        maybe_grid_ref(value.field_len(), ui, id, |ui, label| {
            for i in 0..value.field_len() {
                if label {
                    ui.label(value.name_at(i).unwrap());
                }
                let field = value.field_at(i).unwrap();
                self.ui_for_reflect_ref_with_options(
                    field,
                    ui,
                    id.with(i),
                    inspector_options_struct_field(options, i),
                );
                ui.end_row();
            }
        })
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

    fn ui_for_tuple_struct_ref(
        &mut self,
        value: &dyn TupleStruct,
        ui: &mut egui::Ui,
        id: egui::Id,
        options: &dyn Any,
    ) {
        maybe_grid_ref(value.field_len(), ui, id, |ui, label| {
            for i in 0..value.field_len() {
                if label {
                    ui.label(i.to_string());
                }
                let field = value.field(i).unwrap();
                self.ui_for_reflect_ref_with_options(
                    field,
                    ui,
                    id.with(i),
                    inspector_options_struct_field(options, i),
                );
                ui.end_row();
            }
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

    fn ui_for_tuple_ref(
        &mut self,
        value: &dyn Tuple,
        ui: &mut egui::Ui,
        id: egui::Id,
        options: &dyn Any,
    ) {
        maybe_grid_ref(value.field_len(), ui, id, |ui, label| {
            for i in 0..value.field_len() {
                if label {
                    ui.label(i.to_string());
                }
                let field = value.field(i).unwrap();
                self.ui_for_reflect_ref_with_options(
                    field,
                    ui,
                    id.with(i),
                    inspector_options_struct_field(options, i),
                );
                ui.end_row();
            }
        });
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

    fn ui_for_list_ref(
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
                    self.ui_for_reflect_ref_with_options(val, ui, id.with(i), options)
                });

                if i != len - 1 {
                    ui.separator();
                }
            }
        });
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
                self.ui_for_reflect_ref(key, ui, id.with(i));
                // TODO: iter mut
                self.ui_for_reflect_ref(value, ui, id.with(i));
                ui.end_row();
            }
        });

        changed
    }

    fn ui_for_reflect_map_ref(
        &mut self,
        map: &dyn Map,
        ui: &mut egui::Ui,
        id: egui::Id,
        _options: &dyn Any,
    ) {
        egui::Grid::new(id).show(ui, |ui| {
            for (i, (key, value)) in map.iter().enumerate() {
                self.ui_for_reflect_ref(key, ui, id.with(i));
                self.ui_for_reflect_ref(value, ui, id.with(i));
                ui.end_row();
            }
        });
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

    fn ui_for_array_ref(
        &mut self,
        _value: &dyn Array,
        ui: &mut egui::Ui,
        _id: egui::Id,
        _options: &dyn Any,
    ) {
        ui.label("Array not yet implemented");
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

            changed |= maybe_grid(value.field_len(), ui, id, |ui, label| {
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
                            inspector_options_enum_variant_field(
                                options,
                                active_variant.clone(),
                                i,
                            ),
                        );
                        ui.end_row();
                        changed
                    })
                    .fold(false, or)
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
        let mut changed = false;
        ui.horizontal(|ui| {
            let mut unconstructable_variants = Vec::new();
            egui::ComboBox::new(id.with("select"), "")
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
                        if !variant_is_constructable && !is_active_variant {
                            unconstructable_variants.push(variant_name);
                        }
                        ui.add_enabled_ui(variant_is_constructable, |ui| {
                            if ui
                                .selectable_label(is_active_variant, variant_name)
                                .clicked()
                            {
                                changed = true;
                                if let Ok(dynamic_enum) =
                                    self.construct_default_variant(variant, ui, value)
                                {
                                    value.apply(&dynamic_enum);
                                };
                            }
                        });
                    }

                    false
                });
            if !unconstructable_variants.is_empty() {
                errors::error_message_unconstructable_variants(
                    ui,
                    value.type_name(),
                    &unconstructable_variants,
                );
            }
        });

        (
            changed,
            active_variant.unwrap_or_else(|| Cow::Owned(value.variant_name().to_owned())),
        )
    }

    fn ui_for_enum_ref(
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

            maybe_grid_ref(value.field_len(), ui, id, |ui, label| {
                for i in 0..value.field_len() {
                    if label {
                        if let Some(name) = value.name_at(i) {
                            ui.label(name);
                        } else {
                            ui.label(i.to_string());
                        }
                    }
                    let field_value = value.field_at(i).expect("invalid reflect impl: field len");
                    self.ui_for_reflect_ref_with_options(
                        field_value,
                        ui,
                        id.with(i),
                        inspector_options_enum_variant_field(
                            options,
                            active_variant.to_owned().into(),
                            i,
                        ),
                    );
                    ui.end_row();
                }
            });
        });
    }

    fn ui_for_value(
        &mut self,
        value: &mut dyn Reflect,
        ui: &mut egui::Ui,
        _id: egui::Id,
        _options: &dyn Any,
    ) -> bool {
        errors::error_message_reflect_value_no_impl(ui, value.type_name());
        false
    }

    fn ui_for_value_ref(
        &mut self,
        value: &dyn Reflect,
        ui: &mut egui::Ui,
        _id: egui::Id,
        _options: &dyn Any,
    ) {
        errors::error_message_reflect_value_no_impl(ui, value.type_name());
    }
}

impl<'a, 'c> InspectorUi<'a, 'c> {
    fn reborrow<'s>(&'s mut self) -> InspectorUi<'s, 'c> {
        InspectorUi {
            type_registry: self.type_registry,
            context: self.context,
            short_circuit: self.short_circuit,
            short_circuit_readonly: self.short_circuit_readonly,
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
        value: &dyn Enum,
    ) -> Result<DynamicEnum, ()> {
        let dynamic_variant = match variant {
            VariantInfo::Struct(struct_info) => {
                let mut dynamic_struct = DynamicStruct::default();
                for field in struct_info.iter() {
                    let field_default_value = match self.get_default_value_for(field.type_id()) {
                        Some(value) => value,
                        None => {
                            errors::error_message_no_default_value(ui, field.type_name());
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
                            errors::error_message_no_default_value(ui, field.type_name());
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
        _ => {
            Grid::new(id)
                .show(ui, |ui| {
                    let changed = f(ui, true);
                    changed
                })
                .inner
        }
    }
}

fn maybe_grid_ref(
    i: usize,
    ui: &mut egui::Ui,
    id: egui::Id,
    mut f: impl FnMut(&mut egui::Ui, bool),
) {
    match i {
        0 => {}
        1 => f(ui, false),
        _ => {
            Grid::new(id).show(ui, |ui| {
                let changed = f(ui, true);
                changed
            });
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
    variant: Cow<'static, str>,
    field: usize,
) -> &'a dyn Any {
    options
        .downcast_ref::<InspectorOptions>()
        .and_then(|options| options.get(Target::VariantField(variant, field)))
        .unwrap_or(&())
}

fn or(a: bool, b: bool) -> bool {
    a || b
}
