#[cfg(feature = "savedata")]
use std::path::Path;

#[cfg(feature = "savedata")]
use serde::{de::DeserializeOwned, Serialize};

#[cfg(feature = "savedata")]
use bevy::app::AppExit;

use bevy::{
    prelude::*,
    ecs::Commands,
    render::mesh::Mesh,
    render::{camera::ActiveCameras, render_graph::base, entity::Camera3dBundle},
};

use bevy_fly_camera::{FlyCamera, FlyCameraPlugin};

use bevy_voxel::{
    collections::lod_tree::Voxel,
    render::{
        entity::{generate_chunk_mesh, VoxelExt},
        light::*,
        lod::lod_update,
        prelude::*,
    },
    simple::{Block, MeshType},
    terrain::*,
    world::{ChunkUpdate, Map, MapComponents, MapUpdates},
};

pub const CHUNK_SIZE: u32 = 4;
pub const WORLD_WIDTH: i32 = 256;
pub const WORLD_HEIGHT: i32 = 96;

pub fn main() {
    let params = Program::build()
        .seed(0)
        .noise_type(NoiseType::SuperSimplex)
        .noise_dimensions(NoiseDimensions::Two)
        .chunk_size(CHUNK_SIZE)
        .subdivisions(1)
        .filter(Filter::Bilinear(2))
        .biome_frequency(0.001)
        .biome(
            Biome::build()
                .name("ocean")
                .spawn_probability(0.7)
                .height(-8.0)
                .octave(Octave::new(8.0, 0.01))
                .octave(Octave::new(2.0, 0.05))
                .octave(Octave::new(1.0, 0.10))
                .layer(Layer::new(
                    Block {
                        color: Color::rgb(0.08, 0.08, 0.08),
                        ..Default::default()
                    },
                    f64::INFINITY,
                ))
                .layer(Layer::new(
                    Block {
                        color: Color::rgb(0.5, 0.5, 0.5),
                        ..Default::default()
                    },
                    16.0,
                ))
                .layer(Layer::new(
                    Block {
                        color: Color::rgb(0.76, 0.69, 0.5),
                        ..Default::default()
                    },
                    1.0,
                ))
                .water(Layer::new(
                    Block {
                        color: Color::rgba(0.4, 0.8, 1.0, 0.5),
                        ..Default::default()
                    },
                    0.0,
                ))
                .build(),
        )
        .biome(
            Biome::build()
                .name("plains")
                .spawn_probability(0.5)
                .octave(Octave::new(8.0, 0.01))
                .octave(Octave::new(2.0, 0.05))
                .octave(Octave::new(1.0, 0.10))
                .layer(Layer::new(
                    Block {
                        color: Color::rgb(0.08, 0.08, 0.08),
                        ..Default::default()
                    },
                    f64::INFINITY,
                ))
                .layer(Layer::new(
                    Block {
                        color: Color::rgb(0.5, 0.5, 0.5),
                        ..Default::default()
                    },
                    16.0,
                ))
                .layer(Layer::new(
                    Block {
                        color: Color::rgb(0.396, 0.263, 0.129),
                        ..Default::default()
                    },
                    3.0,
                ))
                .layer(Layer::new(
                    Block {
                        color: Color::rgb(0.0, 0.416, 0.306),
                        ..Default::default()
                    },
                    1.0,
                ))
                .water(Layer::new(
                    Block {
                        color: Color::rgba(0.4, 0.8, 1.0, 0.5),
                        ..Default::default()
                    },
                    0.0,
                ))
                .per_xz(
                    Expression::Ratio(3, 10)
                        .is_true()
                        .and_then(BlockQuery::y_top())
                        .set_block(Block {
                            color: Color::rgb(0.0, 0.6, 0.2),
                            mesh_type: MeshType::Cross,
                            ..Default::default()
                        }),
                )
                .build(),
        )
        .biome(
            Biome::build()
                .name("fields")
                .spawn_probability(0.5)
                .height(4.0)
                .octave(Octave::new(8.0, 0.01))
                .octave(Octave::new(2.0, 0.05))
                .octave(Octave::new(1.0, 0.10))
                .layer(Layer::new(
                    Block {
                        color: Color::rgb(0.08, 0.08, 0.08),
                        ..Default::default()
                    },
                    f64::INFINITY,
                ))
                .layer(Layer::new(
                    Block {
                        color: Color::rgb(0.5, 0.5, 0.5),
                        ..Default::default()
                    },
                    16.0,
                ))
                .layer(Layer::new(
                    Block {
                        color: Color::rgb(0.396, 0.263, 0.129),
                        ..Default::default()
                    },
                    3.0,
                ))
                .layer(Layer::new(
                    Block {
                        color: Color::rgb(0.0, 0.416, 0.306),
                        ..Default::default()
                    },
                    1.0,
                ))
                .water(Layer::new(
                    Block {
                        color: Color::rgba(0.4, 0.8, 1.0, 0.5),
                        ..Default::default()
                    },
                    0.0,
                ))
                .per_xz(
                    Expression::Ratio(4, 10)
                        .is_true()
                        .and_then(BlockQuery::y_top())
                        .set_block(Block {
                            color: Color::rgb(0.6, 0.6, 0.2),
                            mesh_type: MeshType::Cross,
                            ..Default::default()
                        }),
                )
                .build(),
        )
        .biome(
            Biome::build()
                .name("hills")
                .spawn_probability(0.3)
                .height(8.0)
                .octave(Octave::new(24.0, 0.01))
                .octave(Octave::new(2.0, 0.05))
                .octave(Octave::new(1.0, 0.10))
                .layer(Layer::new(
                    Block {
                        color: Color::rgb(0.08, 0.08, 0.08),
                        ..Default::default()
                    },
                    f64::INFINITY,
                ))
                .layer(Layer::new(
                    Block {
                        color: Color::rgb(0.5, 0.5, 0.5),
                        ..Default::default()
                    },
                    16.0,
                ))
                .layer(Layer::new(
                    Block {
                        color: Color::rgb(0.396, 0.263, 0.129),
                        ..Default::default()
                    },
                    3.0,
                ))
                .layer(Layer::new(
                    Block {
                        color: Color::rgb(0.0, 0.416, 0.306),
                        ..Default::default()
                    },
                    1.0,
                ))
                .water(Layer::new(
                    Block {
                        color: Color::rgba(0.4, 0.8, 1.0, 0.5),
                        ..Default::default()
                    },
                    0.0,
                ))
                .build(),
        )
        .build();
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(bevy::diagnostic::FrameTimeDiagnosticsPlugin::default())
        .add_plugin(bevy::diagnostic::PrintDiagnosticsPlugin::default())
        .add_plugin(VoxelRenderPlugin)
        // .add_plugin(FlyCameraPlugin)
        .add_startup_system(setup::<Block>)
        .add_resource(DirectionalLight {
            direction: Vec3::new(0.8, -1.0, 0.5).normalize(),
            intensity: 0.8,
        })
        .add_resource(AmbientLight { intensity: 0.05 })
        .add_resource(params)
        .init_resource::<ExitListenerState>()
        .init_resource::<HeightMap>()
        .add_stage_before(stage::PRE_UPDATE, "stage_terrain_generation")
        .add_stage_after("stage_terrain_generation", "stage_lod_update")
        .add_system_to_stage(
            "stage_terrain_generation",
            terrain_generation::<Block>,
        )
        .add_system_to_stage("stage_lod_update", lod_update::<Block>)
        .add_system_to_stage(stage::UPDATE, infinite_update::<Block>)
        .add_system_to_stage(
            stage::UPDATE,
            light_map_update::<Block, line_drawing::Bresenham3d<i32>>,
        )
        .add_system_to_stage(stage::UPDATE, shaded_light_update::<Block>)
        // .add_system_to_stage(stage::UPDATE, simple_light_update::<Block>)
        .add_system_to_stage(stage::POST_UPDATE, chunk_update::<Block>)
        .add_system_to_stage(stage::POST_UPDATE, save_game::<Block>)
        .run();
}

