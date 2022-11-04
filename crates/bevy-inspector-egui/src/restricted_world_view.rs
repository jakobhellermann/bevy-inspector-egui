//! A view into the world which may only access certain resources and components

use std::any::TypeId;

use bevy_ecs::{change_detection::MutUntyped, prelude::*};
use bevy_reflect::{Reflect, ReflectFromPtr, TypeRegistry};
use smallvec::{smallvec, SmallVec};

pub enum Error {
    NoAccessToResource(TypeId),
    NoAccessToComponent(EntityComponent),

    ResourceDoesNotExist(TypeId),
    ComponentDoesNotExist(EntityComponent),
    NoComponentId(TypeId),
    NoTypeRegistration(TypeId),
    NoTypeData(TypeId, &'static str),
}

type EntityComponent = (Entity, TypeId);

/// A view into the world which may only access certain resources and components. A restricted form of [`&mut World`](bevy_ecs::world::World).
///
/// Can be used to access a value, and give out the remaining "&mut World" somewhere else.
///
/// # Example usage
///
/// ```no_run
/// use bevy_ecs::prelude::*;
/// use std::any::TypeId;
/// use bevy_inspector_egui::restricted_world_view::RestrictedWorldView;
/// # use bevy_asset::Assets;
/// # use bevy_pbr::StandardMaterial;
///
/// let mut world = World::new();
/// let mut world = RestrictedWorldView::new(&mut world);
///
/// let (mut materials, world) = world.split_off_resource(TypeId::of::<Assets<StandardMaterial>>());
/// let materials = materials.get_resource_mut::<Assets<StandardMaterial>>();
///
/// pass_somewhere_else(world);
/// # fn pass_somewhere_else(_: RestrictedWorldView) {}
/// ```
pub struct RestrictedWorldView<'w> {
    world: &'w World,

    // INVARIANT: if `allowed_resources` contains `T` || `unmentioned_allowed`, `world` may be used to access `T` mutably
    allowed_resources: SmallVec<[TypeId; 2]>,
    // INVARIANT: if `allowed_components` contains `(e, c)` || `unmentioned_allowed`, `world` may be used to access component `c` on entity `e` mutably
    allowed_components: SmallVec<[EntityComponent; 1]>,
    unmentioned_allowed: bool,
}

/// Fundamental methods for working with a [`RestrictedWorldView`]
impl<'w> RestrictedWorldView<'w> {
    /// Create a new [`RestrictedWorldView`] with permission to access everything.
    pub fn new(world: &'w mut World) -> RestrictedWorldView<'w> {
        // INVARIANTS: `world` is `&mut` so we have access to everything
        RestrictedWorldView {
            world,
            allowed_resources: SmallVec::new(),
            allowed_components: SmallVec::new(),
            unmentioned_allowed: true,
        }
    }

    /// Get a reference to the inner [`World`].
    ///
    /// # Safety
    /// The returned world reference may only be used to access (mutably or immutably) resources and components
    /// that [`RestrictedWorldView::allows_access_to_resource`] and [`RestrictedWorldView::allows_access_to_component`] return `true` for.
    pub unsafe fn get(&self) -> &'_ World {
        self.world
    }

    /// Whether the resource with the given [`TypeId`] may be accessed from this world view
    pub fn allows_access_to_resource(&self, type_id: TypeId) -> bool {
        self.allowed_resources.contains(&type_id) || self.unmentioned_allowed
    }
    /// Whether the given component at the entity may be accessed from this world view
    pub fn allows_access_to_component(&self, component: EntityComponent) -> bool {
        self.allowed_components.contains(&component) || self.unmentioned_allowed
    }

