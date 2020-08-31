#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use std::mem;

use int_traits::IntTraits;

fn sp_index(x: i32, y: i32, z: i32) -> usize {
    let x = x as usize;
    let y = y as usize;
    let z = z as usize;
    x | y << 1 | z << 2
}

fn dir_index(idx: usize) -> (i32, i32, i32) {
    let x = idx & 1;
    let y = (idx >> 1) & 1;
    let z = (idx >> 2) & 1;
    (x as i32, y as i32, z as i32)
}

fn depth_index(x: i32, y: i32, z: i32, depth: usize) -> usize {
    let width_2 = 1 << (depth - 1);
    
    let mut idx = 0;

    let mut x = x + width_2;
    let mut y = y + width_2;
    let mut z = z + width_2;

    for _ in 0..depth {
        idx <<= 3;
        let bx = x & 1;
        let by = y & 1;
        let bz = z & 1;

        x >>= 1;
        y >>= 1;
        z >>= 1;

        idx |= bx as usize | (by as usize) << 1 | (bz as usize) << 2;
    }
    
    idx
}

fn array_index(mut idx: usize, depth: usize) -> (i32, i32, i32) {
    let width_2 = 1 << (depth - 1);
    
    let mut x = 0;
    let mut y = 0;
    let mut z = 0;

    for i in 0..depth {
        let bx = idx as i32 & 1;
        let by = (idx as i32 >> 1) & 1;
        let bz = (idx as i32 >> 2) & 1;

        x <<= 1;
        y <<= 1;
        z <<= 1;
        
        x = x | bx;
        y = y | by;
        z = z | bz;
                
        idx >>= 3;
    }
    (x - width_2, y - width_2, z - width_2)
}

fn array_index_from((ax, ay, az): (i32, i32, i32), mut idx: usize, depth: usize) -> (i32, i32, i32) {
    let mut x = 0;
    let mut y = 0;
    let mut z = 0;

    for i in 0..depth {
        let bx = idx as i32 & 1;
        let by = (idx as i32 >> 1) & 1;
        let bz = (idx as i32 >> 2) & 1;

        x <<= 1;
        y <<= 1;
        z <<= 1;
        
        x = x | bx;
        y = y | by;
        z = z | bz;
                
        idx >>= 3;
    }
    (x + ax, y + ay, z + az)
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
pub struct LodTree<T> {
    depth: usize,
    len: usize,
    array: Vec<Option<T>>,
}

impl<T> LodTree<T> {
    pub fn new(width: usize) -> Self {
        let mut array = Vec::with_capacity(width.pow(3));
        for _ in 0..width.pow(3) {
            array.push(None);
        }
        Self {
            depth: width.log2(),
            len: 0,
            array,
        }
    }

    pub fn capacity(&self) -> usize {
        self.width() * self.width() * self.width()
    }

    pub fn width(&self) -> usize {
        1 << self.depth
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn clear(&mut self) {
        for elem in &mut self.array {
            *elem = None;
        }
    }

    pub fn insert(&mut self, (x, y, z): (i32, i32, i32), value: T) -> Option<T> {
        if x >= self.width() as i32 / 2
            || x < self.width() as i32 / -2
            || y >= self.width() as i32 / 2
            || y < self.width() as i32 / -2
            || z >= self.width() as i32 / 2
            || z < self.width() as i32 / -2
        {
            return None;
        }
        let idx = depth_index(x, y, z, self.depth);
        let mut result = Some(value);
        mem::swap(&mut self.array[idx], &mut result);
        if result.is_none() {
            self.len += 1;
        }
        result
    }

    pub fn remove(&mut self, (x, y, z): (i32, i32, i32)) -> Option<T> {
        if x >= self.width() as i32 / 2
            || x < self.width() as i32 / -2
            || y >= self.width() as i32 / 2
            || y < self.width() as i32 / -2
            || z >= self.width() as i32 / 2
            || z < self.width() as i32 / -2
        {
            return None;
        }
        let idx = depth_index(x, y, z, self.depth);
        let mut result = None;
        mem::swap(&mut self.array[idx], &mut result);
        if result.is_some() {
            self.len -= 1;
        }
        result
    }

    pub fn get(&self, (x, y, z): (i32, i32, i32)) -> Option<&T> {
        if x >= self.width() as i32 / 2
            || x < self.width() as i32 / -2
            || y >= self.width() as i32 / 2
            || y < self.width() as i32 / -2
            || z >= self.width() as i32 / 2
            || z < self.width() as i32 / -2
        {
            return None;
        }
        let idx = depth_index(x, y, z, self.depth);
        self.array[idx].as_ref()
    }

    pub fn get_mut(&mut self, (x, y, z): (i32, i32, i32)) -> Option<&mut T> {
        if x >= self.width() as i32 / 2
            || x < self.width() as i32 / -2
            || y >= self.width() as i32 / 2
            || y < self.width() as i32 / -2
            || z >= self.width() as i32 / 2
            || z < self.width() as i32 / -2
        {
            return None;
        }
        let idx = depth_index(x, y, z, self.depth);
        self.array[idx].as_mut()
    }

    pub fn contains_key(&self, coords: (i32, i32, i32)) -> bool {
        self.get(coords).is_some()
    }

    pub fn elements(&self) -> impl Iterator<Item = Element<'_, T>> {
        let depth = self.depth;
        self.array.iter().enumerate().flat_map(move |(i, value)| {
            value.as_ref().map(|value| {
                let (x, y, z) = array_index(i, depth);
                Element {
                    x,
                    y,
                    z,
                    width: 1,
                    value,
                }
            })
        })
    }

    pub fn elements_mut(&mut self) -> impl Iterator<Item = ElementMut<'_, T>> {
        let depth = self.depth;
        self.array.iter_mut().enumerate().flat_map(move |(i, value)| {
            value.as_mut().map(|value| {
                let (x, y, z) = array_index(i, depth);
                ElementMut {
                    x,
                    y,
                    z,
                    width: 1,
                    value,
                }
            })
        })
    }
}

