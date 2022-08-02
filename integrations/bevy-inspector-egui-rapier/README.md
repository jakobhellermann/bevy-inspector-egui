# bevy-inspector-egui-rapier

```toml
[dependencies]
bevy-inspector-egui = "0.11"
bevy-inspector-egui-rapier = { version = "0.3", features = ["rapier3d"] }
```

```rust
use bevy::prelude::*;
use bevy_inspector_egui::WorldInspectorPlugin;
use bevy_inspector_egui_rapier::InspectableRapierPlugin;
use bevy_rapier3d::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierDebugRenderPlugin)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(InspectableRapierPlugin) // <--- register the inspectable UI functions for rapier types
        .add_plugin(WorldInspectorPlugin)
        .run();
}
```

## Bevy support table

| bevy    | bevy-inspector-egui | bevy-inspector-egui-rapier | bevy\_rapier
| ------- | ------------------- | -------------------------- | ------
| 0.8     | 0.12                | 0.5                        | 0.16
| 0.7     | 0.11                | 0.4                        | 0.14
| 0.7     | 0.11                | 0.3                        | 0.13
| 0.7     | 0.10                | 0.2                        | 0.12
| 0.6     | 0.9                 | 0.1                        | 0.12
