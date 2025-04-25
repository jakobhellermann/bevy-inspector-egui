use bevy::prelude::*;
use bevy_asset::{ReflectAsset, UntypedAssetId};
use bevy_egui::{EguiContext, EguiContextPass, EguiContextSettings};
use bevy_inspector_egui::bevy_inspector::hierarchy::{hierarchy_ui, SelectedEntities};
use bevy_inspector_egui::bevy_inspector::{
    self, ui_for_entities_shared_components, ui_for_entity_with_children,
};
use bevy_inspector_egui::DefaultInspectorConfigPlugin;
use bevy_math::{DQuat, DVec3};
use std::any::TypeId;
// use bevy_mod_picking::backends::egui::EguiPointer;
// use bevy_mod_picking::prelude::*;
use bevy_reflect::TypeRegistry;
use bevy_render::camera::{CameraProjection, Viewport};
use bevy_window::{PrimaryWindow, Window};
use egui_dock::{DockArea, DockState, NodeIndex, Style};

use transform_gizmo_egui::{Gizmo, GizmoConfig, GizmoExt, GizmoOrientation};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // .add_plugins(bevy_framepace::FramepacePlugin) // reduces input lag
        .add_plugins(bevy_egui::EguiPlugin {
            enable_multipass_for_primary_context: true,
        })
        .add_plugins(DefaultInspectorConfigPlugin)
        // .add_plugins(bevy_mod_picking::plugins::DefaultPickingPlugins)
        .insert_resource(UiState::new())
        .add_systems(Startup, setup)
        .add_systems(EguiContextPass, show_ui_system)
        .add_systems(PostUpdate, set_camera_viewport.after(show_ui_system))
        // .add_systems(Update, auto_add_raycast_target)
        // .add_systems(Update, handle_pick_events)
        .register_type::<Option<Handle<Image>>>()
        .register_type::<AlphaMode>()
        .run();
}

/*
fn auto_add_raycast_target(
    mut commands: Commands,
    query: Query<Entity, (Without<PickRaycastTarget>, With<Handle<Mesh>>)>,
) {
    for entity in &query {
        commands
            .entity(entity)
            .insert((PickRaycastTarget::default(), PickableBundle::default()));
    }
}

fn handle_pick_events(
    mut ui_state: ResMut<UiState>,
    mut click_events: EventReader<PointerClick>,
    mut egui: ResMut<EguiContext>,
    egui_entity: Query<&EguiPointer>,
) {
    let egui_context = egui.ctx_mut();

    for click in click_events.iter() {
        if egui_entity.get(click.target()).is_ok() {
            continue;
        };

        let modifiers = egui_context.input().modifiers;
        let add = modifiers.ctrl || modifiers.shift;

        ui_state
            .selected_entities
            .select_maybe_add(click.target(), add);
    }
}
*/

#[derive(Component)]
struct MainCamera;

fn show_ui_system(world: &mut World) {
    let Ok(egui_context) = world
        .query_filtered::<&mut EguiContext, With<PrimaryWindow>>()
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
    primary_window: Query<&mut Window, With<PrimaryWindow>>,
    egui_settings: Single<&EguiContextSettings>,
    mut cam: Single<&mut Camera, With<MainCamera>>,
) {
    let Ok(window) = primary_window.single() else {
        return;
    };

    let scale_factor = window.scale_factor() * egui_settings.scale_factor;

    let viewport_pos = ui_state.viewport_rect.left_top().to_vec2() * scale_factor;
    let viewport_size = ui_state.viewport_rect.size() * scale_factor;

    let physical_position = UVec2::new(viewport_pos.x as u32, viewport_pos.y as u32);
    let physical_size = UVec2::new(viewport_size.x as u32, viewport_size.y as u32);

    // The desired viewport rectangle at its offset in "physical pixel space"
    let rect = physical_position + physical_size;

    let window_size = window.physical_size();
    // wgpu will panic if trying to set a viewport rect which has coordinates extending
    // past the size of the render target, i.e. the physical window in our case.
    // Typically this shouldn't happen- but during init and resizing etc. edge cases might occur.
    // Simply do nothing in those cases.
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
    state: DockState<EguiWindow>,
    viewport_rect: egui::Rect,
    selected_entities: SelectedEntities,
    selection: InspectorSelection,
    gizmo: Gizmo,
}

