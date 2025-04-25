# Changelog

## Version 0.31.0
- update to `bevy 0.16`

## Version 0.30.0
- add docs and labels for types without fields
- allow color grading controls with negative numbers
- `bevy_egui 0.33` and `egui 0.31`
- fix panic on unsupported color type

## Version 0.29.1
- add enabled-by-default `documentation` feature, which will display docs for reflected components on hover 

## Version 0.29.0
- fix sRGBA colors not being accurate [(#235)](https://github.com/jakobhellermann/bevy-inspector-egui/pull/235)
- add ability to perform custom cilters in hierarchy [(#231)](https://github.com/jakobhellermann/bevy-inspector-egui/pull/231)
- add `InspectorEguiImpl` for `Uuid`
- update to `egui 0.30`, `bevy_egui 0.32`

## Version 0.28.1
- fix inspector options not being respected
- fix handle short circuiting not working
- remove `egui_open_url` from default features as it is rarely used and pulls a lot of dependencies

## Version 0.28.0
- upgrade to `bevy_egui 0.31`
- update `bevy` to 0.15
- add ability to reflect Sets
- add some more auto name associations

## Version 0.27.0
- upgrade to `bevy_egui 0.30` / `egui 0.29`
- update `image` to 0.25

## Version 0.26.0
- update to `bevy_egui` 0.29
- make `RangeInclusive<fXX>` inspectable

## Version 0.25.2
- introduce features for egui clipboard and open\_url to improve wasm compat
- re-enable image and mesh inspector impls

## Version 0.25.1
- don't clamp linearrgba values
- fix render layer inspector
- actually make bevy\_egui/render optional
- put quick.rs plugins in their own schedule to fix system ambiguity

## Version 0.25.0
- update to bevy 0.14, egui 0.28
- make `bevy_render` an optional feature
- put plugin systems in their own Inspect schedule to avoid ambiguities

## Version 0.24.0

- update to egui 0.27 and bevy_egui 0.27
- add dropdown for images handles (thanks to @eupraxia05 https://github.com/jakobhellermann/bevy-inspector-egui/pull/190)
- add entity name filter to world inspector (thanks to @RobWalt https://github.com/jakobhellermann/bevy-inspector-egui/pull/194)

## Version 0.23.4
- fix compilation on wasm32

## Version 0.23.3
- add `InspectorPrimitive` trait to make custom UI for a type more convenient (see [example](../crates/bevy-inspector-egui/examples/basic/custom_type_ui.rs))

## Version 0.23
- bevy `0.13`, egui `0.26`
- show entityid beside name in restrictedworldview
- fix change detection in hierarchy UI
- improve macro path hygiene

## Version 0.22.1
- add `InspectorEguiImpl` for `PathBuf`

## Version 0.22
- update to `egui` `0.24` [#174](https://github.com/jakobhellermann/bevy-inspector-egui/pull/174)
- show entity ID beside name [#172](https://github.com/jakobhellermann/bevy-inspector-egui/pull/172)
- allow horizontal scrolling and improve UI [#166](https://github.com/jakobhellermann/bevy-inspector-egui/pull/166)
- allow inspecting `Range<T>`
- restrict `Time<Virtual>` to positive durations

## Version 0.21
- update to bevy `0.12` (@raffaeleragni, https://github.com/jakobhellermann/bevy-inspector-egui/pull/157)
- add ability to edit and insert elements into maps (@B-Reif, https://github.com/jakobhellermann/bevy-inspector-egui/pull/155)
- show asset path if available in inspector

## Version 0.20
- update egui to `0.23`
- add better list controls (https://github.com/jakobhellermann/bevy-inspector-egui/pull/152 by @B-Reif)

## Version 0.19
- improve some error messages
- update egui

## Version 0.18.4
- don't panic in quick plugin if no `PrimaryWindow` exists

## Version 0.18.3
- fix tab background in `egui_dock` example
- add ui for `RenderLayers`
- update to `syn 2.0`
- add suggestions for a fix in the error message for a missing `InspectorEguiImpl`


## Version 0.18.1
- fix nested entity name not being displayed
- fix `WorldInspectorPlugin` docs
- fix `egui_dock` system order

## Version 0.18.0
- update to `bevy` 0.10
- add `run_if` to `quick::*` plugins
- add `CommandQueue` to context
  - this allows us to display a despawn button for entities
- fix single struct field name not being displayed
- return whether an entity was selected from hierarchy
- don't show error for zero-sized types

## Version 0.17.0
- update to `bevy_egui` 0.19

## Version 0.16.6
- fix bug where the world inspector would only show components until it finds an unregistered one

## Version 0.16.4/0.16.5
- add `EntityOptions` with `display: EntityDisplay::Id | EntityDisplay::Components` (default to components)

## Version 0.16.3
- fix `ui_for_world_entities_filtered` not actually using filter

## Version 0.16.2
- add default `highlight-changes` feature for globally toggling the yellow highlight on changes
  - if you would like more fine-grained control, open an issue

## Version 0.16.1
- add `FilterQueryInspectorPlugin`
- fix `#[derive(InspectorOptions)]` derive on generic type
- wrap entities in collapsing header in `ui_for_world_entities`
- add utility function `bevy_inspector::ui_for_world_entities_filtered<F>`

## Version 0.16
- **There's a migration guide at [MIGRATION_GUIDE_0.15_0.16.md](./MIGRATION_GUIDE_0.15_0.16.md)**
- full rewrite of the crate
    - now centered around `Reflect` instead of the custom `Inspectable` derive macro
    - options can be specified using `#[derive(InspectorOptions)]`
    - `quick::*` plugins like `WorldInspectorPlugin`, `ResourceInspectorPlugin<T>`, `StateInspectorPlugin<S>` and `AssetInspectorPlugin<A>`
    - functions in `bevy_inspector` for bevy-specific debug UI: `ui_for_world`, `ui_for_resource`, `ui_for_value` etc.
- multiediting is supported using `InspectorUi::ui_for_reflect_many`
- read-only UI is now possible with `_readonly` variants

### Breaking changes:
- `InspectorPlugin<T>` got renamed to `quick::ResourceInspectorPlugin<T>` and doesn't automatically insert the resource anymore
- You need to call `register_type` for every type that you want to be able to see



## Version 0.15
- update to bevy_egui 0.18
- impl for `MouseButton`, `KeyCode`
- fix font path in example

## Version 0.14
- implement Inspectable for slices
- update to bevy 0.9

## Version 0.13

- update to `bevy_egui` 0.16
- allow negative scale on transform (thanks to @asherkin)
- implement inspectable for gamepad types (thanks to @johanhelsing)
- fix egui ID reuse
- fix compilation without `bevy_ui` feature

## Version 0.12
- fix compilation without the `bevy_ui` feature

## Version 0.12
- register `Vec3A`
- don't clamp `Val::Px` and `Val::Percent`
- display `Duration` with more precision and only update if changed
- update to bevy `0.8`

## Version 0.11
- update to `bevy_egui` 0.14
- update nalgebra feature to `nalgebra` `0.31`
- implement `Inspectable` for `VecDeque`

## Version 0.10
- update to bevy 0.7
- put `bevy_pbr`, `bevy_sprite`, `bevy_text`, `bevy_ui` behind feature flags (enabled by default)
- implement `Inspectable` for `Mesh2dHandle` (thanks to @tversteeg [https://github.com/jakobhellermann/bevy-inspector-egui/pull/51](#51))

## Version 0.9
- add `nalgebra` features for `bevy-inspector-egui-rapier` integration crate
- update to `bevy_egui` 0.12
- fix allowed value range for `OrthographicProjection`

## Version 0.8
- update to `bevy_egui` 0.11
- sort components in world inspector by default

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
