use bevy::prelude::*;
use bevy_inspector_egui::bevy_ecs_inspector::hierarchy::{hierarchy_ui, SelectedEntities};
use bevy_inspector_egui::bevy_ecs_inspector::{ui_for_all_assets, ui_for_entity, ui_for_resources};
use bevy_inspector_egui::DefaultInspectorConfigPlugin;
use bevy_render::camera::{CameraProjection, Viewport};
use egui_dock::{NodeIndex, Tree};
use egui_gizmo::GizmoMode;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(DefaultInspectorConfigPlugin)
        .add_plugin(bevy_egui::EguiPlugin)
        .insert_resource(UiState::new())
        .add_system_to_stage(CoreStage::PreUpdate, show_ui_system.at_end())
        .add_startup_system(setup)
        .add_system(set_camera_viewport)
        .add_system(set_gizmo_mode)
        .run();
}

#[derive(Component)]
struct MainCamera;

fn show_ui_system(world: &mut World) {
    let mut egui_context = world
        .resource_mut::<bevy_egui::EguiContext>()
        .ctx_mut()
        .clone();

    world.resource_scope::<UiState, _>(|world, mut ui_state| ui_state.ui(world, &mut egui_context));
}

// make camera only render to view not obstructed by UI
fn set_camera_viewport(
    ui_state: Res<UiState>,
    windows: Res<Windows>,
    egui_settings: Res<bevy_egui::EguiSettings>,
    mut cameras: Query<&mut Camera, With<MainCamera>>,
) {
    let mut cam = cameras.single_mut();

    let window = windows.primary();
    let scale_factor = window.scale_factor() * egui_settings.scale_factor;

    let viewport_pos = ui_state.viewport_rect.left_top().to_vec2() * scale_factor as f32;
    let viewport_size = ui_state.viewport_rect.size() * scale_factor as f32;

    cam.viewport = Some(Viewport {
        physical_position: UVec2::new(viewport_pos.x as u32, viewport_pos.y as u32),
        physical_size: UVec2::new(viewport_size.x as u32, viewport_size.y as u32),
        depth: 0.0..1.0,
    });
}

fn set_gizmo_mode(input: Res<Input<KeyCode>>, mut ui_state: ResMut<UiState>) {
    for (key, mode) in [
        (KeyCode::R, GizmoMode::Rotate),
        (KeyCode::T, GizmoMode::Translate),
        (KeyCode::S, GizmoMode::Scale),
    ] {
        if input.just_pressed(key) {
            ui_state.gizmo_mode = mode;
        }
    }
}

#[derive(Resource)]
struct UiState {
    tree: Tree<Window>,
    viewport_rect: egui::Rect,
    selected_entities: SelectedEntities,
    gizmo_mode: GizmoMode,
}

impl UiState {
    pub fn new() -> Self {
        let mut tree = Tree::new(vec![Window::GameView]);
        let [game, _inspector] = tree.split_right(NodeIndex::root(), 0.75, vec![Window::Inspector]);
        let [game, _hierarchy] = tree.split_left(game, 0.2, vec![Window::Hierarchy]);
        let [_game, _bottom] = tree.split_below(game, 0.8, vec![Window::Resources, Window::Assets]);

        Self {
            tree,
            selected_entities: SelectedEntities::default(),
            viewport_rect: egui::Rect::NOTHING,
            gizmo_mode: GizmoMode::Translate,
        }
    }

    fn ui(&mut self, world: &mut World, ctx: &mut egui::Context) {
        let mut tab_viewer = TabViewer {
            world,
            viewport_rect: &mut self.viewport_rect,
            selected_entities: &mut self.selected_entities,
            gizmo_mode: self.gizmo_mode,
        };
        egui_dock::DockArea::new(&mut self.tree).show(ctx, &mut tab_viewer);
    }
}

#[derive(Debug)]
enum Window {
    GameView,
    Hierarchy,
    Resources,
    Assets,
    Inspector,
}

struct TabViewer<'a> {
    world: &'a mut World,
    selected_entities: &'a mut SelectedEntities,
    viewport_rect: &'a mut egui::Rect,
    gizmo_mode: GizmoMode,
}

