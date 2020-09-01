use std::borrow::Cow;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::collections::{
    lod_tree::{Element, ElementMut, Voxel},
    LodTree,
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

pub type ChunkKey = (i32, i32, i32);

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct Chunk<T> {
    position: ChunkKey,
    data: LodTree<T>,
}

impl<T: Voxel> Default for Chunk<T> {
    fn default() -> Self {
        Self::new(1, (0, 0, 0))
    }
}

impl<T: Voxel> Chunk<T> {
    pub fn new(size: u32, position: ChunkKey) -> Self {
        let chunk_size = 1 << size;
        let data = LodTree::new(chunk_size);
        Self { position, data }
    }

    pub fn set_lod(&mut self, lod: usize) {
        self.data.set_lod(lod);
    }

    pub fn lod(&self) -> usize {
        self.data.lod()
    }

    pub fn merge(&mut self) {
        self.data.merge();
    }

    pub fn position(&self) -> (i32, i32, i32) {
        self.position
    }

    pub fn width(&self) -> usize {
        self.data.width()
    }

    pub fn iter(&self) -> impl Iterator<Item = Element<'_, T>> {
        self.data.elements()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = ElementMut<'_, T>> {
        self.data.elements_mut()
    }

    pub fn insert(&mut self, coords: (i32, i32, i32), voxel: T) {
        self.data.insert(coords, voxel);
    }

    pub fn get(&self, coords: (i32, i32, i32)) -> Option<Cow<'_, T>> {
        self.data.get(coords)
    }

    pub fn get_mut(&mut self, coords: (i32, i32, i32)) -> Option<&mut T> {
        self.data.get_mut(coords)
    }

    pub fn contains_key(&self, coords: (i32, i32, i32)) -> bool {
        self.data.contains_key(coords)
    }
}
