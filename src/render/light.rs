use bevy::prelude::*;

use line_drawing::{Bresenham3d, VoxelOrigin, WalkVoxels};

use crate::{
    render::entity::Block,
    world::{ChunkUpdate, Map, MapUpdates},
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
    mut query: Query<(&mut Map<Block>, &mut MapUpdates)>,
) {
    for (mut map, mut update) in &mut query.iter() {
        let mut remove = Vec::new();
        let mut insert = Vec::new();
        for (&(x, y, z, w), update) in &update.updates {
            match update {
                ChunkUpdate::UpdateLightMap => {}
                _ => continue,
            }
            remove.push((x, y, z, w));

            let w_2 = w as i32 / 2;
            let cx = x * w as i32 - w_2;
            let cy = y * w as i32 - w_2;
            let cz = z * w as i32 - w_2;
            let chunk = map.get_mut((cx, cy, cz));
            if chunk.is_none() {
                continue;
            }
            let chunk = chunk.unwrap();

            let light = -directional.direction;

            for elem in chunk.iter_mut() {
                elem.value.shade.top = light.dot(Vec3::new(0.0, 1.0, 0.0)).max(0.0).min(1.0)
                    * directional.intensity
                    + ambient.intensity;
                elem.value.shade.bottom = light.dot(Vec3::new(0.0, -1.0, 0.0)).max(0.0).min(1.0)
                    * directional.intensity
                    + ambient.intensity;
                elem.value.shade.front = light.dot(Vec3::new(0.0, 0.0, 1.0)).max(0.0).min(1.0)
                    * directional.intensity
                    + ambient.intensity;
                elem.value.shade.back = light.dot(Vec3::new(0.0, 0.0, -1.0)).max(0.0).min(1.0)
                    * directional.intensity
                    + ambient.intensity;
                elem.value.shade.left = light.dot(Vec3::new(1.0, 0.0, 0.0)).max(0.0).min(1.0)
                    * directional.intensity
                    + ambient.intensity;
                elem.value.shade.right = light.dot(Vec3::new(-1.0, 0.0, 0.0)).max(0.0).min(1.0)
                    * directional.intensity
                    + ambient.intensity;
            }

            chunk.merge();

            insert.push(((x, y, z, w), ChunkUpdate::UpdateMesh));
        }
        for coords in remove {
            update.updates.remove(&coords);
        }
        for (coords, u) in insert {
            update.updates.insert(coords, u);
        }
    }
}

