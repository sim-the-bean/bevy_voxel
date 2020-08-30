#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use std::{iter, mem, slice};

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

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
pub struct VolumetricTree<T> {
    len: usize,
    root: Node<T>,
}

impl<T> VolumetricTree<T> {
    pub fn new(width: usize) -> Self {
        Self {
            len: 0,
            root: Node::Leaf { width, value: None },
        }
    }

    pub fn capacity(&self) -> usize {
        self.width() * self.width() * self.width()
    }

    pub fn width(&self) -> usize {
        self.root.width()
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn clear(&mut self) {
        let width = self.root.width();
        self.root = Node::Leaf { width, value: None };
    }

    pub fn nodes(&self) -> Nodes<'_, T> {
        match &self.root {
            Node::Leaf { .. } => {
                let width = self.width() as i32 / 2;
                Nodes {
                    x: -width,
                    y: -width,
                    z: -width,
                    idx: 0,
                    nodes: slice::from_ref(&self.root),
                }
            }
            Node::Branch { width, elems } => {
                let width = *width as i32 / 2;
                Nodes {
                    x: -width,
                    y: -width,
                    z: -width,
                    idx: 0,
                    nodes: elems,
                }
            }
        }
    }

    pub fn elements(&self) -> Elements<'_, T> {
        let width = self.width() as i32 / 2;
        let mut idx = vec![];
        let mut node = &self.root;
        loop {
            match node {
                Node::Leaf { value: Some(_), .. } => break,
                Node::Leaf { .. } => break,
                Node::Branch { elems, .. } => {
                    for (i, n) in elems.iter().enumerate() {
                        match n {
                            Node::Branch { .. } | Node::Leaf { value: Some(_), .. } => {
                                node = n;
                                idx.push(i);
                                break;
                            }
                            Node::Leaf { .. } => {}
                        }
                    }
                }
            }
        }
        let empty = match self.root {
            Node::Leaf { value: Some(_), .. } => false,
            Node::Leaf { .. } => true,
            Node::Branch { .. } => false,
        };
        Elements {
            x: -width,
            y: -width,
            z: -width,
            idx,
            node: &self.root,
            empty,
        }
    }

    pub fn elements_mut(&mut self) -> ElementsMut<'_, T> {
        let width = self.width() as i32 / 2;
        let mut idx = vec![];
        let mut node = &mut self.root as *mut _;
        loop {
            match unsafe { &mut *node } {
                Node::Leaf { value: Some(_), .. } => break,
                Node::Leaf { .. } => break,
                Node::Branch { elems, .. } => {
                    for (i, n) in elems.iter_mut().enumerate() {
                        match n {
                            Node::Branch { .. } | Node::Leaf { value: Some(_), .. } => {
                                node = n as *mut _;
                                idx.push(i);
                                break;
                            }
                            Node::Leaf { .. } => {}
                        }
                    }
                }
            }
        }
        let empty = match self.root {
            Node::Leaf { value: Some(_), .. } => false,
            Node::Leaf { .. } => true,
            Node::Branch { .. } => false,
        };
        ElementsMut {
            x: -width,
            y: -width,
            z: -width,
            idx,
            node: &mut self.root,
            empty,
        }
    }
}

impl<T: Clone + PartialEq> VolumetricTree<T> {
    pub fn insert(&mut self, coords: (i32, i32, i32), value: T) -> Option<T> {
        if coords.0 >= self.width() as i32 / 2
            || coords.0 < self.width() as i32 / -2
            || coords.1 >= self.width() as i32 / 2
            || coords.1 < self.width() as i32 / -2
            || coords.2 >= self.width() as i32 / 2
            || coords.2 < self.width() as i32 / -2
        {
            return None;
        }
        let result = self.root.insert(coords, value);
        if result.is_none() {
            self.len += 1;
        }
        result
    }

    pub fn remove(&mut self, coords: (i32, i32, i32)) -> Option<T> {
        if coords.0 >= self.width() as i32 / 2
            || coords.0 < self.width() as i32 / -2
            || coords.1 >= self.width() as i32 / 2
            || coords.1 < self.width() as i32 / -2
            || coords.2 >= self.width() as i32 / 2
            || coords.2 < self.width() as i32 / -2
        {
            return None;
        }
        let result = self.root.remove(coords);
        if result.is_some() {
            self.len -= 1;
        }
        result
    }

