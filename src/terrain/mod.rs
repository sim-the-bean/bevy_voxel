use bevy::prelude::*;

use noise::{NoiseFn, OpenSimplex, Perlin, Seedable, SuperSimplex};

use crate::{
    collections::lod_tree::Voxel,
    world::{Chunk, ChunkUpdate, Map, MapUpdates},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoiseType {
    Perlin,
    OpenSimplex,
    SuperSimplex,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoiseDimensions {
    Two,
    Three,
}

impl Default for NoiseType {
    fn default() -> Self {
        Self::SuperSimplex
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Octave {
    pub amplitude: f64,
    pub frequency: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Layer<T: Voxel> {
    pub block: T,
    pub height: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TerrainGenParameters<T: Voxel> {
    pub seed: u32,
    pub noise_type: NoiseType,
    pub dimensions: NoiseDimensions,
    pub chunk_size: u32,
    pub granularity: u32,
    pub octaves: Vec<Octave>,
    pub layers: Vec<Layer<T>>,
}

impl<T: Voxel> TerrainGenParameters<T> {
    pub fn chunk_width(&self) -> usize {
        2_usize.pow(self.chunk_size - self.granularity)
    }

    pub fn unit_width(&self) -> usize {
        2_usize.pow(self.granularity)
    }

    pub fn generate(&self, coords: (i32, i32, i32)) -> Chunk<T> {
        match self.dimensions {
            NoiseDimensions::Two => match self.noise_type {
                NoiseType::Perlin => terrain_gen2_impl::<_, Perlin>(self, coords),
                NoiseType::OpenSimplex => terrain_gen2_impl::<_, OpenSimplex>(self, coords),
                NoiseType::SuperSimplex => terrain_gen2_impl::<_, SuperSimplex>(self, coords),
            },
            NoiseDimensions::Three => match self.noise_type {
                NoiseType::Perlin => terrain_gen3_impl::<_, Perlin>(self, coords),
                NoiseType::OpenSimplex => terrain_gen3_impl::<_, OpenSimplex>(self, coords),
                NoiseType::SuperSimplex => terrain_gen3_impl::<_, SuperSimplex>(self, coords),
            },
        }
    }
}

pub fn terrain_generation<T: Voxel>(
    params: Res<TerrainGenParameters<T>>,
    mut query: Query<(&mut Map<T>, &mut MapUpdates)>,
) {
    let max_count = 1;
    let mut count = 0;
    for (mut map, mut update) in &mut query.iter() {
        let mut remove = Vec::new();
        let mut insert = Vec::new();
        for (&(x, y, z, w), update) in &update.updates {
            match update {
                ChunkUpdate::GenerateChunk => {}
                _ => continue,
            }
            if count == max_count {
                break;
            }
            count += 1;
            remove.push((x, y, z, w));
            let chunk = params.generate((x, y, z));
            map.insert(chunk);
            let range = 1;
            for lx in -range..=range {
                for ly in -range..=range {
                    for lz in -range..=range {
                        let x = x + lx;
                        let y = y + ly;
                        let z = z + lz;
                        insert.push(((x, y, z, w), ChunkUpdate::UpdateLightMap));
                    }
                }
            }
        }
        for coords in remove {
            update.updates.remove(&coords);
        }
        for (coords, u) in insert {
            if !update.updates.contains_key(&coords) {
                update.updates.insert(coords, u);
            }
        }
    }
}

fn terrain_gen2_impl<T: Voxel, N: NoiseFn<[f64; 2]> + Seedable + Default>(
    params: &TerrainGenParameters<T>,
    (cx, cy, cz): (i32, i32, i32),
) -> Chunk<T> {
    let noise = N::default().set_seed(params.seed);
    let mut chunk = Chunk::new(params.chunk_size, (cx, cy, cz));

    let size = params.chunk_width() as i32;
    let size_2 = size / 2;
    let by = cy * size - size_2;
    for x in -size_2..size_2 {
        let ax = cx * size + x;
        let fx = ax as f64;
        for z in -size_2..size_2 {
            let az = cz * size + z;
            let fz = az as f64;
            let mut height = 0.0;
            for octave in &params.octaves {
                height +=
                    noise.get([fx * octave.frequency, fz * octave.frequency]) * octave.amplitude;
            }
            let mut y = height as i32 - by;
            for layer in params.layers.iter().rev() {
                let layer_height = layer.height as i32;
                for _ in 0..layer_height {
                    y -= 1;
                    if y >= size_2 {
                        continue;
                    }
                    if y < -size_2 {
                        break;
                    }
                    let x = x << params.granularity;
                    let y = y << params.granularity;
                    let z = z << params.granularity;
                    for ix in 0..params.unit_width() as i32 {
                        for iy in 0..params.unit_width() as i32 {
                            for iz in 0..params.unit_width() as i32 {
                                chunk.insert((x + ix, y + iy, z + iz), layer.block.clone());
                            }
                        }
                    }
                }
            }
        }
    }

    chunk
}

fn terrain_gen3_impl<T: Voxel, N: NoiseFn<[f64; 3]> + Seedable + Default>(
    params: &TerrainGenParameters<T>,
    (cx, cy, cz): (i32, i32, i32),
) -> Chunk<T> {
    let noise = N::default().set_seed(params.seed);
    let mut chunk = Chunk::new(params.chunk_size, (cx, cy, cz));

    let size = params.chunk_width() as i32;
    let size_2 = size / 2;
    for x in -size_2..size_2 {
        let ax = cx * size + x;
        let fx = ax as f64;
        for y in -size_2..size_2 {
            let ay = cy * size + y;
            let fy = ay as f64;
            for z in -size_2..size_2 {
                let az = cz * size + z;
                let fz = az as f64;
                let mut height = 0.0;
                for octave in &params.octaves {
                    height += noise.get([
                        fx * octave.frequency,
                        fy * octave.frequency,
                        fz * octave.frequency,
                    ]) * octave.amplitude;
                }

                let mut h = height;
                let mut idx = None;
                for (i, layer) in params.layers.iter().enumerate() {
                    if h < layer.height {
                        idx = Some(i);
                        break;
                    }
                    h -= layer.height;
                }
                if let Some(idx) = idx {
                    let layer = &params.layers[idx];
                    let x = x << params.granularity;
                    let y = y << params.granularity;
                    let z = z << params.granularity;
                    for ix in 0..params.unit_width() as i32 {
                        for iy in 0..params.unit_width() as i32 {
                            for iz in 0..params.unit_width() as i32 {
                                chunk.insert((x + ix, y + iy, z + iz), layer.block.clone());
                            }
                        }
                    }
                }
            }
        }
    }

    chunk
}
