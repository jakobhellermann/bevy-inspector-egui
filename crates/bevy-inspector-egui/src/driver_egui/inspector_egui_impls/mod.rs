use super::InspectorUi;
use bevy_reflect::TypeRegistry;
use std::{
    any::{Any, TypeId},
    time::Instant,
};

mod image;
mod std_impls;

type InspectorEguiImplFn = fn(&mut dyn Any, &mut egui::Ui, &dyn Any, InspectorUi<'_, '_>) -> bool;

#[derive(Clone)]
pub struct InspectorEguiImpl {
    f: InspectorEguiImplFn,
}

impl InspectorEguiImpl {
    pub fn new(f: InspectorEguiImplFn) -> Self {
        InspectorEguiImpl { f }
    }

    pub(crate) fn execute<'a, 'c: 'a>(
        &'a self,
        value: &mut dyn Any,
        ui: &mut egui::Ui,
        options: &dyn Any,
        env: InspectorUi<'_, '_>,
    ) -> bool {
        (self.f)(value, ui, options, env)
    }
}

pub fn register_default_impls(type_registry: &mut TypeRegistry) {
    fn add<T: 'static>(type_registry: &mut TypeRegistry, f: InspectorEguiImplFn) {
        type_registry
            .get_mut(TypeId::of::<T>())
            .unwrap()
            .insert(InspectorEguiImpl::new(f));
    }
    add::<f32>(type_registry, std_impls::number_ui_subint::<f32>);
    add::<f64>(type_registry, std_impls::number_ui_subint::<f64>);
    add::<i8>(type_registry, std_impls::number_ui::<i8>);
    add::<i16>(type_registry, std_impls::number_ui::<i16>);
    add::<i32>(type_registry, std_impls::number_ui::<i32>);
    add::<i64>(type_registry, std_impls::number_ui::<i64>);
    add::<isize>(type_registry, std_impls::number_ui::<isize>);
    add::<u8>(type_registry, std_impls::number_ui::<u8>);
    add::<u16>(type_registry, std_impls::number_ui::<u16>);
    add::<u32>(type_registry, std_impls::number_ui::<u32>);
    add::<u64>(type_registry, std_impls::number_ui::<u64>);
    add::<usize>(type_registry, std_impls::number_ui::<usize>);
    add::<bool>(type_registry, std_impls::bool_ui);
    add::<std::time::Duration>(type_registry, std_impls::duration_ui);
    add::<Instant>(type_registry, std_impls::instant_ui);

    add::<bevy_asset::Handle<bevy_render::texture::Image>>(type_registry, image::image_ui);
}
