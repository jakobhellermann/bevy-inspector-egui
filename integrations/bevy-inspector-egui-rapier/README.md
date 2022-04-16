# bevy-inspector-egui-rapier

```toml
[dependencies]
bevy-inspector-egui = "0.9"
bevy-inspector-egui-rapier = { version = "0.1", features = ["rapier3d"] }
```

```rust
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierRenderPlugin)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(InspectableRapierPlugin) // <--- register the inspectable UI functions for rapier types
        .add_plugin(WorldInpsectorPlugin)
        .run();
}
```

## Bevy support table

| bevy    | bevy-inspector-egui | bevy-inspector-egui-rapier | bevy_rapier
| ------- | ------------------- | -------------------------- | ------
| 0.7     | 0.10                | 0.2                        | 0.12
| 0.6     | 0.9                 | 0.1                        | 0.12
