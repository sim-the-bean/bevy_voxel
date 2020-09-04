use bevy::prelude::*;

use noise::{NoiseFn, OpenSimplex, Perlin, Seedable, SuperSimplex};

use rstar::{PointDistance, RTree, RTreeObject, AABB};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Filter {
    NearestNeighbour,
    Bilinear(i32),
}

impl Filter {
    pub fn aux_width(&self) -> i32 {
        match self {
            Filter::NearestNeighbour => 0,
            Filter::Bilinear(_) => 1,
        }
    }
    
    pub fn as_i32(&self) -> i32 {
        match self {
            Filter::NearestNeighbour => 1,
            Filter::Bilinear(width) => *width,
        }
    }
    
    pub fn as_usize(&self) -> usize {
        self.as_i32() as _
    }
}

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
                debug_assert!((width as u32).is_power_of_two());
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

#[derive(Debug, Clone, PartialEq)]
pub struct TerrainGenParameters<T: Voxel> {
    pub seed: u32,
    pub noise_type: NoiseType,
    pub dimensions: NoiseDimensions,
    pub chunk_size: u32,
    pub granularity: u32,
    pub filter: Filter,
    pub octaves: Vec<Octave>,
    pub layers: Vec<Layer<T>>,
}

impl<T: Voxel> TerrainGenParameters<T> {
    pub fn height_chunk<N: NoiseFn<[f64; 2]> + Seedable + Default>(&self, (cx, cz): (i32, i32)) -> HeightChunk {
        let a = self.filter.aux_width();
        let mut chunk = Vec::with_capacity((self.chunk_width() / self.filter.as_usize() + a as usize).pow(2));
        
        let noise = N::default().set_seed(self.seed);
        let unit_width = self.unit_width() as i32;

        let size = self.chunk_width() as i32 / self.filter.as_i32();
        for x in 0..size + a {
            let ax = cx + x * unit_width * self.filter.as_i32();
            let fx = ax as f64;
            for z in 0..size + a {
                let az = cz + z * unit_width * self.filter.as_i32();
                let fz = az as f64;
                let mut height = 0.0;
                for octave in &self.octaves {
                    height +=
                        noise.get([fx * octave.frequency, fz * octave.frequency]) * octave.amplitude;
                }
                chunk.push(height as f32);
            }
        }

        HeightChunk::new((cx, cz), self.chunk_width().div_euclid(self.filter.as_usize()) + a as usize, self.filter, chunk)
    }
    
    pub fn chunk_width(&self) -> usize {
        2_usize.pow(self.chunk_size - self.granularity)
    }

    pub fn unit_width(&self) -> usize {
        2_usize.pow(self.granularity)
    }

    pub fn generate(&self, height_map: &mut HeightMap, coords: (i32, i32, i32)) -> Chunk<T> {
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
    params: Res<TerrainGenParameters<T>>,
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
            let chunk = params.generate(&mut height_map, (x, y, z));
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
    params: &TerrainGenParameters<T>,
    height_map: &mut HeightMap,
    (cx, cy, cz): (i32, i32, i32),
) -> Chunk<T> {
    let height_chunk = height_map.get_mut_or_else((cx, cz), || {
        params.height_chunk::<N>((cx, cz))
    });
    
    let mut chunk = Chunk::new(params.chunk_size, (cx, cy, cz));
    let unit_width = params.unit_width() as i32;

    let size = params.chunk_width() as i32;
    let by = cy / unit_width;
    for x in 0..size {
        for z in 0..size {
            let height = height_chunk.get((x, z)) as f64;
            let mut y = height as i32 - by;
            for layer in params.layers.iter().rev() {
                let layer_height = layer.height as i32;
                for _ in 0..layer_height {
                    y -= 1;
                    if y >= size {
                        continue;
                    }
                    if y < 0 {
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
    for x in 0..size {
        let ax = cx + x;
        let fx = ax as f64;
        for y in 0..size {
            let ay = cy + y;
            let fy = ay as f64;
            for z in 0..size {
                let az = cz + z;
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
