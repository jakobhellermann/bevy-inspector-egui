use std::{any::TypeId, marker::PhantomData};

use bevy::prelude::*;
use bevy_egui::{egui, EguiContext, EguiPlugin};
use pretty_type_name::pretty_type_name_str;

use crate::{Context, Inspectable, InspectableRegistry};

#[allow(missing_debug_implementations)]
/// Bevy plugin for the inspector.
/// See the [crate-level docs](index.html) for an example on how to use it.
pub struct InspectorPlugin<T> {
    marker: PhantomData<T>,
    exclusive_access: bool,
}

impl<T> Default for InspectorPlugin<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> InspectorPlugin<T> {
    /// Creates a new inspector plugin with access to `World` in the [`Context`](crate::Context).
    pub fn new() -> Self {
        InspectorPlugin {
            exclusive_access: true,
            marker: PhantomData,
        }
    }
    /// Creates a new inspector plugin *without+ access to `World` in the [`Context`](crate::Context).
    /// This has the advantage that the system can be scheduled concurrently to others and may be faster.
    pub fn shared() -> Self {
        InspectorPlugin {
            exclusive_access: false,
            marker: PhantomData,
        }
    }
}

#[derive(Default, Debug)]
struct InspectorWindows(bevy::utils::HashMap<TypeId, String>);
impl InspectorWindows {
    fn contains_id(&self, type_id: TypeId) -> bool {
        self.0.iter().any(|(&id, _)| id == type_id)
    }
    fn contains_name(&self, name: &str) -> bool {
        self.0.iter().any(|(_, n)| n == name)
    }
    fn get_unwrap<T: 'static>(&self) -> &str {
        self.0
            .get(&TypeId::of::<T>())
            .expect("inspector window not properly initialized")
    }
}

impl<T> Plugin for InspectorPlugin<T>
where
    T: Inspectable + FromWorld + Send + Sync + 'static,
{
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<T>();

        T::setup(app);

        // init inspector ui and data resource
        if self.exclusive_access {
            app.add_system(exclusive_access_ui::<T>.exclusive_system());
        } else {
            app.add_system(shared_access_ui::<T>.exclusive_system());
        }

        // init egui
        if !app.world().contains_resource::<EguiContext>() {
            app.add_plugin(EguiPlugin);
        }

        // registeres egui textures
        app.add_system(egui_texture_setup.system());

        let world = app.world_mut();

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
                inspector_windows.0.insert(type_id, full_type_name.into());
            }
        } else {
            inspector_windows.0.insert(type_id, type_name);
        }
    }
}

fn egui_texture_setup(
    mut egui_context: ResMut<EguiContext>,
    mut asset_events: EventReader<AssetEvent<Texture>>,
    mut textures: ResMut<Assets<Texture>>,
) {
    use crate::impls::with_context::id_of_handle;

    for asset_event in asset_events.iter() {
        match asset_event {
            AssetEvent::Created { handle } => {
                let mut handle = handle.clone();
                handle.make_strong(&mut textures);
                egui_context.set_egui_texture(id_of_handle(&handle), handle)
            }
            AssetEvent::Modified { .. } => {}
            AssetEvent::Removed { handle } => {
                egui_context.remove_egui_texture(id_of_handle(handle))
            }
        }
    }
}

fn shared_access_ui<T>(
    mut data: ResMut<T>,
    egui_context: ResMut<EguiContext>,
    inspector_windows: Res<InspectorWindows>,
) where
    T: Inspectable + Send + Sync + 'static,
{
    let ctx = &egui_context.ctx;

    let type_name = inspector_windows.get_unwrap::<T>();

    egui::Window::new(type_name)
        .resizable(false)
        .scroll(true)
        .show(ctx, |ui| {
            default_settings(ui);

            let context = Context::new_shared(ctx);
            data.ui(ui, T::Attributes::default(), &context);
        });
}

fn exclusive_access_ui<T>(world: &mut World)
where
    T: Inspectable + Send + Sync + 'static,
{
    let type_name = {
        let inspector_windows = world.get_resource_mut::<InspectorWindows>().unwrap();
        let type_name = inspector_windows.get_unwrap::<T>();
        type_name.to_string()
    };

    let world_ptr = world as *mut _;

    let world = world.cell();

    let egui_context = world.get_resource::<EguiContext>().unwrap();
    let ctx = &egui_context.ctx;

    let context = unsafe { Context::new_ptr(ctx, world_ptr) };
    let mut data = world.get_resource_mut::<T>().unwrap();

    egui::Window::new(type_name)
        .resizable(false)
        .scroll(true)
        .show(ctx, |ui| {
            default_settings(ui);

            data.ui(ui, T::Attributes::default(), &context);
        });
}

pub(crate) fn default_settings(ui: &mut egui::Ui) {
    ui.style_mut().wrap = Some(false);
}
