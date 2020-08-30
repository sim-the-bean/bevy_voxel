use bevy::prelude::*;
use bevy::render::{
    camera::ActiveCameras,
    draw::Draw,
    mesh::Mesh,
    pipeline::{DynamicBinding, PipelineSpecialization, RenderPipeline, RenderPipelines},
    render_graph::base::{self, MainPass},
};
use bevy::asset::Handle;
use bevy::ecs::Bundle;
use bevy::transform::prelude::{Rotation, Scale, Transform, Translation};

use line_drawing::{Bresenham3d, WalkVoxels, VoxelOrigin};

use crate::world::{Chunk, Shade};
use crate::render::{
    prelude::*,
    render_graph::pipeline,
    entity::{Block, generate_chunk_mesh, ChunkMeshUpdate, CurrentLod},
};

pub fn shaded_light_update<T: VoxelTracer>(
    camera: Res<ActiveCameras>,
    mut query: Query<(&Chunk, &mut CurrentLod)>,
    translation: Query<&Translation>,
) {
    let (camera_x, camera_y, camera_z) = if let Some(camera) = camera.get(base::camera::CAMERA3D) {
        let position = translation.get::<Translation>(camera).unwrap();
        (position.0.x() as i32, position.0.y() as i32, position.0.z() as i32)
    } else {
        (0, 0, 0)
    };
    for (chunk, current_lod) in &mut query.iter() {
        let (x, y, z) = chunk.position();
        let lod = ((camera_x - x * chunk.width(0) as i32).abs() / 128)
            .max((camera_y - y * chunk.width(0) as i32).abs() / 128)
            .max((camera_z - z * chunk.width(0) as i32).abs() / 128) as usize;
        current_lod.lod = lod;
    }
}
