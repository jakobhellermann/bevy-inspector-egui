# bevy-inspector-egui

<div align="center">
  <!-- Crates version -->
  <a href="https://crates.io/crates/bevy-inspector-egui">
    <img src="https://img.shields.io/crates/v/bevy-inspector-egui.svg?style=flat-square"
    alt="Crates.io version" />
  </a>
  <!-- docs.rs docs -->
  <a href="https://docs.rs/bevy-inspector-egui">
    <img src="https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square"
      alt="docs.rs docs" />
  </a>
  <!-- License -->
    <img src="https://img.shields.io/crates/l/bevy-inspector-egui?style=flat-square"
      alt="Download" />
</div>
<br/>

This crate provides a debug interface using [egui](https://github.com/emilk/egui) where you can visually edit the values of your components live.

<img src="./docs/inspector.jpg" alt="demonstration with a running bevy app" width="500"/>

## Usage

You can either inspect a single resource using the `InspectorPlugin`, or use the `WorldInspectorPlugin` to inspect all entities.

## InspectorPlugin

The `InspectorPlugin<T>` will insert a resource of type `T` and display UI for editing that resource.

```rust
use bevy::prelude::*;
use bevy_inspector_egui::{InspectorPlugin, Inspectable};

#[derive(Inspectable, Default)]
struct Data {
    should_render: bool,
    text: String,
    #[inspectable(min = 42.0, max = 100.0)]
    size: f32,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(InspectorPlugin::<Data>::new())
        .run();
}
```

## World inspector

```rust
use bevy::prelude::*;
use bevy_inspector_egui::WorldInspectorPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(WorldInspectorPlugin::new())
        .run();
}
```

<img src="./docs/examples/world_inspector.png" alt="world inspector ui" width="600"/>

You can configure the `WorldInspectorPlugin` by inserting the `WorldInspectorParams` resource.
If you want to only display some components, you may want to use the [InspectorQuery](./examples/README.md#inspector-query-source) instead.

### Custom components in the world inspector

By default, types implementing `Inspectable` will not be displayed in the `WorldInspector`, because the there is no way to know of the trait implementation at runtime.
You can call `world.register_inspectable::<T>()` to tell `bevy-inspector-egui` how that type should be displayed, and it will show up correctly in the world inspector.

Alternatively, you can `#[derive(Reflect)]` and call `world.register_type::<T>()`. This will enable bevy's reflection feature for the type, and it will show up in the world inspector.

```rust
use bevy::prelude::*;
use bevy_inspector_egui::{WorldInspectorPlugin, Inspectable, RegisterInspectable};

#[derive(Inspectable, Component)]
struct InspectableType;

#[derive(Reflect, Component, Default)]
#[reflect(Component)]
struct ReflectedType;

fn main() {
  App::new()
    .add_plugins(DefaultPlugins)
    .add_plugin(WorldInspectorPlugin::new())
    .register_inspectable::<InspectableType>() // tells bevy-inspector-egui how to display the struct in the world inspector
    .register_type::<ReflectedType>() // registers the type in the `bevy_reflect` machinery, so that even without implementing `Inspectable` we can display the struct fields
    .run();
}
```
More examples (with pictures) can be found in the [`examples folder`](examples).

## Bevy support table

| bevy    | bevy-inspector-egui |
| ------- | ------------------- |
| 0.7     | 0.10                |
| 0.6     | 0.9                 |
| 0.6     | 0.8                 |
| 0.6     | 0.7                 |
| 0.5 | 0.5-0.6                 |
| 0.5     | 0.4                 |
| 0.4     | 0.1-0.3             |
