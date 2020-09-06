#[cfg(feature = "savedata")]
use serde::{Deserialize, Serialize};

use bevy::prelude::*;

#[cfg(feature = "savedata")]
use crate::serialize::SerDePartialEq;

use crate::{
    collections::lod_tree::Voxel,
    render::entity::{Face, MeshPart, VoxelExt, Transparent},
    world::{Chunk, Map},
};

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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeshType {
    Cube,
    Cross,
}

impl Default for MeshType {
    fn default() -> Self {
        Self::Cube
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct Block {
    #[cfg_attr(feature = "serde", serde(skip))]
    pub shade: Shade,
    pub color: Color,
    pub mesh_type: MeshType,
}

impl Block {
    pub fn solid(&self) -> bool {
        self.mesh_type == MeshType::Cube && self.color.a == 1.0
    }
    
    pub fn transparent(&self) -> bool {
        self.color.a < 1.0
    }

    fn mesh_cube(
        &self,
        coords: (i32, i32, i32),
        map: &Map<Self>,
        chunk: &Chunk<Self>,
        width: usize,
    ) -> MeshPart {
        let mut positions = Vec::new();
        let mut shades = Vec::new();
        let mut colors = Vec::new();
        let mut indices = Vec::new();

        let mut n = 0;
        if let Some((p, s, c)) =
            generate_top_side(self, map, chunk, coords, width, &mut indices, &mut n)
        {
            positions.extend(&p);
            shades.extend(&s);
            colors.extend(&c);
        }

        if let Some((p, s, c)) =
            generate_bottom_side(self, map, chunk, coords, width, &mut indices, &mut n)
        {
            positions.extend(&p);
            shades.extend(&s);
            colors.extend(&c);
        }

        if let Some((p, s, c)) =
            generate_front_side(self, map, chunk, coords, width, &mut indices, &mut n)
        {
            positions.extend(&p);
            shades.extend(&s);
            colors.extend(&c);
        }

        if let Some((p, s, c)) =
            generate_back_side(self, map, chunk, coords, width, &mut indices, &mut n)
        {
            positions.extend(&p);
            shades.extend(&s);
            colors.extend(&c);
        }

        if let Some((p, s, c)) =
            generate_left_side(self, map, chunk, coords, width, &mut indices, &mut n)
        {
            positions.extend(&p);
            shades.extend(&s);
            colors.extend(&c);
        }

        if let Some((p, s, c)) =
            generate_right_side(self, map, chunk, coords, width, &mut indices, &mut n)
        {
            positions.extend(&p);
            shades.extend(&s);
            colors.extend(&c);
        }

        let transparent = self.color.a < 1.0;

        MeshPart {
            positions,
            shades,
            colors,
            indices,
            transparent: Transparent::from(transparent),
        }
    }

    fn mesh_cross(
        &self,
        coords: (i32, i32, i32),
        _map: &Map<Self>,
        _chunk: &Chunk<Self>,
        width: usize,
    ) -> MeshPart {
        let x = coords.0 as f32;
        let y = coords.1 as f32;
        let z = coords.2 as f32;
        let size = width as f32;

        let positions = vec![
            [x, y, z + size],
            [x, y + size, z + size],
            [x + size, y + size, z],
            [x + size, y, z],
            [x, y + size, z],
            [x, y, z],
            [x + size, y, z + size],
            [x + size, y + size, z + size],
            [x, y + size, z + size],
            [x, y, z + size],
            [x + size, y, z],
            [x + size, y + size, z],
            [x, y, z],
            [x, y + size, z],
            [x + size, y + size, z + size],
            [x + size, y, z + size],
        ];
        let front = self.shade.front;
        let back = self.shade.back;
        let left = self.shade.left;
        let right = self.shade.right;
        let shade_a = (front + left) * 0.5;
        let shade_b = (front + right) * 0.5;
        let shade_c = (back + left) * 0.5;
        let shade_d = (back + right) * 0.5;
        let shades = vec![
            shade_b, shade_b, shade_b, shade_b, shade_d, shade_d, shade_d, shade_d, shade_c,
            shade_c, shade_c, shade_c, shade_a, shade_a, shade_a, shade_a,
        ];
        let colors = vec![self.color.into(); 16];

        let indices = vec![
            0, 1, 2, 2, 3, 0, 4, 5, 6, 6, 7, 4, 8, 9, 10, 10, 11, 8, 12, 13, 14, 14, 15, 12,
        ];
        
        let transparent = self.color.a < 1.0;

        MeshPart {
            positions,
            shades,
            colors,
            indices,
            transparent: Transparent::from(transparent),
        }
    }
}

#[cfg(feature = "savedata")]
impl SerDePartialEq<Self> for Block {
    fn serde_eq(&self, other: &Self) -> bool {
        self.color == other.color
    }
}

impl Voxel for Block {
    fn average(data: &[Self]) -> Option<Self> {
        if data.is_empty() {
            return None;
        } else if data.len() == 1 {
            return Some(data[0].clone());
        };

        let mut color = Color::rgba(0.0, 0.0, 0.0, 0.0);
        let mut top = 0.0_f32;
        let mut bottom = 0.0_f32;
        let mut left = 0.0_f32;
        let mut right = 0.0_f32;
        let mut front = 0.0_f32;
        let mut back = 0.0_f32;

        for block in data {
            top = top.max(block.shade.top);
            bottom = bottom.max(block.shade.bottom);
            left = left.max(block.shade.left);
            right = right.max(block.shade.right);
            front = front.max(block.shade.front);
            back = back.max(block.shade.back);
            color += block.color;
        }

        color *= (data.len() as f32).recip();

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
            mesh_type: MeshType::Cube,
        })
    }