pub fn shaded_light_update(
    directional: Res<DirectionalLight>,
    ambient: Res<AmbientLight>,
    mut query: Query<(&mut Map<Block>, &mut MapUpdates)>,
) {
    for (mut map, mut update) in &mut query.iter() {
        let mut remove = Vec::new();
        let mut insert = Vec::new();
        'outer: for (&(x, y, z, w), update) in &update.updates {
            match update {
                ChunkUpdate::UpdateLight => {}
                _ => continue,
            }

            let w_2 = w as i32 / 2;
            let cx = x * w as i32 - w_2;
            let cy = y * w as i32 - w_2;
            let cz = z * w as i32 - w_2;
            let chunk = map.get((cx, cy, cz)).unwrap();

            let mut light_map = vec![0.0; (chunk.width() + 2).pow(3)];

            let width = chunk.width() as i32;
            let width_2 = width / 2;

            let lm_width = chunk.width() as i32 + 2;
            let lm_width_2 = lm_width / 2;

            for x in -lm_width_2..lm_width_2 {
                for y in -lm_width_2..lm_width_2 {
                    for z in -lm_width_2..lm_width_2 {
                        let mut light = 0.0;
                        let mut count = 0;
                        let range = 1;
                        for lx in -range..=range {
                            for ly in -range..=range {
                                for lz in -range..=range {
                                    let x = x + lx;
                                    let y = y + ly;
                                    let z = z + lz;
                                    if x < -width_2
                                        || x >= width_2
                                        || y < -width_2
                                        || y >= width_2
                                        || z < -width_2
                                        || z >= width_2
                                    {
                                        let sx = if x < -width_2 {
                                            -1
                                        } else if x >= width_2 {
                                            1
                                        } else {
                                            0
                                        };
                                        let sy = if y < -width_2 {
                                            -1
                                        } else if y >= width_2 {
                                            1
                                        } else {
                                            0
                                        };
                                        let sz = if z < -width_2 {
                                            -1
                                        } else if z >= width_2 {
                                            1
                                        } else {
                                            0
                                        };
                                        let cx = cx + width * sx;
                                        let cy = cy + width * sy;
                                        let cz = cz + width * sz;
                                        if let Some(chunk) = map.get((cx, cy, cz)) {
                                            let mut x = x;
                                            let mut y = y;
                                            let mut z = z;
                                            if !chunk.has_light() {
                                                continue 'outer;
                                            }
                                            while x >= width_2 {
                                                x -= width;
                                            }
                                            while x < -width_2 {
                                                x += width;
                                            }
                                            while y >= width_2 {
                                                y -= width;
                                            }
                                            while y < -width_2 {
                                                y += width;
                                            }
                                            while z >= width_2 {
                                                z -= width;
                                            }
                                            while z < -width_2 {
                                                z += width;
                                            }
                                            if let Some(l) = chunk.light((x, y, z)) {
                                                light += l;
                                                count += 1;
                                            }
                                        }
                                    } else {
                                        if let Some(l) = chunk.light((x, y, z)) {
                                            light += l;
                                            count += 1;
                                        }
                                    }
                                }
                            }
                        }
                        if count == 0 {
                            count = 1;
                        }
                        let light = light / count as f32;
                        let idx = ((x + lm_width_2) * lm_width * lm_width) as usize
                            + ((y + lm_width_2) * lm_width) as usize
                            + (z + lm_width_2) as usize;
                        light_map[idx] = light;
                    }
                }
            }

            let chunk = map.get_mut((cx, cy, cz)).unwrap();

            for x in -lm_width_2..lm_width_2 {
                for y in -lm_width_2..lm_width_2 {
                    for z in -lm_width_2..lm_width_2 {
                        let idx = ((x + lm_width_2) * lm_width * lm_width) as usize
                            + ((y + lm_width_2) * lm_width) as usize
                            + (z + lm_width_2) as usize;
                        let light = light_map[idx];
                        if let Some(block) = chunk.get_mut((x, y - 1, z)) {
                            block.shade.top = light;
                        }
                        if let Some(block) = chunk.get_mut((x, y + 1, z)) {
                            block.shade.bottom = light;
                        }
                        if let Some(block) = chunk.get_mut((x, y, z - 1)) {
                            block.shade.front = light;
                        }
                        if let Some(block) = chunk.get_mut((x, y, z + 1)) {
                            block.shade.back = light;
                        }
                        if let Some(block) = chunk.get_mut((x - 1, y, z)) {
                            block.shade.left = light;
                        }
                        if let Some(block) = chunk.get_mut((x + 1, y, z)) {
                            block.shade.right = light;
                        }
                    }
                }
            }

            let light = -directional.direction;

            for elem in chunk.iter_mut() {
                elem.value.shade.top = elem.value.shade.top
                    * light.dot(Vec3::new(0.0, 1.0, 0.0)).max(0.0).min(1.0)
                    * directional.intensity
                    + ambient.intensity;
                elem.value.shade.bottom = elem.value.shade.bottom
                    * light.dot(Vec3::new(0.0, -1.0, 0.0)).max(0.0).min(1.0)
                    * directional.intensity
                    + ambient.intensity;
                elem.value.shade.front = elem.value.shade.front
                    * light.dot(Vec3::new(0.0, 0.0, 1.0)).max(0.0).min(1.0)
                    * directional.intensity
                    + ambient.intensity;
                elem.value.shade.back = elem.value.shade.back
                    * light.dot(Vec3::new(0.0, 0.0, -1.0)).max(0.0).min(1.0)
                    * directional.intensity
                    + ambient.intensity;
                elem.value.shade.left = elem.value.shade.left
                    * light.dot(Vec3::new(1.0, 0.0, 0.0)).max(0.0).min(1.0)
                    * directional.intensity
                    + ambient.intensity;
                elem.value.shade.right = elem.value.shade.right
                    * light.dot(Vec3::new(-1.0, 0.0, 0.0)).max(0.0).min(1.0)
                    * directional.intensity
                    + ambient.intensity;
            }

            chunk.merge();

            remove.push((x, y, z, w));
            insert.push(((x, y, z, w), ChunkUpdate::UpdateMesh));
        }
        for coords in remove {
            update.updates.remove(&coords);
        }
        for (coords, u) in insert {
            update.updates.insert(coords, u);
        }
    }
}

pub fn light_map_update<T: VoxelTracer>(
    directional: Res<DirectionalLight>,
    mut query: Query<(&mut Map<Block>, &mut MapUpdates)>,
) {
    for (mut map, mut update) in &mut query.iter() {
        let mut remove = Vec::new();
        let mut insert = Vec::new();
        for (&(x, y, z, w), update) in &update.updates {
            match update {
                ChunkUpdate::UpdateLightMap => {}
                _ => continue,
            }
            remove.push((x, y, z, w));

            let w_2 = w as i32 / 2;
            let cx = x * w as i32 - w_2;
            let cy = y * w as i32 - w_2;
            let cz = z * w as i32 - w_2;
            let chunk = map.get_mut((cx, cy, cz));
            if chunk.is_none() {
                continue;
            }
            let chunk = chunk.unwrap();

            let mut light_map = vec![None; chunk.width().pow(3)];

            let lm_width = chunk.width() as i32;
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
                            if x < -lm_width_2
                                || y < -lm_width_2
                                || z < -lm_width_2
                                || x >= lm_width_2
                                || y >= lm_width_2
                                || z >= lm_width_2
                            {
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

            for x in -lm_width_2..lm_width_2 {
                for y in -lm_width_2..lm_width_2 {
                    for z in -lm_width_2..lm_width_2 {
                        let idx = ((x + lm_width_2) * lm_width * lm_width) as usize
                            + ((y + lm_width_2) * lm_width) as usize
                            + (z + lm_width_2) as usize;
                        let light = light_map[idx];
                        chunk.insert_light((x, y, z), light.unwrap_or_default());
                    }
                }
            }

            chunk.set_light(true);

            insert.push(((x, y, z, w), ChunkUpdate::UpdateLight));
        }
        for coords in remove {
            update.updates.remove(&coords);
        }
        for (coords, u) in insert {
            update.updates.insert(coords, u);
        }
    }
}
