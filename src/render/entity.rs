#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use bevy::{
    asset::Handle,
    ecs::Bundle,
    render::{
        color::Color,
        draw::Draw,
        mesh::Mesh,
        pipeline::{DynamicBinding, PipelineSpecialization, RenderPipeline, RenderPipelines},
        render_graph::base::MainPass,
    },
    transform::prelude::{Rotation, Scale, Transform, Translation},
};

use crate::{
    collections::lod_tree::Voxel,
    render::{material::VoxelMaterial, render_graph::pipeline},
    world::{Chunk, Shade},
};

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
            top += block.shade().top;
            bottom += block.shade().bottom;
            left += block.shade().left;
            right += block.shade().right;
            front += block.shade().front;
            back += block.shade().back;
            color += block.color();
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

    fn shade(&self) -> Shade {
        self.shade
    }

    fn color(&self) -> Color {
        self.color
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ChunkMeshUpdate {
    pub update_mesh: bool,
    pub update_light: bool,
    pub generate_chunk: bool,
}

#[derive(Bundle)]
pub struct ChunkRenderComponents {
    pub chunk: Chunk<Block>,
    pub mesh_update: ChunkMeshUpdate,
    pub mesh: Handle<Mesh>,
    pub material: Handle<VoxelMaterial>,
    pub main_pass: MainPass,
    pub draw: Draw,
    pub render_pipelines: RenderPipelines,
    pub transform: Transform,
    pub translation: Translation,
    pub rotation: Rotation,
    pub scale: Scale,
}

impl Default for ChunkRenderComponents {
    fn default() -> Self {
        Self {
            chunk: Chunk::default(),
            mesh_update: ChunkMeshUpdate {
                update_mesh: false,
                update_light: true,
                generate_chunk: false,
            },
            render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::specialized(
                pipeline::PIPELINE_HANDLE,
                PipelineSpecialization {
                    dynamic_bindings: vec![
                        // Transform
                        DynamicBinding {
                            bind_group: 2,
                            binding: 0,
                        },
                        // Voxel_material
                        DynamicBinding {
                            bind_group: 1,
                            binding: 0,
                        },
                    ],
                    ..Default::default()
                },
            )]),
            mesh: Default::default(),
            material: Default::default(),
            main_pass: Default::default(),
            draw: Default::default(),
            transform: Default::default(),
            translation: Default::default(),
            rotation: Default::default(),
            scale: Default::default(),
        }
    }
}

fn generate_front_side<T: Voxel>(
    block: &T,
    chunk: &Chunk<T>,
    (x, y, z): (i32, i32, i32),
    width: usize,
    offset: (f32, f32, f32),
    indices: &mut Vec<u32>,
    n: &mut u32,
) -> Option<([[f32; 3]; 4], [f32; 4], [[f32; 4]; 4])> {
    for dx in 0..width as i32 {
        for dy in 0..width as i32 {
            if !chunk.contains_key((x + dx, y + dy, z + width as i32)) {
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
                        block.shade().front,
                        block.shade().front,
                        block.shade().front,
                        block.shade().front,
                    ],
                    [
                        block.color().into(),
                        block.color().into(),
                        block.color().into(),
                        block.color().into(),
                    ],
                ));
            }
        }
    }
    None
}

fn generate_back_side<T: Voxel>(
    block: &T,
    chunk: &Chunk<T>,
    (x, y, z): (i32, i32, i32),
    width: usize,
    offset: (f32, f32, f32),
    indices: &mut Vec<u32>,
    n: &mut u32,
) -> Option<([[f32; 3]; 4], [f32; 4], [[f32; 4]; 4])> {
    for dx in 0..width as i32 {
        for dy in 0..width as i32 {
            if !chunk.contains_key((x + dx, y + dy, z - 1)) {
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
                        block.shade().back,
                        block.shade().back,
                        block.shade().back,
                        block.shade().back,
                    ],
                    [
                        block.color().into(),
                        block.color().into(),
                        block.color().into(),
                        block.color().into(),
                    ],
                ));
            }
        }
    }
    None
}

fn generate_right_side<T: Voxel>(
    block: &T,
    chunk: &Chunk<T>,
    (x, y, z): (i32, i32, i32),
    width: usize,
    offset: (f32, f32, f32),
    indices: &mut Vec<u32>,
    n: &mut u32,
) -> Option<([[f32; 3]; 4], [f32; 4], [[f32; 4]; 4])> {
    for dy in 0..width as i32 {
        for dz in 0..width as i32 {
            if !chunk.contains_key((x - 1, y + dy, z + dz)) {
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
                        block.shade().right,
                        block.shade().right,
                        block.shade().right,
                        block.shade().right,
                    ],
                    [
                        block.color().into(),
                        block.color().into(),
                        block.color().into(),
                        block.color().into(),
                    ],
                ));
            }
        }
    }
    None
}

fn generate_left_side<T: Voxel>(
    block: &T,
    chunk: &Chunk<T>,
    (x, y, z): (i32, i32, i32),
    width: usize,
    offset: (f32, f32, f32),
    indices: &mut Vec<u32>,
    n: &mut u32,
) -> Option<([[f32; 3]; 4], [f32; 4], [[f32; 4]; 4])> {
    for dy in 0..width as i32 {
        for dz in 0..width as i32 {
            if !chunk.contains_key((x + width as i32, y + dy, z + dz)) {
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
                        block.shade().left,
                        block.shade().left,
                        block.shade().left,
                        block.shade().left,
                    ],
                    [
                        block.color().into(),
                        block.color().into(),
                        block.color().into(),
                        block.color().into(),
                    ],
                ));
            }
        }
    }
    None
}