    pub fn get(&self, coords: (i32, i32, i32)) -> Option<&T> {
        if coords.0 >= self.width() as i32 / 2
            || coords.0 < self.width() as i32 / -2
            || coords.1 >= self.width() as i32 / 2
            || coords.1 < self.width() as i32 / -2
            || coords.2 >= self.width() as i32 / 2
            || coords.2 < self.width() as i32 / -2
        {
            return None;
        }
        self.root.get(coords)
    }

    pub fn get_mut(&mut self, coords: (i32, i32, i32)) -> Option<&mut T> {
        if coords.0 >= self.width() as i32 / 2
            || coords.0 < self.width() as i32 / -2
            || coords.1 >= self.width() as i32 / 2
            || coords.1 < self.width() as i32 / -2
            || coords.2 >= self.width() as i32 / 2
            || coords.2 < self.width() as i32 / -2
        {
            return None;
        }
        self.root.get_mut(coords)
    }

    pub fn position(&self, value: &T) -> Option<(i32, i32, i32)> {
        let width = self.width() as i32 / 2;
        self.root.position((-width, -width, -width), value)
    }

    pub fn contains_key(&self, coords: (i32, i32, i32)) -> bool {
        self.get(coords).is_some()
    }

    pub fn contains(&self, value: &T) -> bool {
        self.position(value).is_some()
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Node<T> {
    Leaf { width: usize, value: Option<T> },
    Branch { width: usize, elems: Vec<Node<T>> },
}

impl<T> Default for Node<T> {
    fn default() -> Self {
        Node::Leaf {
            width: 1,
            value: None,
        }
    }
}

impl<T> Node<T> {
    pub fn is_leaf(&self) -> bool {
        match self {
            Node::Leaf { .. } => true,
            Node::Branch { .. } => false,
        }
    }

    pub fn is_branch(&self) -> bool {
        match self {
            Node::Leaf { .. } => false,
            Node::Branch { .. } => true,
        }
    }

    pub fn width(&self) -> usize {
        match self {
            Node::Leaf { width, .. } | Node::Branch { width, .. } => *width,
        }
    }

    pub fn nodes(&self) -> Option<Nodes<'_, T>> {
        match self {
            Node::Leaf { .. } => None,
            Node::Branch { elems, width } => {
                let width = *width as i32 / 2;
                Some(Nodes {
                    x: -width,
                    y: -width,
                    z: -width,
                    idx: 0,
                    nodes: elems,
                })
            }
        }
    }
}

impl<T: Clone + PartialEq> Node<T> {
    fn merge(&mut self) {
        match self {
            Node::Leaf { .. } => {}
            Node::Branch { elems, width } => {
                let first = &elems[0];
                match first {
                    Node::Leaf { .. } => {}
                    Node::Branch { .. } => return,
                }
                for other in &elems[1..8] {
                    if first != other {
                        return;
                    }
                }
                let first = &mut elems[0];
                let mut new = Node::Leaf {
                    width: *width / 2,
                    value: None,
                };
                mem::swap(first, &mut new);
                match &mut new {
                    Node::Leaf { width, .. } => *width *= 2,
                    Node::Branch { width, .. } => *width *= 2,
                }
                *self = new;
            }
        }
    }

    pub fn insert(&mut self, (x, y, z): (i32, i32, i32), value: T) -> Option<T> {
        match self {
            Node::Leaf {
                width,
                value: current_value,
            } => {
                if *width == 1 {
                    let mut value = Some(value);
                    mem::swap(current_value, &mut value);
                    value
                } else {
                    let width_2 = *width as i32 / 2;
                    let width_4 = width_2 / 2;

                    let dx = (x.signum() + 2) / 2;
                    let dy = (y.signum() + 2) / 2;
                    let dz = (z.signum() + 2) / 2;

                    let x = x - width_2 * dx + width_4;
                    let y = y - width_2 * dy + width_4;
                    let z = z - width_2 * dz + width_4;

                    let mut elems = iter::repeat(current_value.clone())
                        .take(8)
                        .map(|value| Node::Leaf {
                            width: width_2 as usize,
                            value,
                        })
                        .collect::<Vec<_>>();
                    elems[sp_index(dx, dy, dz)].insert((x, y, z), value);
                    let mut node = Node::Branch {
                        width: *width,
                        elems,
                    };
                    node.merge();
                    mem::swap(self, &mut node);

                    match node {
                        Node::Leaf { value, .. } => value,
                        _ => unreachable!(),
                    }
                }
            }
            Node::Branch { width, elems } => {
                let width_2 = *width as i32 / 2;
                let width_4 = width_2 / 2;

                let dx = (x.signum() + 2) / 2;
                let dy = (y.signum() + 2) / 2;
                let dz = (z.signum() + 2) / 2;

                let x = x - width_2 * dx + width_4;
                let y = y - width_2 * dy + width_4;
                let z = z - width_2 * dz + width_4;

                let result = elems[sp_index(dx, dy, dz)].insert((x, y, z), value);
                self.merge();
                result
            }
        }
    }

