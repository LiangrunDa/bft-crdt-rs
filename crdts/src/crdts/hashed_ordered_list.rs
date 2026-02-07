use std::collections::HashMap;
use std::hash::Hash;

use slab::Slab;

type Element<I, V> = (I, V, bool); // (id, value, is_deleted)

#[derive(Debug)]
pub(crate) struct Node<I, V> {
    pub(crate) elem: Element<I, V>,
    prev: Option<usize>,
    pub(crate) next: Option<usize>,
}

pub struct OrderedList<I, V>
where
    I: PartialEq + Eq + Hash + Clone + PartialOrd,
    V: PartialEq + Eq + Hash + Clone,
{
    pub(crate) nodes: Slab<Node<I, V>>,
    pub(crate) head: Option<usize>,
    tail: Option<usize>,
    // id -> first occurrence handle (keeps semantics of "first match" for delete/get)
    index: HashMap<I, usize>,
}

impl<I, V> OrderedList<I, V>
where
    I: PartialEq + Eq + Hash + Clone + PartialOrd,
    V: PartialEq + Eq + Hash + Clone,
{
    pub fn new() -> Self {
        Self {
            nodes: Slab::new(),
            head: None,
            tail: None,
            index: HashMap::new(),
        }
    }
    
    pub fn insert_by_id(&mut self, id: I, value: V, after: Option<I>) -> Option<()> {
        let to_insert_id = id.clone();
        let new_handle = self.alloc_node((id, value, false));

        // keep "first occurrence" index
        self.index.entry(to_insert_id).or_insert(new_handle);

        // Determine the scan start position
        let mut cur = match after {
            None => self.head,
            Some(after_id) => {
                let anchor = self.find_first_handle_by_id(&after_id)?;
                self.nodes[anchor].next
            }
        };

        // Scan forward: insert BEFORE the first node whose id < new_id
        while let Some(h) = cur {
            let cur_id = &self.nodes[h].elem.0;
            let new_id = &self.nodes[new_handle].elem.0;
            if cur_id < new_id {
                break;
            }
            cur = self.nodes[h].next;
        }

        // Insert before `cur` (or push_back if cur == None)
        self.insert_before(cur, new_handle);
        Some(())
    }

    pub fn delete_by_id(&mut self, id: I) -> Option<()> {
        let h = self.find_first_handle_by_id(&id)?;
        self.nodes[h].elem.2 = true;
        Some(())
    }

    pub fn get_by_id(&self, id: I) -> Option<Element<I, V>> {
        let h = self.find_first_handle_by_id_ref(&id)?;
        Some(self.nodes[h].elem.clone())
    }

    pub fn get_by_idx(&self, idx: usize) -> Option<Element<I, V>> {
        let mut count = 0;
        let mut cur = self.head;
        while let Some(h) = cur {
            let e = &self.nodes[h].elem;
            if !e.2 {
                if count == idx {
                    return Some(e.clone());
                }
                count += 1;
            }
            cur = self.nodes[h].next;
        }
        None
    }

    pub fn delete_by_idx(&mut self, idx: usize) -> Option<()> {
        let mut count = 0;
        let mut cur = self.head;
        while let Some(h) = cur {
            let e_deleted = self.nodes[h].elem.2;
            if !e_deleted {
                if count == idx {
                    self.nodes[h].elem.2 = true;
                    return Some(());
                }
                count += 1;
            }
            cur = self.nodes[h].next;
        }
        None
    }

    pub fn get_list(&self) -> Vec<V> {
        let mut out = Vec::new();
        let mut cur = self.head;
        while let Some(h) = cur {
            let e = &self.nodes[h].elem;
            if !e.2 {
                out.push(e.1.clone());
            }
            cur = self.nodes[h].next;
        }
        out
    }

    fn alloc_node(&mut self, elem: Element<I, V>) -> usize {
        self.nodes.insert(Node {
            elem,
            prev: None,
            next: None,
        })
    }

    /// Find first handle for an id, matching your LinkedList traversal semantics.
    /// - Fast path: HashMap (first occurrence)
    /// - If stale/missing, fall back to traversal (keeps correctness if needed)
    fn find_first_handle_by_id(&self, id: &I) -> Option<usize> {
        if let Some(&h) = self.index.get(id) {
            return Some(h);
        }
        // fallback traversal (rare if index kept consistent)
        self.find_first_handle_by_traversal(id)
    }

    fn find_first_handle_by_id_ref(&self, id: &I) -> Option<usize> {
        if let Some(&h) = self.index.get(id) {
            return Some(h);
        }
        self.find_first_handle_by_traversal(id)
    }

    fn find_first_handle_by_traversal(&self, id: &I) -> Option<usize> {
        let mut cur = self.head;
        while let Some(h) = cur {
            if &self.nodes[h].elem.0 == id {
                return Some(h);
            }
            cur = self.nodes[h].next;
        }
        None
    }

    /// Insert `new_h` before `before_h` (or append if before_h is None).
    fn insert_before(&mut self, before_h: Option<usize>, new_h: usize) {
        match before_h {
            None => self.push_back_handle(new_h),
            Some(bh) => {
                let prev = self.nodes[bh].prev;

                self.nodes[new_h].next = Some(bh);
                self.nodes[new_h].prev = prev;

                self.nodes[bh].prev = Some(new_h);

                match prev {
                    Some(ph) => self.nodes[ph].next = Some(new_h),
                    None => self.head = Some(new_h),
                }
            }
        }
    }

    fn push_back_handle(&mut self, h: usize) {
        self.nodes[h].next = None;
        self.nodes[h].prev = self.tail;

        match self.tail {
            Some(t) => self.nodes[t].next = Some(h),
            None => self.head = Some(h),
        }
        self.tail = Some(h);
    }
}

