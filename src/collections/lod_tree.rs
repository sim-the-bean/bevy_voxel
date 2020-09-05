use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    mem,
};

use int_traits::IntTraits;

#[cfg(feature = "savedata")]
use crate::{collections::RleTree, serialize::SerDePartialEq};

fn depth_index(mut x: i32, mut y: i32, mut z: i32, depth: usize) -> usize {
    let mut idx = 0;

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
    (x, y, z)
}

#[cfg(feature = "savedata")]
pub trait Voxel: SerDePartialEq<Self> + PartialEq + Clone + Send + Sync + 'static {
    fn average(data: &[Self]) -> Option<Self>;
    fn can_merge(&self) -> bool;
}

#[cfg(not(feature = "savedata"))]
pub trait Voxel: PartialEq + Clone + Send + Sync + 'static {
    fn average(data: &[Self]) -> Option<Self>;
    fn can_merge(&self) -> bool;
}

impl Voxel for f32 {
    fn average(data: &[Self]) -> Option<Self> {
        if data.is_empty() {
            None
        } else {
            Some(data.iter().copied().sum::<f32>() / data.len() as f32)
        }
    }

    fn can_merge(&self) -> bool {
        false
    }
}

impl Voxel for i32 {
    fn average(data: &[Self]) -> Option<Self> {
        if data.is_empty() {
            None
        } else {
            Some(data.iter().copied().sum::<i32>() / data.len() as i32)
        }
    }

