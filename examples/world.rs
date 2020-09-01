use bevy::{asset::Handle, prelude::*, render::mesh::Mesh};

use bevy_voxel::{
    render::{
        entity::{generate_chunk_mesh, Block, ChunkMeshUpdate},
        light::*,
        lod::lod_update,
        prelude::*,
    },
    terrain::*,
    world::Chunk,
};

pub const CHUNK_SIZE: u32 = 5;
pub const WORLD_WIDTH: i32 = 128;
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
        .add_system_to_stage(stage::PRE_UPDATE, lod_update.system())
        .add_system_to_stage(stage::UPDATE, terrain_generation.system())
        .add_system_to_stage(
            stage::UPDATE,
            shaded_light_update::<line_drawing::WalkVoxels<f32, i32>>.system(),
        )
        .add_system_to_stage(stage::POST_UPDATE, chunk_update.system())
        .run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<VoxelMaterial>>,
) {
    let chunk_size = 2_i32.pow(CHUNK_SIZE as u32);
    let world_width_2 = WORLD_WIDTH / chunk_size / 2;
    let world_height = WORLD_HEIGHT / chunk_size;
    for cx in -world_width_2..world_width_2 {
        for cy in 0..world_height {
            for cz in -world_width_2..world_width_2 {
                let chunk = Chunk::new(1, (cx, cy, cz));
                let mesh = meshes.add(generate_chunk_mesh(&chunk));
                // add entities to the world
                commands
                    // chunk
                    .spawn(ChunkRenderComponents {
                        chunk,
                        mesh_update: ChunkMeshUpdate {
                            update_mesh: false,
                            update_light: false,
                            generate_chunk: true,
                        },
                        material: materials.add(VoxelMaterial {
                            albedo: Color::from(Vec4::new(1.0, 1.0, 1.0, 1.0)),
                        }),
                        mesh,
                        ..Default::default()
                    })
                    // camera
                    .spawn(bevy_fly_camera::FlyCamera::default());
            }
        }
    }
}

fn chunk_update(
    mut meshes: ResMut<Assets<Mesh>>,
    mut query: Query<(&Chunk<Block>, &mut ChunkMeshUpdate, &Handle<Mesh>)>,
) {
    for (chunk, mut update, chunk_mesh) in &mut query.iter() {
        if update.update_mesh {
            update.update_mesh = false;
            let mesh = generate_chunk_mesh(&chunk);
            *meshes.get_mut(&chunk_mesh).unwrap() = mesh;
        }
    }
}
