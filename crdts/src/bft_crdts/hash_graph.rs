use std::collections::HashMap;
use std::fmt::Debug;
use sha2::{Digest, Sha256};
use hex;
use tracing::trace;
use crate::serialize::Serialize;

pub type HashType = String;

#[derive(Clone)]
pub struct Node<T: Serialize + Clone> {
    pub predecessors: Vec<HashType>,
    pub value: T,
}

impl <T: Serialize + Clone> Debug for Node<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let hash = self.get_hash();
        write!(f, "{:?}", &hash[0..8])
    }
}

impl <T: Serialize + Clone> Node<T> {
    pub fn get_hash(&self) -> HashType {
        let mut hasher = Sha256::new();
        let mut sorted_preds = self.predecessors.clone();
        sorted_preds.sort();
        for pred in sorted_preds {
            hasher.update(pred.as_bytes());
        }
        let value_bytes = self.value.clone().to_bytes();
        hasher.update(value_bytes);
        let hash = hasher.finalize().to_vec();
        hex::encode(hash)
    }
}

pub struct HashGraph<T: Serialize + Clone> {
    pub nodes: HashMap<HashType, Node<T>>,
    pub heads: Vec<HashType>,
}

impl<T: Serialize + Clone> HashGraph<T> {
    pub fn new() -> Self {
        HashGraph {
            nodes: HashMap::new(),
            heads: vec![],
        }
    }
    
    pub fn is_structurally_valid(&self, node: &Node<T>) -> bool {
        trace!("Begin of is_structurally_valid");
        for pred in &node.predecessors {
            if !self.nodes.contains_key(pred) {
                return false;
            }
        }
        trace!("End of is_structurally_valid");
        true
    }
    
    pub fn add_node(&mut self, node: Node<T>) {
        let hash = node.get_hash();
        self.nodes.insert(hash.clone(), node);
    }
    
    pub fn add_value_with_head_preds(&mut self, value: T) -> Option<HashType> {
        let node = Node {
            predecessors: self.heads.clone(),
            value,
        };
        
        let hash = node.get_hash();
        self.nodes.insert(hash.clone(), node);
        self.heads = vec![hash.clone()];
        Some(hash)
    }

    pub fn get_node(&self, hash: &HashType) -> Option<&Node<T>> {
        self.nodes.get(hash)
    }
    
    pub fn is_ancestor(&self, ancestor: &HashType, descendant: &Node<T>) -> bool {
        if *ancestor == descendant.get_hash() {
            return true;
        }
        for pred in &descendant.predecessors {
            let pred_node = self.get_node(pred);
            match pred_node {
                Some(node) => {
                    if self.is_ancestor(ancestor, node) {
                        return true;
                    }
                }
                None => return false,
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_node() {
        let mut graph: HashGraph<Vec<u8>> = HashGraph::new();
        let hash = graph.add_value_with_head_preds(b"test".to_vec());
        let mut hasher = Sha256::new();
        hasher.update(b"test");
        let expected_hash = hex::encode(hasher.finalize().to_vec());
        assert_eq!(hash.unwrap(), expected_hash);
        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(graph.get_node(&expected_hash).unwrap().value, b"test");
    }
    
    #[test]
    fn test_add_multiple_nodes() {
        let mut graph: HashGraph<Vec<u8>> = HashGraph::new();
        let hash1 = graph.add_value_with_head_preds(b"test1".to_vec());
        let hash2 = graph.add_value_with_head_preds(b"test2".to_vec());
        let mut hasher = Sha256::new();
        hasher.update(b"test1");
        let expected_hash1 = hex::encode(hasher.finalize().to_vec());
        hasher = Sha256::new();
        hasher.update(expected_hash1.clone());
        hasher.update(b"test2");
        let expected_hash2 = hex::encode(hasher.finalize().to_vec());
        
        assert_eq!(hash1.unwrap(), expected_hash1);
        assert_eq!(hash2.unwrap(), expected_hash2);
        assert_eq!(graph.nodes.len(), 2);
        assert_eq!(graph.get_node(&expected_hash1).unwrap().value, b"test1");
        assert_eq!(graph.get_node(&expected_hash2).unwrap().value, b"test2");
        assert_eq!(graph.get_node(&expected_hash2).unwrap().predecessors, vec![expected_hash1]);
    }
}

