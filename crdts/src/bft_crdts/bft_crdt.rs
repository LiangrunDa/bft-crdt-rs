use std::cmp::min;
use std::fmt::Debug;
use crate::bft_crdts::hash_graph::{HashGraph, HashType, Node};
use tracing::{trace};
use crate::serialize::Serialize;
use rand;
use rand::{Rng, RngCore};
use rand::prelude::SliceRandom;
use rand::rngs::StdRng;
use rand::SeedableRng;

pub trait BFTCRDT<O: Serialize + Clone> {
    fn interpret_node(&mut self, node: &Node<O>);
    fn is_sem_valid(&self, op: &Node<O>, hash_graph: &HashGraph<O>) -> bool;
}

pub struct BFTCRDTTester<O: Serialize + Clone, T: BFTCRDT<O>> {
    pub crdt: T,
    pub hash_graph: HashGraph<O>,
}

impl <O: Serialize + Clone + Debug, T: BFTCRDT<O>> BFTCRDTTester<O, T> {
    pub fn new(crdt: T) -> Self {
        BFTCRDTTester {
            crdt,
            hash_graph: HashGraph::new(),
        }
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub fn handle_node(&mut self, remote_node: Node<O>) {
        trace!("Begin of handle_node");
        let struct_valid = self.hash_graph.is_structurally_valid(&remote_node);
        if !struct_valid {
            return;
        }
        let sem_valid = self.crdt.is_sem_valid(&remote_node, &self.hash_graph);
        if sem_valid {
            self.hash_graph.add_node(remote_node.clone());
            trace!("Interpreting node");
            self.crdt.interpret_node(&remote_node);
            trace!("Node interpreted");
        }
        trace!("End of handle_node");
    }
}

/// IMPORTANT: used only for testing
pub struct BFTCRDTGenerator<O: Serialize + Clone, T: BFTCRDT<O>> {
    pub crdt: T,
    pub hash_graph: HashGraph<O>,
    rng: StdRng,
}

impl <O: Serialize + Clone, T: BFTCRDT<O>> BFTCRDTGenerator<O, T> {

    pub fn new(crdt: T, seed: u64) -> Self {
        BFTCRDTGenerator {
            crdt,
            hash_graph: HashGraph::new(),
            rng: StdRng::seed_from_u64(seed),
        }
    }

    /// IMPORTANT: used only for correctness testing 
    pub fn generate_and_interpret_random_struct_valid_node(&mut self, op: O) -> Node<O> {
        let num_preds = (self.rng.next_u32() % 10) as usize;
        let num_preds = min(num_preds, self.hash_graph.nodes.len());
        let mut preds = self.hash_graph.nodes.keys().cloned().collect::<Vec<HashType>>();
        preds.shuffle(&mut self.rng);
        preds.truncate(num_preds);

        let node = Node {
            predecessors: preds,
            value: op,
        };
        if !self.hash_graph.is_structurally_valid(&node) {
            // this should never happen
            panic!("Generated node is not structurally valid");
        }

        let sem_valid = self.crdt.is_sem_valid(&node, &self.hash_graph);
        if sem_valid {
            // we happen to generate a semantically valid node
            self.hash_graph.add_node(node.clone());
            self.crdt.interpret_node(&node);
            node
        } else {
            // more likely we generated a node that is not semantically valid
            node
        }
    }

    /// IMPORTANT: used only for testing 
    pub fn generate_and_interpret_valid_node(&mut self, op: O) -> Node<O> {
        let h = self.hash_graph.add_value_with_head_preds(op.clone());
        if h.is_none() {
            // this should never happen
            panic!("Failed to add local node");
        }
        let node = self.hash_graph.get_node(&h.unwrap()).unwrap();
        self.crdt.interpret_node(&node);
        node.clone()
    }
    
    
    /// IMPORTANT: used only for testing
    pub fn generate_and_interpret_random_node(&mut self, op: O) -> Node<O> {
        let num_preds = (self.rng.next_u32() % 10) as usize;
        let mut preds: Vec<HashType> = vec![];
        
        for _ in 0..num_preds {
            let mut hash = [0u8; 32];
            self.rng.fill(&mut hash);
            let hash = hex::encode(hash);
            preds.push(hash);
        }

        let node = Node {
            predecessors: preds,
            value: op,
        };
        
        let struct_valid = self.hash_graph.is_structurally_valid(&node);
        if !struct_valid {
            // almost always this will be the case
            return node;
        }
        
        let sem_valid = self.crdt.is_sem_valid(&node, &self.hash_graph);
        if sem_valid {
            // we happen to generate a semantically valid node
            self.hash_graph.add_node(node.clone());
            self.crdt.interpret_node(&node);
            return node;
        }
        // more likely we generated a node that is not semantically valid
        node
    }
}

pub struct BFTCRDTHandler<O: Serialize + Clone, T: BFTCRDT<O>> {
    pub crdt: T,
    pub hash_graph: HashGraph<O>,
    pub pending_nodes: Vec<Node<O>>,
}

impl <O: Serialize + Clone, T: BFTCRDT<O>> BFTCRDTHandler<O, T> {
    pub fn new(crdt: T) -> Self {
        BFTCRDTHandler {
            crdt,
            hash_graph: HashGraph::new(),
            pending_nodes: vec![],
        }
    }
    
