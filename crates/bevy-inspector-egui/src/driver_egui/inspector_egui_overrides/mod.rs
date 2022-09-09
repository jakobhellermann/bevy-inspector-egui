use bevy_reflect::TypeRegistry;
use std::time::Instant;

use super::{Context, InspectorUi, ShortCircuitFn};
use std::{
    any::{Any, TypeId},
    collections::HashMap,
    time::Duration,
};

mod image;
mod std_overrides;

type InspectorOverrideFn =
    Box<dyn Fn(&mut dyn Any, &mut egui::Ui, &dyn Any, InspectorUi<'_, '_>) -> bool + Send + Sync>;

pub struct InspectorEguiOverrides {
    fns: HashMap<TypeId, InspectorOverrideFn>,
}

impl Default for InspectorEguiOverrides {
    fn default() -> Self {
        let mut overrides = Self::empty();
        overrides.register::<f32, _>(std_overrides::number_ui_subint);
        overrides.register::<f64, _>(std_overrides::number_ui_subint);
        overrides.register::<i8, _>(std_overrides::number_ui);
        overrides.register::<i16, _>(std_overrides::number_ui);
        overrides.register::<i32, _>(std_overrides::number_ui);
        overrides.register::<i64, _>(std_overrides::number_ui);
        overrides.register::<isize, _>(std_overrides::number_ui);
        overrides.register::<u8, _>(std_overrides::number_ui);
        overrides.register::<u16, _>(std_overrides::number_ui);
        overrides.register::<u32, _>(std_overrides::number_ui);
        overrides.register::<u64, _>(std_overrides::number_ui);
        overrides.register::<usize, _>(std_overrides::number_ui);
        overrides.register::<bool, _>(std_overrides::bool_ui);

        overrides.register::<Duration, _>(std_overrides::duration);
        overrides.register::<Instant, _>(std_overrides::instant);

        overrides.register::<bevy_asset::Handle<bevy_render::texture::Image>, _>(image::image_ui);

        overrides
    }
}

impl InspectorEguiOverrides {
    fn empty() -> InspectorEguiOverrides {
        InspectorEguiOverrides {
            fns: HashMap::new(),
        }
    }

    pub fn register<T: 'static, F>(&mut self, f: F)
    where
        F: Fn(&mut T, &mut egui::Ui, &dyn Any, InspectorUi<'_, '_>) -> bool + Send + Sync + 'static,
    {
        let type_id = TypeId::of::<T>();
        let callback = Box::new(
            move |value: &mut dyn Any,
                  ui: &mut egui::Ui,
                  options: &dyn Any,
                  env: InspectorUi<'_, '_>| {
                let value: &mut T = value.downcast_mut().unwrap();
                f(value, ui, options, env)
            },
        ) as InspectorOverrideFn;
        self.fns.insert(type_id, callback);
    }

    pub(crate) fn try_execute_mut<'a, 'c: 'a>(
        &'a self,
        value: &mut dyn Any,
        ui: &mut egui::Ui,
        options: &dyn Any,
        context: &'a mut Context<'c>,
        type_registry: &'a TypeRegistry,
        short_circuit_fn: ShortCircuitFn,
    ) -> Option<bool> {
        let type_id = Any::type_id(value);
        self.fns.get(&type_id).map(|f| {
            f(
                value,
                ui,
                options,
                InspectorUi::new(type_registry, self, context, Some(short_circuit_fn)),
            )
        })
    }
}
