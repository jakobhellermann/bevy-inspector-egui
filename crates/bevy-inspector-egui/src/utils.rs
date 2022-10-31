use bevy_ecs::change_detection::{DetectChanges, MutUntyped};
use bevy_ecs::ptr::PtrMut;

// workaround for https://github.com/bevyengine/bevy/pull/6430
pub fn mut_untyped_split<'a>(mut mut_untyped: MutUntyped<'a>) -> (PtrMut<'a>, impl FnOnce() + 'a) {
    // bypass_change_detection returns a `&mut PtrMut` which is basically useless, because all its methods take `self`
    let ptr = mut_untyped.bypass_change_detection();
    // SAFETY: this is exactly the same PtrMut, just not in a `&mut`. The old one is no longer accessible
    let ptr = unsafe { PtrMut::new(std::ptr::NonNull::new_unchecked(ptr.as_ptr())) };

    (ptr, move || mut_untyped.set_changed())
}

pub mod guess_entity_name {
    use bevy_core::Name;
    use bevy_ecs::{prelude::*, world::EntityRef};
    use bevy_reflect::TypeRegistry;

    /// Guesses an appropriate entity name like `Light (6)` or falls back to `Entity (8)`
    pub fn entity_name(world: &World, type_registry: &TypeRegistry, entity: Entity) -> String {
        match world.get_entity(entity) {
            Some(entity) => guess_entity_name_inner(world, entity, type_registry),
            None => format!("Entity {} (inexistent)", entity.id()),
        }
    }

    fn guess_entity_name_inner(
        world: &World,
        entity: EntityRef,
        type_registry: &TypeRegistry,
    ) -> String {
        let id = entity.id().id();

        if let Some(name) = entity.get::<Name>() {
            return name.as_str().to_string();
        }

        #[rustfmt::skip]
        let associations = &[
            ("bevy_core_pipeline::core_3d::camera_3d::Camera3d", "Camera3d"),
            ("bevy_core_pipeline::core_2d::camera_2d::Camera2d", "Camera2d"),
            ("bevy_pbr::light::PointLight", "PointLight"),
            ("bevy_pbr::light::DirectionalLight", "DirectionalLight"),
            ("bevy_text::text::Text", "Text"),
            ("bevy_ui::ui_node::Node", "Node"),
            ("bevy_asset::handle::Handle<bevy_pbr::pbr_material::StandardMaterial>", "Pbr Mesh")

        ];

        let type_names = entity.archetype().components().filter_map(|id| {
            let type_id = world.components().get_info(id)?.type_id()?;
            let registration = type_registry.get(type_id)?;
            Some(registration.type_name())
        });

        for component_type in type_names {
            if let Some(name) = associations
                .iter()
                .find_map(|&(name, matches)| (component_type == name).then_some(matches))
            {
                return format!("{} ({:?})", name, id);
            }
        }

        format!("Entity ({:?})", id)
    }
}
