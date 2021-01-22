use std::{any::TypeId, marker::PhantomData};

use bevy::prelude::*;
use bevy_egui::{egui, EguiContext, EguiPlugin};

use crate::{Context, Inspectable};

#[derive(Default)]
#[allow(missing_debug_implementations)]
/// Bevy plugin for the inspector.
/// See the [crate-level docs](index.html) for an example on how to use it.
pub struct InspectorPlugin<T> {
    marker: PhantomData<T>,
    thread_local: bool,
}

impl<T> InspectorPlugin<T> {
    /// Creates a new inspector plugin, where the <Inspectable> implementations
    /// *do not* have access to `bevy::ecs::Resources` in the [`Context`](crate::Context::resources)
    pub fn new() -> Self {
        InspectorPlugin {
            thread_local: false,
            marker: PhantomData,
        }
    }
    /// Creates a new inspector plugin wich access to `bevy::ecs::Resources`.
    /// The disadvantage is, that the ui system has to run in a thread local system, which may hurt performance a bit.
    pub fn thread_local() -> Self {
        InspectorPlugin {
            thread_local: true,
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
    T: Inspectable + Default + Send + Sync + 'static,
{
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<T>();

        // init inspector ui and data resource
        if self.thread_local {
            app.add_system(thread_local_ui::<T>.system());
        } else {
            app.add_system(ui::<T>.system());
        }

        // init egui
        if !app.resources().contains::<EguiContext>() {
            app.add_plugin(EguiPlugin);
        }

        // add entry to `InspectorWindows`
        let mut inspector_windows = app
            .resources_mut()
            .get_or_insert_with(InspectorWindows::default);

        let type_id = TypeId::of::<T>();
        let full_type_name = std::any::type_name::<T>();

        if inspector_windows.contains_id(type_id) {
            panic!(
                "InspectorPlugin for {} is already registered",
                full_type_name,
            );
        }

        let type_name = full_type_name.rsplit("::").next().unwrap_or("Inspector");
        if inspector_windows.contains_name(type_name) {
            if inspector_windows.contains_name(full_type_name) {
                panic!("two types with different type_id but same type_name");
            } else {
                inspector_windows.0.insert(type_id, full_type_name.into());
            }
        } else {
            inspector_windows.0.insert(type_id, type_name.into());
        }
    }
}

fn ui<T>(
    mut data: ResMut<T>,
    mut egui_context: ResMut<EguiContext>,
    inspector_windows: Res<InspectorWindows>,
) where
    T: Inspectable + Send + Sync + 'static,
{
    let ctx = &mut egui_context.ctx;

    let type_name = inspector_windows.get_unwrap::<T>();

    egui::Window::new(type_name)
        .resizable(false)
        .show(ctx, |ui| {
            let context = Context::default();
            data.ui(ui, T::Attributes::default(), &context);
        });
}

fn thread_local_ui<T>(_world: &mut World, resources: &mut Resources)
where
    T: Inspectable + Send + Sync + 'static,
{
    let mut egui_context = resources.get_mut::<EguiContext>().unwrap();
    let inspector_windows = resources.get::<InspectorWindows>().unwrap();

    let mut data = resources.get_mut::<T>().unwrap();

    let ctx = &mut egui_context.ctx;

    let type_name = inspector_windows.get_unwrap::<T>();

    egui::Window::new(type_name)
        .resizable(false)
        .show(ctx, |ui| {
            let context = Context::new(resources);
            data.ui(ui, T::Attributes::default(), &context);
        });
}
