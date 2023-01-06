# Migration Guide from 0.15 to 0.16

- `WorldInspectorPlugin` got moved to `quick::WorldInspectorPlugin`
    - `WorldInspectorParams` got removed. Copy the implementation of `WorldInspectorPlugin` and tweak it to your liking. If you have any feature ideas that you think would be beneficial to have built in, please open an issue.
- `InspectorPlugin` got moved to `quick::ResourceInspectorPlugin`
    - it doesn't automatically insert the resource anymore
    - you need to derive `Reflect` instead of `Inspectable` and call `.register_type`
- `InspectorQuery` doesn't exist anymore. To show all entities matching a query, use `.add_plugin(FilterQueryInspectorPlugin::<With<Transform>>::default())`,
for anything else you have to write your own UI logic:
```rust
let mut query = world.query::<&Handle<StandardMaterial>>();
let handles: Vec<Handle<_>> = query.iter(world).cloned().collect();
for mut handle in handles {
    bevy_inspector_egui::bevy_inspector::ui_for_value(
        &mut handle,
        ui,
        world
    );
}
```
- if you used `value.ui(ui, options, context)` directly, new usage looks like this:
    - `bevy_inspector::ui_for_value(&mut value, ui, world)` if you want to be able to resolve handles inside your type
    - `reflect_inspector::ui_for_value(&mut value, ui, type_registry)` if you don't have `World` access
    - if you want more pass options:
    <details>
    <summary>example passing options</summary>

    ```rust
    let world: &mut World;

    let context = Context {
        // can also be None or Context::default()
        world: Some(world.into()),
    };
    // InspectorUi::new_no_short_circuit can be used if you don't need to be able to resolve bevy_asset handles.
    let env = InspectorUi::for_bevy(type_registry, context);

    let changed = env.ui_for_reflect_with_options(
        &mut value,
        ui,
        // some types like `Quat`s may store state. If you have display multiple values next to each other, make sure to pass different IDs. Otherwise you can use `egui::Id::null()`.
        egui::Id::new("ui"),
        // whatever options you want to pass. Look at the `inspector_options` docs for more info.
        &NumberOptions::positive(),
    );
    ```
    </details>
- if you used options like `#[inspectable(min = 0.0)]`, you now have to write
```rust
#[derive(Reflect, InspectorOptions)]
#[reflect(InspectorOptions)]
struct MyStruct {
    #[inspector(min = 0.0, max = 10.0, step = 0.01)]
    value: f32,
}

app.register_type::<MyStruct>();
```
- if you had your own custom UI implementation for a type and registered it using the `InspectableRegistry`, you can now instead add `InspectorEguiImpl` type data to the `TypeRegistry` for your type


If you have any feedback or are missing some features from the old release, please open an issue.
