use bevy::prelude::*;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub use crate::collections::volumetric_tree::Node as ChunkNode;
use crate::collections::{
    volumetric_tree::{Element, ElementMut},
    VolumetricTree,
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

pub trait Voxel: PartialEq + Clone {
    fn average(data: &[Self]) -> Option<Self>;

    fn shade(&self) -> Shade {
        Shade::default()
    }

    fn color(&self) -> Color {
        Color::rgba(1.0, 1.0, 1.0, 1.0)
    }
}

pub type ChunkKey = (i32, i32, i32);

pub type Lod = usize;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct Chunk<T> {
    position: ChunkKey,
    data: Vec<VolumetricTree<T>>,
}

impl<T: Voxel> Default for Chunk<T> {
    fn default() -> Self {
        Self::new(1, (0, 0, 0))
    }
}

impl<T: Voxel> Chunk<T> {
    pub fn new(size: u32, position: ChunkKey) -> Self {
        let mut data = Vec::new();
        for i in 0..size {
            let chunk_size = 2_usize.pow(size - i);
            data.push(VolumetricTree::new(chunk_size));
        }
        Self { position, data }
    }

    fn update_lod(&mut self, mut coords: (i32, i32, i32)) {
        for lod in 0..self.data.len() - 1 {
            let (mut x, mut y, mut z) = coords;
            x &= !1;
            y &= !1;
            z &= !1;
            let mut array = Vec::new();
            array.extend(self.data[lod].get((x, y, z)).cloned());
            array.extend(self.data[lod].get((x, y, z + 1)).cloned());
            array.extend(self.data[lod].get((x, y + 1, z)).cloned());
            array.extend(self.data[lod].get((x, y + 1, z + 1)).cloned());
            array.extend(self.data[lod].get((x + 1, y, z)).cloned());
            array.extend(self.data[lod].get((x + 1, y, z + 1)).cloned());
            array.extend(self.data[lod].get((x + 1, y + 1, z)).cloned());
            array.extend(self.data[lod].get((x + 1, y + 1, z + 1)).cloned());
            let block = T::average(&array);
            let x = coords.0 / 2;
            let y = coords.1 / 2;
            let z = coords.2 / 2;
            coords = (x, y, z);
            if let Some(block) = block {
                self.data[lod + 1].insert(coords, block);
            } else {
                self.data[lod + 1].remove(coords);
            }
        }
    }

    pub fn position(&self) -> (i32, i32, i32) {
        self.position
    }

    pub fn width(&self, lod: Lod) -> usize {
        self.data[lod].width()
    }

    pub fn iter(&self, lod: Lod) -> impl Iterator<Item = Element<'_, T>> {
        self.data[lod].elements()
    }

    pub fn iter_mut(&mut self, lod: Lod) -> impl Iterator<Item = ElementMut<'_, T>> {
        self.data[lod].elements_mut()
    }

    pub fn insert(&mut self, coords: (i32, i32, i32), voxel: T) {
        self.data[0].insert(coords, voxel);
        self.update_lod(coords);
    }

    pub fn get(&self, coords: (i32, i32, i32)) -> Option<&T> {
        self.data[0].get(coords)
    }

    pub fn get_mut(&mut self, coords: (i32, i32, i32)) -> Option<&mut T> {
        self.data[0].get_mut(coords)
    }

    pub fn get_at(&self, lod: Lod, coords: (i32, i32, i32)) -> Option<&T> {
        self.data[lod].get(coords)
    }

    pub fn get_mut_at(&mut self, lod: Lod, coords: (i32, i32, i32)) -> Option<&mut T> {
        self.data[lod].get_mut(coords)
    }
}
