use std::collections::HashMap;
use sha3::{Digest, Sha3_256};

type Hash = Vec<u8>;
pub(crate) struct Node<T: AsRef<[u8]>> {
    predecessors: Vec<Hash>,
    value: T,
}

pub(crate) struct HashGraph<T: AsRef<[u8]>> {
    nodes: HashMap<Hash, Node<T>>,
}

impl<T: AsRef<[u8]>> HashGraph<T> {
    pub(crate) fn new() -> Self {
        HashGraph {
            nodes: HashMap::new(),
        }
    }

    pub(crate) fn is_structurally_valid(&self, node: &Node<T>) -> bool {
        for pred in &node.predecessors {
            if !self.nodes.contains_key(pred) {
                return false;
            }
        }
        true
    }

    pub(crate) fn add_node(&mut self, predecessors: Vec<Hash>, value: T) -> bool {
        let node = Node {
            predecessors,
            value,
        };
        if !self.is_structurally_valid(&node) {
            return false;
        }

        let mut hasher = Sha3_256::new();
        for pred in &node.predecessors {
            hasher.update(pred);
        }
        hasher.update(node.value.as_ref());
        let hash = hasher.finalize().to_vec();
        self.nodes.insert(hash.clone(), node);
        true
    }

    pub(crate) fn get_node(&self, hash: &Hash) -> Option<&Node<T>> {
        self.nodes.get(hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_node() {
        let mut graph: HashGraph<&[u8]> = HashGraph::new();
        let success = graph.add_node(vec![], "test".as_bytes());
        assert!(success);
        let mut hasher = Sha3_256::new();
        hasher.update("test".as_bytes());
        let expected_hash = hasher.finalize().to_vec();
        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(graph.get_node(&expected_hash).unwrap().value, "test".as_bytes());
    }
}

