# Changelog

## Version 0.7
- update `egui` to 0.16 and `bevy` to 0.6
- ability to filter entities by name in world inspector
- add `app.register_inspectable` for easy `InspectableRegistry` access
- add `highlight_changes` to world inspector params
- use `&mut Context` instead of `&Context`
- fix external changes to quaternions not being displayed

## Version 0.6
- update to `egui` 0.13, `bevy_egui` 0.6
- update to rapier `0.10`
- rename `rapier` feature to `rapier3d`
- fix `Quat` implementation to not modify unrelated state
- expose `InspectorWindows` to allow controlling whether inspector ui is shown

## Version 0.5.1
### Added
- add `rapier2d` feature

## Version 0.5
- support for multiple windows using `InspectorPlugin::new.on_window(window_id)` ([example](https://github.com/jakobhellermann/bevy-inspector-egui/blob/main/examples/two_windows.rs))
- `InspectorQuerySingle` (like `.single_mut` on a query)
- `ignore`, `read_only` and `wrapper` built-in attributes
- optional `ui_ctx`

## Version 0.4

### Added

- many new implementations (`Handle<Texture>`, `ColorMaterial`, `Entity`, `World`, UI stuff, tuples)
- `WorldInspectorPlugin` for displaying the whole entity tree in a UI panel
- `widgets::InspectorQuery`: display entity tree for select entities
  Use like
  ```rust
  #[derive(Inspectable)]
  struct Inspector {
    root_elements: InspectorQuery<Entity, Without<Parent>>,
    collider: InspectorQuery<&'static mut Transform, With<Collider0>>,
  }
  ```
- `widgets::InspectableButton`: sends event upon button click. Usage looks like
- derive `Inspectable` for enums with data
- drag and drop into texture

### Changed

- `Inspectable::ui` now returns a bool which indicates whether any data has changed. You can now check via `Res<T>::changed` whether data was modified.
- rename `InspectorPlugin::thread_local -> new, InspectorPlugin::new -> shared`
- require `FromResources` instead of `Default` on the inspectable data
- update to `bevy 0.5`
- `Inspectable::Attributes` require `Clone`
- show `Vec2` as two number fields by default, use `#[inspectable(visual)]` for old behaviour
- properly give ids to egui
- mark inspected components in the world inspector as mutated
- quaternions are now displayed as euler angles by default

### Fixed

- UI fixes for derived structs
- clamp number types to their `min`/`max` values if set

## Version 0.3.0

- another change to the `Inspectable` trait, it now gets a context from which bevy's `Resources` can be access (provided it is started in thread-local mode)

  ```rust
  struct Context<'a> {
    resources: Option<&'a bevy::ecs::Resources>
  }

  trait Inspectable {
      type Attributes: Default;
      fn ui(&mut self, ui: &mut egui::Ui, options: Self::Attributes, context: &Context);
  }
  ```

  This allows implementations for things like `Handle<T>` that need access so more information.

  When access to the resources is needed, add the plugin like

  ```rust
  add.add_plugin(InspectorPlugin::<T>::thread_local())
  ```

- implementations of `Inspectable` for `StandardMaterial` and `Range{,Inclusive}<T>`

## Version 0.2.0

### Added

- [impl Inspectable for the remaining number types](https://github.com/jakobhellermann/bevy-inspector-egui/commit/b072035c1f14baf189274e9a166f2bae1adf2f70)
- [impl Inspectable Mat{3,4}](https://github.com/jakobhellermann/bevy-inspector-egui/commit/070e2a45dec945a83e42ea268cc384cf6857a7bc)
- [`ReflectedUI` wrapper type for automatically figuring out how to display a type based on the `Reflect` impl](https://github.com/jakobhellermann/bevy-inspector-egui/commit/caf1c31c25349d29011d5772bba7eb2709879eb5)
  ```rust
  #[derive(Inspectable, Default, Debug)]
  struct Data { timer: ReflectedUI<Timer> }
  ```
- [allow multiple inspector windows](https://github.com/jakobhellermann/bevy-inspector-egui/commit/980de51e181fe486ac74312474c33a34e0a77293)

### Changed

- [rename NumberAttributes::step to speed](https://github.com/jakobhellermann/bevy-inspector-egui/commit/b2aeb1735bdf0b5d8d68386e22f1e73437cbf733)
- [use the relecant number type to specify min and max of the NumberAttributes instead of always f64](https://github.com/jakobhellermann/bevy-inspector-egui/commit/28fa524390874026b0087b5bfac18b5e15ad1eec)
- [simplify inspectable trait, rename FieldOptions to Options](https://github.com/jakobhellermann/bevy-inspector-egui/commit/af032d42ac7f816ccc17a79eaf2e19f3768bb968)
  The trait now looks like this:
  ```rust
  trait Inspectable {
      type Attributes: Default;
      fn ui(&mut self, ui: &mut egui::Ui, options: Self::Attributes);
  }
  ```
- [try to convert Attributes using From::from](https://github.com/jakobhellermann/bevy-inspector-egui/commit/43e84d885ca080a6f2e62bbcfb396f27f92237fd)

### Fixed

- [normalize quaternion in `Transform` so that it stays a valid rotation](https://github.com/jakobhellermann/bevy-inspector-egui/commit/48fce89f7692408bad4841b126f7e68d8995fffd)
- [disallow negative scale](https://github.com/jakobhellermann/bevy-inspector-egui/commit/c1ffc5d8898d2db882d60f88f8010c6121ca41ad)

## Version 0.1.0

### Added

- first version with `Inspectable` support for number types `u8`, `i32` and `f{32,64}`, `String` and `Vec<T>` and bevy's `Color`, `Transform`, `Quat`, `Vec{2,3,4}`
- derive `Inspectable` for unit enums (displays a dropdown)
- derive `Inspectable` for struct
- `#[inspectable(label = x, collapse)]` builtins for struct derives
- `nightly` features for array `Inspectable` impls
