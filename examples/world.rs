use bevy::{prelude::*, render::mesh::Mesh};

use bevy_voxel::{
    render::{
        entity::{generate_chunk_mesh, Block},
        light::*,
        lod::lod_update,
        prelude::*,
    },
    terrain::*,
    world::{ChunkUpdate, Map, MapComponents, MapUpdates},
};

pub const CHUNK_SIZE: u32 = 5;
pub const WORLD_WIDTH: i32 = 64;
pub const WORLD_HEIGHT: i32 = 64;

pub fn main() {
    let params = TerrainGenParameters {
        seed: 0,
        noise_type: NoiseType::SuperSimplex,
        dimensions: NoiseDimensions::Two,
        chunk_size: CHUNK_SIZE,
        granularity: 1,
        octaves: vec![
            Octave {
                amplitude: 8.0,
                frequency: 0.01,
            },
            Octave {
                amplitude: 2.0,
                frequency: 0.05,
            },
            Octave {
                amplitude: 1.0,
                frequency: 0.10,
            },
        ],
        layers: vec![
            Layer {
                color: Color::rgb(0.08, 0.08, 0.08),
                height: f64::INFINITY,
            },
            Layer {
                color: Color::rgb(0.5, 0.5, 0.5),
                height: 16.0,
            },
            Layer {
                color: Color::rgb(0.396, 0.263, 0.129),
                height: 3.0,
            },
            Layer {
                color: Color::rgb(0.0, 0.416, 0.306),
                height: 1.0,
            },
        ],
    };
    App::build()
        .add_default_plugins()
        .add_plugin(VoxelRenderPlugin::default())
        .add_plugin(bevy_fly_camera::FlyCameraPlugin)
        .add_startup_system(setup.system())
        .add_resource(DirectionalLight {
            direction: Vec3::new(0.8, -1.0, 0.5).normalize(),
            intensity: 0.8,
        })
        .add_resource(AmbientLight { intensity: 0.05 })
        .add_resource(params)
        .add_stage_before(stage::PRE_UPDATE, "stage_terrain_generation")
        .add_stage_after("stage_terrain_generation", "stage_lod_update")
        .add_system_to_stage("stage_terrain_generation", terrain_generation.system())
        .add_system_to_stage("stage_lod_update", lod_update.system())
        .add_system_to_stage(
            stage::UPDATE,
            light_map_update::<line_drawing::Bresenham3d<i32>>.system(),
        )
        .add_system_to_stage(stage::UPDATE, shaded_light_update.system())
        .add_system_to_stage(stage::POST_UPDATE, chunk_update.system())
        .run();
}

/// set up a simple 3D scene
fn setup(mut commands: Commands) {
    let mut update = MapUpdates::default();
    let chunk_size = 2_i32.pow(CHUNK_SIZE as u32);
    let world_width_2 = WORLD_WIDTH / chunk_size / 2;
    let world_height = WORLD_HEIGHT / chunk_size;
    for cx in -world_width_2..world_width_2 {
        for cy in 0..world_height {
            for cz in -world_width_2..world_width_2 {
                update.updates.insert(
                    (cx, cy, cz, chunk_size as usize),
                    ChunkUpdate::GenerateChunk,
                );
            }
        }
    }
    commands
        .spawn(MapComponents {
            map_update: update,
            ..Default::default()
        })
        .spawn(bevy_fly_camera::FlyCamera::default());
}

fn chunk_update(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<VoxelMaterial>>,
    mut query: Query<(&Map<Block>, &mut MapUpdates)>,
) {
    for (map, mut update) in &mut query.iter() {
        let mut remove = Vec::new();
        for (&(x, y, z, w), update) in &update.updates {
            match update {
                ChunkUpdate::UpdateMesh => {}
                _ => continue,
            }
            remove.push((x, y, z, w));

            let w_2 = w as i32 / 2;
            let cx = x * w as i32 - w_2;
            let cy = y * w as i32 - w_2;
            let cz = z * w as i32 - w_2;
            let chunk = map.get((cx, cy, cz)).unwrap();

            let mesh = generate_chunk_mesh(&map, &chunk);
            if let Some(mesh) = mesh {
                commands.spawn(ChunkRenderComponents {
                    mesh: meshes.add(mesh),
                    material: materials.add(VoxelMaterial {
                        albedo: Color::WHITE,
                    }),
                    ..Default::default()
                });
            }
        }
        for coords in remove {
            update.updates.remove(&coords);
        }
    }
}