impl UiState {
    pub fn new() -> Self {
        let mut state = DockState::new(vec![EguiWindow::GameView]);
        let tree = state.main_surface_mut();
        let [game, _inspector] =
            tree.split_right(NodeIndex::root(), 0.75, vec![EguiWindow::Inspector]);
        let [game, _hierarchy] = tree.split_left(game, 0.2, vec![EguiWindow::Hierarchy]);
        let [_game, _bottom] =
            tree.split_below(game, 0.8, vec![EguiWindow::Resources, EguiWindow::Assets]);

        Self {
            state,
            selected_entities: SelectedEntities::default(),
            selection: InspectorSelection::Entities,
            viewport_rect: egui::Rect::NOTHING,
            gizmo: Gizmo::default(),
        }
    }

    fn ui(&mut self, world: &mut World, ctx: &mut egui::Context) {
        let mut tab_viewer = TabViewer {
            world,
            viewport_rect: &mut self.viewport_rect,
            selected_entities: &mut self.selected_entities,
            selection: &mut self.selection,
            gizmo: &mut self.gizmo,
        };
        DockArea::new(&mut self.state)
            .style(Style::from_egui(ctx.style().as_ref()))
            .show(ctx, &mut tab_viewer);
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
    gizmo: &'a mut Gizmo,
}

impl egui_dock::TabViewer for TabViewer<'_> {
    type Tab = EguiWindow;

    fn ui(&mut self, ui: &mut egui_dock::egui::Ui, window: &mut Self::Tab) {
        let type_registry = self.world.resource::<AppTypeRegistry>().0.clone();
        let type_registry = type_registry.read();

        match window {
            EguiWindow::GameView => {
                *self.viewport_rect = ui.clip_rect();

                draw_gizmo(ui, &mut self.gizmo, self.world, self.selected_entities);
            }
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
    }

    fn title(&mut self, window: &mut Self::Tab) -> egui_dock::egui::WidgetText {
        format!("{window:?}").into()
    }

    fn clear_background(&self, window: &Self::Tab) -> bool {
        !matches!(window, EguiWindow::GameView)
    }
}

#[allow(unused)]
fn draw_gizmo(
    ui: &mut egui::Ui,
    gizmo: &mut Gizmo,
    world: &mut World,
    selected_entities: &SelectedEntities,
) {
    let (cam_transform, projection) = world
        .query_filtered::<(&GlobalTransform, &Projection), With<MainCamera>>()
        .single(world)
        .expect("Camera not found");
    let view_matrix = Mat4::from(cam_transform.affine().inverse());
    let projection_matrix = projection.get_clip_from_view();

    if selected_entities.len() != 1 {
        #[allow(clippy::needless_return)]
        return;
    }

    for selected in selected_entities.iter() {
        let Some(transform) = world.get::<Transform>(selected) else {
            continue;
        };
        let model_matrix = transform.compute_matrix();

        gizmo.update_config(GizmoConfig {
            view_matrix: view_matrix.as_dmat4().into(),
            projection_matrix: projection_matrix.as_dmat4().into(),
            orientation: GizmoOrientation::Local,
            ..Default::default()
        });
        let transform = transform_gizmo_egui::math::Transform::from_scale_rotation_translation(
            transform.scale.as_dvec3(),
            transform.rotation.as_dquat(),
            transform.translation.as_dvec3(),
        );
        let Some((result, transforms)) = gizmo.interact(ui, &[transform]) else {
            continue;
        };
        let new = transforms[0];

        let mut transform = world.get_mut::<Transform>(selected).unwrap();
        *transform = Transform {
            translation: DVec3::from(new.translation).as_vec3(),
            rotation: DQuat::from_array(<[f64; 4]>::from(new.rotation)).as_quat(),
            scale: DVec3::from(new.scale).as_vec3(),
        };
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
) {
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
        MainCamera,
        // PickRaycastSource,
    ));
}
