use std::{borrow::Cow, collections::HashMap};
#[cfg(feature = "savedata")]
use std::{
    fs::{self, File},
    io::Read,
    path::Path,
};

#[cfg(feature = "serde")]
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use rstar::{PointDistance, RTree, RTreeObject, AABB};

use bevy::prelude::*;
use bevy::ecs::Bundle;

#[cfg(feature = "savedata")]
use crate::collections::RleTree;

use crate::collections::{
    lod_tree::{Element, ElementMut, Voxel},
    LodTree,
};

#[cfg(feature = "savedata")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SaveData<T> {
    position: (i32, i32, i32),
    data: RleTree<T>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Chunk<T> {
    position: (i32, i32, i32),
    data: LodTree<T>,
    light: LodTree<f32>,
    has_light: bool,
    entity: Option<Entity>,
}

impl<T: Voxel> Chunk<T> {
    pub fn new(size: u32, position: (i32, i32, i32)) -> Self {
        let chunk_size = 1 << size;
        let data = LodTree::new(chunk_size);
        let light = LodTree::new(chunk_size);
        Self {
            position,
            data,
            light,
            has_light: false,
            entity: None,
        }
    }

    pub fn entity(&self) -> Option<Entity> {
        self.entity
    }
    
    pub fn set_entity(&mut self, e: Entity) {
        self.entity = Some(e);
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

#[cfg(feature = "savedata")]
impl<T: Voxel + Serialize + DeserializeOwned> Chunk<T> {
    pub fn load<R: Read>(reader: R) -> bincode::Result<Self> {
        Ok(Self::from(bincode::deserialize_from::<_, SaveData<T>>(
            reader,
        )?))
    }

    pub fn serializable(&self) -> SaveData<T> {
        SaveData {
            position: self.position,
            data: RleTree::with_tree(&self.data),
        }
    }
}

#[cfg(feature = "savedata")]
impl<T: Voxel> From<SaveData<T>> for Chunk<T> {
    fn from(save: SaveData<T>) -> Self {
        let data = LodTree::from(save.data);
        let width = data.width();
        Self {
            position: save.position,
            data,
            light: LodTree::new(width),
            has_light: false,
            entity: None,
        }
    }
}

impl<T: Voxel> RTreeObject for Chunk<T> {
    type Envelope = AABB<[i32; 3]>;

    fn envelope(&self) -> Self::Envelope {
        let w = self.width() as i32;
        let x0 = self.position.0;
        let y0 = self.position.1;
        let z0 = self.position.2;
        let x1 = self.position.0 + w - 1;
        let y1 = self.position.1 + w - 1;
        let z1 = self.position.2 + w - 1;
        AABB::from_corners([x0, y0, z0], [x1, y1, z1])
    }
}

impl<T: Voxel> PointDistance for Chunk<T> {
    fn distance_2(&self, point: &[i32; 3]) -> i32 {
        self.envelope().distance_2(point)
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
        let (x, y, z) = value.position;
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

#[cfg(feature = "savedata")]
impl<T: Voxel + Serialize + DeserializeOwned> Map<T> {
    pub fn save<P: AsRef<Path>>(&self, save_directory: P) -> bincode::Result<()> {
        let save_directory = save_directory.as_ref();
        fs::create_dir_all(save_directory)?;
        for chunk in &self.map {
            let mut path = save_directory.to_path_buf();
            let (x, y, z) = chunk.position();
            path.push(format!("chunk.{}.{}.{}.gz", x, y, z));
            let file = File::create(path)?;
            let savedata = chunk.serializable();
            bincode::serialize_into(
                flate2::write::GzEncoder::new(file, flate2::Compression::default()),
                &savedata,
            )?;
        }
        Ok(())
    }

    pub fn load<P: AsRef<Path>>(save_directory: P) -> bincode::Result<Self> {
        let save_directory = save_directory.as_ref();
        let mut chunks = Vec::new();
        for entry in save_directory.read_dir()? {
            let file = flate2::read::GzDecoder::new(File::open(entry?.path())?);
            let chunk = Chunk::load(file)?;
            chunks.push(chunk);
        }
        Ok(Self::with_chunks(chunks))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ChunkUpdate {
    GenerateChunk,
    UpdateLightMap,
    UpdateLight,
    UpdateMesh,
}

#[derive(Default, Debug, Clone)]
pub struct MapUpdates {
    pub updates: HashMap<(i32, i32, i32), ChunkUpdate>,
}

#[derive(Default, Bundle)]
pub struct MapComponents {
    pub map_update: MapUpdates,
}
