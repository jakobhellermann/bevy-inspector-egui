use std::{any::TypeId, marker::PhantomData};

use bevy::prelude::*;
use bevy_egui::{egui, EguiContext, EguiPlugin};

use crate::Inspectable;

#[derive(Default)]
#[allow(missing_debug_implementations)]
/// Bevy plugin for the inspector.
/// See the [crate-level docs](index.html) for an example on how to use it.
pub struct InspectorPlugin<T>(PhantomData<T>);

impl<T> InspectorPlugin<T> {
    pub fn new() -> Self {
        InspectorPlugin(PhantomData)
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
}

impl<T> Plugin for InspectorPlugin<T>
where
    T: Inspectable + Default + Send + Sync + 'static,
{
    fn build(&self, app: &mut AppBuilder) {
        // init inspector ui and data resource
        app.init_resource::<T>().add_system(ui::<T>.system());

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
    mut egui_context: ResMut<EguiContext>,
    inspector_windows: Res<InspectorWindows>,
    mut data: ResMut<T>,
) where
    T: Inspectable + Send + Sync + 'static,
{
    let ctx = &mut egui_context.ctx;

    let type_name = inspector_windows
        .0
        .get(&TypeId::of::<T>())
        .expect("inspector window not properly initialized");

    egui::Window::new(type_name)
        .resizable(false)
        .show(ctx, |ui| {
            data.ui(ui, T::Attributes::default());
        });
}