    /// Splits this view into one view that only has access the the resource `resource` (`.0`), and the rest (`.1`).
    pub fn split_off_resource(
        &mut self,
        resource: TypeId,
    ) -> (RestrictedWorldView<'_>, RestrictedWorldView<'_>) {
        assert!(self.allows_access_to_resource(resource));

        // INVARIANTS: `self` had `resource` access, so `split` has access if we remove it from `self`
        let split = RestrictedWorldView {
            world: self.world,
            allowed_resources: smallvec![resource],
            allowed_components: SmallVec::new(),
            unmentioned_allowed: false,
        };
        let rest = RestrictedWorldView {
            world: self.world,
            allowed_resources: {
                let mut allowed_resources = self.allowed_resources.clone();
                allowed_resources.retain(|e| *e != resource);
                allowed_resources
            },
            allowed_components: self.allowed_components.clone(),
            unmentioned_allowed: self.unmentioned_allowed,
        };

        (split, rest)
    }

    /// Like [`RestrictedWorldView::split_off_resource`], but takes `self` and returns `'w` lifetimes.
    pub fn split_off_resource_typed<R: Resource>(
        self,
    ) -> Option<(Mut<'w, R>, RestrictedWorldView<'w>)> {
        let type_id = TypeId::of::<R>();
        assert!(self.allows_access_to_resource(type_id));

        // SAFETY: `self` had `R` access, so we have unique access if we remove it from `self`
        let resource = unsafe { self.world.get_resource_unchecked_mut::<R>()? };

        let rest = RestrictedWorldView {
            world: self.world,
            allowed_resources: {
                let mut allowed_resources = self.allowed_resources.clone();
                allowed_resources.retain(|e| *e != type_id);
                allowed_resources
            },
            allowed_components: self.allowed_components.clone(),
            unmentioned_allowed: self.unmentioned_allowed,
        };

        Some((resource, rest))
    }

    /// Splits this view into one view that only has access the the component `component.1` at the entity `component.0` (`.0`), and the rest (`.1`).
    pub fn split_off_component(
        &mut self,
        component: EntityComponent,
    ) -> (RestrictedWorldView<'_>, RestrictedWorldView<'_>) {
        assert!(self.allows_access_to_component(component));

        // INVARIANTS: `self` had `component` access, so `split` has access if we remove it from `self`
        let split = RestrictedWorldView {
            world: self.world,
            allowed_resources: SmallVec::new(),
            allowed_components: smallvec![component],
            unmentioned_allowed: false,
        };
        let rest = RestrictedWorldView {
            world: self.world,
            allowed_resources: self.allowed_resources.clone(),
            allowed_components: {
                let mut allowed_components = self.allowed_components.clone();
                allowed_components.retain(|e| *e != component);
                allowed_components
            },
            unmentioned_allowed: self.unmentioned_allowed,
        };

        (split, rest)
    }
}

