use bevy::prelude::*;

use line_drawing::{Bresenham3d, VoxelOrigin, WalkVoxels};

use crate::{
    render::entity::{Face, VoxelExt},
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

pub fn simple_light_update<T: VoxelExt>(
    directional: Res<DirectionalLight>,
    ambient: Res<AmbientLight>,
    mut query: Query<(&mut Map<T>, &mut MapUpdates)>,
) {
    for (mut map, mut update) in &mut query.iter() {
        let mut remove = Vec::new();
        let mut insert = Vec::new();
        for (&(x, y, z), update) in &update.updates {
            match update {
                ChunkUpdate::UpdateLightMap => {}
                _ => continue,
            }
            remove.push((x, y, z));

            let chunk = map.get_mut((x, y, z));
            if chunk.is_none() {
                continue;
            }
            let chunk = chunk.unwrap();

            let light = -directional.direction;

            for elem in chunk.iter_mut() {
                elem.value.set_shade(
                    Face::Top,
                    light.dot(Vec3::new(0.0, 1.0, 0.0)).max(0.0).min(1.0) * directional.intensity
                        + ambient.intensity,
                );
                elem.value.set_shade(
                    Face::Bottom,
                    light.dot(Vec3::new(0.0, -1.0, 0.0)).max(0.0).min(1.0) * directional.intensity
                        + ambient.intensity,
                );
                elem.value.set_shade(
                    Face::Front,
                    light.dot(Vec3::new(0.0, 0.0, 1.0)).max(0.0).min(1.0) * directional.intensity
                        + ambient.intensity,
                );
                elem.value.set_shade(
                    Face::Back,
                    light.dot(Vec3::new(0.0, 0.0, -1.0)).max(0.0).min(1.0) * directional.intensity
                        + ambient.intensity,
                );
                elem.value.set_shade(
                    Face::Left,
                    light.dot(Vec3::new(1.0, 0.0, 0.0)).max(0.0).min(1.0) * directional.intensity
                        + ambient.intensity,
                );
                elem.value.set_shade(
                    Face::Right,
                    light.dot(Vec3::new(-1.0, 0.0, 0.0)).max(0.0).min(1.0) * directional.intensity
                        + ambient.intensity,
                );
            }

            chunk.merge();

            insert.push(((x, y, z), ChunkUpdate::UpdateMesh));
        }
        for coords in remove {
            update.updates.remove(&coords);
        }
        for (coords, u) in insert {
            update.updates.insert(coords, u);
        }
    }
}

pub fn shaded_light_update<T: VoxelExt>(
    directional: Res<DirectionalLight>,
    ambient: Res<AmbientLight>,
    mut query: Query<(&mut Map<T>, &mut MapUpdates)>,
) {
    for (mut map, mut update) in &mut query.iter() {
        let mut remove = Vec::new();
        let mut insert = Vec::new();
        'outer: for (&(cx, cy, cz), update) in &update.updates {
            match update {
                ChunkUpdate::UpdateLight => {}
                _ => continue,
            }

            let chunk = map.get((cx, cy, cz)).unwrap();

            let mut light_map = vec![0.0; (chunk.width() + 2).pow(3)];

            let width = chunk.width() as i32;

            let lm_width = chunk.width() as i32 + 2;

            for x in -1..lm_width - 1 {
                for y in -1..lm_width - 1 {
                    for z in -1..lm_width - 1 {
                        let mut light = 0.0;
                        let mut count = 0;
                        let range = 1;
                        for lx in -range..=range {
                            for ly in -range..=range {
                                for lz in -range..=range {
                                    let x = x + lx;
                                    let y = y + ly;
                                    let z = z + lz;
                                    if x < 0
                                        || x >= width
                                        || y < 0
                                        || y >= width
                                        || z < 0
                                        || z >= width
                                    {
                                        let sx = if x < 0 {
                                            -1
                                        } else if x >= width {
                                            1
                                        } else {
                                            0
                                        };
                                        let sy = if y < 0 {
                                            -1
                                        } else if y >= width {
                                            1
                                        } else {
                                            0
                                        };
                                        let sz = if z < 0 {
                                            -1
                                        } else if z >= width {
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
                                            while x >= width {
                                                x -= width;
                                            }
                                            while x < 0 {
                                                x += width;
                                            }
                                            while y >= width {
                                                y -= width;
                                            }
                                            while y < 0 {
                                                y += width;
                                            }
                                            while z >= width {
                                                z -= width;
                                            }
                                            while z < 0 {
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
                        let idx = ((x + 1) * lm_width * lm_width) as usize
                            + ((y + 1) * lm_width) as usize
                            + (z + 1) as usize;
                        light_map[idx] = light;
                    }
                }
            }

            let chunk = map.get_mut((cx, cy, cz)).unwrap();

            let dir = -directional.direction;

            for elem in chunk.iter_mut() {
                let x = elem.x;
                let y = elem.y;
                let z = elem.z;
                let block = elem.value;

                let idx = ((x + 1) * lm_width * lm_width) as usize
                    + ((y + 2) * lm_width) as usize
                    + (z + 1) as usize;
                let light = light_map[idx];
                block.set_shade(
                    Face::Top,
                    light
                        * dir.dot(Vec3::new(0.0, 1.0, 0.0)).max(0.0).min(1.0)
                        * directional.intensity
                        + ambient.intensity,
                );

                let idx = ((x + 1) * lm_width * lm_width) as usize
                    + (y * lm_width) as usize
                    + (z + 1) as usize;
                let light = light_map[idx];
                block.set_shade(
                    Face::Bottom,
                    light
                        * dir.dot(Vec3::new(0.0, -1.0, 0.0)).max(0.0).min(1.0)
                        * directional.intensity
                        + ambient.intensity,
                );

                let idx = ((x + 1) * lm_width * lm_width) as usize
                    + ((y + 1) * lm_width) as usize
                    + (z + 2) as usize;
                let light = light_map[idx];
                block.set_shade(
                    Face::Front,
                    light
                        * dir.dot(Vec3::new(0.0, 0.0, 1.0)).max(0.0).min(1.0)
                        * directional.intensity
                        + ambient.intensity,
                );

                let idx = ((x + 1) * lm_width * lm_width) as usize
                    + ((y + 1) * lm_width) as usize
                    + z as usize;
                let light = light_map[idx];
                block.set_shade(
                    Face::Back,
                    light
                        * dir.dot(Vec3::new(0.0, 0.0, -1.0)).max(0.0).min(1.0)
                        * directional.intensity
                        + ambient.intensity,
                );

                let idx = ((x + 2) * lm_width * lm_width) as usize
                    + ((y + 1) * lm_width) as usize
                    + (z + 1) as usize;
                let light = light_map[idx];
                block.set_shade(
                    Face::Left,
                    light
                        * dir.dot(Vec3::new(1.0, 0.0, 0.0)).max(0.0).min(1.0)
                        * directional.intensity
                        + ambient.intensity,
                );

                let idx = (x * lm_width * lm_width) as usize
                    + ((y + 1) * lm_width) as usize
                    + (z + 1) as usize;
                let light = light_map[idx];
                block.set_shade(
                    Face::Right,
                    light
                        * dir.dot(Vec3::new(-1.0, 0.0, 0.0)).max(0.0).min(1.0)
                        * directional.intensity
                        + ambient.intensity,
                );
            }

            chunk.merge();

            remove.push((cx, cy, cz));
            insert.push(((cx, cy, cz), ChunkUpdate::UpdateMesh));
        }
        for coords in remove {
            update.updates.remove(&coords);
        }
        for (coords, u) in insert {
            update.updates.insert(coords, u);
        }
    }
}

pub fn light_map_update<T: VoxelExt, R: VoxelTracer>(
    directional: Res<DirectionalLight>,
    mut query: Query<(&mut Map<T>, &mut MapUpdates)>,
) {
    for (mut map, mut update) in &mut query.iter() {
        let mut remove = Vec::new();
        let mut insert = Vec::new();
        for (&(cx, cy, cz), update) in &update.updates {
            match update {
                ChunkUpdate::UpdateLightMap => {}
                _ => continue,
            }
            remove.push((cx, cy, cz));

            let chunk = map.get_mut((cx, cy, cz));
            if chunk.is_none() {
                continue;
            }
            let chunk = chunk.unwrap();

            let mut light_map = vec![None; chunk.width().pow(3)];

            let lm_width = chunk.width() as i32;

            for y in 0..lm_width {
                for x in 0..lm_width {
                    for z in 0..lm_width {
                        let idx = (x * lm_width * lm_width) as usize
                            + (y * lm_width) as usize
                            + z as usize;
                        if light_map[idx].is_some() {
                            continue;
                        }

                        let light_source =
                            Vec3::new(x as _, y as _, z as _) + directional.direction * -100.0;
                        let mut light = 1.0;
                        for (x, y, z) in R::new(
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
                            if x < 0
                                || y < 0
                                || z < 0
                                || x >= lm_width
                                || y >= lm_width
                                || z >= lm_width
                            {
                                continue;
                            }
                            let idx = (x * lm_width * lm_width) as usize
                                + (y * lm_width) as usize
                                + z as usize;
                            if let Some(map) = light_map.get_mut(idx) {
                                if map.is_none() {
                                    *map = Some(light);
                                }
                            }
                        }
                    }
                }
            }

            for x in 0..lm_width {
                for y in 0..lm_width {
                    for z in 0..lm_width {
                        let idx = (x * lm_width * lm_width) as usize
                            + (y * lm_width) as usize
                            + z as usize;
                        let light = light_map[idx];
                        chunk.insert_light((x, y, z), light.unwrap_or_default());
                    }
                }
            }

            chunk.set_light(true);

            insert.push(((cx, cy, cz), ChunkUpdate::UpdateLight));
        }
        for coords in remove {
            update.updates.remove(&coords);
        }
        for (coords, u) in insert {
            update.updates.insert(coords, u);
        }
    }
}
