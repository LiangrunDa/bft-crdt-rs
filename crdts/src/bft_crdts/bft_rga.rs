use std::hash::Hash;
use crate::bft_crdts::hash_graph::{HashGraph, HashType, Node};
use crate::bft_crdts::bft_crdt::BFTCRDT;
use crate::crdts::ordered_list::OrderedList;
use sha3::{Digest, Sha3_256};

//  The ID of each element in RGA affects the position of the element in the list, since 
//   $\isa{insert-body}$ skips over the elements that have greater IDs than the inserted element. 
//   Therefore, we cannot simply use the hash of the node containing the Insert operation as its ID, 
//   as this would prevent the operation generator from controlling the exact position of the inserted 
//   element. Instead, we use a pair consisting of an ID that is chosen by the generating peer and the 
//   hash of the node containing the Insert operation as the element ID. When comparing element IDs, 
//   we first compare the IDs chosen by the peers. If they are identical, we then compare the hashes. 
//   This approach ensures that the operation generator can control the position of the inserted element, 
//   while still guaranteeing the uniqueness of the ID.

type RGAID<I> = (I, HashType);

#[derive(Debug, Clone)]
pub enum BFTRGAOp<I, V> {
    // v, i, ei
    Insert(V, I, Option<RGAID<I>>),
    // ei
    Delete(RGAID<I>),
}

impl<I, V> Into<Vec<u8>> for BFTRGAOp<I, V>
where
    I: Eq + Hash + Clone + Into<Vec<u8>> + PartialOrd,
    V: Eq + Hash + Clone + Into<Vec<u8>>,
{
    fn into(self) -> Vec<u8> {
        match self {
            BFTRGAOp::Insert(v, i, rga_id) => {
                let mut bytes = vec![];
                bytes.extend_from_slice(&v.into());
                bytes.extend_from_slice(&i.into());
                match rga_id {
                    Some((id, hash)) => {
                        bytes.extend_from_slice(&id.into());
                        bytes.extend_from_slice(&hash);
                    }
                    None => {}
                }
                bytes
            }
            BFTRGAOp::Delete(rga_id) => {
                let mut bytes = vec![];
                bytes.extend_from_slice(&rga_id.0.into());
                bytes.extend_from_slice(&rga_id.1);
                bytes
            }
        }
    }
}

pub struct BFTRGA<I, V>
where
    I: Eq + Hash + Clone + Into<Vec<u8>> + PartialOrd,
    V: Eq + Hash + Clone + Into<Vec<u8>>,
{
    elements: OrderedList<RGAID<I>, V>,
}

impl <I, V> BFTCRDT<BFTRGAOp<I, V>> for BFTRGA<I, V>
where
    I: Eq + Hash + Clone + Into<Vec<u8>> + PartialOrd,
    V: Eq + Hash + Clone + Into<Vec<u8>>,
{
    fn interpret_node(&mut self, node: &Node<BFTRGAOp<I, V>>) {
        let op = &node.value;
        match op {
            BFTRGAOp::Insert(value, id, after) => {
                let h = node.get_hash();
                self.elements.insert_by_id((id.clone(), h.clone()), value.clone(), after.clone());
            }
            BFTRGAOp::Delete(eid) => {
                self.elements.delete_by_id(eid.clone());
            }
        }
    }

    fn is_sem_valid(&self, node: &Node<BFTRGAOp<I, V>>, hash_graph: &HashGraph<BFTRGAOp<I, V>>) -> bool {
        let op = &node.value;
        match op {
            // ‹is_rga_sem_valid C H G (hs, Insert v i ei) = (
            //     case ei of 
            //       None ⇒ True
            //     | Some ii ⇒ (∃hs' v' i' ei'. 
            //         (hs', Insert v' i' ei') ∈ G ∧
            //         C (hs', Insert v' i' ei') (hs, Insert v i ei) ∧ 
            //         H (hs', Insert v' i' ei') = snd ii ∧
            //         i' = (fst ii)) ∧ 
            //         H (hs, Insert v i ei) ≠ snd ii
            //   )›
            BFTRGAOp::Insert(v, i, ei) => {
                match ei {
                    Some((id, hash)) => {
                        let ref_node_res = hash_graph.get_node(hash); // H (hs', Insert v' i' ei') = snd ii
                        if let Some(ref_node) = ref_node_res {
                            if let BFTRGAOp::Insert(v2, i2, ei2) = &node.value {
                                let ref_hash = ref_node.get_hash();
                                let hash = node.get_hash();
                                if hash_graph.is_ancestor(&ref_hash, &hash) && id == i2 {
                                    true
                                } else {
                                    false
                                }
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    }
                    None => {
                        true
                    }
                }
            }
            BFTRGAOp::Delete(eid) => {
                false
            }
        }
    }
}

impl<I, V> BFTRGA<I, V> 
where 
    I: Eq + Hash + Clone + Into<Vec<u8>> + PartialOrd,
    V: Eq + Hash + Clone + Into<Vec<u8>>,
{
    pub fn new() -> Self {
        BFTRGA {
            elements: OrderedList::new(),
        }
    }

    pub fn get(&self, idx: usize) -> Option<V> {
        self.elements.get_by_idx(idx).map(|(_, v, _)| v)
    }

    pub fn get_list(&self) -> Vec<V> {
        self.elements.get_list()
    }

    pub fn insert(&mut self, idx: usize, value: V, iid: I) -> Option<BFTRGAOp<I, V>> {
        if idx == 0 {
            Some(BFTRGAOp::Insert(value, iid, None))
        } else {
            let prev = self.elements.get_by_idx(idx - 1);
            if let Some((id, _, _)) = prev {
                Some(BFTRGAOp::Insert(value, iid, Some(id.clone())))
            } else {
                None
            }
        }
    }
    
    pub fn delete(&mut self, idx: usize) -> Option<BFTRGAOp<I, V>> {
        let elem = self.elements.get_by_idx(idx);
        if let Some((id, _, _)) = elem {
            Some(BFTRGAOp::Delete(id.clone()))
        } else {
            None
        }
    }
}