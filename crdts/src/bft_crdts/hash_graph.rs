use std::collections::HashMap;
use sha3::{Digest, Sha3_256};
use hex;

pub type HashType = String;

#[derive(Debug, Clone)]
pub struct Node<T: Into<Vec<u8>> + Clone> {
    pub predecessors: Vec<HashType>,
    pub value: T,
}

impl <T: Into<Vec<u8>> + Clone> Node<T> {
    pub fn get_hash(&self) -> HashType {
        let mut hasher = Sha3_256::new();
        for pred in &self.predecessors {
            hasher.update(pred);
        }
        let value_bytes = self.value.clone().into();
        hasher.update(value_bytes);
        let hash = hasher.finalize().to_vec();
        hex::encode(hash)
    }
}

pub struct HashGraph<T: Into<Vec<u8>> + Clone> {
    nodes: HashMap<HashType, Node<T>>,
    heads: Vec<HashType>,
}

impl<T: Into<Vec<u8>> + Clone> HashGraph<T> {
    pub fn new() -> Self {
        HashGraph {
            nodes: HashMap::new(),
            heads: vec![],
        }
    }
    
    

    pub fn is_structurally_valid(&self, node: &Node<T>) -> bool {
        for pred in &node.predecessors {
            if !self.nodes.contains_key(pred) {
                return false;
            }
        }
        true
    }
    
    pub fn add_remote_node(&mut self, node: Node<T>) {
        let hash = node.get_hash();
        self.nodes.insert(hash.clone(), node);
    }
    
    pub fn add_local_node(&mut self, value: T) -> Option<HashType> {
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
    
    pub fn is_ancestor(&self, ancestor: &HashType, descendant: &HashType) -> bool {
        if ancestor == descendant {
            return true;
        }
        let descendant_node = self.get_node(descendant).unwrap();
        for pred in &descendant_node.predecessors {
            if self.is_ancestor(ancestor, pred) {
                return true;
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
        let hash = graph.add_local_node(b"test".to_vec());
        let mut hasher = Sha3_256::new();
        hasher.update(b"test");
        let expected_hash = hex::encode(hasher.finalize().to_vec());
        assert_eq!(hash.unwrap(), expected_hash);
        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(graph.get_node(&expected_hash).unwrap().value, b"test");
    }
    
    #[test]
    fn test_add_multiple_nodes() {
        let mut graph: HashGraph<Vec<u8>> = HashGraph::new();
        let hash1 = graph.add_local_node(b"test1".to_vec());
        let hash2 = graph.add_local_node(b"test2".to_vec());
        let mut hasher = Sha3_256::new();
        hasher.update(b"test1");
        let expected_hash1 = hex::encode(hasher.finalize().to_vec());
        hasher = Sha3_256::new();
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

