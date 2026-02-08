use std::collections::{HashMap, VecDeque};
use std::fmt::{Debug, Display};
use std::cell::RefCell;
use sha2::{Digest, Sha256};
use hex;
use tracing::trace;
use ahash::AHashSet;
use crate::serialize::Serialize;

// Use fixed-size array for zero-allocation hash storage
pub type HashType = [u8; 32];

#[derive(Clone)]
pub struct Node<T: Serialize + Clone> {
    pub predecessors: Vec<HashType>,
    pub value: T,
    pub cached_hash: RefCell<Option<HashType>>,
}

impl <T: Serialize + Clone + Display> Display for Node<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let hash = self.get_hash();
        // Convert first 4 bytes to hex for display
        let hash_str = hex::encode(&hash[0..4]);
        write!(f, "Node {{ preds: [..], hash: {}, value: {} }}", hash_str, self.value)
    }
}

impl <T: Serialize + Clone> Debug for Node<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let hash = self.get_hash();
        // Display first 4 bytes in hex
        write!(f, "{}", hex::encode(&hash[0..4]))
    }
}

impl <T: Serialize + Clone> Node<T> {
    pub fn get_hash(&self) -> HashType {
        // Check if hash is already cached
        if let Some(cached) = self.cached_hash.borrow().as_ref() {
            return *cached;  // Copy is cheap for [u8; 32]
        }
        
        // Compute hash using SHA-256
        let mut hasher = Sha256::new();
        
        // Sort by reference to avoid cloning the entire vector
        let mut pred_refs: Vec<&HashType> = self.predecessors.iter().collect();
        pred_refs.sort_unstable();
        
        for pred in pred_refs {
            hasher.update(pred);
        }
        
        // Use hash_into for zero-allocation hashing!
        self.value.hash_into(&mut hasher);
        
        // Get hash as fixed-size array - zero allocations!
        let hash_result = hasher.finalize();
        let hash: HashType = hash_result.into();
        
        // Cache the computed hash
        *self.cached_hash.borrow_mut() = Some(hash);
        
        hash
    }
}

pub struct HashGraph<T: Serialize + Clone> {
    pub nodes: HashMap<HashType, Node<T>>,
    heads: Vec<HashType>,
}

impl<T: Serialize + Clone> HashGraph<T> {
    pub fn new() -> Self {
        HashGraph {
            nodes: HashMap::new(),
            heads: vec![],
        }
    }
    
    pub fn _has_cycle(&self) -> bool {
        // from node hash to in_degree
        let mut in_degree_map = HashMap::new();
        let nodes = self.nodes.clone();
        for node in nodes.clone() {
            for pred in node.1.predecessors {
                in_degree_map.insert(pred, in_degree_map.get(&pred).unwrap_or(&0) + 1);
            }
        }
        
        let mut queue = VecDeque::new();
        for node in nodes.clone() {
            let in_degree = in_degree_map.get(&node.0);
            match in_degree {
                Some(d) => {
                    if *d == 0 {
                        queue.push_back(node.0);
                    }
                }
                None => queue.push_back(node.0)
            }
            
        }
        for (hash, in_degree) in in_degree_map.clone() {
            if in_degree == 0 {
                queue.push_back(hash);
            }
        }
        let mut count = 0;
        while let Some(h) = queue.pop_front() {
            count += 1;
            let node = nodes.get(&h).unwrap();
            for pred in node.predecessors.clone() {
                in_degree_map.insert(pred, in_degree_map.get(&pred).unwrap() - 1);
                if *in_degree_map.get(&pred).unwrap() == 0 {
                    queue.push_back(pred);
                };
            }
        }
        count != nodes.len()
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
        
        // Remove predecessors from heads
        self.heads.retain(|head| !node.predecessors.contains(head));
        
        // Add new node to heads
        self.heads.push(hash);
        
        // Add node to graph
        self.nodes.insert(hash, node);
    }
    
