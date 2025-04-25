use bevy::{
    asset::{ReflectAsset, UntypedAssetId},
    color::palettes::tailwind::*,
    picking::pointer::{PointerAction, PointerInput, PointerInteraction},
    prelude::*,
};
use bevy_camera::{Viewport, visibility::RenderLayers};
use bevy_egui::{
    EguiContext, EguiContextSettings, EguiGlobalSettings, EguiPrimaryContextPass,
    PrimaryEguiContext,
};
use bevy_inspector_egui::DefaultInspectorConfigPlugin;
use bevy_inspector_egui::bevy_inspector::hierarchy::{SelectedEntities, hierarchy_ui};
use bevy_inspector_egui::bevy_inspector::{
    self, ui_for_entities_shared_components, ui_for_entity_with_children,
};
use bevy_reflect::TypeRegistry;
use bevy_window::{PrimaryWindow, Window};
use egui::LayerId;
use egui_tiles::TileId;
use std::any::TypeId;
use transform_gizmo_bevy::{GizmoCamera, GizmoTarget, TransformGizmoPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(MeshPickingPlugin)
        .add_plugins(TransformGizmoPlugin)
        // .add_plugins(bevy_framepace::FramepacePlugin) // reduces input lag
        .add_plugins(bevy_egui::EguiPlugin::default())
        .add_plugins(DefaultInspectorConfigPlugin)
        .insert_resource(UiState::new())
        .add_systems(Startup, setup)
        .add_systems(EguiPrimaryContextPass, show_ui_system)
        .add_systems(PostUpdate, set_camera_viewport.after(show_ui_system))
        .add_systems(Update, draw_mesh_intersections)
        .add_systems(PostUpdate, handle_pick_events)
        .register_type::<Option<Handle<Image>>>()
        .register_type::<AlphaMode>()
        .run();
}

fn draw_mesh_intersections(pointers: Query<&PointerInteraction>, mut gizmos: Gizmos) {
    for (point, normal) in pointers
        .iter()
        .filter_map(|interaction| interaction.get_nearest_hit())
        .filter_map(|(_entity, hit)| hit.position.zip(hit.normal))
    {
        gizmos.sphere(point, 0.05, RED_500);
        gizmos.arrow(point, point + normal.normalize() * 0.5, PINK_100);
    }
}

fn handle_pick_events(
    mut ui_state: ResMut<UiState>,
    mut click_events: MessageReader<PointerInput>,
    pointers: Query<&PointerInteraction>,
    button: Res<ButtonInput<KeyCode>>,
    gizmo_targets: Query<(Entity, &GizmoTarget)>,
    mut commands: Commands,
) {
    if !ui_state.pointer_in_viewport {
        return;
    }
    for event in click_events.read() {
        if let PointerAction::Press(PointerButton::Primary) = event.action {
            if gizmo_targets.iter().any(|(_, target)| target.is_focused()) {
                continue;
            }

            for interaction in pointers {
                for (entity, _) in interaction.as_slice() {
                    let add = button.any_pressed([KeyCode::ControlLeft, KeyCode::ShiftLeft]);
                    ui_state.selected_entities.select_maybe_add(*entity, add);

                    for (target, _) in gizmo_targets.iter() {
                        if !ui_state.selected_entities.contains(target) {
                            commands.entity(target).remove::<GizmoTarget>();
                        }
                    }
                    for selected in ui_state.selected_entities.iter() {
                        if !gizmo_targets.contains(selected) {
                            commands.entity(selected).insert(GizmoTarget::default());
                        }
                    }
                }
            }
        }
    }
}

fn show_ui_system(world: &mut World) {
    let Ok(egui_context) = world
        .query_filtered::<&mut EguiContext, With<PrimaryEguiContext>>()
        .single(world)
    else {
        return;
    };
    let mut egui_context = egui_context.clone();

    world.resource_scope::<UiState, _>(|world, mut ui_state| {
        ui_state.ui(world, egui_context.get_mut())
    });
}

