use serde::{Deserialize, Serialize};

use bevy::{prelude::*, render::mesh::Mesh};

use bevy_voxel::{
    collections::lod_tree::Voxel,
    render::{
        entity::{generate_chunk_mesh, Face, MeshPart, VoxelExt},
        light::*,
        lod::lod_update,
        prelude::*,
    },
    terrain::*,
    world::{Chunk, ChunkUpdate, Map, MapComponents, MapUpdates},
};

pub const CHUNK_SIZE: u32 = 5;
pub const WORLD_WIDTH: i32 = 128;
pub const WORLD_HEIGHT: i32 = 64;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Shade {
    pub top: f32,
    pub bottom: f32,
    pub front: f32,
    pub back: f32,
    pub left: f32,
    pub right: f32,
}

impl Shade {
    pub fn zero() -> Self {
        Shade {
            top: 0.0,
            bottom: 0.0,
            front: 0.0,
            back: 0.0,
            left: 0.0,
            right: 0.0,
        }
    }
}

impl Default for Shade {
    fn default() -> Self {
        Shade {
            top: 1.0,
            bottom: 1.0,
            front: 1.0,
            back: 1.0,
            left: 1.0,
            right: 1.0,
        }
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct Block {
    pub shade: Shade,
    pub color: Color,
}

impl Voxel for Block {
    fn average(data: &[Self]) -> Option<Self> {
        if data.is_empty() {
            return None;
        }

        let mut color = Color::rgba(0.0, 0.0, 0.0, 0.0);
        let mut top = 0.0;
        let mut bottom = 0.0;
        let mut left = 0.0;
        let mut right = 0.0;
        let mut front = 0.0;
        let mut back = 0.0;

        for block in data {
            top += block.shade.top;
            bottom += block.shade.bottom;
            left += block.shade.left;
            right += block.shade.right;
            front += block.shade.front;
            back += block.shade.back;
            color += block.color;
        }

        color *= (data.len() as f32).recip();
        top *= (data.len() as f32).recip();
        bottom *= (data.len() as f32).recip();
        left *= (data.len() as f32).recip();
        right *= (data.len() as f32).recip();
        front *= (data.len() as f32).recip();
        back *= (data.len() as f32).recip();

        Some(Self {
            color,
            shade: Shade {
                top,
                bottom,
                left,
                right,
                front,
                back,
            },
        })
    }

    fn can_merge(&self) -> bool {
        true
    }
}

impl VoxelExt for Block {
    fn mesh(
        &self,
        coords: (i32, i32, i32),
        map: &Map<Self>,
        chunk: &Chunk<Self>,
        width: usize,
        offset: (f32, f32, f32),
    ) -> MeshPart {
        let mut positions = Vec::new();
        let mut shades = Vec::new();
        let mut colors = Vec::new();
        let mut indices = Vec::new();

        let mut n = 0;
        if let Some((p, s, c)) = generate_top_side(
            self,
            map,
            chunk,
            coords,
            width,
            offset,
            &mut indices,
            &mut n,
        ) {
            positions.extend(&p);
            shades.extend(&s);
            colors.extend(&c);
        }

        if let Some((p, s, c)) = generate_bottom_side(
            self,
            map,
            chunk,
            coords,
            width,
            offset,
            &mut indices,
            &mut n,
        ) {
            positions.extend(&p);
            shades.extend(&s);
            colors.extend(&c);
        }

        if let Some((p, s, c)) = generate_front_side(
            self,
            map,
            chunk,
            coords,
            width,
            offset,
            &mut indices,
            &mut n,
        ) {
            positions.extend(&p);
            shades.extend(&s);
            colors.extend(&c);
        }

        if let Some((p, s, c)) = generate_back_side(
            self,
            map,
            chunk,
            coords,
            width,
            offset,
            &mut indices,
            &mut n,
        ) {
            positions.extend(&p);
            shades.extend(&s);
            colors.extend(&c);
        }

        if let Some((p, s, c)) = generate_left_side(
            self,
            map,
            chunk,
            coords,
            width,
            offset,
            &mut indices,
            &mut n,
        ) {
            positions.extend(&p);
            shades.extend(&s);
            colors.extend(&c);
        }

        if let Some((p, s, c)) = generate_right_side(
            self,
            map,
            chunk,
            coords,
            width,
            offset,
            &mut indices,
            &mut n,
        ) {
            positions.extend(&p);
            shades.extend(&s);
            colors.extend(&c);
        }

        MeshPart {
            positions,
            shades,
            colors,
            indices,
        }
    }

    fn set_shade(&mut self, face: Face, light: f32) {
        match face {
            Face::Top => self.shade.top = light,
            Face::Bottom => self.shade.bottom = light,
            Face::Front => self.shade.front = light,
            Face::Back => self.shade.back = light,
            Face::Left => self.shade.left = light,
            Face::Right => self.shade.right = light,
        }
    }

    fn shade(&mut self, face: Face) -> Option<f32> {
        match face {
            Face::Top => Some(self.shade.top),
            Face::Bottom => Some(self.shade.bottom),
            Face::Front => Some(self.shade.front),
            Face::Back => Some(self.shade.back),
            Face::Left => Some(self.shade.left),
            Face::Right => Some(self.shade.right),
        }
    }
}

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
        .run();
}

/// set up a simple 3D scene
fn setup(mut commands: Commands) {
    let mut update = MapUpdates::default();
    let chunk_size = 2_i32.pow(CHUNK_SIZE as u32);
    let world_width_2 = WORLD_WIDTH / chunk_size / 2;
    let world_height = WORLD_HEIGHT / chunk_size;
    for cx in -world_width_2..world_width_2 {
        for cy in 0..world_height {
            for cz in -world_width_2..world_width_2 {
                update.updates.insert(
                    (cx, cy, cz, chunk_size as usize),
                    ChunkUpdate::GenerateChunk,
                );
            }
        }
    }
    commands
        .spawn(MapComponents { map_update: update })
        .with(Map::<Block>::default())
        .spawn(bevy_fly_camera::FlyCamera::default());
}

fn chunk_update<T: VoxelExt>(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<VoxelMaterial>>,
    mut query: Query<(&Map<T>, &mut MapUpdates)>,
) {
    for (map, mut update) in &mut query.iter() {
        let mut remove = Vec::new();
        for (&(x, y, z, w), update) in &update.updates {
            match update {
                ChunkUpdate::UpdateMesh => {}
                _ => continue,
            }
            remove.push((x, y, z, w));

            let w_2 = w as i32 / 2;
            let cx = x * w as i32 - w_2;
            let cy = y * w as i32 - w_2;
            let cz = z * w as i32 - w_2;
            let chunk = map.get((cx, cy, cz)).unwrap();

            let mesh = generate_chunk_mesh(&map, &chunk);
            if let Some(mesh) = mesh {
                commands.spawn(ChunkRenderComponents {
                    mesh: meshes.add(mesh),
                    material: materials.add(VoxelMaterial {
                        albedo: Color::WHITE,
                    }),
                    ..Default::default()
                });
            }
        }
        for coords in remove {
            update.updates.remove(&coords);
        }
    }
}

fn generate_front_side(
    block: &Block,
    map: &Map<Block>,
    chunk: &Chunk<Block>,
    (x, y, z): (i32, i32, i32),
    width: usize,
    offset: (f32, f32, f32),
    indices: &mut Vec<u32>,
    n: &mut u32,
) -> Option<([[f32; 3]; 4], [f32; 4], [[f32; 4]; 4])> {
    let width = width as i32;
    let cw = chunk.width() as i32;
    let cw_2 = cw / 2;
    for dx in 0..width {
        for dy in 0..width {
            let render = if z + width >= cw_2 {
                let (cx, cy, cz) = chunk.position();
                let cx = cx * cw - cw_2;
                let cy = cy * cw - cw_2;
                let cz = cz * cw - cw_2 + cw;
                if let Some(chunk) = map.get((cx, cy, cz)) {
                    !chunk.contains_key((x + dx, y + dy, -cw_2))
                } else {
                    false
                }
            } else {
                !chunk.contains_key((x + dx, y + dy, z + width))
            };
            if render {
                let size = width as f32;
                let x = x as f32 + offset.0;
                let y = y as f32 + offset.1;
                let z = z as f32 + offset.2;
                indices.extend(&[*n + 0, *n + 1, *n + 2, *n + 2, *n + 3, *n + 0]);
                *n += 4;
                return Some((
                    [
                        [x, y, z + size],
                        [x + size, y, z + size],
                        [x + size, y + size, z + size],
                        [x, y + size, z + size],
                    ],
                    [
                        block.shade.front,
                        block.shade.front,
                        block.shade.front,
                        block.shade.front,
                    ],
                    [
                        block.color.into(),
                        block.color.into(),
                        block.color.into(),
                        block.color.into(),
                    ],
                ));
            }
        }
    }
    None
}

fn generate_back_side(
    block: &Block,
    map: &Map<Block>,
    chunk: &Chunk<Block>,
    (x, y, z): (i32, i32, i32),
    width: usize,
    offset: (f32, f32, f32),
    indices: &mut Vec<u32>,
    n: &mut u32,
) -> Option<([[f32; 3]; 4], [f32; 4], [[f32; 4]; 4])> {
    let width = width as i32;
    let cw = chunk.width() as i32;
    let cw_2 = cw / 2;
    for dx in 0..width {
        for dy in 0..width {
            let render = if z - 1 < -cw_2 {
                let (cx, cy, cz) = chunk.position();
                let cx = cx * cw - cw_2;
                let cy = cy * cw - cw_2;
                let cz = cz * cw - cw_2 - cw;
                if let Some(chunk) = map.get((cx, cy, cz)) {
                    !chunk.contains_key((x + dx, y + dy, cw_2 - 1))
                } else {
                    false
                }
            } else {
                !chunk.contains_key((x + dx, y + dy, z - 1))
            };
            if render {
                let size = width as f32;
                let x = x as f32 + offset.0;
                let y = y as f32 + offset.1;
                let z = z as f32 + offset.2;
                indices.extend(&[*n + 0, *n + 1, *n + 2, *n + 2, *n + 3, *n + 0]);
                *n += 4;
                return Some((
                    [
                        [x, y + size, z],
                        [x + size, y + size, z],
                        [x + size, y, z],
                        [x, y, z],
                    ],
                    [
                        block.shade.back,
                        block.shade.back,
                        block.shade.back,
                        block.shade.back,
                    ],
                    [
                        block.color.into(),
                        block.color.into(),
                        block.color.into(),
                        block.color.into(),
                    ],
                ));
            }
        }
    }
    None
}

fn generate_right_side(
    block: &Block,
    map: &Map<Block>,
    chunk: &Chunk<Block>,
    (x, y, z): (i32, i32, i32),
    width: usize,
    offset: (f32, f32, f32),
    indices: &mut Vec<u32>,
    n: &mut u32,
) -> Option<([[f32; 3]; 4], [f32; 4], [[f32; 4]; 4])> {
    let width = width as i32;
    let cw = chunk.width() as i32;
    let cw_2 = cw / 2;
    for dy in 0..width {
        for dz in 0..width {
            let render = if x - 1 < -cw_2 {
                let (cx, cy, cz) = chunk.position();
                let cx = cx * cw - cw_2 - cw;
                let cy = cy * cw - cw_2;
                let cz = cz * cw - cw_2;
                if let Some(chunk) = map.get((cx, cy, cz)) {
                    !chunk.contains_key((cw_2 - 1, y + dy, z + dz))
                } else {
                    false
                }
            } else {
                !chunk.contains_key((x - 1, y + dy, z + dz))
            };
            if render {
                let size = width as f32;
                let x = x as f32 + offset.0;
                let y = y as f32 + offset.1;
                let z = z as f32 + offset.2;
                indices.extend(&[*n + 0, *n + 1, *n + 2, *n + 2, *n + 3, *n + 0]);
                *n += 4;
                return Some((
                    [
                        [x, y, z],
                        [x, y, z + size],
                        [x, y + size, z + size],
                        [x, y + size, z],
                    ],
                    [
                        block.shade.right,
                        block.shade.right,
                        block.shade.right,
                        block.shade.right,
                    ],
                    [
                        block.color.into(),
                        block.color.into(),
                        block.color.into(),
                        block.color.into(),
                    ],
                ));
            }
        }
    }
    None
}

fn generate_left_side(
    block: &Block,
    map: &Map<Block>,
    chunk: &Chunk<Block>,
    (x, y, z): (i32, i32, i32),
    width: usize,
    offset: (f32, f32, f32),
    indices: &mut Vec<u32>,
    n: &mut u32,
) -> Option<([[f32; 3]; 4], [f32; 4], [[f32; 4]; 4])> {
    let width = width as i32;
    let cw = chunk.width() as i32;
    let cw_2 = cw / 2;
    for dy in 0..width {
        for dz in 0..width {
            let render = if x + width >= cw_2 {
                let (cx, cy, cz) = chunk.position();
                let cx = cx * cw - cw_2 + cw;
                let cy = cy * cw - cw_2;
                let cz = cz * cw - cw_2;
                if let Some(chunk) = map.get((cx, cy, cz)) {
                    !chunk.contains_key((-cw_2, y + dy, z + dz))
                } else {
                    false
                }
            } else {
                !chunk.contains_key((x + width, y + dy, z + dz))
            };
            if render {
                let size = width as f32;
                let x = x as f32 + offset.0;
                let y = y as f32 + offset.1;
                let z = z as f32 + offset.2;
                indices.extend(&[*n + 0, *n + 1, *n + 2, *n + 2, *n + 3, *n + 0]);
                *n += 4;
                return Some((
                    [
                        [x + size, y, z],
                        [x + size, y + size, z],
                        [x + size, y + size, z + size],
                        [x + size, y, z + size],
                    ],
                    [
                        block.shade.left,
                        block.shade.left,
                        block.shade.left,
                        block.shade.left,
                    ],
                    [
                        block.color.into(),
                        block.color.into(),
                        block.color.into(),
                        block.color.into(),
                    ],
                ));
            }
        }
    }
    None
}

fn generate_top_side(
    block: &Block,
    map: &Map<Block>,
    chunk: &Chunk<Block>,
    (x, y, z): (i32, i32, i32),
    width: usize,
    offset: (f32, f32, f32),
    indices: &mut Vec<u32>,
    n: &mut u32,
) -> Option<([[f32; 3]; 4], [f32; 4], [[f32; 4]; 4])> {
    let width = width as i32;
    let cw = chunk.width() as i32;
    let cw_2 = cw / 2;
    for dx in 0..width {
        for dz in 0..width {
            let render = if y + width >= cw_2 {
                let (cx, cy, cz) = chunk.position();
                let cx = cx * cw - cw_2;
                let cy = cy * cw - cw_2 + cw;
                let cz = cz * cw - cw_2;
                if let Some(chunk) = map.get((cx, cy, cz)) {
                    !chunk.contains_key((x + dx, -cw_2, z + dz))
                } else {
                    false
                }
            } else {
                !chunk.contains_key((x + dx, y + width, z + dz))
            };
            if render {
                let size = width as f32;
                let x = x as f32 + offset.0;
                let y = y as f32 + offset.1;
                let z = z as f32 + offset.2;
                indices.extend(&[*n + 0, *n + 1, *n + 2, *n + 2, *n + 3, *n + 0]);
                *n += 4;
                return Some((
                    [
                        [x + size, y + size, z],
                        [x, y + size, z],
                        [x, y + size, z + size],
                        [x + size, y + size, z + size],
                    ],
                    [
                        block.shade.top,
                        block.shade.top,
                        block.shade.top,
                        block.shade.top,
                    ],
                    [
                        block.color.into(),
                        block.color.into(),
                        block.color.into(),
                        block.color.into(),
                    ],
                ));
            }
        }
    }
    None
}

fn generate_bottom_side(
    block: &Block,
    map: &Map<Block>,
    chunk: &Chunk<Block>,
    (x, y, z): (i32, i32, i32),
    width: usize,
    offset: (f32, f32, f32),
    indices: &mut Vec<u32>,
    n: &mut u32,
) -> Option<([[f32; 3]; 4], [f32; 4], [[f32; 4]; 4])> {
    let width = width as i32;
    let cw = chunk.width() as i32;
    let cw_2 = cw / 2;
    for dx in 0..width {
        for dz in 0..width {
            let render = if y - 1 < -cw_2 {
                let (cx, cy, cz) = chunk.position();
                let cx = cx * cw - cw_2;
                let cy = cy * cw - cw_2 - cw;
                let cz = cz * cw - cw_2;
                if let Some(chunk) = map.get((cx, cy, cz)) {
                    !chunk.contains_key((x + dx, cw_2 - 1, z + dz))
                } else {
                    false
                }
            } else {
                !chunk.contains_key((x + dx, y - 1, z + dz))
            };
            if render {
                let size = width as f32;
                let x = x as f32 + offset.0;
                let y = y as f32 + offset.1;
                let z = z as f32 + offset.2;
                indices.extend(&[*n + 0, *n + 1, *n + 2, *n + 2, *n + 3, *n + 0]);
                *n += 4;
                return Some((
                    [
                        [x + size, y, z + size],
                        [x, y, z + size],
                        [x, y, z],
                        [x + size, y, z],
                    ],
                    [
                        block.shade.bottom,
                        block.shade.bottom,
                        block.shade.bottom,
                        block.shade.bottom,
                    ],
                    [
                        block.color.into(),
                        block.color.into(),
                        block.color.into(),
                        block.color.into(),
                    ],
                ));
            }
        }
    }
    None
}