    pub fn add_value_with_head_preds(&mut self, value: T) -> Option<HashType> {
        let node = Node {
            predecessors: self.heads.clone(),
            value,
            cached_hash: RefCell::new(None),
        };
        
        let hash = node.get_hash();
        self.nodes.insert(hash, node);
        self.heads = vec![hash];
        Some(hash)
    }

    pub fn get_node(&self, hash: &HashType) -> Option<&Node<T>> {
        self.nodes.get(hash)
    }
    
    pub fn is_ancestor(&self, ancestor: &HashType, descendant: &Node<T>) -> bool {
        self.is_ancestor_bfs(ancestor, descendant)
    }
    
    fn is_ancestor_recurse(&self, ancestor: &HashType, descendant: &Node<T>, visited: &mut AHashSet<HashType>) -> bool {
        let current_hash = descendant.get_hash();
        
        // if we have visited this node and returned, it means ancestor is not an ancestor of this node and we don't need to waste time on this node
        if visited.contains(&current_hash) {
            return false;
        }
        
        visited.insert(current_hash);
        
        if *ancestor == descendant.get_hash() {
            return true;
        }
        if descendant.predecessors.is_empty() {
            // we have reached the first node
            return false;
        }
        for pred in &descendant.predecessors {
            let pred_node = self.get_node(pred);
            match pred_node {
                Some(node) => {
                    // if 'ancestor' is an ancestor of any predecessor, return true
                    if self.is_ancestor_recurse(ancestor, node, visited) {
                        return true;
                    }
                }
                None => return false,
            }
        }
        // if 'ancestor' is not an ancestor of any predecessor (or this is a node without any predecessor, 
        // i.e the first node), return false
        false
    }
    
    fn is_ancestor_dfs(&self, ancestor: &HashType, descendant: &Node<T>) -> bool {
        let mut visited = AHashSet::new();
        let mut stack = Vec::new();
        
        stack.push(descendant);
        
        while let Some(current_node) = stack.pop() {
            let current_hash = current_node.get_hash();
            
            if visited.contains(&current_hash) {
                continue;
            }
            
            visited.insert(current_hash);
            
            if *ancestor == current_hash {
                return true;
            }
            
            if current_node.predecessors.is_empty() {
                continue;
            }
            
            for pred in &current_node.predecessors {
                if let Some(pred_node) = self.get_node(pred) {
                    stack.push(pred_node);
                } else {
                    return false;
                }
            }
        }
        
        false
    }

    fn is_ancestor_bfs(&self, ancestor: &HashType, descendant: &Node<T>) -> bool {
        let mut visited = AHashSet::new();
        let mut queue = VecDeque::new();
        
        queue.push_back(descendant);
        
        while let Some(current_node) = queue.pop_front() {
            let current_hash = current_node.get_hash();
            
            if !visited.insert(current_hash) {
                continue;
            }
            
            if *ancestor == current_hash {
                return true;
            }
            
            if current_node.predecessors.is_empty() {
                continue;
            }
            
            for pred in &current_node.predecessors {
                if let Some(pred_node) = self.get_node(pred) {
                    queue.push_back(pred_node);
                } else {
                    return false;
                }
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
        let expected_hash: HashType = hasher.finalize().into();
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
        let expected_hash1: HashType = hasher.finalize().into();
        hasher = Sha256::new();
        hasher.update(&expected_hash1);
        hasher.update(b"test2");
        let expected_hash2: HashType = hasher.finalize().into();
        
        assert_eq!(hash1.unwrap(), expected_hash1);
        assert_eq!(hash2.unwrap(), expected_hash2);
        assert_eq!(graph.nodes.len(), 2);
        assert_eq!(graph.get_node(&expected_hash1).unwrap().value, b"test1");
        assert_eq!(graph.get_node(&expected_hash2).unwrap().value, b"test2");
        assert_eq!(graph.get_node(&expected_hash2).unwrap().predecessors, vec![expected_hash1]);
    }
}

