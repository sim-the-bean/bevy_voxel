use bevy::prelude::*;

use noise::{NoiseFn, OpenSimplex, Perlin, Seedable, SuperSimplex};
use rand::SeedableRng;
use rstar::{PointDistance, RTree, RTreeObject, AABB};

use crate::{
    collections::lod_tree::Voxel,
    world::{Chunk, ChunkUpdate, Map, MapUpdates},
};

pub mod dsl;

pub use dsl::*;

#[derive(Debug, Clone)]
pub struct HeightChunk {
    position: (i32, i32),
    width: usize,
    filter: Filter,
    array: Vec<f32>,
}

impl HeightChunk {
    pub fn new(position: (i32, i32), width: usize, filter: Filter, array: Vec<f32>) -> Self {
        Self { position, width, filter, array }
    }

    pub fn get(&self, (x, z): (i32, i32)) -> f32 {
        match self.filter {
            Filter::NearestNeighbour => self.array[(x * self.width as i32 + z) as usize],
            Filter::Bilinear(width) => {
                let bx = x % width;
                let bz = z % width;
                let x = x / width;
                let z = z / width;
                let a = self.array[(x * self.width as i32 + z) as usize];
                let b = self.array[((x + 1) * self.width as i32 + z) as usize];
                let c = self.array[(x * self.width as i32 + z + 1) as usize];
                let d = self.array[((x + 1) * self.width as i32 + z + 1) as usize];
                let recip_width = (width as f32).recip();
                let rx = bx as f32 * recip_width;
                let rz = bz as f32 * recip_width;
                let x0 = a + (b - a) * rx;
                let x1 = c + (d - c) * rx;
                let z = x0 + (x1 - x0) * rz;
                z
            }
        }
    }

    pub fn insert(&mut self, (x, z): (i32, i32), value: f32) {
        self.array[(x * self.width as i32 + z) as usize] = value;
    }
}

impl RTreeObject for HeightChunk {
    type Envelope = AABB<[i32; 2]>;

    fn envelope(&self) -> Self::Envelope {
        let w = self.width as i32;
        let x0 = self.position.0;
        let y0 = self.position.1;
        let x1 = self.position.0 + w - 1;
        let y1 = self.position.1 + w - 1;
        AABB::from_corners([x0, y0], [x1, y1])
    }
}

impl PointDistance for HeightChunk {
    fn distance_2(&self, point: &[i32; 2]) -> i32 {
        self.envelope().distance_2(point)
    }
}

#[derive(Default, Debug, Clone)]
pub struct HeightMap {
    map: RTree<HeightChunk>,
}

impl HeightMap {
    pub fn new() -> Self {
        Self { map: RTree::new() }
    }

    pub fn with_chunks(initial: Vec<HeightChunk>) -> Self {
        Self {
            map: RTree::bulk_load(initial),
        }
    }

    pub fn get(&self, (x, z): (i32, i32)) -> Option<&HeightChunk> {
        self.map.locate_at_point(&[x, z])
    }

    pub fn get_mut(&mut self, (x, z): (i32, i32)) -> Option<&mut HeightChunk> {
        self.map.locate_at_point_mut(&[x, z])
    }

    pub fn get_mut_or_else<F: FnOnce() -> HeightChunk>(&mut self, (x, z): (i32, i32), f: F) -> &mut HeightChunk {
        if self.get((x, z)).is_some() {
            self.get_mut((x, z)).unwrap()
        } else {
            self.insert(f());
            self.get_mut((x, z)).unwrap()
        }
    }

    pub fn insert(&mut self, value: HeightChunk) {
        let (x, z) = value.position;
        self.map.remove_at_point(&[x, z]);
        self.map.insert(value);
    }

    pub fn remove(&mut self, (x, z): (i32, i32)) -> Option<HeightChunk> {
        self.map.remove_at_point(&[x, z])
    }
}

impl<T: Voxel> Program<T> {
    pub fn height_chunk<N: NoiseFn<[f64; 2]> + Seedable + Default>(&self, (cx, cz): (i32, i32)) -> HeightChunk {
        let a = self.filter.aux_width();
        let mut chunk = Vec::with_capacity((self.chunk_width() / self.filter.as_usize() + a as usize).pow(2));
        
        let noise = N::default().set_seed(self.seed);
        let unit_width = self.unit_width() as i32;

        let size = self.chunk_width() as i32 / self.filter.as_i32();
        
        let mut biome_map = Vec::with_capacity(chunk.capacity());
        
        for x in 0..size + a {
            let ax = cx + x * unit_width * self.filter.as_i32();
            let fx = ax as f64;
            for z in 0..size + a {
                let az = cz + z * unit_width * self.filter.as_i32();
                let fz = az as f64;
                let mut height = noise.get([fx * self.biome_frequency, fz * self.biome_frequency]) * 0.5 + 0.5;
                let mut idx = 0_usize;
                for (i, biome) in self.biomes.iter().enumerate() {
                    if height < biome.prob {
                        idx = i;
                        break;
                    }
                    height -= biome.prob;
                }
                biome_map.push(idx);
            }
        }
        
        for x in 0..size + a {
            let ax = cx + x * unit_width * self.filter.as_i32();
            let fx = ax as f64;
            for z in 0..size + a {
                let az = cz + z * unit_width * self.filter.as_i32();
                let fz = az as f64;
                let biome = biome_map[(x * (size + a) + z) as usize];
                let biome = &self.biomes[biome];
                let mut height = 0.0;
                for octave in &biome.octaves {
                    height +=
                        noise.get([fx * octave.frequency, fz * octave.frequency]) * octave.amplitude;
                }
                chunk.push(height as f32);
            }
        }

        HeightChunk::new((cx, cz), self.chunk_width().div_euclid(self.filter.as_usize()) + a as usize, self.filter, chunk)
    }
    
