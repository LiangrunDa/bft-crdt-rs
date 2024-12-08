use crate::crdts::crdt::CRDT;
use crate::crdts::ordered_list::OrderedList;
use std::hash::Hash;

pub enum RGAOp<I, V>
where
    I: PartialEq + Eq + Hash + Clone,
    V: PartialEq + Eq + Hash + Clone,
{
    Insert(I, V, Option<I>),
    Delete(I),
}

pub struct RGA<I, V>
where
    I: PartialEq + Eq + Hash + Clone + PartialOrd,
    V: PartialEq + Eq + Hash + Clone,
{
    elements: OrderedList<I, V>,
}

impl<I, V> RGA<I, V>
where
    I: PartialEq + Eq + Hash + Clone + PartialOrd,
    V: PartialEq + Eq + Hash + Clone,
{
    pub fn new() -> Self {
        RGA {
            elements: OrderedList::new(),
        }
    }

    pub fn get(&self, idx: usize) -> Option<V> {
        self.elements.get_by_idx(idx).map(|(_, v, _)| v)
    }

    pub fn get_list(&self) -> Vec<V> {
        self.elements.get_list()
    }

    pub fn insert(&mut self, idx: usize, value: V, elem_id: I) -> Option<RGAOp<I, V>> {
        if idx == 0 {
            Some(RGAOp::Insert(elem_id, value, None))
        } else {
            let prev = self.elements.get_by_idx(idx - 1);
            if let Some((id, _, _)) = prev {
                Some(RGAOp::Insert(elem_id, value, Some(id.clone())))
            } else {
                None
            }
        }
    }
    
    pub fn delete(&mut self, idx: usize) -> Option<RGAOp<I, V>> {
        let elem = self.elements.get_by_idx(idx);
        if let Some((id, _, _)) = elem {
            Some(RGAOp::Delete(id.clone()))
        } else {
            None
        }
    }
}

impl<I, V> CRDT<RGAOp<I, V>> for RGA<I, V>
where
    I: PartialEq + Eq + Hash + Clone + PartialOrd,
    V: PartialEq + Eq + Hash + Clone,
{
    fn interpret_op(&mut self, op: &RGAOp<I, V>) {
        match op {
            RGAOp::Insert(id, value, after) => self.elements.insert_by_id(id.clone(), value.clone(), after.clone()),
            RGAOp::Delete(id) => self.elements.delete_by_id(id.clone()),
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert() {
        let mut rga = RGA::new();
        let insert_op = rga.insert(0, "a", 0).unwrap();
        rga.interpret_op(&insert_op);
        let insert_op = rga.insert(1, "b", 1).unwrap();
        rga.interpret_op(&insert_op);
        let insert_op = rga.insert(2, "c", 2).unwrap();
        rga.interpret_op(&insert_op);
        let insert_op = rga.insert(1, "d", 3).unwrap();
        rga.interpret_op(&insert_op);
        assert_eq!(rga.get_list(), vec!["a", "d", "b", "c"]);
    }
    
    #[test]
    fn test_delete() {
        let mut rga = RGA::new();
        let insert_op = rga.insert(0, "a", 0).unwrap();
        rga.interpret_op(&insert_op);
        let insert_op = rga.insert(1, "b", 1).unwrap();
        rga.interpret_op(&insert_op);
        let insert_op = rga.insert(2, "c", 2).unwrap();
        rga.interpret_op(&insert_op);
        let delete_op = rga.delete(1).unwrap();
        rga.interpret_op(&delete_op);
        assert_eq!(rga.get_list(), vec!["a", "c"]);
    }
    
    #[test]
    fn test_insert_invalid() {
        let mut rga = RGA::new();
        let insert_op = rga.insert(1, "a", 0);
        assert!(insert_op.is_none());
    }
}