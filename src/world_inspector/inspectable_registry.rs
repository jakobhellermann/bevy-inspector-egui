use crate::{Context, Inspectable};
use bevy::reflect::TypeRegistryInternal;
use bevy::utils::HashMap;
use bevy::{ecs::TypeInfo, prelude::*};
use bevy_egui::egui;
use std::any::TypeId;

type InspectCallback = Box<dyn Fn(*mut u8, &mut egui::Ui, &Resources) -> () + Send + Sync>;

#[allow(missing_debug_implementations)]
/// The `InspectableRegistry` can be used to tell the [`WorldInspectorPlugin`](crate::WorldInspectorPlugin)
/// that a type implements [`Inspectable`](crate::Inspectable).
pub struct InspectableRegistry {
    impls: HashMap<TypeId, InspectCallback>,
}

impl Default for InspectableRegistry {
    fn default() -> Self {
        let mut this = InspectableRegistry {
            impls: HashMap::default(),
        };

        this.register::<std::ops::Range<f32>>();

        this.register::<Transform>();
        this.register::<GlobalTransform>();
        this.register::<Quat>();

        #[cfg(feature = "rapier")]
        {
            this.register::<bevy_rapier3d::rapier::dynamics::MassProperties>();
            this.register::<bevy_rapier3d::rapier::dynamics::RigidBody>();
            this.register::<bevy_rapier3d::physics::RigidBodyHandleComponent>();
        }

        this
    }
}

impl InspectableRegistry {
    /// Register type `T` so that it can be displayed by the [`WorldInspectorPlugin`](crate::WorldInspectorPlugin).
    pub fn register<T: Inspectable + 'static>(&mut self) {
        let type_id = TypeId::of::<T>();
        let callback = Box::new(|ptr: *mut u8, ui: &mut egui::Ui, resources: &Resources| {
            let value: &mut T = unsafe { std::mem::transmute(ptr) };
            value.ui(
                ui,
                <T as Inspectable>::Attributes::default(),
                &Context::new(resources),
            )
        }) as InspectCallback;
        self.impls.insert(type_id, callback);
    }

    pub(crate) fn try_execute(
        &self,
        value: &mut dyn Reflect,
        ui: &mut egui::Ui,
        resources: &Resources,
    ) -> bool {
        if let Some(inspect_callback) = self.impls.get(&value.type_id()) {
            let ptr = value as *mut dyn Reflect as *mut u8;
            inspect_callback(ptr, ui, resources);
            true
        } else {
            false
        }
    }

    pub(crate) fn generate(
        &self,
        world: &World,
        resources: &Resources,
        archetype_index: usize,
        entity_index: usize,
        type_info: &TypeInfo,
        type_registry: &TypeRegistryInternal,
        ui: &mut egui::Ui,
    ) -> bool {
        let archetype = &world.archetypes[archetype_index];

        let ptr = unsafe {
            archetype
                .get_dynamic(type_info.id(), type_info.layout().size(), entity_index)
                .unwrap()
                .as_ptr()
        };

        if let Some(f) = self.impls.get(&type_info.id()) {
            f(ptr, ui, resources);
            return true;
        }

        let success = (|| {
            let registration = type_registry.get(type_info.id())?;
            let reflect_component = registration.data::<ReflectComponent>()?;
            let reflected =
                unsafe { reflect_component.reflect_component_mut(archetype, entity_index) };
            crate::reflect::ui_for_reflect(reflected, ui, &Context::new(resources));
            Some(())
        })();

        success.is_some()
    }
}
