#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{
    collections::lod_tree::{LodTree, Voxel},
    serialize::SerDePartialEq,
};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Node<T> {
    pub value: Option<T>,
    pub len: usize,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
pub struct RleTree<T> {
    array: Vec<Node<T>>,
}

impl<T: Voxel> RleTree<T> {
    pub fn with_tree(tree: &LodTree<T>) -> Self {
        let mut array = Vec::<Node<T>>::new();
        for elem in tree.opt_elements() {
            if let Some(last) = array.last_mut() {
                if last.value.serde_eq(&elem.value) {
                    last.len += elem.width.pow(3);
                    continue;
                }
            }
            array.push(Node {
                value: elem.value.clone(),
                len: elem.width.pow(3),
            });
        }
        Self { array }
    }
}

impl<T: Voxel> IntoIterator for RleTree<T> {
    type IntoIter = std::vec::IntoIter<Self::Item>;
    type Item = Node<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.array.into_iter()
    }
}