    pub fn remove(&mut self, (x, y, z): (i32, i32, i32)) -> Option<T> {
        match self {
            Node::Leaf {
                width,
                value: current_value,
            } => {
                if *width == 1 {
                    let mut value = None;
                    mem::swap(current_value, &mut value);
                    value
                } else {
                    let width_2 = *width as i32 / 2;
                    let width_4 = width_2 / 2;

                    let dx = (x.signum() + 2) / 2;
                    let dy = (y.signum() + 2) / 2;
                    let dz = (z.signum() + 2) / 2;

                    let x = x - width_2 * dx + width_4;
                    let y = y - width_2 * dy + width_4;
                    let z = z - width_2 * dz + width_4;

                    let mut elems = iter::repeat(current_value.clone())
                        .take(8)
                        .map(|value| Node::Leaf {
                            width: width_2 as usize,
                            value,
                        })
                        .collect::<Vec<_>>();
                    elems[sp_index(dx, dy, dz)].remove((x, y, z));
                    let mut node = Node::Branch {
                        width: *width,
                        elems,
                    };
                    node.merge();
                    mem::swap(self, &mut node);

                    match node {
                        Node::Leaf { value, .. } => value,
                        _ => unreachable!(),
                    }
                }
            }
            Node::Branch { width, elems } => {
                let width_2 = *width as i32 / 2;
                let width_4 = width_2 / 2;

                let dx = (x.signum() + 2) / 2;
                let dy = (y.signum() + 2) / 2;
                let dz = (z.signum() + 2) / 2;

                let x = x - width_2 * dx + width_4;
                let y = y - width_2 * dy + width_4;
                let z = z - width_2 * dz + width_4;

                let result = elems[sp_index(dx, dy, dz)].remove((x, y, z));
                self.merge();
                result
            }
        }
    }

    pub fn get(&self, (x, y, z): (i32, i32, i32)) -> Option<&T> {
        match self {
            Node::Leaf { value, .. } => value.as_ref(),
            Node::Branch { width, elems } => {
                let width_2 = *width as i32 / 2;
                let width_4 = width_2 / 2;

                let dx = (x.signum() + 2) / 2;
                let dy = (y.signum() + 2) / 2;
                let dz = (z.signum() + 2) / 2;

                let x = x - width_2 * dx + width_4;
                let y = y - width_2 * dy + width_4;
                let z = z - width_2 * dz + width_4;

                elems[sp_index(dx, dy, dz)].get((x, y, z))
            }
        }
    }

    pub fn get_mut(&mut self, (x, y, z): (i32, i32, i32)) -> Option<&mut T> {
        match self {
            Node::Leaf { value, .. } => value.as_mut(),
            Node::Branch { width, elems } => {
                let width_2 = *width as i32 / 2;
                let width_4 = width_2 / 2;

                let dx = (x.signum() + 2) / 2;
                let dy = (y.signum() + 2) / 2;
                let dz = (z.signum() + 2) / 2;

                let x = x - width_2 * dx + width_4;
                let y = y - width_2 * dy + width_4;
                let z = z - width_2 * dz + width_4;

                elems[sp_index(dx, dy, dz)].get_mut((x, y, z))
            }
        }
    }