/// set up a simple 3D scene
fn setup<T: Voxel>(
    commands: &mut Commands,
    params: Res<Program<T>>,
    mut height_map: ResMut<HeightMap>,
) {
    let mut update = MapUpdates::default();
    let chunk_size = 2_i32.pow(CHUNK_SIZE as u32);
    let world_width_2 = WORLD_WIDTH / chunk_size / 2;
    let world_height = WORLD_HEIGHT / chunk_size;

    commands
        .spawn(Camera3dBundle {
            transform: Transform::from_translation(Vec3::new(0.0, WORLD_HEIGHT as f32 - chunk_size as f32, 0.0))
                .looking_at(Vec3::default(), Vec3::unit_y()),
            ..Default::default()
        });
        // .with(FlyCamera::default());

    if let Some(save_directory) = std::env::args().skip(1).next() {
        let save_directory: &Path = save_directory.as_ref();
        if save_directory.exists() {
            for cx in -world_width_2..world_width_2 {
                for cy in -1..world_height - 1 {
                    for cz in -world_width_2..world_width_2 {
                        let x = cx * chunk_size;
                        let y = cy * chunk_size;
                        let z = cz * chunk_size;
                        update
                            .updates
                            .insert((x, y, z), ChunkUpdate::UpdateLightMap);
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
                update.updates.insert((x, y, z), ChunkUpdate::GenerateChunk);
            }
        }
    }
    
    let mut map = Vec::new();

    for x in -world_width_2..world_width_2 {
        for y in -1..world_height - 1 {
            for z in -world_width_2..world_width_2 {
                let x = x * chunk_size;
                let y = y * chunk_size;
                let z = z * chunk_size;
                let chunk = params.execute(&mut height_map, (x, y, z));
                map.push(chunk);
                update.updates.insert((x, y, z), ChunkUpdate::UpdateLightMap);
            }
        }
    }
    commands
        .spawn(MapComponents { map_update: update })
        .with(Map::<T>::with_chunks(map));
}

fn chunk_update<T: VoxelExt>(
    commands: &mut Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<VoxelMaterial>>,
    mut maps: Query<(&mut Map<T>, &mut MapUpdates)>,
    chunks: Query<&Handle<Mesh>>
) {
    for (mut map, mut update) in &mut maps.iter_mut() {
        let mut remove = Vec::new();
        for (&(x, y, z), update) in &update.updates {
            match update {
                ChunkUpdate::UpdateMesh => {},
                _ => continue,
            }
            remove.push((x, y, z));

            let chunk = map.get((x, y, z)).unwrap();

            let (mesh, t_mesh) = generate_chunk_mesh(&map, &chunk);

            if let Some(mesh) = mesh {
                let chunk = map.get_mut((x, y, z)).unwrap();
                if let Some(e) = chunk.entity() {
                    *meshes.get_mut(chunks.get(e).unwrap()).unwrap() = mesh;
                } else {
                    commands.spawn(ChunkRenderComponents {
                        mesh: meshes.add(mesh),
                        material: materials.add(VoxelMaterial {
                            albedo: Color::WHITE,
                        }),
                        transform: Transform::from_translation(Vec3::new(x as f32, y as f32, z as f32)),
                        ..Default::default()
                    });
                    chunk.set_entity(commands.current_entity().unwrap());
                }
            }
            
            if let Some(mesh) = t_mesh {
                let chunk = map.get_mut((x, y, z)).unwrap();
                if let Some(e) = chunk.transparent_entity() {
                    *meshes.get_mut(chunks.get(e).unwrap()).unwrap() = mesh;
                } else {
                    commands.spawn(ChunkRenderComponents {
                        mesh: meshes.add(mesh),
                        material: materials.add(VoxelMaterial {
                            albedo: Color::WHITE,
                        }),
                        transform: Transform::from_translation(Vec3::new(x as f32, y as f32, z as f32)),
                        ..Default::default()
                    });
                    chunk.set_transparent_entity(commands.current_entity().unwrap());
                }
            }
        }
        for coords in remove {
            update.updates.remove(&coords);
        }
    }
}

pub fn infinite_update<T: Voxel>(
    camera: Res<ActiveCameras>,
    mut query: Query<(&Map<T>, &mut MapUpdates)>,
    transform: Query<&Transform>
) {
    let (camera_x, camera_z) = if let Some(camera) = camera.get(base::camera::CAMERA3D) {
        let position = transform.get(camera).unwrap();
        (
            position.translation.x as i32,
            position.translation.z as i32,
        )
    } else {
        (0, 0)
    };
    
    let range = 8;
    let chunk_size = 2_i32.pow(CHUNK_SIZE as u32);
    let world_height = WORLD_HEIGHT / chunk_size;
    
    for (map, mut update) in query.iter_mut() {
        let x = camera_x / chunk_size;
        let z = camera_z / chunk_size;
        for x in x - range..=x + range {
            for z in z - range..=z + range {
                for y in -1..world_height - 1 {
                    let x = x * chunk_size;
                    let y = y * chunk_size;
                    let z = z * chunk_size;
                    if map.get((x, y, z)).is_none() {
                        update.updates.insert((x, y, z), ChunkUpdate::GenerateChunk);
                    }
                }
            }
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
    query: Query<&Map<T>>,
) {
    if let Some(_) = state.reader.iter(&exit_events).next() {
        if let Some(save_directory) = std::env::args().skip(1).next() {
            let save_directory: &Path = save_directory.as_ref();
            for map in query.iter() {
                map.save(save_directory).expect(&format!(
                    "couldn't save map to {}",
                    save_directory.display()
                ));
            }
        }
    }
}
