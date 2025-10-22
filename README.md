# bevy-inspector-egui

Examples can be found at [`./crates/bevy-inspector-egui/examples`](./crates/bevy-inspector-egui/examples/).

This crate contains

- general purpose machinery for displaying [`Reflect`](bevy_reflect::Reflect) values in [reflect_inspector],
- a way of associating arbitrary options with fields and enum variants in [inspector_options]
- utility functions for displaying bevy resource, entities and assets in [bevy_inspector]
- some drop-in plugins in [quick] to get you started without any code necessary.

The changelog can be found at [`docs/CHANGELOG.md`](./docs/CHANGELOG.md).

# Use case 1: Quick plugins

These plugins can be easily added to your app, but don't allow for customization of the presentation and content.

## WorldInspectorPlugin

Displays the world's entities, resources and assets.

![image of the world inspector](https://raw.githubusercontent.com/jakobhellermann/bevy-inspector-egui/main/docs/images/world_inspector.png)

```rust
use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin::default())
        .add_plugins(WorldInspectorPlugin::new())
        .run();
}
```

## ResourceInspectorPlugin

Display a single resource in a window.

![image of the resource inspector](https://raw.githubusercontent.com/jakobhellermann/bevy-inspector-egui/main/docs/images/resource_inspector.png)

```rust
use bevy::prelude::*;
use bevy_inspector_egui::prelude::*;
use bevy_inspector_egui::quick::ResourceInspectorPlugin;

// `InspectorOptions` are completely optional
#[derive(Reflect, Resource, Default, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
struct Configuration {
    name: String,
    #[inspector(min = 0.0, max = 1.0)]
    option: f32,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<Configuration>() // `ResourceInspectorPlugin` won't initialize the resource
        .register_type::<Configuration>() // you need to register your type to display it
        .add_plugins(EguiPlugin::default())
        .add_plugins(ResourceInspectorPlugin::<Configuration>::default())
        // also works with built-in resources, as long as they are `Reflect`
        .add_plugins(ResourceInspectorPlugin::<Time>::default())
        .run();
}
```

<hr>

There is also the [`StateInspectorPlugin`](quick::StateInspectorPlugin) and the [`AssetInspectorPlugin`](quick::AssetInspectorPlugin).

# Use case 2: Manual UI

The [quick] plugins don't allow customization of the egui window or its content, but you can easily build your own UI:

```rust
use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::prelude::*;
use std::any::TypeId;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin::default())
        .add_plugins(bevy_inspector_egui::DefaultInspectorConfigPlugin) // adds default options and `InspectorEguiImpl`s
        .add_systems(EguiPrimaryContextPass, inspector_ui)
        .run();
}

fn inspector_ui(world: &mut World) {
    let Ok(egui_context) = world
        .query_filtered::<&mut EguiContext, With<PrimaryEguiContext>>()
        .get_single(world)
    else {
        return;
    };
    let mut egui_context = egui_context.clone();

    egui::Window::new("UI").show(egui_context.get_mut(), |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
            // equivalent to `WorldInspectorPlugin`
            bevy_inspector_egui::bevy_inspector::ui_for_world(world, ui);

            egui::CollapsingHeader::new("Materials").show(ui, |ui| {
                bevy_inspector_egui::bevy_inspector::ui_for_assets::<StandardMaterial>(world, ui);
            });

            ui.heading("Entities");
            bevy_inspector_egui::bevy_inspector::ui_for_world_entities(world, ui);
        });
    });
}
```

Pair this with a crate like [`egui_dock`](https://docs.rs/egui_dock/latest/egui_dock/) and you have your own editor in less than 100 lines: [`examples/egui_dock.rs`](https://github.com/jakobhellermann/bevy-inspector-egui/blob/main/crates/bevy-inspector-egui/examples/integrations/egui_dock.rs).
![image of the egui_dock example](https://raw.githubusercontent.com/jakobhellermann/bevy-inspector-egui/main/docs/images/egui_dock.png)

## Cargo features

- `highlight_changes` - highlight changed values every frame.
  Ideally this should be runtime-configurable, but it was implemented like this as a stopgap solution. If you'd like to configure this at runtime, please open an issue to let me know it's more of a priority.
- `bevy_pbr` (default): register default options for `bevy_pbr` types. You should disable this if you don't use `bevy_pbr` to reduce the dependency footprint.
- `bevy_gizmos`: enable inspecting of `GizmoConfigGroup`

## FAQ

**Q: How do I change the names of the entities in the world inspector?**

**A:** You can insert the [`Name`](https://docs.rs/bevy_core/latest/bevy_core/struct.Name.html) component.

**Q: What if I just want to display a single value without passing in the whole `&mut World`?**

**A:** You can use `reflect_inspector::ui_for_value`. Note that displaying things like `Handle<StandardMaterial>` won't be able to display the asset's value.

**Q:** Can I change how exactly my type is displayed?

**A:** Implement `InspectorPrimitive` and call `app.register_type_data::<T, InspectorEguiImpl>`.

[reflect_inspector]: https://docs.rs/bevy-inspector-egui/latest/bevy_inspector_egui/reflect_inspector
[inspector_options]: https://docs.rs/bevy-inspector-egui/latest/bevy_inspector_egui/inspector_options
[quick]: https://docs.rs/bevy-inspector-egui/latest/bevy_inspector_egui/quick
[bevy_inspector]: https://docs.rs/bevy-inspector-egui/latest/bevy_inspector_egui/bevy_inspector

## Bevy support table

| bevy | bevy-inspector-egui |
| ---- | ------------------- |
| 0.17 | 0.35                |
| 0.17 | 0.34                |
| 0.16 | 0.33                |
| 0.16 | 0.32                |
| 0.16 | 0.31                |
| 0.15 | 0.30                |
| 0.15 | 0.29                |
| 0.15 | 0.28                |
| 0.14 | 0.27                |
| 0.14 | 0.26                |
| 0.14 | 0.25                |
| 0.13 | 0.24                |
| 0.13 | 0.23                |
| 0.12 | 0.22                |
| 0.12 | 0.21                |
| 0.11 | 0.19-0.20           |
| 0.10 | 0.18                |
| 0.9  | 0.14-0.17           |
| 0.8  | 0.12-0.13           |
| 0.7  | 0.10-0.11           |
| 0.6  | 0.9                 |
| 0.6  | 0.8                 |
| 0.6  | 0.7                 |
| 0.5  | 0.5-0.6             |
| 0.5  | 0.4                 |
| 0.4  | 0.1-0.3             |
