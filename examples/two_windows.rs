use bevy::prelude::*;
use bevy::render::{render_graph::RenderGraph, RenderApp};
use bevy::window::{CreateWindow, WindowId};
use bevy_inspector_egui::{Inspectable, InspectorPlugin};
use once_cell::sync::Lazy;

#[derive(Inspectable, Default)]
struct Primary {
    x: f32,
}
#[derive(Inspectable, Default)]
struct Secondary {
    y: f32,
}

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .add_startup_system(create_new_window)
        .add_plugin(InspectorPlugin::<Primary>::new())
        .add_plugin(InspectorPlugin::<Secondary>::new().on_window(*SECOND_WINDOW_ID));

    let render_app = app.sub_app_mut(RenderApp);
    let mut graph = render_app.world.get_resource_mut::<RenderGraph>().unwrap();

    bevy_egui::setup_pipeline(
        &mut graph,
        bevy_egui::RenderGraphConfig {
            window_id: *SECOND_WINDOW_ID,
            egui_pass: SECONDARY_EGUI_PASS,
        },
    );

    app.run();
}

static SECOND_WINDOW_ID: Lazy<WindowId> = Lazy::new(WindowId::new);
const SECONDARY_EGUI_PASS: &str = "secondary_egui_pass";

fn create_new_window(mut create_window_events: EventWriter<CreateWindow>) {
    let window_id = *SECOND_WINDOW_ID;

    create_window_events.send(CreateWindow {
        id: window_id,
        descriptor: WindowDescriptor {
            width: 800.,
            height: 600.,
            title: "Second window".to_string(),
            ..Default::default()
        },
    });
}
