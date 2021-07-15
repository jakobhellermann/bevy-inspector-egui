#![allow(clippy::type_complexity, clippy::identity_op)]
use bevy::{math::Vec3Swizzles, prelude::*, render::render_resource::PrimitiveTopology};
use bevy_inspector_egui::{Inspectable, InspectorPlugin};
use noise::NoiseFn;

#[derive(Default)]
struct GenerateEvent;

#[derive(Inspectable)]
struct Data {
    #[inspectable(min = 0.2, max = 2.0)]
    radius: f32,
    #[inspectable(min = 2, max = 24)]
    resolution: u8,
    color: Color,
    #[inspectable(collapse)]
    noise_settings: NoiseSettings,
}

#[derive(Clone, Inspectable)]
pub struct NoiseSettings {
    #[inspectable(min = 1, max = 4)]
    octaves: u8,
    #[inspectable(min = 0.1, max = 3.0, speed = 0.1)]
    persistence: f32,
    #[inspectable(min = 0.0, max = 2.0)]
    strength: f32,
    #[inspectable(min = 0.0, max = 2.0)]
    base_roughness: f32,
    #[inspectable(min = 0.0, max = 5.0)]
    roughness: f32,
    #[inspectable(min = 0.0, max = 3.0)]
    min_value: f32,
    #[inspectable(visual, min = Vec2::splat(-2.0), max = Vec2::splat(2.0))]
    offset: Vec2,
}

impl Default for Data {
    fn default() -> Self {
        Data {
            radius: 1.4,
            resolution: 16,
            color: Color::rgb(0.4, 0.2, 0.6),

            noise_settings: Default::default(),
        }
    }
}

impl Default for NoiseSettings {
    fn default() -> Self {
        Self {
            octaves: 5,
            persistence: 0.54,
            strength: 1.0,
            base_roughness: 0.71,
            roughness: 1.83,
            min_value: 1.1,
            offset: Vec2::ZERO,
        }
    }
}

fn main() {
    #[cfg(debug_assertions)]
    eprintln!("Try running with --release, it is much more responsive.");

    App::new()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(ClearColor(Color::rgb(0.058, 0.078, 0.098)))
        .add_plugins(DefaultPlugins)
        .add_plugin(InspectorPlugin::<Data>::new())
        .add_startup_system(setup.system())
        .add_system(generate.system())
        .run();
}

fn generate(
    data: Res<Data>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<(&Planet, &Handle<Mesh>, &Handle<StandardMaterial>)>,
) {
    if !data.is_changed() {
        return;
    }

    for (_, mesh, material) in query.iter() {
        let mesh = meshes.get_mut(mesh.clone()).unwrap();
        let material = materials.get_mut(material.clone()).unwrap();

        *mesh = data.as_mesh();
        material.base_color = data.color;
    }
}

#[derive(Component)]
struct Planet;

fn setup(
    mut commands: Commands,
    data: Res<Data>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = data.as_mesh();
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(mesh),
            material: materials.add(data.color.into()),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..Default::default()
        })
        .insert(Planet);
    commands.spawn_bundle(PointLightBundle {
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..Default::default()
    });
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(-3.0, 5.0, 10.0).looking_at(Vec3::default(), Vec3::Y),
        ..Default::default()
    });
}

impl Data {
    fn as_mesh(&self) -> Mesh {
        let planet_shape = PlanetShape::new(self.resolution, self.radius, &self.noise_settings);
        planet_shape.into()
    }
}

pub struct PlanetShape<'a> {
    resolution: u8,
    radius: f32,
    noise: &'a NoiseSettings,
}
impl<'a> PlanetShape<'a> {
    pub fn new(resolution: u8, radius: f32, noise_settings: &'a NoiseSettings) -> PlanetShape<'a> {
        assert!(resolution > 0);

        PlanetShape {
            resolution,
            radius,
            noise: noise_settings,
        }
    }

    fn elevation_at_point(&self, point: Vec3) -> f32 {
        let settings = &self.noise;
        let noise_fn = noise::SuperSimplex::new();

        let mut noise_value = 0.0;
        let mut frequency = settings.base_roughness;
        let mut amplitude = 1.0;

        for _ in 0..settings.octaves {
            let p = point * frequency + settings.offset.extend(0.0);
            let p = [p.x as f64, p.y as f64, p.z as f64];

            let v = (noise_fn.get(p) + 1.0) * 0.5;

            noise_value += v as f32 * amplitude;
            frequency *= settings.roughness;
            amplitude *= settings.persistence;
        }

        noise_value = (noise_value - settings.min_value).max(0.0);

        noise_value * settings.strength
    }

    fn point_on_planet(&self, point_on_unit_sphere: Vec3) -> Vec3 {
        let elevation = self.elevation_at_point(point_on_unit_sphere);
        point_on_unit_sphere * self.radius * (1.0 + elevation)
    }

    fn mesh_attributes(&self) -> (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<[f32; 2]>, Vec<u32>) {
        let n_attributes: u32 = self.resolution as u32 * self.resolution as u32 * 6;
        let mut positions: Vec<[f32; 3]> = vec![[0.0, 0.0, 0.0]; n_attributes as usize];
        let mut normals: Vec<[f32; 3]> = vec![[0.0, 0.0, 0.0]; n_attributes as usize];
        let uvs: Vec<[f32; 2]> = vec![[0.0, 0.0]; n_attributes as usize];

        let n_indices = (self.resolution as u32 - 1) * (self.resolution as u32 - 1) * 2 * 3 * 6;
        let mut indices = vec![0u32; n_indices as usize];

        let mut tri_index = 0;

        let faces = [Vec3::X, -Vec3::X, Vec3::Y, -Vec3::Y, Vec3::Z, -Vec3::Z];
        let mut i = 0;
        for &local_up in &faces {
            let axis_a = local_up.yzx();
            let axis_b = local_up.cross(axis_a);

            for y in 0..self.resolution {
                for x in 0..self.resolution {
                    let percent: Vec2 =
                        Vec2::new(x as f32, y as f32) / (self.resolution as f32 - 1.0);
                    let point_on_unit_cube: Vec3 = local_up
                        + (percent.x - 0.5) * 2.0 * axis_a
                        + (percent.y - 0.5) * 2.0 * axis_b;

                    let point_on_unit_sphere = point_on_unit_cube.normalize();
                    positions[i as usize] = self.point_on_planet(point_on_unit_sphere).into();
                    normals[i as usize] = point_on_unit_sphere.into();

                    if x != self.resolution - 1 && y != self.resolution - 1 {
                        indices[tri_index + 0] = i;
                        indices[tri_index + 1] = i + self.resolution as u32 + 1;
                        indices[tri_index + 2] = i + self.resolution as u32;

                        indices[tri_index + 3] = i;
                        indices[tri_index + 4] = i + 1;
                        indices[tri_index + 5] = i + self.resolution as u32 + 1;

                        tri_index += 6;
                    }

                    i += 1;
                }
            }
        }

        (positions, normals, uvs, indices)
    }
}

impl From<PlanetShape<'_>> for Mesh {
    fn from(s: PlanetShape) -> Self {
        let (positions, normals, uvs, indices) = s.mesh_attributes();

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh.set_indices(Some(bevy::render::mesh::Indices::U32(indices)));
        mesh
    }
}