// make camera only render to view not obstructed by UI
fn set_camera_viewport(
    ui_state: Res<UiState>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut cam: Single<&mut Camera, Without<PrimaryEguiContext>>,
    egui_settings: Single<&EguiContextSettings>,
) {
    let scale_factor = window.scale_factor() * egui_settings.scale_factor;

    let viewport_pos = ui_state.viewport_rect.left_top().to_vec2() * scale_factor;
    let viewport_size = ui_state.viewport_rect.size() * scale_factor;

    let physical_position = UVec2::new(viewport_pos.x as u32, viewport_pos.y as u32);
    let physical_size = UVec2::new(viewport_size.x as u32, viewport_size.y as u32);

    let rect = physical_position + physical_size;

    let window_size = window.physical_size();
    if rect.x <= window_size.x && rect.y <= window_size.y {
        cam.viewport = Some(Viewport {
            physical_position,
            physical_size,
            depth: 0.0..1.0,
        });
    }
}

#[derive(Eq, PartialEq)]
enum InspectorSelection {
    Entities,
    Resource(TypeId, String),
    Asset(TypeId, String, UntypedAssetId),
}

#[derive(Resource)]
struct UiState {
    state: egui_tiles::Tree<EguiWindow>,
    viewport_rect: egui::Rect,
    selected_entities: SelectedEntities,
    selection: InspectorSelection,
    pointer_in_viewport: bool,
}

impl UiState {
    pub fn new() -> Self {
        let mut tiles = egui_tiles::Tiles::default();

        let left = vec![tiles.insert_pane(EguiWindow::Hierarchy)];
        let left = tiles.insert_tab_tile(left);

        let center = vec![tiles.insert_pane(EguiWindow::GameView)];
        let center = tiles.insert_tab_tile(center);

        let right = vec![tiles.insert_pane(EguiWindow::Inspector)];
        let right = tiles.insert_tab_tile(right);

        let bottom = vec![
            tiles.insert_pane(EguiWindow::Resources),
            tiles.insert_pane(EguiWindow::Assets),
        ];
        let bottom = tiles.insert_tab_tile(bottom);

        let lc = vec![left, center];
        let lc = tiles.insert_horizontal_tile(lc);
        set_linear_share(&mut tiles, lc, center, 4.);

        let lcb = vec![lc, bottom];
        let lcb = tiles.insert_vertical_tile(lcb);
        set_linear_share(&mut tiles, lcb, lc, 4.);

        let all = vec![lcb, right];
        let all = tiles.insert_horizontal_tile(all);
        set_linear_share(&mut tiles, all, lcb, 4.);

        Self {
            state: egui_tiles::Tree::new("tiles", all, tiles),
            selected_entities: SelectedEntities::default(),
            selection: InspectorSelection::Entities,
            viewport_rect: egui::Rect::NOTHING,
            pointer_in_viewport: false,
        }
    }

    fn ui(&mut self, world: &mut World, ctx: &mut egui::Context) {
        let mut tab_viewer = TabViewer {
            world,
            viewport_rect: &mut self.viewport_rect,
            selected_entities: &mut self.selected_entities,
            selection: &mut self.selection,
            pointer_in_viewport: &mut self.pointer_in_viewport,
        };

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(&ctx, |ui| {
                self.state.ui(&mut tab_viewer, ui);
            });
    }
}

impl Default for UiState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
enum EguiWindow {
    GameView,
    Hierarchy,
    Resources,
    Assets,
    Inspector,
}

struct TabViewer<'a> {
    world: &'a mut World,
    selected_entities: &'a mut SelectedEntities,
    selection: &'a mut InspectorSelection,
    viewport_rect: &'a mut egui::Rect,
    pointer_in_viewport: &'a mut bool,
}

