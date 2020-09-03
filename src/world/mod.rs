use std::{borrow::Cow, collections::HashMap};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use rstar::{PointDistance, RTree, RTreeObject, AABB};

use bevy::ecs::Bundle;

use crate::collections::{
    lod_tree::{Element, ElementMut, Voxel},
    LodTree,
};

pub type ChunkKey = (i32, i32, i32);

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct Chunk<T> {
    position: ChunkKey,
    data: LodTree<T>,
    light: LodTree<f32>,
    has_light: bool,
}

impl<T: Voxel> Chunk<T> {
    pub fn new(size: u32, position: ChunkKey) -> Self {
        let chunk_size = 1 << size;
        let data = LodTree::new(chunk_size);
        let light = LodTree::new(chunk_size);
        Self {
            position,
            data,
            light,
            has_light: false,
        }
    }

    pub fn has_light(&self) -> bool {
        self.has_light
    }

    pub fn set_light(&mut self, light: bool) {
        self.has_light = light;
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

    pub fn lights(&self) -> impl Iterator<Item = Element<'_, f32>> {
        self.light.elements()
    }

    pub fn lights_mut(&mut self) -> impl Iterator<Item = ElementMut<'_, f32>> {
        self.light.elements_mut()
    }

    pub fn insert(&mut self, coords: (i32, i32, i32), voxel: T) {
        self.data.insert(coords, voxel);
    }

    pub fn insert_light(&mut self, coords: (i32, i32, i32), light: f32) {
        self.light.insert(coords, light);
    }

    pub fn get(&self, coords: (i32, i32, i32)) -> Option<Cow<'_, T>> {
        self.data.get(coords)
    }

    pub fn get_mut(&mut self, coords: (i32, i32, i32)) -> Option<&mut T> {
        self.data.get_mut(coords)
    }

    pub fn light(&self, coords: (i32, i32, i32)) -> Option<f32> {
        self.light.get(coords).map(Cow::into_owned)
    }

    pub fn light_mut(&mut self, coords: (i32, i32, i32)) -> Option<&mut f32> {
        self.light.get_mut(coords)
    }

    pub fn contains_key(&self, coords: (i32, i32, i32)) -> bool {
        self.data.contains_key(coords)
    }
}

impl<T: Voxel> RTreeObject for Chunk<T> {
    type Envelope = AABB<[i32; 3]>;

    fn envelope(&self) -> Self::Envelope {
        let w = self.width() as i32;
        let w_2 = w / 2;
        let x0 = self.position.0 * w - w_2;
        let y0 = self.position.1 * w - w_2;
        let z0 = self.position.2 * w - w_2;
        let x1 = self.position.0 * w + w_2 - 1;
        let y1 = self.position.1 * w + w_2 - 1;
        let z1 = self.position.2 * w + w_2 - 1;
        AABB::from_corners([x0, y0, z0], [x1, y1, z1])
    }
}

impl<T: Voxel> PointDistance for Chunk<T> {
    fn distance_2(&self, point: &[i32; 3]) -> i32 {
        let w = self.width() as i32;
        let w_2 = w / 2;
        let x = self.position.0 * w - w_2;
        let y = self.position.1 * w - w_2;
        let z = self.position.2 * w - w_2;
        let dx = x - point[0];
        let dy = y - point[1];
        let dz = z - point[2];
        dx * dx + dy * dy + dz * dz
    }
}

/// The map represents visible chunks.
#[derive(Default, Debug, Clone)]
pub struct Map<T: Voxel> {
    map: RTree<Chunk<T>>,
}

impl<T: Voxel> Map<T> {
    pub fn new() -> Self {
        Self { map: RTree::new() }
    }

    pub fn with_chunks(initial: Vec<Chunk<T>>) -> Self {
        Self {
            map: RTree::bulk_load(initial),
        }
    }

    pub fn get(&self, (x, y, z): (i32, i32, i32)) -> Option<&Chunk<T>> {
        self.map.locate_at_point(&[x, y, z])
    }

    pub fn get_mut(&mut self, (x, y, z): (i32, i32, i32)) -> Option<&mut Chunk<T>> {
        self.map.locate_at_point_mut(&[x, y, z])
    }

    pub fn insert(&mut self, value: Chunk<T>) {
        let w = value.width() as i32;
        let x = value.position.0 * w;
        let y = value.position.1 * w;
        let z = value.position.2 * w;
        self.map.remove_at_point(&[x, y, z]);
        self.map.insert(value);
    }

    pub fn remove(&mut self, (x, y, z): (i32, i32, i32)) -> Option<Chunk<T>> {
        self.map.remove_at_point(&[x, y, z])
    }

    pub fn iter(&self) -> impl Iterator<Item = &'_ Chunk<T>> {
        self.map.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &'_ mut Chunk<T>> {
        self.map.iter_mut()
    }
}

#[derive(Debug, Clone)]
pub enum ChunkUpdate {
    UpdateMesh,
    UpdateLight,
    UpdateLightMap,
    GenerateChunk,
}

#[derive(Default, Debug, Clone)]
pub struct MapUpdates {
    pub updates: HashMap<(i32, i32, i32, usize), ChunkUpdate>,
}

#[derive(Default, Bundle)]
pub struct MapComponents {
    pub map_update: MapUpdates,
}