fn generate_top_side<T: Voxel>(
    block: &T,
    chunk: &Chunk<T>,
    (x, y, z): (i32, i32, i32),
    width: usize,
    offset: (f32, f32, f32),
    indices: &mut Vec<u32>,
    n: &mut u32,
) -> Option<([[f32; 3]; 4], [f32; 4], [[f32; 4]; 4])> {
    for dx in 0..width as i32 {
        for dz in 0..width as i32 {
            if !chunk.contains_key((x + dx, y + width as i32, z + dz)) {
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
                        block.shade().top,
                        block.shade().top,
                        block.shade().top,
                        block.shade().top,
                    ],
                    [
                        block.color().into(),
                        block.color().into(),
                        block.color().into(),
                        block.color().into(),
                    ],
                ));
            }
        }
    }
    None
}

fn generate_bottom_side<T: Voxel>(
    block: &T,
    chunk: &Chunk<T>,
    (x, y, z): (i32, i32, i32),
    width: usize,
    offset: (f32, f32, f32),
    indices: &mut Vec<u32>,
    n: &mut u32,
) -> Option<([[f32; 3]; 4], [f32; 4], [[f32; 4]; 4])> {
    for dx in 0..width as i32 {
        for dz in 0..width as i32 {
            if !chunk.contains_key((x + dx, y - 1, z + dz)) {
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
                        block.shade().bottom,
                        block.shade().bottom,
                        block.shade().bottom,
                        block.shade().bottom,
                    ],
                    [
                        block.color().into(),
                        block.color().into(),
                        block.color().into(),
                        block.color().into(),
                    ],
                ));
            }
        }
    }
    None
}

pub fn generate_chunk_mesh<T: Voxel>(chunk: &Chunk<T>) -> Mesh {
    let mut positions = Vec::new();
    let mut shades = Vec::new();
    let mut colors = Vec::new();
    let mut indices = Vec::new();
    let mut n = 0;

    let coords = chunk.position();
    let chunk_width = chunk.width() as f32;
    let cx = coords.0 as f32 * chunk_width;
    let cy = coords.1 as f32 * chunk_width;
    let cz = coords.2 as f32 * chunk_width;

    for elem in chunk.iter() {
        let top = generate_top_side(
            &*elem.value,
            chunk,
            (elem.x, elem.y, elem.z),
            elem.width,
            (cx, cy, cz),
            &mut indices,
            &mut n,
        );
        let bottom = generate_bottom_side(
            &*elem.value,
            chunk,
            (elem.x, elem.y, elem.z),
            elem.width,
            (cx, cy, cz),
            &mut indices,
            &mut n,
        );
        let right = generate_right_side(
            &*elem.value,
            chunk,
            (elem.x, elem.y, elem.z),
            elem.width,
            (cx, cy, cz),
            &mut indices,
            &mut n,
        );
        let left = generate_left_side(
            &*elem.value,
            chunk,
            (elem.x, elem.y, elem.z),
            elem.width,
            (cx, cy, cz),
            &mut indices,
            &mut n,
        );
        let front = generate_front_side(
            &*elem.value,
            chunk,
            (elem.x, elem.y, elem.z),
            elem.width,
            (cx, cy, cz),
            &mut indices,
            &mut n,
        );
        let back = generate_back_side(
            &*elem.value,
            chunk,
            (elem.x, elem.y, elem.z),
            elem.width,
            (cx, cy, cz),
            &mut indices,
            &mut n,
        );

        if let Some(top) = top {
            positions.extend(&top.0);
            shades.extend(&top.1);
            colors.extend(&top.2);
        }

        if let Some(bottom) = bottom {
            positions.extend(&bottom.0);
            shades.extend(&bottom.1);
            colors.extend(&bottom.2);
        }

        if let Some(right) = right {
            positions.extend(&right.0);
            shades.extend(&right.1);
            colors.extend(&right.2);
        }

        if let Some(left) = left {
            positions.extend(&left.0);
            shades.extend(&left.1);
            colors.extend(&left.2);
        }

        if let Some(front) = front {
            positions.extend(&front.0);
            shades.extend(&front.1);
            colors.extend(&front.2);
        }

        if let Some(back) = back {
            positions.extend(&back.0);
            shades.extend(&back.1);
            colors.extend(&back.2);
        }
    }

    Mesh {
        primitive_topology: bevy::render::pipeline::PrimitiveTopology::TriangleList,
        attributes: vec![
            bevy::render::mesh::VertexAttribute {
                name: From::from("Voxel_Position"),
                values: bevy::render::mesh::VertexAttributeValues::Float3(positions),
            },
            bevy::render::mesh::VertexAttribute {
                name: From::from("Voxel_Shade"),
                values: bevy::render::mesh::VertexAttributeValues::Float(shades),
            },
            bevy::render::mesh::VertexAttribute {
                name: From::from("Voxel_Color"),
                values: bevy::render::mesh::VertexAttributeValues::Float4(colors),
            },
        ],
        indices: Some(indices),
    }
}
