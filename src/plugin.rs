use std::marker::PhantomData;

use bevy::prelude::*;
use bevy_egui::{egui, EguiContext, EguiPlugin};

use crate::{Inspectable, Options};

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

impl<T> Plugin for InspectorPlugin<T>
where
    T: Inspectable + Default + Send + Sync + 'static,
{
    fn build(&self, app: &mut AppBuilder) {
        app.add_plugin(EguiPlugin)
            .init_resource::<T>()
            .add_system(ui::<T>.system());
    }
}

fn ui<T>(mut egui_context: ResMut<EguiContext>, mut data: ResMut<T>)
where
    T: Inspectable + Send + Sync + 'static,
{
    let ctx = &mut egui_context.ctx;

    egui::Window::new("Inspector")
        .resizable(false)
        .show(ctx, |ui| {
            data.ui(ui, Options::default());
        });
}