impl egui_tiles::Behavior<EguiWindow> for TabViewer<'_> {
    fn pane_ui(
        &mut self,
        ui: &mut egui::Ui,
        _: egui_tiles::TileId,
        window: &mut EguiWindow,
    ) -> egui_tiles::UiResponse {
        if !matches!(window, EguiWindow::GameView) {
            ui.painter().rect(
                ui.available_rect_before_wrap(),
                0.0,
                ui.style().visuals.panel_fill,
                egui::Stroke::NONE,
                egui::StrokeKind::Outside,
            );
        }

        let type_registry = self.world.resource::<AppTypeRegistry>().0.clone();
        let type_registry = type_registry.read();

        match *window {
            EguiWindow::GameView => *self.viewport_rect = ui.clip_rect(),
            EguiWindow::Hierarchy => {
                let selected = hierarchy_ui(self.world, ui, self.selected_entities);
                if selected {
                    *self.selection = InspectorSelection::Entities;
                }
            }
            EguiWindow::Resources => select_resource(ui, &type_registry, self.selection),
            EguiWindow::Assets => select_asset(ui, &type_registry, self.world, self.selection),
            EguiWindow::Inspector => match *self.selection {
                InspectorSelection::Entities => match self.selected_entities.as_slice() {
                    &[entity] => ui_for_entity_with_children(self.world, entity, ui),
                    entities => ui_for_entities_shared_components(self.world, entities, ui),
                },
                InspectorSelection::Resource(type_id, ref name) => {
                    ui.label(name);
                    bevy_inspector::by_type_id::ui_for_resource(
                        self.world,
                        type_id,
                        ui,
                        name,
                        &type_registry,
                    )
                }
                InspectorSelection::Asset(type_id, ref name, handle) => {
                    ui.label(name);
                    bevy_inspector::by_type_id::ui_for_asset(
                        self.world,
                        type_id,
                        handle,
                        ui,
                        &type_registry,
                    );
                }
            },
        }

        *self.pointer_in_viewport = ui
            .ctx()
            .rect_contains_pointer(LayerId::background(), self.viewport_rect.shrink(16.));

        egui_tiles::UiResponse::None
    }

    fn tab_title_for_pane(&mut self, pane: &EguiWindow) -> egui::WidgetText {
        format!("{pane:?}").into()
    }

    fn simplification_options(&self) -> egui_tiles::SimplificationOptions {
        egui_tiles::SimplificationOptions {
            prune_single_child_tabs: false,
            ..default()
        }
    }
}

fn select_resource(
    ui: &mut egui::Ui,
    type_registry: &TypeRegistry,
    selection: &mut InspectorSelection,
) {
    let mut resources: Vec<_> = type_registry
        .iter()
        .filter(|registration| registration.data::<ReflectResource>().is_some())
        .map(|registration| {
            (
                registration.type_info().type_path_table().short_path(),
                registration.type_id(),
            )
        })
        .collect();
    resources.sort_by(|(name_a, _), (name_b, _)| name_a.cmp(name_b));

    for (resource_name, type_id) in resources {
        let selected = match *selection {
            InspectorSelection::Resource(selected, _) => selected == type_id,
            _ => false,
        };

        if ui.selectable_label(selected, resource_name).clicked() {
            *selection = InspectorSelection::Resource(type_id, resource_name.to_string());
        }
    }
}

