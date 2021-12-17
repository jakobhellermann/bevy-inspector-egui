use std::{any::TypeId, marker::PhantomData};

use bevy::{ecs::component::ComponentTicks, prelude::*, window::WindowId};
use bevy_egui::{egui, EguiContext, EguiPlugin};
use pretty_type_name::pretty_type_name_str;

use crate::{Context, Inspectable, InspectableRegistry};

#[allow(missing_debug_implementations)]
/// Bevy plugin for the inspector.
/// See the [crate-level docs](index.html) for an example on how to use it.
pub struct InspectorPlugin<T> {
    marker: PhantomData<T>,
    exclusive_access: bool,
    initial_value: Option<Box<dyn Fn(&mut World) -> T + Send + Sync + 'static>>,
    window_id: WindowId,
}

impl<T: Default + Send + Sync + 'static> Default for InspectorPlugin<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: FromWorld + Send + Sync + 'static> InspectorPlugin<T> {
    /// Creates a new inspector plugin with access to `World` in the [`Context`](crate::Context).
    pub fn new() -> Self {
        InspectorPlugin {
            exclusive_access: true,
            marker: PhantomData,
            initial_value: Some(Box::new(T::from_world)),
            window_id: WindowId::primary(),
        }
    }
}

impl<T> InspectorPlugin<T> {
    /// Same as [`InspectorPlugin::new`], but doesn't automatically insert the `T` resource.
    pub fn new_insert_manually() -> Self {
        InspectorPlugin {
            marker: PhantomData,
            exclusive_access: true,
            initial_value: None,
            window_id: WindowId::primary(),
        }
    }

    /// Specifies that the inspector has no access to `World` in the [`Context`](crate::Context).
    /// This has the advantage that the system can be scheduled concurrently to others.
    pub fn shared(self) -> Self {
        InspectorPlugin {
            exclusive_access: false,
            ..self
        }
    }

    /// Sets the window the inspector should be displayed on
    pub fn on_window(self, window_id: WindowId) -> Self {
        InspectorPlugin { window_id, ..self }
    }
}

#[derive(Clone)]
/// Metadata for a inspector window
pub struct InspectorWindowData {
    /// The name of the egui window. Must be unique.
    pub name: String,
    /// The window the ui should be displayed on
    pub window_id: WindowId,
    /// Whether the ui is currently shown
    pub visible: bool,
}
#[derive(Default)]
/// Can be used to control whether inspector windows are shown
pub struct InspectorWindows(bevy::utils::HashMap<TypeId, InspectorWindowData>);
impl InspectorWindows {
    fn insert<T: 'static>(&mut self, name: String, window_id: WindowId) {
        let data = InspectorWindowData {
            name,
            window_id,
            visible: true,
        };
        self.0.insert(TypeId::of::<T>(), data);
    }
    fn contains_id(&self, type_id: TypeId) -> bool {
        self.0.iter().any(|(&id, _)| id == type_id)
    }
    fn contains_name(&self, name: &str) -> bool {
        self.0.iter().any(|(_, data)| data.name == name)
    }
    /// Returns the [InspectorWindowData] for the type `T`.
    #[track_caller]
    pub fn window_data<T: 'static>(&self) -> &InspectorWindowData {
        self.0
            .get(&TypeId::of::<T>())
            .ok_or_else(|| {
                format!(
                    "inspector window `{}` not initialized",
                    std::any::type_name::<T>()
                )
            })
            .unwrap()
    }
    /// Returns a mutable reference to the [InspectorWindowData] for the type `T`.
    #[track_caller]
    pub fn window_data_mut<T: 'static>(&mut self) -> &mut InspectorWindowData {
        self.0
            .get_mut(&TypeId::of::<T>())
            .ok_or_else(|| {
                format!(
                    "inspector window `{}` not initialized",
                    std::any::type_name::<T>()
                )
            })
            .unwrap()
    }
}