impl<I, V> Clone for OrderedList<I, V>
where
    I: PartialEq + Eq + Hash + Clone + PartialOrd,
    V: PartialEq + Eq + Hash + Clone,
{
    fn clone(&self) -> Self {
        let mut out = OrderedList::new();

        // old_handle -> new_handle
        let mut remap: HashMap<usize, usize> = HashMap::new();

        // rebuild nodes in list order
        let mut cur = self.head;
        while let Some(h) = cur {
            let new_h = out.alloc_node(self.nodes[h].elem.clone());
            remap.insert(h, new_h);

            // link with tail
            out.push_back_handle(new_h);

            cur = self.nodes[h].next;
        }

        // rebuild index
        let mut cur2 = self.head;
        while let Some(h) = cur2 {
            let id = self.nodes[h].elem.0.clone();
            let nh = *remap.get(&h).unwrap();
            out.index.entry(id).or_insert(nh);
            cur2 = self.nodes[h].next;
        }

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert() {
        let mut list = OrderedList::new();
        list.insert_by_id(1, "a", None);
        list.insert_by_id(2, "b", Some(1));
        list.insert_by_id(3, "c", Some(2));

        assert_eq!(list.get_list(), vec!["a", "b", "c"]);
    }

    #[test]
    fn test_delete() {
        let mut list = OrderedList::new();
        list.insert_by_id(1, "a", None);
        list.insert_by_id(2, "b", Some(1));
        list.insert_by_id(3, "c", Some(2));
        list.delete_by_id(2);
        assert_eq!(list.get_list(), vec!["a", "c"]);
    }

    #[test]
    fn test_insert_at_the_same_position() {
        let mut list = OrderedList::new();
        list.insert_by_id(1, "a", None);
        list.insert_by_id(2, "b", Some(1));
        list.insert_by_id(3, "c", Some(1));
        assert_eq!(list.get_list(), vec!["a", "c", "b"]);
    }

    #[test]
    fn concurrent_insert_commute() {
        let mut list = OrderedList::new();
        list.insert_by_id(1, "a", None);
        list.insert_by_id(2, "b", Some(1));
        list.insert_by_id(3, "c", Some(1));

        let mut list2 = OrderedList::new();
        list2.insert_by_id(1, "a", None);
        list2.insert_by_id(3, "c", Some(1));
        list2.insert_by_id(2, "b", Some(1));

        assert_eq!(list.get_list(), list2.get_list());
    }

    #[test]
    fn concurrent_delete_commute() {
        let mut list = OrderedList::new();
        list.insert_by_id(1, "a", None);
        list.insert_by_id(2, "b", Some(1));

        let mut list2 = list.clone();

        list.delete_by_id(1);
        list.delete_by_id(2);
        list2.delete_by_id(2);
        list2.delete_by_id(1);

        assert_eq!(list.get_list(), list2.get_list());
    }

    #[test]
    fn concurrent_insert_delete_commute() {
        let mut list = OrderedList::new();
        list.insert_by_id(1, "a", None);
        list.insert_by_id(2, "b", Some(1));
        list.insert_by_id(3, "c", Some(2));

        let mut list2 = list.clone();

        list.insert_by_id(2, "d", Some(1));
        list.delete_by_id(1);

        list2.delete_by_id(1);
        list2.insert_by_id(2, "d", Some(1));

        assert_eq!(list.get_list(), list2.get_list());
    }
}
