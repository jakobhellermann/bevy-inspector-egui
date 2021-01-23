use std::{any::TypeId, marker::PhantomData};

use bevy::prelude::*;
use bevy_egui::{egui, EguiContext, EguiPlugin};

use crate::{Context, Inspectable, InspectableWithContext};

#[derive(Default)]
#[allow(missing_debug_implementations)]
/// Bevy plugin for the inspector.
/// See the [crate-level docs](index.html) for an example on how to use it.
pub struct InspectorPlugin<T>(PhantomData<T>);

#[derive(Default)]
#[allow(missing_debug_implementations)]
/// Bevy plugin for using [`InspectableWithContext`](crate::InspectableWithContext).
pub struct ThreadLocalInspectorPlugin<T>(PhantomData<T>);

impl<T: Inspectable> InspectorPlugin<T> {
    pub fn new() -> Self {
        InspectorPlugin(PhantomData)
    }
}
impl<T: InspectableWithContext> ThreadLocalInspectorPlugin<T> {
    pub fn new() -> Self {
        ThreadLocalInspectorPlugin(PhantomData)
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

fn build_inspector_plugin<T>(app: &mut AppBuilder)
where
    T: Default + Send + Sync + 'static,
{
    // init inspector ui and data resource
    app.init_resource::<T>();

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
        assert!(
            !inspector_windows.contains_name(full_type_name),
            "two types with different type_id but same type_name"
        );
        inspector_windows.0.insert(type_id, full_type_name.into());
    } else {
        inspector_windows.0.insert(type_id, type_name.into());
    }
}

impl<T> Plugin for InspectorPlugin<T>
where
    T: Inspectable + Default + Send + Sync + 'static,
{
    fn build(&self, app: &mut AppBuilder) {
        build_inspector_plugin::<T>(app);
        app.add_system(ui::<T>.system());
    }
}

impl<T> Plugin for ThreadLocalInspectorPlugin<T>
where
    T: InspectableWithContext + Default + Send + Sync + 'static,
{
    fn build(&self, app: &mut AppBuilder) {
        build_inspector_plugin::<T>(app);
        app.add_system(thread_local_ui::<T>.system());
    }
}

fn ui<T>(
    mut egui_context: ResMut<EguiContext>,
    inspector_windows: Res<InspectorWindows>,
    mut data: ResMut<T>,
) where
    T: Inspectable + Send + Sync + 'static,
{
    let ctx = &mut egui_context.ctx;

    let type_name = inspector_windows.get_unwrap::<T>();
    egui::Window::new(type_name)
        .resizable(false)
        .show(ctx, |ui| {
            data.ui(ui, T::Attributes::default());
        });
}

fn thread_local_ui<T>(_: &mut World, resources: &mut Resources)
where
    T: InspectableWithContext + Send + Sync + 'static,
{
    let egui_context = resources.get::<EguiContext>().unwrap();
    let inspector_windows = resources.get_mut::<InspectorWindows>().unwrap();
    let mut data = resources.get_mut::<T>().unwrap();

    let type_name = inspector_windows.get_unwrap::<T>();

    let ctx = &egui_context.ctx;
    egui::Window::new(type_name)
        .resizable(false)
        .show(ctx, |ui| {
            let context = Context { resources };
            data.ui_with_context(ui, T::Attributes::default(), &context);
        });
}