    fn can_merge(&self) -> bool {
        self.mesh_type == MeshType::Cube
    }
}

impl VoxelExt for Block {
    fn mesh(
        &self,
        coords: (i32, i32, i32),
        map: &Map<Self>,
        chunk: &Chunk<Self>,
        width: usize,
    ) -> MeshPart {
        match self.mesh_type {
            MeshType::Cube => self.mesh_cube(coords, map, chunk, width),
            MeshType::Cross => self.mesh_cross(coords, map, chunk, width),
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

fn generate_front_side(
    block: &Block,
    map: &Map<Block>,
    chunk: &Chunk<Block>,
    (x, y, z): (i32, i32, i32),
    width: usize,
    indices: &mut Vec<u32>,
    n: &mut u32,
) -> Option<([[f32; 3]; 4], [f32; 4], [[f32; 4]; 4])> {
    let width = width as i32;
    let cw = chunk.width() as i32;
    for dx in 0..width {
        for dy in 0..width {
            let render = if z + width >= cw {
                let (cx, cy, cz) = chunk.position();
                let cz = cz + cw;
                if let Some(chunk) = map.get((cx, cy, cz)) {
                    !chunk
                        .get((x + dx, y + dy, 0))
                        .map(|other| block.solid() && other.solid() || block.transparent() && other.transparent())
                        .unwrap_or(false)
                } else {
                    false
                }
            } else {
                !chunk
                    .get((x + dx, y + dy, z + width))
                    .map(|other| block.solid() && other.solid() || block.transparent() && other.transparent())
                    .unwrap_or(false)
            };
            if render {
                let size = width as f32;
                let x = x as f32;
                let y = y as f32;
                let z = z as f32;
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
    indices: &mut Vec<u32>,
    n: &mut u32,
) -> Option<([[f32; 3]; 4], [f32; 4], [[f32; 4]; 4])> {
    let width = width as i32;
    let cw = chunk.width() as i32;
    for dx in 0..width {
        for dy in 0..width {
            let render = if z - 1 < 0 {
                let (cx, cy, cz) = chunk.position();
                let cz = cz - cw;
                if let Some(chunk) = map.get((cx, cy, cz)) {
                    !chunk
                        .get((x + dx, y + dy, cw - 1))
                        .map(|other| block.solid() && other.solid() || block.transparent() && other.transparent())
                        .unwrap_or(false)
                } else {
                    false
                }
            } else {
                !chunk
                    .get((x + dx, y + dy, z - 1))
                    .map(|other| block.solid() && other.solid() || block.transparent() && other.transparent())
                    .unwrap_or(false)
            };
            if render {
                let size = width as f32;
                let x = x as f32;
                let y = y as f32;
                let z = z as f32;
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
    indices: &mut Vec<u32>,
    n: &mut u32,
) -> Option<([[f32; 3]; 4], [f32; 4], [[f32; 4]; 4])> {
    let width = width as i32;
    let cw = chunk.width() as i32;
    for dy in 0..width {
        for dz in 0..width {
            let render = if x - 1 < 0 {
                let (cx, cy, cz) = chunk.position();
                let cx = cx - cw;
                if let Some(chunk) = map.get((cx, cy, cz)) {
                    !chunk
                        .get((cw - 1, y + dy, z + dz))
                        .map(|other| block.solid() && other.solid() || block.transparent() && other.transparent())
                        .unwrap_or(false)
                } else {
                    false
                }
            } else {
                !chunk
                    .get((x - 1, y + dy, z + dz))
                    .map(|other| block.solid() && other.solid() || block.transparent() && other.transparent())
                    .unwrap_or(false)
            };
            if render {
                let size = width as f32;
                let x = x as f32;
                let y = y as f32;
                let z = z as f32;
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
    indices: &mut Vec<u32>,
    n: &mut u32,
) -> Option<([[f32; 3]; 4], [f32; 4], [[f32; 4]; 4])> {
    let width = width as i32;
    let cw = chunk.width() as i32;
    for dy in 0..width {
        for dz in 0..width {
            let render = if x + width >= cw {
                let (cx, cy, cz) = chunk.position();
                let cx = cx + cw;
                if let Some(chunk) = map.get((cx, cy, cz)) {
                    !chunk
                        .get((0, y + dy, z + dz))
                        .map(|other| block.solid() && other.solid() || block.transparent() && other.transparent())
                        .unwrap_or(false)
                } else {
                    false
                }
            } else {
                !chunk
                    .get((x + width, y + dy, z + dz))
                    .map(|other| block.solid() && other.solid() || block.transparent() && other.transparent())
                    .unwrap_or(false)
            };
            if render {
                let size = width as f32;
                let x = x as f32;
                let y = y as f32;
                let z = z as f32;
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
    indices: &mut Vec<u32>,
    n: &mut u32,
) -> Option<([[f32; 3]; 4], [f32; 4], [[f32; 4]; 4])> {
    let width = width as i32;
    let cw = chunk.width() as i32;
    for dx in 0..width {
        for dz in 0..width {
            let render = if y + width >= cw {
                let (cx, cy, cz) = chunk.position();
                let cy = cy + cw;
                if let Some(chunk) = map.get((cx, cy, cz)) {
                    !chunk
                        .get((x + dx, 0, z + dz))
                        .map(|other| block.solid() && other.solid() || block.transparent() && other.transparent())
                        .unwrap_or(false)
                } else {
                    false
                }
            } else {
                !chunk
                    .get((x + dx, y + width, z + dz))
                    .map(|other| block.solid() && other.solid() || block.transparent() && other.transparent())
                    .unwrap_or(false)
            };
            if render {
                let size = width as f32;
                let x = x as f32;
                let y = y as f32;
                let z = z as f32;
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
    indices: &mut Vec<u32>,
    n: &mut u32,
) -> Option<([[f32; 3]; 4], [f32; 4], [[f32; 4]; 4])> {
    let width = width as i32;
    let cw = chunk.width() as i32;
    for dx in 0..width {
        for dz in 0..width {
            let render = if y - 1 < 0 {
                let (cx, cy, cz) = chunk.position();
                let cy = cy - cw;
                if let Some(chunk) = map.get((cx, cy, cz)) {
                    !chunk
                        .get((x + dx, cw - 1, z + dz))
                        .map(|other| block.solid() && other.solid() || block.transparent() && other.transparent())
                        .unwrap_or(false)
                } else {
                    false
                }
            } else {
                !chunk
                    .get((x + dx, y - 1, z + dz))
                    .map(|other| block.solid() && other.solid() || block.transparent() && other.transparent())
                    .unwrap_or(false)
            };
            if render {
                let size = width as f32;
                let x = x as f32;
                let y = y as f32;
                let z = z as f32;
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
