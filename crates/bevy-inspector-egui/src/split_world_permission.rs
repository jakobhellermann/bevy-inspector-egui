use bevy_ecs::prelude::*;
use smallvec::SmallVec;
use std::any::TypeId;

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

    pub fn allows_access_to(&self, type_id: TypeId) -> bool {
        self.except_resource.map_or(false, |allow| allow == type_id)
    }
}
pub struct OnlyResourceAccessWorld<'a> {
    world: &'a World,
    except_resources: SmallVec<[TypeId; 2]>,
}
impl<'a> OnlyResourceAccessWorld<'a> {
    /// # Safety
    /// The returned world must only be used to access resources (possibly mutably), but it may not access the resource with the type id returned by `except`.
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
}
