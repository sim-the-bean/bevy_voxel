#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use std::{borrow::Cow, collections::HashMap, mem};

use int_traits::IntTraits;

fn depth_index(x: i32, y: i32, z: i32, depth: usize) -> usize {
    let width_2 = 1 << (depth - 1);

    let mut idx = 0;

    let mut x = x + width_2;
    let mut y = y + width_2;
    let mut z = z + width_2;

    for i in 0..depth {
        let bx = x & 1;
        let by = y & 1;
        let bz = z & 1;

        x >>= 1;
        y >>= 1;
        z >>= 1;

        idx |= (bx as usize | (by as usize) << 1 | (bz as usize) << 2) << 3 * i;
    }

    idx
}

fn array_index(idx: usize, depth: usize) -> (i32, i32, i32) {
    let width_2 = 1 << (depth - 1);

    let mut x = 0;
    let mut y = 0;
    let mut z = 0;

    for i in (0..depth).rev() {
        let b = idx >> 3 * i;
        let bx = b as i32 & 1;
        let by = (b as i32 >> 1) & 1;
        let bz = (b as i32 >> 2) & 1;

        x <<= 1;
        y <<= 1;
        z <<= 1;

        x |= bx;
        y |= by;
        z |= bz;
    }
    (x - width_2, y - width_2, z - width_2)
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Node<T> {
    Ref(usize),
    Value(Option<T>, usize),
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
pub struct LodTree<T> {
    depth: usize,
    len: usize,
    array: Vec<Node<T>>,
}

impl<T: Clone + PartialEq> LodTree<T> {
    pub fn new(width: usize) -> Self {
        let mut array = Vec::with_capacity(width.pow(3));
        for _ in 0..width.pow(3) {
            array.push(Node::Value(None, 1));
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
            *elem = Node::Value(None, 1);
        }
    }

    pub fn merge(&mut self) {
        for d in 1..=self.depth {
            let skip = 8_usize.pow(d as u32 - 1);

            let mut merges = Vec::new();
            let mut pivot = None;
            let iter = self.array.iter().enumerate().filter_map(|(i, elem)| {
                if i % skip == 0 {
                    Some((i, elem))
                } else {
                    None
                }
            });

            for (i, node) in iter {
                if i & 7 == 0 {
                    let mut i = i;
                    let mut node = node;
                    pivot = loop {
                        match node {
                            Node::Ref(idx) => {
                                node = &self.array[*idx];
                                i = *idx;
                            }
                            Node::Value(value, _) => break Some((value, i)),
                        }
                    };
                    continue;
                }
                if let Some((pivot, pivot_idx)) = pivot {
                    let mut i = i;
                    let mut node = node;
                    let elem = loop {
                        match node {
                            Node::Ref(idx) => {
                                node = &self.array[*idx];
                                i = *idx;
                            }
                            Node::Value(value, _) => break value,
                        }
                    };
                    if elem == pivot {
                        merges.push((i, pivot_idx));
                    }
                }
            }

            let mut pivot_map = HashMap::<_, usize>::new();
            for (idx, pivot_idx) in merges {
                *pivot_map.entry(pivot_idx).or_default() += 1;
                self.array[idx] = Node::Ref(pivot_idx);
            }

            for (pivot_idx, count) in pivot_map {
                debug_assert!(count < 8, "count is not < 8: {}", count);
                if count == 7 {
                    match &mut self.array[pivot_idx] {
                        Node::Value(_, width) => *width *= 2,
                        _ => unreachable!(),
                    }
                }
            }
        }
    }

    pub fn insert(&mut self, (x, y, z): (i32, i32, i32), value: T) -> Option<Cow<'_, T>> {
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
        let mut result = Node::Value(Some(value), 1);
        mem::swap(&mut self.array[idx], &mut result);

        let mut result_ref;
        let mut depth = 0;
        match result {
            Node::Ref(idx) => {
                depth += 1;
                result_ref = &mut self.array[idx] as *mut _;
            }
            Node::Value(value, _) => {
                return value.map(Cow::Owned);
            }
        }

        loop {
            match unsafe { &mut *result_ref } {
                Node::Ref(idx) => {
                    depth += 1;
                    result_ref = &mut self.array[*idx] as *mut _;
                }
                Node::Value(value, width) => {
                    *width >>= depth;
                    return value.as_ref().map(Cow::Borrowed);
                }
            }
        }
    }

    pub fn remove(&mut self, (x, y, z): (i32, i32, i32)) -> Option<Cow<'_, T>> {
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
        let mut result = Node::Value(None, 1);
        mem::swap(&mut self.array[idx], &mut result);

        let mut result_ref;
        let mut depth = 0;
        match result {
            Node::Ref(idx) => {
                depth += 1;
                result_ref = &mut self.array[idx] as *mut _;
            }
            Node::Value(value, _) => {
                return value.map(Cow::Owned);
            }
        }

        loop {
            match unsafe { &mut *result_ref } {
                Node::Ref(idx) => {
                    depth += 1;
                    result_ref = &mut self.array[*idx] as *mut _;
                }
                Node::Value(value, width) => {
                    *width >>= depth;
                    return value.as_ref().map(Cow::Borrowed);
                }
            }
        }
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
        let mut result_ref = &self.array[idx];

        loop {
            match result_ref {
                Node::Ref(idx) => {
                    result_ref = &self.array[*idx];
                }
                Node::Value(value, _) => return value.as_ref(),
            }
        }
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
        let mut result_ref = &mut self.array[idx] as *mut _;

        loop {
            match unsafe { &mut *result_ref } {
                Node::Ref(idx) => {
                    result_ref = &mut self.array[*idx] as *mut _;
                }
                Node::Value(value, _) => return value.as_mut(),
            }
        }
    }

    pub fn contains_key(&self, coords: (i32, i32, i32)) -> bool {
        self.get(coords).is_some()
    }

    pub fn elements(&self) -> impl Iterator<Item = Element<'_, T>> {
        let depth = self.depth;
        let array = &self.array[..];
        array
            .iter()
            .enumerate()
            .flat_map(move |(mut i, mut value)| {
                let (idx, value, width) = loop {
                    match value {
                        Node::Ref(idx) => {
                            value = &array[*idx];
                            i = *idx;
                        }
                        Node::Value(value, width) => break (i, value, *width),
                    }
                };
                value.as_ref().map(|value| {
                    let (x, y, z) = array_index(idx, depth);
                    Element {
                        x,
                        y,
                        z,
                        width,
                        value,
                    }
                })
            })
    }

    pub fn elements_mut(&mut self) -> impl Iterator<Item = ElementMut<'_, T>> {
        let depth = self.depth;
        let array = &mut self.array as *mut Vec<_>;
        self.array
            .iter_mut()
            .enumerate()
            .flat_map(move |(mut i, mut value)| {
                let (idx, value, width) = loop {
                    match value {
                        Node::Ref(idx) => {
                            let array = unsafe { &mut *array };
                            value = &mut array[*idx];
                            i = *idx;
                        }
                        Node::Value(value, width) => break (i, value, *width),
                    }
                };
                value.as_mut().map(|value| {
                    let (x, y, z) = array_index(idx, depth);
                    ElementMut {
                        x,
                        y,
                        z,
                        width,
                        value,
                    }
                })
            })
    }
}

