//! Utilities for splitting the world into two disjoint views for safe access

use bevy_ecs::{change_detection::MutUntyped, component::ComponentId, prelude::*};
use bevy_reflect::{Reflect, ReflectFromPtr, TypeRegistry};
use smallvec::SmallVec;
use std::any::TypeId;

use crate::utils;

/// Splits the [`World`] into to disjoint views, a [`NoResourceRefsWorld`] which can only be used for component accesses,
/// and a [`OnlyResourceAccessWorld`] can only be used to access resources.
///
/// The `except_resource` specifies a resource, that **can** be used in the [`NoResourceRefsWorld`], and **cannot** be used in the [`OnlyResourceAccessWorld`].
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
            except_resources: match except_resource {
                Some(id) => smallvec::smallvec![id],
                None => SmallVec::new(),
            },
        },
    )
}

pub enum Error {
    ResourceDoesNotExist(TypeId),
    NoTypeRegistration(TypeId),
    NoTypeData(TypeId, &'static str),
}

/// This view into the world must not be used to access any resources, except for the resource [`NoResourceRefsWorld::allows_access_to`] returns true for.
///
/// Components may be accessed.
pub struct NoResourceRefsWorld<'a> {
    world: &'a World,
    except_resource: Option<TypeId>,
}
impl<'a> NoResourceRefsWorld<'a> {
    /// # Safety
    /// This view into the world must not be used to access any resources, except for the resource [`NoResourceRefsWorld::allows_access_to`] returns true for.
    pub unsafe fn get(&self) -> &World {
        self.world
    }

    pub fn allows_access_to(&self, type_id: TypeId) -> bool {
        self.except_resource.map_or(false, |allow| allow == type_id)
    }

    pub fn get_entity_component(
        &mut self,
        entity: Entity,
        component_id: ComponentId,
    ) -> Option<MutUntyped<'_>> {
        // SAFETY: The NoResourceRefsWorld may access components, and we take &mut self so the return is unique
        unsafe {
            self.world
                .entity(entity)
                .get_unchecked_mut_by_id(component_id)
        }
    }

    /// Panics if component does not exist or type_id has no associated ComponentId
    pub fn get_entity_component_reflect(
        &mut self,
        entity: Entity,
        type_id: TypeId,
        type_registry: &TypeRegistry,
    ) -> Result<(&'_ mut dyn Reflect, impl FnOnce() + '_), Error> {
        let component_id = self.world.components().get_id(type_id).unwrap();
        let value = self.get_entity_component(entity, component_id).unwrap();

        // SAFETY: value matches type_id
        unsafe { mut_untyped_to_reflect(value, type_registry, type_id) }
    }

    pub fn get_resource_mut_by_id(&mut self, type_id: TypeId) -> Result<MutUntyped<'a>, Error> {
        assert!(self.allows_access_to(type_id));
        // SAFETY: we only access type_id for which we have access to
        let world = self.world;

        let component_id = world
            .components()
            .get_resource_id(type_id)
            .ok_or(Error::ResourceDoesNotExist(type_id))?;
        // SAFETY: this NoResourceRefs world is the only world allowed to access this resource,
        // and we borrow &mut self
        let value = unsafe {
            world
                .get_resource_unchecked_mut_by_id(component_id)
                .ok_or(Error::ResourceDoesNotExist(type_id))?
        };

        Ok(value)
    }

    pub fn get_resource_reflect_mut_by_id(
        &mut self,
        type_id: TypeId,
        type_registry: &TypeRegistry,
    ) -> Result<(&'a mut dyn Reflect, impl FnOnce() + 'a), Error> {
        let resource = self.get_resource_mut_by_id(type_id)?;
        // SAFETY: resource has type type_id
        let (value, set_changed) =
            unsafe { mut_untyped_to_reflect(resource, type_registry, type_id)? };
        Ok((value, set_changed))
    }
}

/// This view into the world may only be used to access resources (possibly mutably), except for the one [`OnlyResourceAccessWorld::forbids_access_to`] returns true for.
pub struct OnlyResourceAccessWorld<'a> {
    world: &'a World,
    except_resources: SmallVec<[TypeId; 2]>,
}
impl<'a> OnlyResourceAccessWorld<'a> {
    /// # Safety
    /// This view into the world may only be used to access resources (possibly mutably), except for the one [`OnlyResourceAccessWorld::forbids_access_to`] returns true for.
    pub unsafe fn get(&self) -> &World {
        self.world
    }

    pub fn forbids_access_to(&self, type_id: TypeId) -> bool {
        self.except_resources.contains(&type_id)
    }

    /// # Safety
    /// While this new more restricted world is used, the only resource that the current world may have access to is
    /// the newly forbidden one.
    pub unsafe fn with_more_restriction(
        &self,
        forbid_resource: TypeId,
    ) -> OnlyResourceAccessWorld<'a> {
        let mut except_resources = self.except_resources.clone();
        except_resources.push(forbid_resource);
        OnlyResourceAccessWorld {
            world: self.world,
            except_resources,
        }
    }

    pub fn get_resource_mut<R: Resource>(&mut self) -> Option<Mut<'a, R>> {
        // SAFETY: &mut self, get_resource_unchecked_mut asserts !forbids_access_to
        unsafe { self.get_resource_unchecked_mut::<R>() }
    }

    pub fn get_two_resources_mut<R1: Resource, R2: Resource>(
        &mut self,
    ) -> (Option<Mut<'a, R1>>, Option<Mut<'a, R2>>) {
        assert_ne!(TypeId::of::<R1>(), TypeId::of::<R2>());
        // SAFETY: &mut self, get_resource_unchecked_mut asserts !forbids_access_to, R1!=R2
        let r1 = unsafe { self.get_resource_unchecked_mut::<R1>() };
        // SAFETY: &mut self, get_resource_unchecked_mut asserts !forbids_access_to, R1!=R2
        let r2 = unsafe { self.get_resource_unchecked_mut::<R2>() };
        (r1, r2)
    }

    /// # Safety
    /// This will allow aliased mutable access to the given resource type. The caller must ensure
    /// that there is either only one mutable access or multiple immutable accesses at a time.
    unsafe fn get_resource_unchecked_mut<R: Resource>(&self) -> Option<Mut<'a, R>> {
        assert!(!self.forbids_access_to(TypeId::of::<R>()));
        // SAFETY: we only access type_id for which we have access to
        let world = self.world;

        // SAFETY: deferred to caller
        unsafe { world.get_resource_unchecked_mut::<R>() }
    }
}

// SAFETY: MutUntyped is of type with `type_id`
unsafe fn mut_untyped_to_reflect<'a>(
    resource: MutUntyped<'a>,
    type_registry: &TypeRegistry,
    type_id: TypeId,
) -> Result<(&'a mut dyn Reflect, impl FnOnce() + 'a), Error> {
    let registration = type_registry
        .get(type_id)
        .ok_or(Error::NoTypeRegistration(type_id))?;
    let reflect_from_ptr = registration
        .data::<ReflectFromPtr>()
        .ok_or(Error::NoTypeData(type_id, "ReflectFromPtr"))?;

    let (ptr, set_changed) = utils::mut_untyped_split(resource);
    assert_eq!(reflect_from_ptr.type_id(), type_id);
    let value = unsafe { reflect_from_ptr.as_reflect_ptr_mut(ptr) };

    Ok((value, set_changed))
}