    fn can_merge(&self) -> bool {
        true
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Node<T> {
    Ref(usize),
    Value(Option<T>, usize),
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
pub struct LodTree<T> {
    lod: usize,
    depth: usize,
    len: usize,
    array: Vec<Node<T>>,
}

impl<T: Voxel> LodTree<T> {
    pub fn new(width: usize) -> Self {
        let mut array = Vec::with_capacity(width.pow(3));
        for _ in 0..width.pow(3) {
            array.push(Node::Value(None, 1));
        }
        Self {
            lod: 0,
            depth: width.log2(),
            len: 0,
            array,
        }
    }

    pub fn set_lod(&mut self, lod: usize) {
        self.lod = lod;
    }

    pub fn lod(&self) -> usize {
        self.lod
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

            let mut merges = HashMap::<_, Vec<_>>::new();
            let mut pivot = None;
            let iter = self
                .array
                .iter()
                .enumerate()
                .filter_map(
                    |(i, elem)| {
                        if i % skip == 0 {
                            Some((i, elem))
                        } else {
                            None
                        }
                    },
                )
                .enumerate();

            for (count, (i, node)) in iter {
                if count & 7 == 0 {
                    let mut i = i;
                    let mut node = node;
                    pivot = loop {
                        match node {
                            Node::Ref(idx) => {
                                node = &self.array[*idx];
                                i = *idx;
                            }
                            Node::Value(value, width) => break Some((value, width, i)),
                        }
                    };
                    continue;
                }
                if let Some((pivot, pivot_width, pivot_idx)) = pivot {
                    let mut i = i;
                    let mut node = node;
                    let (elem, width) = loop {
                        match node {
                            Node::Ref(idx) => {
                                node = &self.array[*idx];
                                i = *idx;
                            }
                            Node::Value(value, width) => break (value, width),
                        }
                    };
                    if elem.as_ref().map(|v| v.can_merge()).unwrap_or(true) && elem == pivot && width == pivot_width {
                        merges.entry(pivot_idx).or_default().push(i);
                    }
                }
            }

            for (pivot_idx, idxs) in merges {
                debug_assert!(idxs.len() < 8, "idxs.len() is not < 8: {}", idxs.len());
                if idxs.len() == 7 {
                    for idx in idxs {
                        self.array[idx] = Node::Ref(pivot_idx);
                    }
                    match &mut self.array[pivot_idx] {
                        Node::Value(_, width) => *width *= 2,
                        _ => unreachable!(),
                    }
                }
            }
        }
    }

    pub fn insert(&mut self, (x, y, z): (i32, i32, i32), value: T) -> Option<Cow<'_, T>> {
        if x >= self.width() as i32
            || x < 0
            || y >= self.width() as i32
            || y < 0
            || z >= self.width() as i32
            || z < 0
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
        if x >= self.width() as i32
            || x < 0
            || y >= self.width() as i32
            || y < 0
            || z >= self.width() as i32
            || z < 0
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

    pub fn get_mut(&mut self, (x, y, z): (i32, i32, i32)) -> Option<&mut T> {
        if x >= self.width() as i32
            || x < 0
            || y >= self.width() as i32
            || y < 0
            || z >= self.width() as i32
            || z < 0
        {
            return None;
        }
        let idx = depth_index(x, y, z, self.depth);
        let result_ref = &mut self.array[idx] as *mut _;
        let mut result = &mut self.array[idx] as *mut _;

        let value = loop {
            match unsafe { &mut *result } {
                Node::Ref(idx) => {
                    result = &mut self.array[*idx] as &mut _;
                }
                Node::Value(value, width) => {
                    *width = 1;
                    break value.clone();
                }
            }
        };
        unsafe {
            *result_ref = Node::Value(value, 1);
            match &mut *result_ref {
                Node::Value(value, _) => value.as_mut(),
                _ => unreachable!(),
            }
        }
    }

    pub fn get(&self, (x, y, z): (i32, i32, i32)) -> Option<Cow<'_, T>> {
        if x >= self.width() as i32
            || x < 0
            || y >= self.width() as i32
            || y < 0
            || z >= self.width() as i32
            || z < 0
        {
            return None;
        }
        
        if self.lod == 0 {
            self.get_impl((x, y, z)).map(Cow::Borrowed)
        } else {
            let width = 1 << self.lod;
            let mask = width - 1;
            let x = x & !mask;
            let y = y & !mask;
            let z = z & !mask;
            let start = depth_index(x, y, z, self.depth);
            let end = start + width.pow(3) as usize;
            // TODO: optimize this
            let array = self.array[start..end]
                .iter()
                .flat_map(|mut value| loop {
                    match value {
                        Node::Ref(idx) => {
                            value = &self.array[*idx];
                        }
                        Node::Value(value, _) => return value.clone(),
                    }
                })
                .collect::<Vec<_>>();
            T::average(&array).map(Cow::Owned)
        }
    }

    fn get_impl(&self, (x, y, z): (i32, i32, i32)) -> Option<&T> {
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

    pub fn contains_key(&self, coords: (i32, i32, i32)) -> bool {
        self.get_impl(coords).is_some()
    }

    pub fn opt_elements(&self) -> impl Iterator<Item = OptElement<'_, T>> {
        let depth = self.depth;
        let mut set = HashSet::new();
        self.array
            .iter()
            .enumerate()
            .flat_map(move |(mut i, mut node)| {
                let (idx, value, width) = loop {
                    match node {
                        Node::Ref(idx) => {
                            node = &self.array[*idx];
                            i = *idx;
                        }
                        Node::Value(value, width) => break (i, value, *width),
                    }
                };
                if set.contains(&idx) {
                    return None;
                }
                set.insert(idx);
                let (x, y, z) = array_index(idx, depth);
                Some(OptElement {
                    x,
                    y,
                    z,
                    width,
                    value,
                })
            })
    }

    pub fn elements(&self) -> impl Iterator<Item = Element<'_, T>> {
        let depth = self.depth;
        let mut set = HashSet::new();
        let width = 1_usize << self.lod;
        let volume = width.pow(3);
        self.array
            .chunks(volume)
            .map(|slice| slice.iter().enumerate())
            .enumerate()
            .flat_map(move |(big_i, node)| {
                let mut result = Element {
                    x: 0,
                    y: 0,
                    z: 0,
                    width: 0,
                    value: unsafe { mem::MaybeUninit::uninit().assume_init() },
                };
                let array = node
                    .flat_map(|(small_i, mut value)| {
                        let mut i = big_i * volume + small_i;
                        let (idx, value, width) = loop {
                            match value {
                                Node::Ref(idx) => {
                                    value = &self.array[*idx];
                                    i = *idx;
                                }
                                Node::Value(value, width) => break (i, value, *width),
                            }
                        };
                        let (vx, vy, vz) = array_index(idx, depth);
                        result.x = vx;
                        result.y = vy;
                        result.z = vz;
                        result.width = width.max(1 << self.lod);
                        value.clone()
                    })
                    .collect::<Vec<_>>();
                let mask = result.width as i32 - 1;
                result.x &= !mask;
                result.y &= !mask;
                result.z &= !mask;
                if set.contains(&(result.x, result.y, result.z, result.width)) {
                    mem::forget(result.value);
                    return None;
                }
                set.insert((result.x, result.y, result.z, result.width));
                let avg = T::average(&array);
                if let Some(value) = avg {
                    let mut value = Cow::Owned(value);
                    mem::swap(&mut result.value, &mut value);
                    mem::forget(value);
                    Some(result)
                } else {
                    mem::forget(result.value);
                    None
                }
            })
    }

