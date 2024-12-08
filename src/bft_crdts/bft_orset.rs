use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use crate::bft_crdts::hash_graph::HashGraph;

pub enum BFTORSetOp<E, I> {
    Add(E),
    Remove(E, Vec<I>),
}

impl<E, I> Into<Vec<u8>> for BFTORSetOp<E, I>
where
    E: Eq + Hash + Clone + Into<Vec<u8>>,
    I: PartialEq + Eq + Hash + Clone + Into<Vec<u8>>,
{
    fn into(self) -> Vec<u8> {
        match self {
            BFTORSetOp::Add(e) => {
                let mut bytes = vec![0];
                bytes.extend_from_slice(&e.into());
                bytes
            }
            BFTORSetOp::Remove(e, ids) => {
                let mut bytes = vec![1];
                bytes.extend_from_slice(&e.into());
                for id in ids {
                    bytes.extend_from_slice(&id.into());
                }
                bytes
            }
        }
    }
} 

impl<E, I> Clone for BFTORSetOp<E, I>
where
    E: Eq + Hash + Clone + Into<Vec<u8>>,
    I: PartialEq + Eq + Hash + Clone + Into<Vec<u8>>,
{
    fn clone(&self) -> Self {
        match self {
            BFTORSetOp::Add(e) => BFTORSetOp::Add(e.clone()),
            BFTORSetOp::Remove(e, ids) => BFTORSetOp::Remove(e.clone(), ids.clone()),
        }
    }
}

pub struct BFTORSet<E, I>
where
    E: Eq + Hash + Clone + Into<Vec<u8>>,
    I: PartialEq + Eq + Hash + Clone + Into<Vec<u8>>,
{
    elements: HashMap<E, HashSet<I>>,
    hash_graph: HashGraph<BFTORSetOp<E, I>>,
}

impl<E, I> BFTORSet<E, I>
where
    E: Eq + Hash + Clone + Into<Vec<u8>>,
    I: PartialEq + Eq + Hash + Clone + Into<Vec<u8>>,
{
    pub fn new() -> Self {
        BFTORSet {
            elements: HashMap::new(),
            hash_graph: HashGraph::new(),
        }
    }

    pub fn add(&mut self, e: E) -> BFTORSetOp<E, I> {
        BFTORSetOp::Add(e)
    }

    pub fn remove(&mut self, e: E, ids: Vec<I>) -> BFTORSetOp<E, I> {
        BFTORSetOp::Remove(e, ids)
    }

    pub fn is_in(&self, e: E) -> bool {
        self.elements.get(&e).map_or(false, |ids| !ids.is_empty())
    }

    pub fn get_ids(&self, e: E) -> HashSet<I> {
        self.elements.get(&e).cloned().unwrap_or_default()
    }

}
