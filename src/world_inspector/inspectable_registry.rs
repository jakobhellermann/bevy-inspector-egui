use crate::{Context, Inspectable};
use bevy::asset::HandleId;
use bevy::math::{DVec2, DVec3, DVec4};
use bevy::pbr::{Clusters, CubemapVisibleEntities, VisiblePointLights};
use bevy::render::camera::{DepthCalculation, ScalingMode, WindowOrigin};
use bevy::render::primitives::{CubemapFrusta, Frustum, Plane};
use bevy::render::render_resource::PrimitiveTopology;
use bevy::render::view::VisibleEntities;
use bevy::utils::HashMap;
use bevy::{pbr::AmbientLight, prelude::*};
use bevy_egui::egui;
use std::any::{Any, TypeId};

pub(crate) type InspectCallback =
    Box<dyn Fn(*mut u8, &mut egui::Ui, &mut Context) -> bool + Send + Sync>;

macro_rules! register {
    ($this:ident $($ty:ty),* $(,)?) => {
        $($this.register::<$ty>();)*
        $($this.register::<Option<$ty>>();)*
    };
}

/// The `InspectableRegistry` can be used to tell the [`WorldInspectorPlugin`](crate::WorldInspectorPlugin)
/// how to display a type.
/// ```rust,no_run
/// use bevy::prelude::*;
/// use bevy_inspector_egui::{Inspectable, InspectableRegistry};
///
/// #[derive(Inspectable)]
/// struct CustomType;
///
/// fn main() {
///     let mut app = App::new();
///     let mut registry = app
///         .world
///         .get_resource_mut::<InspectableRegistry>()
///         .unwrap();
///     registry.register::<CustomType>();
/// }
/// ```
pub struct InspectableRegistry {
    pub(crate) impls: HashMap<TypeId, InspectCallback>,
}

impl InspectableRegistry {
    /// Register type `T` so that it can be displayed by the [`WorldInspectorPlugin`](crate::WorldInspectorPlugin).
    pub fn register<T: Inspectable + 'static>(&mut self) {
        self.register_raw::<T, _>(|value, ui, context| {
            value.ui(ui, <T as Inspectable>::Attributes::default(), context)
        });
    }

    /// Registers a type that doesn't need to implement [`Inspectable`](crate::Inspectable)
    pub fn register_raw<T: 'static, F>(&mut self, f: F)
    where
        F: Fn(&mut T, &mut egui::Ui, &mut Context) -> bool + Send + Sync + 'static,
    {
        let type_id = TypeId::of::<T>();
        let callback = Box::new(
            move |ptr: *mut u8, ui: &mut egui::Ui, context: &mut Context| {
                let value: &mut T = unsafe { &mut *(ptr as *mut T) };
                f(value, ui, context)
            },
        ) as InspectCallback;
        self.impls.insert(type_id, callback);
    }

    /// Variant of [`InspectableRegistry::register`] which returns self by-value.
    /// Allows
    /// ```rust,no_run
    /// # use bevy::prelude::*;
    /// # use bevy_inspector_egui::InspectableRegistry;
    /// # #[derive(bevy_inspector_egui::Inspectable)] struct MyType;
    /// App::new()
    ///   .insert_resource(InspectableRegistry::default().with::<MyType>())
    ///   .run();
    /// ```
    #[must_use]
    pub fn with<T: Inspectable + 'static>(mut self) -> Self {
        self.register::<T>();
        self
    }

    /// Try to run the provided inspectable callback and return whether it has changed the value.
    pub fn try_execute(
        &self,
        value: &mut dyn Any,
        ui: &mut egui::Ui,
        context: &mut Context,
    ) -> Result<bool, ()> {
        let type_id = (*value).type_id();
        if let Some(inspect_callback) = self.impls.get(&type_id) {
            // SAFETY: we maintain the invariant that any callback in the hashmap receives a the type with the type_id specified in the key
            let ptr = value as *mut dyn Any as *mut u8;
            let changed = inspect_callback(ptr, ui, context);
            Ok(changed)
        } else {
            Err(())
        }
    }
}

impl Default for InspectableRegistry {
    fn default() -> Self {
        let mut this = InspectableRegistry {
            impls: HashMap::default(),
        };

        // #[reflect_value]
        register!(this Entity, HandleId);
        register!(this IVec2, IVec3, IVec4, UVec2, UVec3, Vec2, DVec2, DVec3, DVec4, Vec3, Vec4, Mat3, Mat4, Quat);
        register!(this u8, u16, u32, u64, usize, i8, i16, i32, i64, isize, bool, f32, f64, String);
        this.register::<Transform>();
        this.register::<GlobalTransform>();

        this.register::<std::ops::Range<f32>>();
        this.register::<std::time::Duration>();

        this.register::<Color>();
        this.register::<TextureAtlasSprite>();
        this.register::<TextureAtlas>();
        this.register::<PointLight>();
        this.register::<DirectionalLight>();
        this.register::<StandardMaterial>();
        this.register::<PrimitiveTopology>();
        this.register::<Mesh>();
        this.register::<bevy::sprite::Rect>();

        this.register::<WindowOrigin>();
        this.register::<ScalingMode>();
        this.register::<DepthCalculation>();
        this.register::<VisibleEntities>();
        this.register::<VisiblePointLights>();
        this.register::<CubemapVisibleEntities>();
        this.register::<CubemapFrusta>();
        this.register::<Frustum>();
        this.register::<Plane>();
        this.register::<Clusters>();

        this.register::<Handle<Image>>();
        this.register::<Handle<StandardMaterial>>();
        this.register::<Handle<TextureAtlas>>();
        this.register::<Handle<Mesh>>();

        this.register::<ClearColor>();
        this.register::<AmbientLight>();

        register!(this Display, Style, Size<f32>, Size<Val>, Val, bevy::ui::FocusPolicy);
        register!(this VerticalAlign, HorizontalAlign, TextAlignment, TextStyle, TextSection, Text);
        register!(this PositionType, Direction, FlexDirection, FlexWrap, AlignItems, AlignSelf, JustifyContent);

        #[cfg(feature = "rapier")]
        {
            use bevy_rapier3d::prelude::*;
            register!(this
                RigidBodyType, RigidBodyPosition, RigidBodyVelocity, RigidBodyMassProps, RigidBodyMassPropsFlags,
                RigidBodyForces, RigidBodyActivation, RigidBodyDamping, RigidBodyDominance, RigidBodyCcd, RigidBodyChanges,
                ColliderType, ColliderPosition, ColliderMaterial, CoefficientCombineRule, ColliderFlags, RigidBodyHandle,
                ActiveCollisionTypes, ActiveHooks, ActiveEvents, InteractionGroups, ColliderChanges, ColliderParent,
                ColliderPositionSync, SharedShape, RigidBodyColliders, ColliderMassProps
            );
        }
        #[cfg(feature = "rapier2d")]
        {
            use bevy_rapier2d::prelude::*;
            register!(this
                RigidBodyType, RigidBodyPosition, RigidBodyVelocity, RigidBodyMassProps, RigidBodyMassPropsFlags,
                RigidBodyForces, RigidBodyActivation, RigidBodyDamping, RigidBodyDominance, RigidBodyCcd, RigidBodyChanges,
                ColliderType, ColliderPosition, ColliderMaterial, CoefficientCombineRule, ColliderFlags, RigidBodyHandle,
                ActiveCollisionTypes, ActiveHooks, ActiveEvents, InteractionGroups, ColliderChanges, ColliderParent,
                ColliderPositionSync, SharedShape, RigidBodyColliders, ColliderMassProps
            );
        }

        this
    }
}