impl<T> Plugin for InspectorPlugin<T>
where
    T: Inspectable + Send + Sync + 'static,
{
    fn build(&self, app: &mut App) {
        if let Some(get_value) = &self.initial_value {
            let world = &mut app.world;
            let resource = get_value(world);
            app.insert_resource(resource);
        }

        T::setup(app);

        // init inspector ui and data resource
        if self.exclusive_access {
            app.add_system(exclusive_access_ui::<T>.exclusive_system());
        } else {
            app.add_system(shared_access_ui::<T>.exclusive_system());
        }

        // init egui
        if !app.world.contains_resource::<EguiContext>() {
            app.add_plugin(EguiPlugin);
        }

        let world = &mut app.world;

        // add entry to `InspectorWindows`
        world.get_resource_or_insert_with(InspectableRegistry::default);
        let mut inspector_windows = world.get_resource_or_insert_with(InspectorWindows::default);

        let type_id = TypeId::of::<T>();
        let full_type_name = std::any::type_name::<T>();

        if inspector_windows.contains_id(type_id) {
            panic!(
                "InspectorPlugin for {} is already registered",
                full_type_name,
            );
        }

        let type_name: String = pretty_type_name_str(full_type_name);
        if inspector_windows.contains_name(&type_name) {
            if inspector_windows.contains_name(full_type_name) {
                panic!("two types with different type_id but same type_name");
            } else {
                inspector_windows.insert::<T>(full_type_name.into(), self.window_id);
            }
        } else {
            inspector_windows.insert::<T>(type_name, self.window_id);
        }
    }
}

fn shared_access_ui<T>(
    data: Option<ResMut<T>>,
    egui_context: ResMut<EguiContext>,
    inspector_windows: Res<InspectorWindows>,
) where
    T: Inspectable + Send + Sync + 'static,
{
    let mut data = match data {
        Some(data) => data,
        None => return,
    };

    let window_data = inspector_windows.window_data::<T>();
    let ctx = egui_context.ctx_for_window(window_data.window_id);

    if !window_data.visible {
        return;
    }
    egui::Window::new(&window_data.name)
        .resizable(false)
        .vscroll(true)
        .show(egui_context.ctx(), |ui| {
            default_settings(ui);

            let context = Context::new_shared(Some(ctx));
            data.ui(ui, T::Attributes::default(), &context);
        });
}

fn exclusive_access_ui<T>(world: &mut World)
where
    T: Inspectable + Send + Sync + 'static,
{
    let window_data = {
        let inspector_windows = world.get_resource_mut::<InspectorWindows>().unwrap();
        let window_data = inspector_windows.window_data::<T>();
        window_data.clone()
    };

    let world_ptr = world as *mut _;

    let egui_context = unsafe { world.get_resource_unchecked_mut::<EguiContext>().unwrap() };

    let ctx = match egui_context.try_ctx_for_window(window_data.window_id) {
        Some(ctx) => ctx,
        None => return,
    };

    let context = unsafe { Context::new_ptr(Some(ctx), world_ptr) };
    let mut data = match get_silent_mut_unchecked::<T>(world) {
        Some(data) => data,
        None => return,
    };

    let mut changed = false;

    if !window_data.visible {
        return;
    }
    egui::Window::new(window_data.name)
        .resizable(false)
        .vscroll(true)
        .show(ctx, |ui| {
            default_settings(ui);

            let value = data.get_mut_silent();
            changed = value.ui(ui, T::Attributes::default(), &context);
        });

    if changed {
        data.mark_changed();
    }
}

pub(crate) fn default_settings(ui: &mut egui::Ui) {
    ui.style_mut().wrap = Some(false);
}

fn get_silent_mut_unchecked<T: Send + Sync + 'static>(world: &World) -> Option<SilentMut<T>> {
    let component_id = world.components().get_resource_id(TypeId::of::<T>())?;

    let resource_archetype = world.archetypes().resource();
    let unique_components = resource_archetype.unique_components();
    let column = unique_components.get(component_id).and_then(|column| {
        if column.is_empty() {
            None
        } else {
            Some(column)
        }
    })?;

    let value = unsafe {
        SilentMut {
            value: &mut *column.get_data_ptr().as_ptr().cast::<T>(),
            component_ticks: &mut *(column.get_ticks_ptr() as *mut ComponentTicks),
            change_tick: world.read_change_tick(),
        }
    };
    Some(value)
}

struct SilentMut<'a, T> {
    pub(crate) value: &'a mut T,
    pub(crate) component_ticks: &'a mut ComponentTicks,
    pub(crate) change_tick: u32,
}
impl<'a, T> SilentMut<'a, T> {
    fn get_mut_silent(&mut self) -> &mut T {
        self.value
    }
    fn mark_changed(&mut self) {
        self.component_ticks.set_changed(self.change_tick);
    }
}