impl egui_dock::TabViewer for TabViewer<'_> {
    type Tab = Window;

    fn ui(&mut self, ui: &mut egui::Ui, window: &mut Self::Tab) {
        match window {
            Window::GameView => {
                (*self.viewport_rect, _) =
                    ui.allocate_exact_size(ui.available_size(), egui::Sense::hover());

                let (cam_transform, projection) = self
                    .world
                    .query_filtered::<(&GlobalTransform, &Projection), With<MainCamera>>()
                    .single(self.world);
                let view_matrix = Mat4::from(cam_transform.affine().inverse());
                let projection_matrix = projection.get_projection_matrix();

                for selected in self.selected_entities.iter() {
                    let transform = self.world.get::<Transform>(selected).unwrap();
                    let model_matrix = transform.compute_matrix();

                    let Some(result) = egui_gizmo::Gizmo::new(selected)
                        .model_matrix(model_matrix.to_cols_array_2d())
                        .view_matrix(view_matrix.to_cols_array_2d())
                        .projection_matrix(projection_matrix.to_cols_array_2d())
                        .orientation(egui_gizmo::GizmoOrientation::Local)
                        .mode(self.gizmo_mode)
                        .interact(ui)
                    else { continue };

                    let mut transform = self.world.get_mut::<Transform>(selected).unwrap();
                    *transform =
                        Transform::from_matrix(Mat4::from_cols_array_2d(&result.transform));
                }
            }
            Window::Hierarchy => hierarchy_ui(self.world, ui, self.selected_entities),
            Window::Resources => ui_for_resources(self.world, ui),
            Window::Assets => ui_for_all_assets(self.world, ui),
            Window::Inspector => {
                for entity in self.selected_entities.iter() {
                    ui_for_entity(self.world, entity, ui, self.selected_entities.len() > 1);
                }
            }
        }
    }

    fn title(&mut self, window: &mut Self::Tab) -> egui::WidgetText {
        format!("{window:?}").into()
    }

    fn clear_background(&self, window: &Self::Tab) -> bool {
        !matches!(window, Window::GameView)
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
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Box::new(
            box_size,
            box_thickness,
            box_size,
        ))),
        transform,
        material: materials.add(StandardMaterial {
            base_color: Color::rgb(0.63, 0.065, 0.05),
            ..Default::default()
        }),
        ..Default::default()
    });
    // right - green
    let mut transform = Transform::from_xyz(box_offset, box_offset, 0.0);
    transform.rotate(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2));
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Box::new(
            box_size,
            box_thickness,
            box_size,
        ))),
        transform,
        material: materials.add(StandardMaterial {
            base_color: Color::rgb(0.14, 0.45, 0.091),
            ..Default::default()
        }),
        ..Default::default()
    });
    // bottom - white
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Box::new(
            box_size + 2.0 * box_thickness,
            box_thickness,
            box_size,
        ))),
        material: materials.add(StandardMaterial {
            base_color: Color::rgb(0.725, 0.71, 0.68),
            ..Default::default()
        }),
        ..Default::default()
    });
    // top - white
    let transform = Transform::from_xyz(0.0, 2.0 * box_offset, 0.0);
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Box::new(
            box_size + 2.0 * box_thickness,
            box_thickness,
            box_size,
        ))),
        transform,
        material: materials.add(StandardMaterial {
            base_color: Color::rgb(0.725, 0.71, 0.68),
            ..Default::default()
        }),
        ..Default::default()
    });
    // back - white
    let mut transform = Transform::from_xyz(0.0, box_offset, -box_offset);
    transform.rotate(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2));
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Box::new(
            box_size + 2.0 * box_thickness,
            box_thickness,
            box_size + 2.0 * box_thickness,
        ))),
        transform,
        material: materials.add(StandardMaterial {
            base_color: Color::rgb(0.725, 0.71, 0.68),
            ..Default::default()
        }),
        ..Default::default()
    });

    // ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.02,
    });
    // top light
    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane { size: 0.4 })),
            transform: Transform::from_matrix(Mat4::from_scale_rotation_translation(
                Vec3::ONE,
                Quat::from_rotation_x(std::f32::consts::PI),
                Vec3::new(0.0, box_size + 0.5 * box_thickness, 0.0),
            )),
            material: materials.add(StandardMaterial {
                base_color: Color::WHITE,
                emissive: Color::WHITE * 100.0,
                ..Default::default()
            }),
            ..Default::default()
        })
        .with_children(|builder| {
            builder.spawn(PointLightBundle {
                point_light: PointLight {
                    color: Color::WHITE,
                    intensity: 25.0,
                    ..Default::default()
                },
                transform: Transform::from_translation((box_thickness + 0.05) * Vec3::Y),
                ..Default::default()
            });
        });
    // directional light
    const HALF_SIZE: f32 = 10.0;
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 10000.0,
            shadow_projection: OrthographicProjection {
                left: -HALF_SIZE,
                right: HALF_SIZE,
                bottom: -HALF_SIZE,
                top: HALF_SIZE,
                near: -10.0 * HALF_SIZE,
                far: 10.0 * HALF_SIZE,
                ..Default::default()
            },
            ..Default::default()
        },
        transform: Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::PI / 2.0)),
        ..Default::default()
    });

    // camera
    commands
        .spawn(Camera3dBundle {
            transform: Transform::from_xyz(0.0, box_offset, 4.0)
                .looking_at(Vec3::new(0.0, box_offset, 0.0), Vec3::Y),
            ..Default::default()
        })
        .insert(MainCamera);
}
