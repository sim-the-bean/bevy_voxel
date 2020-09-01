use bevy::prelude::*;

use line_drawing::{Bresenham3d, VoxelOrigin, WalkVoxels};

use crate::{
    render::entity::{Block, ChunkMeshUpdate},
    world::Chunk,
};

pub trait VoxelTracer: Iterator<Item = (i32, i32, i32)> {
    fn new(start: (i32, i32, i32), end: (i32, i32, i32)) -> Self;
}

impl VoxelTracer for Bresenham3d<i32> {
    fn new(start: (i32, i32, i32), end: (i32, i32, i32)) -> Self {
        Self::new(start, end)
    }
}

impl VoxelTracer for WalkVoxels<f32, i32> {
    fn new(start: (i32, i32, i32), end: (i32, i32, i32)) -> Self {
        Self::new(
            (start.0 as f32, start.1 as f32, start.2 as f32),
            (end.0 as f32, end.1 as f32, end.2 as f32),
            &VoxelOrigin::Center,
        )
    }
}

pub struct DirectionalLight {
    pub direction: Vec3,
    pub intensity: f32,
}

pub struct AmbientLight {
    pub intensity: f32,
}

pub fn simple_light_update(
    directional: Res<DirectionalLight>,
    ambient: Res<AmbientLight>,
    mut query: Query<(&mut Chunk<Block>, &mut ChunkMeshUpdate)>,
) {
    for (mut chunk, mut update) in &mut query.iter() {
        let lod = chunk.lod();
        let flod = 2.0_f32.powf(lod as f32);
        if rand::random::<f32>() < 1.0 - 0.01 * flod {
            continue;
        }
        if !update.update_light {
            continue;
        }
        update.update_light = false;
        update.update_mesh = true;
        let light = -directional.direction;
        for elem in chunk.iter_mut() {
            elem.value.shade.top =
                light.dot(Vec3::new(0.0, 1.0, 0.0)) * directional.intensity + ambient.intensity;
            elem.value.shade.bottom =
                light.dot(Vec3::new(0.0, -1.0, 0.0)) * directional.intensity + ambient.intensity;
            elem.value.shade.front =
                light.dot(Vec3::new(0.0, 0.0, 1.0)) * directional.intensity + ambient.intensity;
            elem.value.shade.back =
                light.dot(Vec3::new(0.0, 0.0, -1.0)) * directional.intensity + ambient.intensity;
            elem.value.shade.left =
                light.dot(Vec3::new(1.0, 0.0, 0.0)) * directional.intensity + ambient.intensity;
            elem.value.shade.right =
                light.dot(Vec3::new(-1.0, 0.0, 0.0)) * directional.intensity + ambient.intensity;
        }
    }
}

pub fn shaded_light_update<T: VoxelTracer>(
    directional: Res<DirectionalLight>,
    ambient: Res<AmbientLight>,
    mut query: Query<(&mut Chunk<Block>, &mut ChunkMeshUpdate)>,
) {
    for (mut chunk, mut update) in &mut query.iter() {
        let lod = chunk.lod();
        let flod = 2.0_f32.powf(lod as f32);
        if rand::random::<f32>() < 1.0 - 0.01 * flod {
            continue;
        }
        if !update.update_light {
            continue;
        }
        update.update_light = false;
        update.update_mesh = true;

        let mut light_map = vec![None; (chunk.width() + 2).pow(3)];

        let lm_width = chunk.width() as i32 + 2;
        let lm_width_2 = lm_width / 2;

        for y in -lm_width_2..lm_width_2 {
            for x in -lm_width_2..lm_width_2 {
                for z in -lm_width_2..lm_width_2 {
                    let idx = ((x + lm_width_2) * lm_width * lm_width) as usize
                        + ((y + lm_width_2) * lm_width) as usize
                        + (z + lm_width_2) as usize;
                    if light_map[idx].is_some() {
                        continue;
                    }

                    let light_source =
                        Vec3::new(x as _, y as _, z as _) + directional.direction * -100.0;
                    let mut light = 1.0;
                    for (x, y, z) in T::new(
                        (
                            light_source.x() as _,
                            light_source.y() as _,
                            light_source.z() as _,
                        ),
                        (x, y, z),
                    ) {
                        let block = chunk.get((x, y, z));
                        if block.is_some() {
                            light = 0.0;
                        }
                        if x < -lm_width_2 || y < -lm_width_2 || z < -lm_width_2 {
                            continue;
                        }
                        let idx = ((x + lm_width_2) * lm_width * lm_width) as usize
                            + ((y + lm_width_2) * lm_width) as usize
                            + (z + lm_width_2) as usize;
                        if let Some(map) = light_map.get_mut(idx) {
                            if map.is_none() {
                                *map = Some(light);
                            }
                        }
                    }
                }
            }
        }

        let width = chunk.width() as i32;
        let width_2 = width / 2;

        for x in -width_2..width_2 {
            for y in -width_2..width_2 {
                for z in -width_2..width_2 {
                    let idx = ((x + lm_width_2) * lm_width * lm_width) as usize
                        + ((y + lm_width_2) * lm_width) as usize
                        + (z + lm_width_2) as usize;
                    if let Some(light) = light_map[idx] {
                        if let Some(mut block) = chunk.get((x, y - 1, z)) {
                            block.to_mut().shade.top = light;
                            let block = *block;
                            chunk.insert((x, y - 1, z), block);
                        }
                        if let Some(mut block) = chunk.get((x, y + 1, z)) {
                            block.to_mut().shade.bottom = light;
                            let block = *block;
                            chunk.insert((x, y + 1, z), block);
                        }
                        if let Some(mut block) = chunk.get((x, y, z - 1)) {
                            block.to_mut().shade.front = light;
                            let block = *block;
                            chunk.insert((x, y, z - 1), block);
                        }
                        if let Some(mut block) = chunk.get((x, y, z + 1)) {
                            block.to_mut().shade.back = light;
                            let block = *block;
                            chunk.insert((x, y, z + 1), block);
                        }
                        if let Some(mut block) = chunk.get((x - 1, y, z)) {
                            block.to_mut().shade.left = light;
                            let block = *block;
                            chunk.insert((x - 1, y, z), block);
                        }
                        if let Some(mut block) = chunk.get((x + 1, y, z)) {
                            block.to_mut().shade.right = light;
                            let block = *block;
                            chunk.insert((x + 1, y, z), block);
                        }
                    }
                }
            }
        }

        let light = -directional.direction;

        for elem in chunk.iter_mut() {
            elem.value.shade.top =
                elem.value.shade.top * light.dot(Vec3::new(0.0, 1.0, 0.0)) * directional.intensity
                    + ambient.intensity;
            elem.value.shade.bottom = elem.value.shade.bottom
                * light.dot(Vec3::new(0.0, -1.0, 0.0))
                * directional.intensity
                + ambient.intensity;
            elem.value.shade.front = elem.value.shade.front
                * light.dot(Vec3::new(0.0, 0.0, 1.0))
                * directional.intensity
                + ambient.intensity;
            elem.value.shade.back = elem.value.shade.back
                * light.dot(Vec3::new(0.0, 0.0, -1.0))
                * directional.intensity
                + ambient.intensity;
            elem.value.shade.left =
                elem.value.shade.left * light.dot(Vec3::new(1.0, 0.0, 0.0)) * directional.intensity
                    + ambient.intensity;
            elem.value.shade.right = elem.value.shade.right
                * light.dot(Vec3::new(-1.0, 0.0, 0.0))
                * directional.intensity
                + ambient.intensity;
        }
    }
}