impl<T: PartialEq> LodTree<T> {
    pub fn position(&self, value: &T) -> Option<(i32, i32, i32)> {
        for (mut i, mut elem) in self.array.iter().enumerate() {
            let elem = loop {
                match elem {
                    Node::Ref(idx) => {
                        elem = &self.array[*idx];
                        i = *idx;
                    }
                    Node::Value(value, _) => break value,
                }
            };
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
    pub fn index() {
        let depth = 2;
        let idx0 = 3;
        let (x, y, z) = array_index(idx0, depth);
        let idx1 = depth_index(x, y, z, depth);
        assert_eq!(idx0, idx1);
    }

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

        assert!(vt
            .elements()
            .map(|elem| ((elem.x, elem.y, elem.z), *elem.value, elem.width))
            .all(|elem| {
                [
                    ((-2, -2, -2), -2, 1),
                    ((-1, -1, -1), -1, 1),
                    ((0, 0, 0), 2, 1),
                ]
                .contains(&elem)
            }));
    }

    #[test]
    fn diagonal() {
        let mut vt = LodTree::<i32>::new(4);
        vt.insert((-2, -2, -2), -2);
        vt.insert((-1, -1, -1), -1);
        vt.insert((0, 0, 0), 0);
        vt.insert((1, 1, 1), 1);

        assert!(vt
            .elements()
            .map(|elem| ((elem.x, elem.y, elem.z), *elem.value, elem.width))
            .all(|elem| {
                [
                    ((-2, -2, -2), -2, 1),
                    ((-1, -1, -1), -1, 1),
                    ((0, 0, 0), 0, 1),
                    ((1, 1, 1), 1, 1),
                ]
                .contains(&elem)
            }));
    }

    #[test]
    pub fn small() {
        let mut vt = LodTree::<i32>::new(4);
        vt.insert((-1, -1, -1), 0);
        vt.insert((-1, -1, -2), 0);
        vt.insert((-1, -2, -1), 0);
        vt.insert((-1, -2, -2), 0);
        vt.insert((-2, -1, -1), 0);
        vt.insert((-2, -1, -2), 0);
        vt.insert((-2, -2, -1), 0);
        vt.insert((-2, -2, -2), 0);

        vt.merge();

        println!("{:?}", vt);
    }

    #[test]
    pub fn merge() {
        let mut vt = LodTree::<i32>::new(8);
        vt.insert((2, 2, 2), 0);
        vt.insert((2, 2, 3), 0);
        vt.insert((2, 3, 2), 0);
        vt.insert((2, 3, 3), 0);
        vt.insert((3, 2, 2), 0);
        vt.insert((3, 2, 3), 0);
        vt.insert((3, 3, 2), 0);
        vt.insert((3, 3, 3), 0);

        vt.merge();

        assert_eq!(vt.position(&0), Some((2, 2, 2)));

        assert_eq!(vt.get((2, 2, 2)), Some(&0));
        assert_eq!(vt.get((2, 2, 3)), Some(&0));
        assert_eq!(vt.get((2, 3, 2)), Some(&0));
        assert_eq!(vt.get((2, 3, 3)), Some(&0));
        assert_eq!(vt.get((3, 2, 2)), Some(&0));
        assert_eq!(vt.get((3, 2, 3)), Some(&0));
        assert_eq!(vt.get((3, 3, 2)), Some(&0));
        assert_eq!(vt.get((3, 3, 3)), Some(&0));

        let a = vt.get((2, 2, 2)).unwrap() as *const _;
        let b = vt.get((2, 2, 3)).unwrap() as *const _;
        let c = vt.get((2, 3, 2)).unwrap() as *const _;
        let d = vt.get((2, 3, 3)).unwrap() as *const _;
        let e = vt.get((3, 2, 2)).unwrap() as *const _;
        let f = vt.get((3, 2, 3)).unwrap() as *const _;
        let g = vt.get((3, 3, 2)).unwrap() as *const _;
        let h = vt.get((3, 3, 3)).unwrap() as *const _;

        assert_eq!(a, b);
        assert_eq!(a, c);
        assert_eq!(a, d);
        assert_eq!(a, e);
        assert_eq!(a, f);
        assert_eq!(a, g);
        assert_eq!(a, h);
    }
}