/// Some safe methods for getting values out of the [`RestrictedWorldView`].
/// Also has some methods for getting values in their [`Reflect`] form.
impl<'w> RestrictedWorldView<'w> {
    /// Gets a mutable reference to the resource of the given type
    pub fn get_resource_mut<R: Resource>(&mut self) -> Result<Mut<'_, R>, Error> {
        // SAFETY: &mut self
        unsafe { self.get_resource_unchecked_mut() }
    }

    /// Gets mutable reference to two resources. Panics if `R1 = R2`.
    pub fn get_two_resources_mut<R1: Resource, R2: Resource>(
        &mut self,
    ) -> (Result<Mut<'_, R1>, Error>, Result<Mut<'_, R2>, Error>) {
        assert_ne!(TypeId::of::<R1>(), TypeId::of::<R2>());
        // SAFETY: &mut self, R1!=R2
        let r1 = unsafe { self.get_resource_unchecked_mut::<R1>() };
        // SAFETY: &mut self, R1!=R2
        let r2 = unsafe { self.get_resource_unchecked_mut::<R2>() };

        (r1, r2)
    }

    /// # Safety
    /// This method does validate that we have access to `R`, but takes `&self`
    /// and as such doesn't check unique access.
    unsafe fn get_resource_unchecked_mut<R: Resource>(&self) -> Result<Mut<'_, R>, Error> {
        let type_id = TypeId::of::<R>();
        if !self.allows_access_to_resource(type_id) {
            return Err(Error::NoAccessToResource(type_id));
        }

        // SAFETY: we have access to `type_id` and borrow `&mut self`
        let value = unsafe {
            self.world
                .get_resource_unchecked_mut::<R>()
                .ok_or(Error::ResourceDoesNotExist(type_id))?
        };

        Ok(value)
    }

    /// Gets a mutable reference in form of a [`&mut dyn Reflect`](bevy_reflect::Reflect) to the resource given by `type_id`.
    ///
    /// Returns an error if the type does not register [`Reflect`] or [`ReflectResource`].
    ///
    /// Also returns a `impl FnOnce()` to mark the value as changed.
    pub fn get_resource_reflect_mut_by_id(
        &mut self,
        type_id: TypeId,
        type_registry: &TypeRegistry,
    ) -> Result<(&'_ mut dyn Reflect, impl FnOnce() + '_), Error> {
        if !self.allows_access_to_resource(type_id) {
            return Err(Error::NoAccessToResource(type_id));
        }

        let component_id = self
            .world
            .components()
            .get_resource_id(type_id)
            .ok_or(Error::ResourceDoesNotExist(type_id))?;

        // SAFETY: we have access to `type_id` and borrow `&mut self`
        let value = unsafe {
            self.world
                .get_resource_unchecked_mut_by_id(component_id)
                .ok_or(Error::ResourceDoesNotExist(type_id))?
        };

        // SAFETY: value is of type type_id
        let value = unsafe { mut_untyped_to_reflect(value, type_registry, type_id)? };

        Ok(value)
    }

    /// Gets a mutable reference in form of a [`&mut dyn Reflect`](bevy_reflect::Reflect) to a component at an entity.
    ///
    /// Returns an error if the type does not register [`Reflect`] or [`ReflectResource`].
    ///
    /// Also returns a `impl FnOnce()` to mark the value as changed.
    pub fn get_entity_component_reflect(
        &mut self,
        entity: Entity,
        component: TypeId,
        type_registry: &TypeRegistry,
    ) -> Result<(&'_ mut dyn Reflect, impl FnOnce() + '_), Error> {
        if !self.allows_access_to_component((entity, component)) {
            return Err(Error::NoAccessToComponent((entity, component)));
        }

        let component_id = self
            .world
            .components()
            .get_id(component)
            .ok_or(Error::NoComponentId(component))?;

        // SAFETY: we have access to (entity, component) and borrow `&mut self`
        let value = unsafe {
            self.world
                .entity(entity)
                .get_unchecked_mut_by_id(component_id)
                .ok_or(Error::ComponentDoesNotExist((entity, component)))?
        };

        // SAFETY: value is of type component
        unsafe { mut_untyped_to_reflect(value, type_registry, component) }
    }
}

// SAFETY: MutUntyped is of type with `type_id`
unsafe fn mut_untyped_to_reflect<'a>(
    value: MutUntyped<'a>,
    type_registry: &TypeRegistry,
    type_id: TypeId,
) -> Result<(&'a mut dyn Reflect, impl FnOnce() + 'a), Error> {
    let registration = type_registry
        .get(type_id)
        .ok_or(Error::NoTypeRegistration(type_id))?;
    let reflect_from_ptr = registration
        .data::<ReflectFromPtr>()
        .ok_or(Error::NoTypeData(type_id, "ReflectFromPtr"))?;

    let (ptr, set_changed) = crate::utils::mut_untyped_split(value);
    assert_eq!(reflect_from_ptr.type_id(), type_id);
    // SAFETY: ptr is of type type_id as required in safety contract, type_id was checked above
    let value = unsafe { reflect_from_ptr.as_reflect_ptr_mut(ptr) };

    Ok((value, set_changed))
}
