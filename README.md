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

In order for custom components to show up in the inspector, you have to:

1. Add an `InspectableRegistry` to the world as a resource
2. Register the component to the `InspectableRegistry`
3. `#[derive(Inspectable)]` for the component
4. Add an inspector plugin

For bevy components don't need to be registered, nor do they require an `InspectableRegistry`.

## Example
### InspectableRegistry

```rust
use bevy::prelude::*;
use bevy_inspector_egui::InspectableRegistry;

let mut registry = app.insert_resource(InspectableRegistry::default())
                      .world_mut()
                      .get_resource_mut::<InspectableRegistry>()
                      .expect("InspectableRegistry not initiated");

registry.register::<Data>();
registry.registre::<OtherComponent>();
```

### Component

```rust
use bevy::prelude::*;
use bevy_inspector_egui::Inspectable;

#[derive(Inspectable, Default)]
struct Data {
    should_render: bool,
    text: String,
    #[inspectable(min = 42.0, max = 100.0)]
    size: f32,
}
```

### Inspector

If you want to display all world entities you can add the `WorldInspectorPlugin`:
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

You can configure it by inserting the `WorldInspectorParams` resource.
If you want to only display some components, you may want to use the [InspectorQuery](./examples/README.md#inspector-query-source) instead.

<img src="./docs/examples/world_inspector.png" alt="world inspector ui" width="600"/>

Alternatively you could use a simpler `InspectorPlugin`:
```rust
use bevy::prelude::*;
use bevy_inspector_egui::InspectorPlugin;

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(InspectorPlugin::<Data>::new())
        .run();
}
```

More examples (with pictures) can be found in the [`examples folder`](examples).

## Bevy support table

| bevy    | bevy-inspector-egui |
| ------- | ------------------- |
| 0.5-0.6 | 0.5                 |
| 0.5     | 0.4                 |
| 0.4     | 0.1-0.3             |
