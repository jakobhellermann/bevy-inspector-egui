use crate::{Context, Inspectable};
use bevy::render::camera::{DepthCalculation, ScalingMode, VisibleEntities, WindowOrigin};
use bevy::{pbr::AmbientLight, prelude::*};
use bevy::{render::pipeline::PrimitiveTopology, utils::HashMap};
use bevy_egui::egui;
use std::any::TypeId;

pub(crate) type InspectCallback =
    Box<dyn Fn(*mut u8, &mut egui::Ui, &Context) -> bool + Send + Sync>;

macro_rules! register {
    ($this:ident $($ty:ty),* $(,)?) => {
        $($this.register::<$ty>();)*
    };
}

/// The `InspectableRegistry` can be used to tell the [`WorldInspectorPlugin`](crate::WorldInspectorPlugin)
/// how to display a type.
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
        F: Fn(&mut T, &mut egui::Ui, &Context) -> bool + Send + Sync + 'static,
    {
        let type_id = TypeId::of::<T>();
        let callback = Box::new(move |ptr: *mut u8, ui: &mut egui::Ui, context: &Context| {
            let value: &mut T = unsafe { &mut *(ptr as *mut T) };
            f(value, ui, context)
        }) as InspectCallback;
        self.impls.insert(type_id, callback);
    }

    /// Variant of [`InspectableRegistry::register`] which returns self by-value.
    /// Allows
    /// ```rust,no_run
    /// # use bevy::prelude::*;
    /// # use bevy_inspector_egui::InspectableRegistry;
    /// # #[derive(bevy_inspector_egui::Inspectable)] struct MyType;
    /// App::build()
    ///   .insert_resource(InspectableRegistry::default().with::<MyType>())
    ///   .run();
    /// ```
    pub fn with<T: Inspectable + 'static>(mut self) -> Self {
        self.register::<T>();
        self
    }

    pub(crate) fn try_execute(
        &self,
        value: &mut dyn Reflect,
        ui: &mut egui::Ui,
        context: &Context,
    ) -> Result<bool, ()> {
        if let Some(inspect_callback) = self.impls.get(&value.type_id()) {
            let ptr = value as *mut dyn Reflect as *mut u8;
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

        this.register::<std::ops::Range<f32>>();
        this.register::<std::time::Duration>();

        this.register::<Transform>();
        this.register::<GlobalTransform>();
        this.register::<Quat>();

        this.register::<Color>();
        this.register::<bevy::asset::HandleId>();
        this.register::<TextureAtlasSprite>();
        this.register::<TextureAtlas>();
        this.register::<Light>();
        this.register::<StandardMaterial>();
        this.register::<ColorMaterial>();
        this.register::<PrimitiveTopology>();
        this.register::<Mesh>();
        this.register::<bevy::sprite::Rect>();

        this.register::<WindowOrigin>();
        this.register::<ScalingMode>();
        this.register::<DepthCalculation>();
        this.register::<VisibleEntities>();

        this.register::<Handle<Texture>>();
        this.register::<Handle<StandardMaterial>>();
        this.register::<Handle<ColorMaterial>>();
        this.register::<Handle<TextureAtlas>>();
        this.register::<Handle<Mesh>>();

        this.register::<ClearColor>();
        this.register::<AmbientLight>();

        register!(this Display, Style, Size<f32>, Size<Val>, Val, bevy::ui::FocusPolicy);
        register!(this VerticalAlign, HorizontalAlign, TextAlignment, TextStyle, TextSection, Text);
        register!(this PositionType, Direction, FlexDirection, FlexWrap, AlignItems, AlignSelf, JustifyContent);

        #[cfg(feature = "rapier")]
        register!(this bevy_rapier3d::rapier::dynamics::MassProperties, bevy_rapier3d::rapier::dynamics::RigidBody, bevy_rapier3d::physics::RigidBodyHandleComponent);
        #[cfg(feature = "rapier2d")]
        register!(this bevy_rapier2d::rapier::dynamics::MassProperties, bevy_rapier2d::rapier::dynamics::RigidBody, bevy_rapier2d::physics::RigidBodyHandleComponent);

        this
    }
}