fn select_asset(
    ui: &mut egui::Ui,
    type_registry: &TypeRegistry,
    world: &World,
    selection: &mut InspectorSelection,
) {
    let mut assets: Vec<_> = type_registry
        .iter()
        .filter_map(|registration| {
            let reflect_asset = registration.data::<ReflectAsset>()?;
            Some((
                registration.type_info().type_path_table().short_path(),
                registration.type_id(),
                reflect_asset,
            ))
        })
        .collect();
    assets.sort_by(|(name_a, ..), (name_b, ..)| name_a.cmp(name_b));

    for (asset_name, asset_type_id, reflect_asset) in assets {
        let handles: Vec<_> = reflect_asset.ids(world).collect();

        ui.collapsing(format!("{asset_name} ({})", handles.len()), |ui| {
            for handle in handles {
                let selected = match *selection {
                    InspectorSelection::Asset(_, _, selected_id) => selected_id == handle,
                    _ => false,
                };

                if ui
                    .selectable_label(selected, format!("{handle:?}"))
                    .clicked()
                {
                    *selection =
                        InspectorSelection::Asset(asset_type_id, asset_name.to_string(), handle);
                }
            }
        });
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut egui_global_settings: ResMut<EguiGlobalSettings>,
) {
    egui_global_settings.auto_create_primary_context = false;

    let box_size = 2.0;
    let box_thickness = 0.15;
    let box_offset = (box_size + box_thickness) / 2.0;

    // left - red
    let mut transform = Transform::from_xyz(-box_offset, box_offset, 0.0);
    transform.rotate(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2));

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(box_size, box_thickness, box_size))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.63, 0.065, 0.05),
            ..Default::default()
        })),
        transform,
    ));
    // right - green
    let mut transform = Transform::from_xyz(box_offset, box_offset, 0.0);
    transform.rotate(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2));
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(box_size, box_thickness, box_size))),
        transform,
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.14, 0.45, 0.091),
            ..Default::default()
        })),
    ));
    // bottom - white
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(
            box_size + 2.0 * box_thickness,
            box_thickness,
            box_size,
        ))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.725, 0.71, 0.68),
            ..Default::default()
        })),
    ));
    // top - white
    let transform = Transform::from_xyz(0.0, 2.0 * box_offset, 0.0);
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(
            box_size + 2.0 * box_thickness,
            box_thickness,
            box_size,
        ))),
        transform,
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.725, 0.71, 0.68),
            ..Default::default()
        })),
    ));
    // back - white
    let mut transform = Transform::from_xyz(0.0, box_offset, -box_offset);
    transform.rotate(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2));
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(
            box_size + 2.0 * box_thickness,
            box_thickness,
            box_size + 2.0 * box_thickness,
        ))),
        transform,
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.725, 0.71, 0.68),
            ..Default::default()
        })),
    ));

    // ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.02,
        ..default()
    });
    // top light
    commands
        .spawn((
            Mesh3d(meshes.add(Plane3d::default().mesh().size(0.4, 0.4))),
            Transform::from_matrix(Mat4::from_scale_rotation_translation(
                Vec3::ONE,
                Quat::from_rotation_x(std::f32::consts::PI),
                Vec3::new(0.0, box_size + 0.5 * box_thickness, 0.0),
            )),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::WHITE,
                emissive: LinearRgba::WHITE * 100.0,
                ..Default::default()
            })),
        ))
        .with_children(|builder| {
            builder.spawn((
                PointLight {
                    color: Color::WHITE,
                    intensity: 25000.0,
                    ..Default::default()
                },
                Transform::from_translation((box_thickness + 0.05) * Vec3::Y),
            ));
        });
    // directional light
    commands.spawn((
        DirectionalLight {
            illuminance: 2000.0,
            ..default()
        },
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::PI / 2.0)),
    ));

    // camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, box_offset, 4.0)
            .looking_at(Vec3::new(0.0, box_offset, 0.0), Vec3::Y),
        GizmoCamera,
        // PickRaycastSource,
    ));

    // egui camera
    commands.spawn((
        Camera2d,
        Name::new("Egui Camera"),
        PrimaryEguiContext,
        RenderLayers::none(),
        Camera {
            order: 1,
            clear_color: ClearColorConfig::None,
            ..default()
        },
    ));
}

#[track_caller]
fn set_linear_share<Pane>(
    tiles: &mut egui_tiles::Tiles<Pane>,
    container: TileId,
    tab: TileId,
    share: f32,
) {
    match tiles.get_mut(container).unwrap() {
        egui_tiles::Tile::Container(egui_tiles::Container::Linear(linear)) => {
            /*if !linear.shares.iter().find(|&(&x, _)| x == tab).is_some() {
                panic!(
                    "Expected {tab:?} in {container:?}, found {:?}",
                    linear.shares.iter().collect::<Vec<_>>()
                );
            }*/
            linear.shares.set_share(tab, share)
        }
        _ => unreachable!(),
    }
}