    pub fn position(&self, coords: (i32, i32, i32), value: &T) -> Option<(i32, i32, i32)> {
        match self {
            Node::Leaf {
                value: Some(this), ..
            } if this == value => Some(coords),
            Node::Leaf { .. } => None,
            Node::Branch { width, elems } => {
                for (i, node) in elems.iter().enumerate() {
                    let (mut x, mut y, mut z) = coords;
                    let (dx, dy, dz) = dir_index(i);
                    let width_2 = *width as i32 / 2;
                    x += dx * width_2;
                    y += dy * width_2;
                    z += dz * width_2;
                    let result = node.position((x, y, z), value);
                    if result.is_some() {
                        return result;
                    }
                }
                None
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry<'a, T> {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub node: &'a Node<T>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Nodes<'a, T> {
    x: i32,
    y: i32,
    z: i32,
    idx: usize,
    nodes: &'a [Node<T>],
}

impl<'a, T> Iterator for Nodes<'a, T> {
    type Item = Entry<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        let idx = self.idx;
        if idx == self.nodes.len() {
            return None;
        }
        self.idx += 1;
        let (dx, dy, dz) = dir_index(idx);
        let x = self.x + dx;
        let y = self.y + dy;
        let z = self.z + dz;
        Some(Entry {
            x,
            y,
            z,
            node: &self.nodes[idx],
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Elements<'a, T> {
    x: i32,
    y: i32,
    z: i32,
    idx: Vec<usize>,
    node: &'a Node<T>,
    empty: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ElementsMut<'a, T> {
    x: i32,
    y: i32,
    z: i32,
    idx: Vec<usize>,
    node: &'a mut Node<T>,
    empty: bool,
}

impl<'a, T> Iterator for Elements<'a, T> {
    type Item = Element<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.empty {
            return None;
        }

        let mut next = None;
        let mut nodes = Vec::with_capacity(self.idx.len());
        let mut node = self.node;
        let mut x = self.x;
        let mut y = self.y;
        let mut z = self.z;
        for &idx in &self.idx {
            nodes.push(node);
            match node {
                Node::Leaf { .. } => unreachable!(),
                Node::Branch { elems, width } => {
                    let width_2 = *width as i32 / 2;
                    let (dx, dy, dz) = dir_index(idx);
                    x += dx * width_2;
                    y += dy * width_2;
                    z += dz * width_2;
                    node = &elems[idx];
                }
            }
        }
        nodes.push(node);
        match node {
            Node::Leaf {
                value: Some(value),
                width,
            } => {
                next = Some(Element {
                    x,
                    y,
                    z,
                    width: *width,
                    value,
                });
            }
            _ => return next,
        }
        let mut changed = false;
        while !self.idx.is_empty() {
            let mut idx = *self.idx.last().unwrap();
            let node = *nodes.last().unwrap();
            match node {
                Node::Leaf { value: Some(_), .. } if changed => {
                    break;
                }
                Node::Leaf { .. } => {
                    changed = true;
                    idx += 1;
                    *self.idx.last_mut().unwrap() = idx;

                    if idx == 8 {
                        while idx == 8 && !self.idx.is_empty() {
                            self.idx.pop();
                            nodes.pop();
                            if let Some(i) = self.idx.last_mut() {
                                *i += 1;
                                idx = *i;
                            }
                        }
                    }

                    nodes.pop();
                    if let Some(node) = nodes.last() {
                        match node {
                            Node::Branch { elems, .. } => {
                                nodes.push(&elems[idx]);
                            }
                            _ => unreachable!(),
                        }
                    }
                }
                Node::Branch { elems, .. } => {
                    self.idx.push(0);
                    nodes.push(&elems[0]);
                    changed = true;
                }
            }
        }

        if self.idx.is_empty() {
            self.empty = true;
        }

        next
    }
}

impl<'a, T> Iterator for ElementsMut<'a, T> {
    type Item = ElementMut<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.empty {
            return None;
        }

        let mut nodes = Vec::with_capacity(self.idx.len());
        let mut node = self.node as *mut _;
        let mut x = self.x;
        let mut y = self.y;
        let mut z = self.z;
        for &idx in &self.idx {
            nodes.push(node);
            match unsafe { &mut *node } {
                Node::Leaf { .. } => unreachable!(),
                Node::Branch { elems, width } => {
                    let width_2 = *width as i32 / 2;
                    let (dx, dy, dz) = dir_index(idx);
                    x += dx * width_2;
                    y += dy * width_2;
                    z += dz * width_2;
                    node = &mut elems[idx] as *mut _;
                }
            }
        }
        nodes.push(node);
        let value_ptr = match unsafe { &mut *node } {
            Node::Leaf {
                value: Some(value),
                width,
            } => (value as *mut _, x, y, z, *width),
            _ => return None,
        };
        let mut changed = false;
        while !self.idx.is_empty() {
            let mut idx = *self.idx.last().unwrap();
            let node = *nodes.last().unwrap();
            match unsafe { &mut *node } {
                Node::Leaf { value: Some(_), .. } if changed => {
                    break;
                }
                Node::Leaf { .. } => {
                    changed = true;
                    idx += 1;
                    *self.idx.last_mut().unwrap() = idx;

                    if idx == 8 {
                        while idx == 8 && !self.idx.is_empty() {
                            self.idx.pop();
                            nodes.pop();
                            if let Some(i) = self.idx.last_mut() {
                                *i += 1;
                                idx = *i;
                            }
                        }
                    }

                    nodes.pop();
                    if let Some(&node) = nodes.last() {
                        match unsafe { &mut *node } {
                            Node::Branch { elems, .. } => {
                                nodes.push(&mut elems[idx] as *mut _);
                            }
                            _ => unreachable!(),
                        }
                    }
                }
                Node::Branch { elems, .. } => {
                    self.idx.push(0);
                    nodes.push(&mut elems[0] as *mut _);
                    changed = true;
                }
            }
        }

        if self.idx.is_empty() {
            self.empty = true;
        }

        Some(ElementMut {
            x: value_ptr.1,
            y: value_ptr.2,
            z: value_ptr.3,
            value: unsafe { &mut *value_ptr.0 },
            width: value_ptr.4,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn insert() {
        let mut vt = VolumetricTree::<i32>::new(8);
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
        let mut vt = VolumetricTree::<i32>::new(8);
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
    pub fn auto_merge() {
        let mut vt = VolumetricTree::<i32>::new(8);
        vt.insert((2, 2, 2), 0);
        vt.insert((2, 2, 3), 0);
        vt.insert((2, 3, 2), 0);
        vt.insert((2, 3, 3), 0);
        vt.insert((3, 2, 2), 0);
        vt.insert((3, 2, 3), 0);
        vt.insert((3, 3, 2), 0);
        vt.insert((3, 3, 3), 0);

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

    #[test]
    #[cfg(feature = "serde")]
    fn serde() {
        let mut vt = VolumetricTree::<i32>::new(8);
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

        let deserialized: VolumetricTree<i32> = serde_json::from_str(&serialized).unwrap();

        assert_eq!(vt, deserialized);
    }

    #[test]
    fn elements() {
        let mut vt = VolumetricTree::<i32>::new(4);
        vt.insert((-2, -2, -2), -2);
        vt.insert((-1, -1, -1), -1);
        vt.insert((0, 0, 0), 2);
        vt.insert((0, 0, 1), 2);
        vt.insert((0, 1, 0), 2);
        vt.insert((0, 1, 1), 2);
        vt.insert((1, 0, 0), 2);
        vt.insert((1, 0, 1), 2);
        vt.insert((1, 1, 0), 2);
        vt.insert((1, 1, 1), 2);

        assert_eq!(
            vt.elements()
                .map(|elem| ((elem.x, elem.y, elem.z), *elem.value, elem.width))
                .collect::<Vec<_>>(),
            &[
                ((-2, -2, -2), -2, 1),
                ((-1, -1, -1), -1, 1),
                ((0, 0, 0), 2, 2),
            ],
        );
    }

    #[test]
    fn diagnonal() {
        let mut vt = VolumetricTree::<i32>::new(4);
        vt.insert((-2, -2, -2), -2);
        vt.insert((-1, -1, -1), -1);
        vt.insert((0, 0, 0), 0);
        vt.insert((1, 1, 1), 1);

        assert_eq!(
            vt.elements()
                .map(|elem| ((elem.x, elem.y, elem.z), *elem.value, elem.width))
                .collect::<Vec<_>>(),
            &[
                ((-2, -2, -2), -2, 1),
                ((-1, -1, -1), -1, 1),
                ((0, 0, 0), 0, 1),
                ((1, 1, 1), 1, 1),
            ],
        );
    }
}
