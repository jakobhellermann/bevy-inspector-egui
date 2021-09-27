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
    App::build()
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
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(WorldInspectorPlugin::new())
        .run();
}
```

<img src="./docs/examples/world_inspector.png" alt="world inspector ui" width="600"/>


You can configure the `WorldInspectorPlugin` by inserting the `WorldInspectorParams` resource.
If you want to only display some components, you may want to use the [InspectorQuery](./examples/README.md#inspector-query-source) instead.

If you want custom types to be displayed in the inspector, you'll need to register them on the `InspectableRegistry`:

```rust
use bevy::prelude::*;
use bevy_inspector_egui::InspectableRegistry;

fn main() {
  let mut app = App::build();
  // add plugins and systems

  let mut registry = app
      .world_mut()
      .get_resource_mut::<InspectableRegistry>()
      .unwrap();
  registry.register::<Data>();
  registry.register::<OtherComponent>();
}
```
More examples (with pictures) can be found in the [`examples folder`](examples).

## Bevy support table

| bevy    | bevy-inspector-egui |
| ------- | ------------------- |
| 0.5-0.6 | 0.5                 |
| 0.5     | 0.4                 |
| 0.4     | 0.1-0.3             |
