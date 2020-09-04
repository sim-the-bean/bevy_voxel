#[cfg(feature = "savedata")]
use std::path::Path;

#[cfg(feature = "savedata")]
use serde::{de::DeserializeOwned, Serialize};

#[cfg(feature = "savedata")]
use bevy::app::AppExit;

use bevy::{prelude::*, render::mesh::Mesh};

use bevy_fly_camera::FlyCamera;

use bevy_voxel::{
    render::{
        entity::{generate_chunk_mesh, VoxelExt},
        light::*,
        lod::lod_update,
        prelude::*,
    },
    simple::Block,
    terrain::*,
    world::{ChunkUpdate, Map, MapComponents, MapUpdates},
};

pub const CHUNK_SIZE: u32 = 5;
pub const WORLD_WIDTH: i32 = 512;
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
                block: Block {
                    color: Color::rgb(0.08, 0.08, 0.08),
                    ..Default::default()
                },
                height: f64::INFINITY,
            },
            Layer {
                block: Block {
                    color: Color::rgb(0.5, 0.5, 0.5),
                    ..Default::default()
                },
                height: 16.0,
            },
            Layer {
                block: Block {
                    color: Color::rgb(0.396, 0.263, 0.129),
                    ..Default::default()
                },
                height: 3.0,
            },
            Layer {
                block: Block {
                    color: Color::rgb(0.0, 0.416, 0.306),
                    ..Default::default()
                },
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
        .init_resource::<ExitListenerState>()
        .add_stage_before(stage::PRE_UPDATE, "stage_terrain_generation")
        .add_stage_after("stage_terrain_generation", "stage_lod_update")
        .add_system_to_stage(
            "stage_terrain_generation",
            terrain_generation::<Block>.system(),
        )
        .add_system_to_stage("stage_lod_update", lod_update::<Block>.system())
        .add_system_to_stage(
            stage::UPDATE,
            light_map_update::<Block, line_drawing::Bresenham3d<i32>>.system(),
        )
        .add_system_to_stage(stage::UPDATE, shaded_light_update::<Block>.system())
        .add_system_to_stage(stage::POST_UPDATE, chunk_update::<Block>.system())
        .add_system_to_stage(stage::POST_UPDATE, save_game::<Block>.system())
        .run();
}

/// set up a simple 3D scene
fn setup(mut commands: Commands) {
    let mut update = MapUpdates::default();
    let chunk_size = 2_i32.pow(CHUNK_SIZE as u32);
    let world_width_2 = WORLD_WIDTH / chunk_size / 2;
    let world_height = WORLD_HEIGHT / chunk_size;

    commands.spawn(FlyCamera {
        translation: Translation::new(0.0, WORLD_HEIGHT as f32 - chunk_size as f32, 0.0),
        ..Default::default()
    });

    if let Some(save_directory) = std::env::args().skip(1).next() {
        let save_directory: &Path = save_directory.as_ref();
        if save_directory.exists() {
            for cx in -world_width_2..world_width_2 {
                for cy in 0..world_height {
                    for cz in -world_width_2..world_width_2 {
                        let x = cx * chunk_size;
                        let y = cy * chunk_size;
                        let z = cz * chunk_size;
                        update.updates.insert(
                            (x, y, z),
                            ChunkUpdate::UpdateLightMap,
                        );
                    }
                }
            }
            commands.spawn(MapComponents { map_update: update }).with(
                Map::<Block>::load(save_directory).expect(&format!(
                    "couldn't load map from {}",
                    save_directory.display()
                )),
            );
            return;
        }
    }

    for cx in -world_width_2..world_width_2 {
        for cy in -1..world_height - 1 {
            for cz in -world_width_2..world_width_2 {
                let x = cx * chunk_size;
                let y = cy * chunk_size;
                let z = cz * chunk_size;
                update.updates.insert(
                    (x, y, z),
                    ChunkUpdate::GenerateChunk,
                );
            }
        }
    }
    commands
        .spawn(MapComponents { map_update: update })
        .with(Map::<Block>::default());
}

fn chunk_update<T: VoxelExt>(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<VoxelMaterial>>,
    mut query: Query<(&Map<T>, &mut MapUpdates)>,
) {
    for (map, mut update) in &mut query.iter() {
        let mut remove = Vec::new();
        for (&(x, y, z), update) in &update.updates {
            match update {
                ChunkUpdate::UpdateMesh => {}
                _ => continue,
            }
            remove.push((x, y, z));

            let chunk = map.get((x, y, z)).unwrap();

            let mesh = generate_chunk_mesh(&map, &chunk);
            if let Some(mesh) = mesh {
                commands.spawn(ChunkRenderComponents {
                    mesh: meshes.add(mesh),
                    material: materials.add(VoxelMaterial {
                        albedo: Color::WHITE,
                    }),
                    translation: Translation::new(x as f32, y as f32, z as f32),
                    ..Default::default()
                });
            }
        }
        for coords in remove {
            update.updates.remove(&coords);
        }
    }
}

#[cfg(feature = "savedata")]
#[derive(Default)]
pub struct ExitListenerState {
    reader: EventReader<AppExit>,
}

#[cfg(feature = "savedata")]
fn save_game<T: VoxelExt + Serialize + DeserializeOwned>(
    mut state: ResMut<ExitListenerState>,
    exit_events: Res<Events<AppExit>>,
    mut query: Query<&Map<T>>,
) {
    if let Some(_) = state.reader.iter(&exit_events).next() {
        if let Some(save_directory) = std::env::args().skip(1).next() {
            let save_directory: &Path = save_directory.as_ref();
            for map in &mut query.iter() {
                map.save(save_directory).expect(&format!(
                    "couldn't save map to {}",
                    save_directory.display()
                ));
            }
        }
    }
}
