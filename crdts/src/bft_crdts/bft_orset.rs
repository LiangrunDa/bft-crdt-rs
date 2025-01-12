use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use crate::bft_crdts::bft_crdt::BFTCRDT;
use crate::bft_crdts::hash_graph::{HashGraph, Node};
use sha2::{Digest, Sha256};
use crate::bft_crdts::hash_graph::HashType;
use crate::serialize::Serialize;

type ORSetID = HashType; // in BFT ORSet, ID is the hash value of the element's Add operation

#[derive(Debug, Clone)]
pub enum BFTORSetOp<E> {
    Add(E),
    Remove(E, Vec<ORSetID>),
}

impl<E> Serialize for BFTORSetOp<E>
where
    E: Eq + Hash + Clone + Serialize,
{
    fn to_bytes(&self) -> Vec<u8> {
        match self {
            BFTORSetOp::Add(e) => {
                let mut bytes = vec![];
                bytes.extend_from_slice(&e.to_bytes());
                bytes
            }
            BFTORSetOp::Remove(e, ids) => {
                let mut bytes = vec![];
                let mut sorted_ids = ids.clone();
                sorted_ids.sort();
                for id in sorted_ids.iter() {
                    bytes.extend_from_slice(id.as_bytes());
                }
                bytes.extend_from_slice(&e.to_bytes());
                bytes
            }
        }
    }
}

pub struct BFTORSet<E>
where
    E: Eq + Hash + Clone + Serialize,
{
    pub elements: HashMap<E, HashSet<ORSetID>>,
}

impl<E> BFTORSet<E>
where
    E: Eq + Hash + Clone + Serialize,
{
    pub fn new() -> Self {
        BFTORSet {
            elements: HashMap::new(),
        }
    }

    pub fn add(&mut self, e: E) -> BFTORSetOp<E> {
        BFTORSetOp::Add(e)
    }

    pub fn remove(&mut self, e: E, ids: Vec<ORSetID>) -> BFTORSetOp<E> {
        BFTORSetOp::Remove(e, ids)
    }
    
    pub fn remove_elem(&mut self, e: E) -> BFTORSetOp<E> {
        let ids = self.elements.get(&e).cloned().unwrap_or_default();
        BFTORSetOp::Remove(e, ids.into_iter().collect())
    }

    pub fn is_in(&self, e: E) -> bool {
        self.elements.get(&e).map_or(false, |ids| !ids.is_empty())
    }

    pub fn get_ids(&self, e: E) -> HashSet<ORSetID> {
        self.elements.get(&e).cloned().unwrap_or_default()
    }
    
    pub fn get_set(&self) -> HashSet<E> {
        // only elements that have non-empty ids
        self.elements.iter().filter(|(_, ids)| !ids.is_empty()).map(|(e, _)| e.clone()).collect()
    }
    
    pub fn get_map(&self) -> HashMap<E, HashSet<ORSetID>> {
        self.elements.clone()
    }
    
}

impl<E> BFTCRDT<BFTORSetOp<E>> for BFTORSet<E>
where
    E: Eq + Hash + Clone + Serialize,
{
    fn interpret_node(&mut self, node: &Node<BFTORSetOp<E>>) {
        let op = &node.value;
        match op {
            BFTORSetOp::Add(e) => {
                let id = node.get_hash();
                self.elements.entry(e.clone()).or_insert(HashSet::new()).insert(id);
            }
            BFTORSetOp::Remove(e, ids) => {
                if let Some(e_ids) = self.elements.get_mut(e) {
                    for id in ids {
                        e_ids.remove(id);
                    }
                }
            }
        }
    }

    fn is_sem_valid(&self, node: &Node<BFTORSetOp<E>>, hash_graph: &HashGraph<BFTORSetOp<E>>) -> bool {
        // fun is_orset_sem_valid :: ‹('hash, 'a) ORSetC ⇒ ('hash, 'a) ORSetH ⇒ ('hash, 'a) ORSetN set ⇒ ('hash, 'a) ORSetN ⇒ bool› where
        //   ‹is_orset_sem_valid C H S (hs, Add e) = True›
        // | ‹is_orset_sem_valid C H S (hs, Rem is e) = 
        //     (∀i ∈ is. ∃ n ∈ S. (C n (hs, Rem is e)) ∧ (snd n = Add e) ∧ (H n = i))›
        let op = node.clone().value;
        match op {
            BFTORSetOp::Add(e) => true,
            BFTORSetOp::Remove(e1, ids) => {
                ids.iter().all(|id| { // ∀i ∈ is
                    let h = id;
                    let rem_node = hash_graph.get_node(h); // ∃ n ∈ S. H n = i
                    match rem_node {
                        Some(n) => {
                            let hn = n.get_hash();
                            if let BFTORSetOp::Add(e2) = &n.value {
                                hash_graph.is_ancestor(&hn, node) // (C n (hs, Rem is e))
                            } else {
                                false
                            }
                        }
                        None => false
                    }
                })
            }
        }
    }
}