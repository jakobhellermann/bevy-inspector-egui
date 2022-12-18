# Examples

- `basic` - Basic features of the crate
  - [`inspector_options.rs`](./basic/inspector_options.rs) Shows how to use `InspectorOptions` derive to tweak the UI
- `quick` - Demonstrations of the quick plugins
  - [`world_inspector.rs`](./quick/world_inspector.rs) Example of the `WorldInspectorPlugin`
  - [`resource_inspector.rs`](./quick/resource_inspector.rs) Example of the `ResourceInspectorPlugin`
  - [`state_inspector.rs`](./quick/state_inspector.rs) Example of the `StateInspectorPlugin`
- `integrations` - examples showing how to integrate `bevy-inspector-egui` into your app in different ways
  - [`egui_dock.rs`](./integrations/egui_dock.rs) Full features examples of building your own mini-editor using `egui_dock` and `egui_gizmo`
  - [`side_panel.rs`](./integrations/side_panel.rs) Example of using a custom UI layout