    pub fn elements_mut(&mut self) -> impl Iterator<Item = ElementMut<'_, T>> {
        let depth = self.depth;
        let array = &mut self.array as *mut Vec<_>;
        self.array
            .iter_mut()
            .enumerate()
            .flat_map(move |(i, mut value)| {
                let idx = i;
                let orig = value as *mut Node<T>;
                let value = loop {
                    match value {
                        Node::Ref(idx) => {
                            let array: &mut Vec<Node<T>> = unsafe { &mut *array };
                            value = &mut array[*idx];
                        }
                        Node::Value(value, width) => {
                            *width = 1;
                            break value.clone();
                        }
                    }
                };
                let value = unsafe {
                    *orig = Node::Value(value, 1);
                    match &mut *orig {
                        Node::Value(value, _) => value,
                        _ => unreachable!(),
                    }
                };
                value.as_mut().map(|value| {
                    let (x, y, z) = array_index(idx, depth);
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

#[cfg(feature = "savedata")]
impl<T: Voxel> From<RleTree<T>> for LodTree<T> {
    fn from(tree: RleTree<T>) -> Self {
        let mut array = Vec::new();
        let mut len = 0;
        for node in tree {
            len += node.value.as_ref().map(|_| node.len).unwrap_or_default();
            let mut remaining = node.len;
            if node.len.is_power_of_two() && node.len.log2() % 3 == 0 {
                let width = node.len.cbrt();
                let idx = array.len();
                array.push(Node::Value(node.value, width));
                for _ in 1..node.len {
                    array.push(Node::Ref(idx));
                }
            } else {
                let next = node.len.next_power_of_two();
                let iter = (0..next.log2()).rev()
                    .filter_map(|idx| if idx % 3 == 0 { Some(idx) } else { None });
                for i in iter {
                    let part = 1 << i;
                    while part <= remaining {
                        let width = part.cbrt();
                        let idx = array.len();
                        array.push(Node::Value(node.value.clone(), width));
                        for _ in 1..part {
                            array.push(Node::Ref(idx));
                        }
                        remaining -= part;
                    }
                }
            }
        }
        Self {
            lod: 0,
            depth: array.len().cbrt().log2(),
            len,
            array,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OptElement<'a, T> {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub width: usize,
    pub value: &'a Option<T>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Element<'a, T: Clone> {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub width: usize,
    pub value: Cow<'a, T>,
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
        let mut vt = LodTree::<i32>::new(4);
        vt.insert((0, 0, 0), 0);
        vt.insert((1, 1, 1), 1);
        vt.insert((2, 2, 2), 2);
        vt.insert((3, 3, 3), 3);

        assert_eq!(vt.position(&0), Some((0, 0, 0)));
        assert_eq!(vt.position(&1), Some((1, 1, 1)));
        assert_eq!(vt.position(&2), Some((2, 2, 2)));
        assert_eq!(vt.position(&3), Some((3, 3, 3)));

        assert_eq!(vt.get((0, 0, 0)).unwrap().into_owned(), 0);
        assert_eq!(vt.get((1, 1, 1)).unwrap().into_owned(), 1);
        assert_eq!(vt.get((2, 2, 2)).unwrap().into_owned(), 2);
        assert_eq!(vt.get((3, 3, 3)).unwrap().into_owned(), 3);
                   
    }

    #[test]
    pub fn remove() {
        let mut vt = LodTree::<i32>::new(8);
        vt.insert((0, 0, 0), 0);
        vt.insert((1, 1, 1), 1);
        vt.insert((2, 2, 2), 2);
        vt.insert((3, 3, 3), 3);

        vt.remove((0, 0, 0));

        assert_eq!(vt.position(&0), None);
        assert_eq!(vt.position(&1), Some((1, 1, 1)));
        assert_eq!(vt.position(&2), Some((2, 2, 2)));
        assert_eq!(vt.position(&3), Some((3, 3, 3)));

        assert_eq!(vt.get((0, 0, 0)), None);
        assert_eq!(vt.get((1, 1, 1)).unwrap().into_owned(), 1);
        assert_eq!(vt.get((2, 2, 2)).unwrap().into_owned(), 2);
        assert_eq!(vt.get((3, 3, 3)).unwrap().into_owned(), 3);
    }

    #[test]
    fn elements() {
        let mut vt = LodTree::<i32>::new(4);
        vt.insert((0, 0, 0), 0);
        vt.insert((0, 0, 1), 1);
        vt.insert((2, 0, 0), 2);

        assert_eq!(vt.elements().count(), 3);
        assert!(vt
            .elements()
            .map(|elem| ((elem.x, elem.y, elem.z), *elem.value, elem.width))
            .all(|elem| {
                [
                    ((0, 0, 0), 0, 1),
                    ((0, 0, 1), 1, 1),
                    ((2, 0, 0), 2, 1),
                ]
                .contains(&elem)
            }));
    }

    #[test]
    pub fn merge() {
        let mut vt = LodTree::<i32>::new(4);
        vt.insert((2, 2, 2), 0);
        vt.insert((2, 2, 3), 0);
        vt.insert((2, 3, 2), 0);
        vt.insert((2, 3, 3), 0);
        vt.insert((3, 2, 2), 0);
        vt.insert((3, 2, 3), 0);
        vt.insert((3, 3, 2), 0);
        vt.insert((3, 3, 3), 0);

        vt.merge();
        
        assert_eq!(vt.elements().count(), 1);

        assert_eq!(vt.position(&0), Some((2, 2, 2)));

        assert_eq!(vt.get((2, 2, 2)).unwrap().into_owned(), 0);
        assert_eq!(vt.get((2, 2, 3)).unwrap().into_owned(), 0);
        assert_eq!(vt.get((2, 3, 2)).unwrap().into_owned(), 0);
        assert_eq!(vt.get((2, 3, 3)).unwrap().into_owned(), 0);
        assert_eq!(vt.get((3, 2, 2)).unwrap().into_owned(), 0);
        assert_eq!(vt.get((3, 2, 3)).unwrap().into_owned(), 0);
        assert_eq!(vt.get((3, 3, 2)).unwrap().into_owned(), 0);
        assert_eq!(vt.get((3, 3, 3)).unwrap().into_owned(), 0);

        let a = &*vt.get((2, 2, 2)).unwrap() as *const _;
        let b = &*vt.get((2, 2, 3)).unwrap() as *const _;
        let c = &*vt.get((2, 3, 2)).unwrap() as *const _;
        let d = &*vt.get((2, 3, 3)).unwrap() as *const _;
        let e = &*vt.get((3, 2, 2)).unwrap() as *const _;
        let f = &*vt.get((3, 2, 3)).unwrap() as *const _;
        let g = &*vt.get((3, 3, 2)).unwrap() as *const _;
        let h = &*vt.get((3, 3, 3)).unwrap() as *const _;

        assert_eq!(a, b);
        assert_eq!(a, c);
        assert_eq!(a, d);
        assert_eq!(a, e);
        assert_eq!(a, f);
        assert_eq!(a, g);
        assert_eq!(a, h);
    }
}