impl<T: PartialEq> LodTree<T> {
    pub fn position(&self, value: &T) -> Option<(i32, i32, i32)> {
        for (i, elem) in self.array.iter().enumerate() {
            if elem.as_ref() == Some(value) {
                return Some(array_index(i, self.depth));
            }
        }
        None
    }

    pub fn contains(&self, value: &T) -> bool {
        self.position(value).is_some()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Element<'a, T> {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub width: usize,
    pub value: &'a T,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ElementMut<'a, T> {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub width: usize,
    pub value: &'a mut T,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn insert() {
        let mut vt = LodTree::<i32>::new(8);
        vt.insert((-4, -4, -4), -4);
        vt.insert((-3, -3, -3), -3);
        vt.insert((-2, -2, -2), -2);
        vt.insert((-1, -1, -1), -1);
        vt.insert((0, 0, 0), 0);
        vt.insert((1, 1, 1), 1);
        vt.insert((2, 2, 2), 2);
        vt.insert((3, 3, 3), 3);

        assert_eq!(vt.position(&-1), Some((-1, -1, -1)));
        assert_eq!(vt.position(&-2), Some((-2, -2, -2)));
        assert_eq!(vt.position(&-3), Some((-3, -3, -3)));
        assert_eq!(vt.position(&-4), Some((-4, -4, -4)));
        assert_eq!(vt.position(&0), Some((0, 0, 0)));
        assert_eq!(vt.position(&1), Some((1, 1, 1)));
        assert_eq!(vt.position(&2), Some((2, 2, 2)));
        assert_eq!(vt.position(&3), Some((3, 3, 3)));

        assert_eq!(vt.get((-1, -1, -1)), Some(&-1));
        assert_eq!(vt.get((-2, -2, -2)), Some(&-2));
        assert_eq!(vt.get((-3, -3, -3)), Some(&-3));
        assert_eq!(vt.get((-4, -4, -4)), Some(&-4));
        assert_eq!(vt.get((0, 0, 0)), Some(&0));
        assert_eq!(vt.get((1, 1, 1)), Some(&1));
        assert_eq!(vt.get((2, 2, 2)), Some(&2));
        assert_eq!(vt.get((3, 3, 3)), Some(&3));
    }

    #[test]
    pub fn remove() {
        let mut vt = LodTree::<i32>::new(8);
        vt.insert((-4, -4, -4), -4);
        vt.insert((-3, -3, -3), -3);
        vt.insert((-2, -2, -2), -2);
        vt.insert((-1, -1, -1), -1);
        vt.insert((0, 0, 0), 0);
        vt.insert((1, 1, 1), 1);
        vt.insert((2, 2, 2), 2);
        vt.insert((3, 3, 3), 3);

        vt.remove((0, 0, 0));

        assert_eq!(vt.position(&-1), Some((-1, -1, -1)));
        assert_eq!(vt.position(&-2), Some((-2, -2, -2)));
        assert_eq!(vt.position(&-3), Some((-3, -3, -3)));
        assert_eq!(vt.position(&-4), Some((-4, -4, -4)));
        assert_eq!(vt.position(&0), None);
        assert_eq!(vt.position(&1), Some((1, 1, 1)));
        assert_eq!(vt.position(&2), Some((2, 2, 2)));
        assert_eq!(vt.position(&3), Some((3, 3, 3)));

        assert_eq!(vt.get((-1, -1, -1)), Some(&-1));
        assert_eq!(vt.get((-2, -2, -2)), Some(&-2));
        assert_eq!(vt.get((-3, -3, -3)), Some(&-3));
        assert_eq!(vt.get((-4, -4, -4)), Some(&-4));
        assert_eq!(vt.get((0, 0, 0)), None);
        assert_eq!(vt.get((1, 1, 1)), Some(&1));
        assert_eq!(vt.get((2, 2, 2)), Some(&2));
        assert_eq!(vt.get((3, 3, 3)), Some(&3));
    }

    #[test]
    #[cfg(feature = "serde")]
    fn serde() {
        let mut vt = LodTree::<i32>::new(8);
        vt.insert((-4, -4, -4), -4);
        vt.insert((-3, -3, -3), -3);
        vt.insert((-2, -2, -2), -2);
        vt.insert((-1, -1, -1), -1);
        vt.insert((0, 0, 0), 0);
        vt.insert((1, 1, 1), 1);
        vt.insert((2, 2, 2), 2);
        vt.insert((2, 2, 3), 2);
        vt.insert((2, 3, 2), 2);
        vt.insert((2, 3, 3), 2);
        vt.insert((3, 2, 2), 2);
        vt.insert((3, 2, 3), 2);
        vt.insert((3, 3, 2), 2);
        vt.insert((3, 3, 3), 2);

        let serialized = serde_json::to_string(&vt).unwrap();

        let deserialized: LodTree<i32> = serde_json::from_str(&serialized).unwrap();

        assert_eq!(vt, deserialized);
    }

    #[test]
    fn elements() {
        let mut vt = LodTree::<i32>::new(4);
        vt.insert((-2, -2, -2), -2);
        vt.insert((-1, -1, -1), -1);
        vt.insert((0, 0, 0), 2);

        assert!(
            vt.elements()
                .map(|elem| ((elem.x, elem.y, elem.z), *elem.value, elem.width))
                .all(|elem| {
                    [
                        ((-2, -2, -2), -2, 1),
                        ((-1, -1, -1), -1, 1),
                        ((0, 0, 0), 2, 1),
                    ].contains(&elem)
                })
        );
    }

    #[test]
    fn diagonal() {
        let mut vt = LodTree::<i32>::new(4);
        vt.insert((-2, -2, -2), -2);
        vt.insert((-1, -1, -1), -1);
        vt.insert((0, 0, 0), 0);
        vt.insert((1, 1, 1), 1);

        assert!(
            vt.elements()
                .map(|elem| ((elem.x, elem.y, elem.z), *elem.value, elem.width))
                .all(|elem| {
                    [
                        ((-2, -2, -2), -2, 1),
                        ((-1, -1, -1), -1, 1),
                        ((0, 0, 0), 0, 1),
                        ((1, 1, 1), 1, 1),
                    ].contains(&elem)
                })
        );
    }
}
