# Examples

- `basic` - Basic features of the crate
  - [`inspector_options.rs`](./basic/inspector_options.rs) Shows how to use `InspectorOptions` derive to tweak the UI
  - [`resource_inspector_manual.rs`](./basic/resource_inspector_manual.rs) Shows how to customize and build your own inspector windows
- `quick` - Demonstrations of the quick plugins
  - [`world_inspector.rs`](./quick/world_inspector.rs) Example of the `WorldInspectorPlugin`
  - [`world_inspector_mouse_check.rs`](./quick/world_inspector_mouse_check.rs) Example of the `WorldInspectorPlugin` with camera and checking for egui components
  - [`resource_inspector.rs`](./quick/resource_inspector.rs) Example of the `ResourceInspectorPlugin`
  - [`filter_query_inspector.rs`](./quick/filter_query_inspector.rs) Example of the `FilterQueryInspectorPlugin`
  - [`asset_inspector.rs`](./quick/asset_inspector.rs) Example of the `AssetInspectorPlugin`
  - [`state_inspector.rs`](./quick/state_inspector.rs) Example of the `StateInspectorPlugin`
- `integrations` - examples showing how to integrate `bevy-inspector-egui` into your app in different ways
  - [`egui_dock.rs`](./integrations/egui_dock.rs) Full features examples of building your own mini-editor using `egui_dock` and `egui_gizmo`
  - [`side_panel.rs`](./integrations/side_panel.rs) Example of using a custom UI layout
