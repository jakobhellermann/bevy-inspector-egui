# Examples


### Simple Demo ([source](demo.rs))
<img src="../docs/examples/demo.png" alt="demo example" width="500"/>

### Planet Generation ([source](planet_demo.rs))
<img src="../docs/examples/planet_demo.png" alt="example" width="500" />

### World Inspector ([source](world.rs))
<img src="../docs/examples/world_inspector.png" alt="example" width="500" />

<details>
  <summary>Example code</summary>

```rust
use bevy_inspector_egui::WorldInspectorPlugin;

fn main() {
    App::build()
        .add_plugin(WorldInspectorPlugin::new())
        // ...
}
```
</details>

### Texture ([source](with_context.rs))
<img src="../docs/examples/texture.png" alt="example" width="500" />

### Rapier Integration ([source](rapier.rs))
<img src="../docs/examples/rapier.png" alt="example" width="500" />

- requires `rapier` feature

### Inspector Query ([source](inspector_query.rs))
<img src="../docs/examples/inspector_query.png" alt="example" width="500" />

<details>
  <summary>Example code</summary>

```rust
use bevy_inspector_egui::widgets::InspectorQuery;

#[derive(Inspectable)]
struct Inspector {
    root_elements: InspectorQuery<Without<Parent>>
}

```
</details>

### Entity ([source](entity.rs))
<img src="../docs/examples/entity.png" alt="example" width="500" />

### Multiple Inspectors ([source](multiple_inspectors.rs))
<img src="../docs/examples/multiple_inspectors.png" alt="example" width="500" />

### New egui window ([source](new_egui_window.rs))
<img src="../docs/examples/new_egui_window.png" alt="example" width="500" />

### Reflected UI ([source](reflected_ui.rs))
<img src="../docs/examples/reflected_ui.png" alt="example" width="500" />

<details>
  <summary>Example code</summary>

Sometimes you want to include types not implementing `Inspectable` in your inspector. If said type implements `Reflect`, you can use the `ReflectedUI` wrapper type:

```rust
use bevy::prelude::*;
use bevy_inspector_egui::widgets::ReflectedUI;

#[derive(Inspectable)]
struct Inspector {
    timer: ReflectedUI<Timer>,
}
```
</details>

### Transform ([source](transform.rs))
<img src="../docs/examples/transform.png" alt="example" width="500" />
