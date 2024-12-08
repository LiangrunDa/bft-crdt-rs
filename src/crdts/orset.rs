use std::collections::HashMap;
use std::collections::HashSet;
use crate::crdts::crdt::CRDT;
use std::hash::Hash;
use std::cmp::Eq;

pub enum ORSetOp<E, I> {
    Add(E, I),
    Remove(E, Vec<I>),
}

pub struct ORSet<E: Eq + Hash + Clone, I: PartialEq + Eq + Hash + Clone> {
    elements: HashMap<E, HashSet<I>>,
}

impl<E: Eq + Hash + Clone, I: PartialEq + Eq + Hash + Clone> ORSet<E, I> {
    pub fn new() -> Self {
        ORSet {
            elements: HashMap::new(),
        }
    }

    pub fn add(&mut self, e: E, i: I) -> ORSetOp<E, I> {
        ORSetOp::Add(e, i)
    }

    pub fn remove(&mut self, e: E, ids: Vec<I>) -> ORSetOp<E, I> {
        ORSetOp::Remove(e, ids)
    }

    pub fn is_in(&self, e: E) -> bool {
        self.elements.get(&e).map_or(false, |ids| !ids.is_empty())
    }

    pub fn get_ids(&self, e: E) -> HashSet<I> {
        self.elements.get(&e).cloned().unwrap_or_default()
    }

}

impl<E: Eq + Hash + Clone, I: PartialEq + Eq + Hash + Clone> CRDT<ORSetOp<E, I>> for ORSet<E, I> {
    fn interpret_op(&mut self, op: &ORSetOp<E, I>) {
        match op {
            ORSetOp::Add(e, i) => {
                self.elements.entry(e.clone()).or_insert(HashSet::new()).insert(i.clone());
            }
            ORSetOp::Remove(e, ids) => {
                self.elements.entry(e.clone()).or_insert(HashSet::new()).retain(|id| !ids.contains(id));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_orset() {
        let mut orset = ORSet::new();
        let add_op = orset.add("a", "1");
        orset.interpret_op(&add_op);
        let add_op = orset.add("a", "2");
        orset.interpret_op(&add_op);
        let remove_op = orset.remove("a", vec!["1"]);
        orset.interpret_op(&remove_op);
        assert!(orset.is_in("a"));
        assert!(!orset.is_in("b"));
        assert_eq!(orset.get_ids("a"), HashSet::from(["2"]));
        assert_eq!(orset.get_ids("b"), HashSet::new());
    }

    #[test]
    fn test_remove_not_in() {
        let mut orset = ORSet::new();
        let remove_op = orset.remove("a", vec!["1"]);
        orset.interpret_op(&remove_op);
        assert!(!orset.is_in("a"));

        let add_op = orset.add("a", "1");
        orset.interpret_op(&add_op);
        assert!(orset.is_in("a"));
    }
}
