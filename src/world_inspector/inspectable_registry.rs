use crate::{Context, Inspectable};
use bevy::{ecs::ComponentFlags, utils::HashMap};
use bevy::{ecs::Location, reflect::TypeRegistryInternal};
use bevy::{ecs::TypeInfo, prelude::*};
use bevy_egui::egui;
use std::any::TypeId;

type InspectCallback = Box<dyn Fn(*mut u8, &mut egui::Ui, &Context) + Send + Sync>;

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

        this.register::<Color>();
        this.register::<Handle<Texture>>();
        this.register::<Handle<StandardMaterial>>();
        this.register::<Handle<ColorMaterial>>();

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
        let callback = Box::new(|ptr: *mut u8, ui: &mut egui::Ui, context: &Context| {
            let value: &mut T = unsafe { &mut *(ptr as *mut T) };
            value.ui(ui, <T as Inspectable>::Attributes::default(), context)
        }) as InspectCallback;
        self.impls.insert(type_id, callback);
    }

    pub(crate) fn try_execute(
        &self,
        value: &mut dyn Reflect,
        ui: &mut egui::Ui,
        context: &Context,
    ) -> bool {
        if let Some(inspect_callback) = self.impls.get(&value.type_id()) {
            let ptr = value as *mut dyn Reflect as *mut u8;
            inspect_callback(ptr, ui, context);
            true
        } else {
            false
        }
    }

    /// Safety:
    /// The `location` must point to a valid archetype and index,
    /// and the function must have unique access to the components.
    #[allow(unused_unsafe)]
    pub(crate) unsafe fn generate(
        &self,
        world: &World,
        resources: &Resources,
        location: Location,
        type_info: &TypeInfo,
        type_registry: &TypeRegistryInternal,
        ui: &mut egui::Ui,
    ) -> bool {
        let archetype = &world.archetypes[location.archetype as usize];

        unsafe {
            let type_state = archetype.get_type_state(type_info.id()).unwrap();
            let flags = &mut *type_state.component_flags().as_ptr().add(location.index);
            flags.insert(ComponentFlags::MUTATED);
        };

        let ptr = unsafe {
            archetype
                .get_dynamic(type_info.id(), type_info.layout().size(), location.index)
                .unwrap()
                .as_ptr()
        };

        let context = Context::new(world, resources);

        if let Some(f) = self.impls.get(&type_info.id()) {
            f(ptr, ui, &context);
            return true;
        }

        let success = (|| {
            let registration = type_registry.get(type_info.id())?;
            let reflect_component = registration.data::<ReflectComponent>()?;
            let reflected =
                unsafe { reflect_component.reflect_component_mut(archetype, location.index) };
            crate::reflect::ui_for_reflect(reflected, ui, &context);
            Some(())
        })();

        success.is_some()
    }
}