    pub fn chunk_width(&self) -> usize {
        2_usize.pow(self.chunk_size - self.subdivisions)
    }

    pub fn unit_width(&self) -> usize {
        2_usize.pow(self.subdivisions)
    }

    pub fn execute(&self, height_map: &mut HeightMap, coords: (i32, i32, i32)) -> Chunk<T> {
        match self.dimensions {
            NoiseDimensions::Two => match self.noise_type {
                NoiseType::Perlin => terrain_gen2_impl::<_, Perlin>(self, height_map, coords),
                NoiseType::OpenSimplex => terrain_gen2_impl::<_, OpenSimplex>(self, height_map, coords),
                NoiseType::SuperSimplex => terrain_gen2_impl::<_, SuperSimplex>(self, height_map, coords),
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
    params: Res<Program<T>>,
    mut height_map: ResMut<HeightMap>,
    mut query: Query<(&mut Map<T>, &mut MapUpdates)>,
) {
    let max_count = 1;
    let mut count = 0;
    for (mut map, mut map_update) in &mut query.iter() {
        let mut remove = Vec::new();
        let mut insert = Vec::new();
        for (&(x, y, z), update) in &map_update.updates {
            match update {
                ChunkUpdate::GenerateChunk => {}
                _ => continue,
            }
            if count == max_count {
                break;
            }
            count += 1;
            remove.push((x, y, z));
            let chunk = params.execute(&mut height_map, (x, y, z));
            let width = chunk.width() as i32;
            map.insert(chunk);
            let range = 1;
            for lx in -range..=range {
                for ly in -range..=range {
                    for lz in -range..=range {
                        let x = x + lx * width;
                        let y = y + ly * width;
                        let z = z + lz * width;
                        if lx != 0 && ly != 0 && lz != 0 {
                            if let Some(u) = map_update.updates.get(&(x, y, z)) {
                                if u > &ChunkUpdate::UpdateLightMap {
                                    insert.push(((x, y, z), ChunkUpdate::UpdateLightMap));
                                }
                                continue;
                            }
                        }
                        insert.push(((x, y, z), ChunkUpdate::UpdateLightMap));
                    }
                }
            }
        }
        for coords in remove {
            map_update.updates.remove(&coords);
        }
        for (coords, u) in insert {
            if !map_update.updates.contains_key(&coords) {
                map_update.updates.insert(coords, u);
            }
        }
    }
}

fn terrain_gen2_impl<T: Voxel, N: NoiseFn<[f64; 2]> + Seedable + Default>(
    params: &Program<T>,
    height_map: &mut HeightMap,
    (cx, cy, cz): (i32, i32, i32),
) -> Chunk<T> {
    let height_chunk = height_map.get_mut_or_else((cx, cz), || {
        params.height_chunk::<N>((cx, cz))
    });
    
    let mut chunk = Chunk::new(params.chunk_size, (cx, cy, cz));
    let unit_width = params.unit_width() as i32;

    let size = params.chunk_width() as i32;
        
    let noise = N::default().set_seed(params.seed);
    let mut biome_map = Vec::with_capacity(params.chunk_size.pow(2) as usize);
    
    for x in 0..size {
        let ax = cx + x * unit_width * params.filter.as_i32();
        let fx = ax as f64;
        for z in 0..size {
            let az = cz + z * unit_width * params.filter.as_i32();
            let fz = az as f64;
            let mut height = noise.get([fx * params.biome_frequency, fz * params.biome_frequency]) * 0.5 + 0.5;
            let mut idx = 0_usize;
            for (i, biome) in params.biomes.iter().enumerate() {
                if height < biome.prob {
                    idx = i;
                    break;
                }
                height -= biome.prob;
            }
            biome_map.push(idx);
        }
    }
    
    let by = cy / unit_width;
    for x in 0..size {
        for z in 0..size {
            let biome = biome_map[(x * size + z) as usize];
            let biome = &params.biomes[biome];
            let height = height_chunk.get((x, z)) as f64;
            let mut y = height as i32 - by;
            for layer in biome.layers.iter().rev() {
                let layer_height = layer.height as i32;
                for _ in 0..layer_height {
                    y -= 1;
                    if y >= size {
                        continue;
                    }
                    if y < 0 {
                        break;
                    }
                    let x = x << params.subdivisions;
                    let y = y << params.subdivisions;
                    let z = z << params.subdivisions;
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

    let mut rng = rand::rngs::SmallRng::seed_from_u64((cx as u64) << 32 | cz as u64);
    
    for x in 0..size {
        for z in 0..size {
            let biome = biome_map[(x * size + z) as usize];
            let biome = &params.biomes[biome];
            let x = x << params.subdivisions;
            let z = z << params.subdivisions;
            for stmt in &biome.per_xz {
                let result = stmt.execute(&mut rng, Some((x, z)), &chunk);
                if let Some(diff) = result.block {
                    for ux in 0..diff.size.0 {
                        for uy in 0..diff.size.1 {
                            for uz in 0..diff.size.2 {
                                for ix in 0..params.unit_width() as i32 {
                                    for iy in 0..params.unit_width() as i32 {
                                        for iz in 0..params.unit_width() as i32 {
                                            let x = diff.at.0 + ux as i32 + ix;
                                            let y = diff.at.1 + uy as i32 + iy;
                                            let z = diff.at.2 + uz as i32 + iz;
                                            chunk.insert((x, y, z), diff.data[ux * diff.size.1 * diff.size.2 + uy * diff.size.2 + uz].clone());
                                        }
                                    }
                                }
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
    _params: &Program<T>,
    (_cx, _cy, _cz): (i32, i32, i32),
) -> Chunk<T> {
    todo!()
}