    pub fn handle_local_op(&mut self, op: O) {
        let h = self.hash_graph.add_value_with_head_preds(op.clone());
        if h.is_none() {
            // this should never happen
            panic!("Failed to add local node");
        }
        let node = self.hash_graph.get_node(&h.unwrap()).unwrap();
        self.crdt.interpret_node(&node);
    }

    pub fn handle_remote_node(&mut self, remote_node: Node<O>) {
        let struct_valid = self.hash_graph.is_structurally_valid(&remote_node);
        if !struct_valid {
            self.pending_nodes.push(remote_node);
            return ;
        }
        let sem_valid = self.crdt.is_sem_valid(&remote_node, &self.hash_graph);
        if sem_valid {
            self.hash_graph.add_node(remote_node.clone());
            self.crdt.interpret_node(&remote_node);
            self.handle_pending_nodes();
        } else {
            self.pending_nodes.push(remote_node);
        }
    }
    
    pub fn handle_pending_nodes(&mut self) {
        let mut changed = false;
        while !self.pending_nodes.is_empty() {
            let mut new_pending_nodes = vec![];
            for node in self.pending_nodes.drain(..) {
                let struct_valid = self.hash_graph.is_structurally_valid(&node);
                if struct_valid {
                    if self.crdt.is_sem_valid(&node, &self.hash_graph) {
                        self.hash_graph.add_node(node.clone());
                        self.crdt.interpret_node(&node);
                        changed = true;
                    }
                } else {
                    new_pending_nodes.push(node);
                }
            }
            if !changed {
                break;
            }
            self.pending_nodes = new_pending_nodes;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::bft_crdts::bft_orset::BFTORSet;
    use crate::bft_crdts::bft_rga::BFTRGA;
    use super::*;
    
    #[test]
    fn test_bft_orset_handler() {
        let orset = BFTORSet::new();
        let mut handler = BFTCRDTHandler::new(orset);
        let add_op = handler.crdt.add("a");
        handler.handle_local_op(add_op.clone());
        assert_eq!(handler.crdt.get_set().len(), 1);
        assert_eq!(handler.crdt.is_in("a"), true);
        let add_op = handler.crdt.add("b");
        handler.handle_local_op(add_op.clone());
        assert_eq!(handler.crdt.get_set().len(), 2);
        assert_eq!(handler.crdt.is_in("b"), true);
        assert_eq!(handler.crdt.is_in("a"), true);
        let remove_op = handler.crdt.remove_elem("a");
        handler.handle_local_op(remove_op.clone());
        assert_eq!(handler.crdt.get_set().len(), 1);
        assert_eq!(handler.crdt.is_in("a"), false);
        assert_eq!(handler.crdt.is_in("b"), true);
    }
    
    #[test]
    fn test_bft_rga_handler() {
        let rga = BFTRGA::new();
        let mut handler = BFTCRDTHandler::new(rga);
        let insert_op = handler.crdt.insert(0, "a", "0").unwrap();
        handler.handle_local_op(insert_op.clone());
        let insert_op = handler.crdt.insert(1, "b", "1").unwrap();
        handler.handle_local_op(insert_op.clone());
        let insert_op = handler.crdt.insert(2, "c", "2").unwrap();
        handler.handle_local_op(insert_op.clone());
        let insert_op = handler.crdt.insert(1, "d", "3").unwrap();
        handler.handle_local_op(insert_op.clone());
        assert_eq!(handler.crdt.get_list(), vec!["a", "d", "b", "c"]);
        let delete_op = handler.crdt.delete(1).unwrap();
        handler.handle_local_op(delete_op.clone());
        assert_eq!(handler.crdt.get_list(), vec!["a", "b", "c"]);
    }
}