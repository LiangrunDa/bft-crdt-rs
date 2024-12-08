use std::hash::Hash;
use crate::crdts::linked_list::LinkedList;
use crate::crdts::linked_list::CursorMut;

type Element<I, V> = (I, V, bool);

pub struct OrderedList<I: PartialEq + Eq + Hash + Clone + PartialOrd, V: PartialEq + Eq + Hash + Clone> {
    elements: LinkedList<Element<I, V>>,
}

impl<I: PartialEq + Eq + Hash + Clone + PartialOrd, V: PartialEq + Eq + Hash + Clone> OrderedList<I, V> {
    pub fn new() -> Self {
        OrderedList {
            elements: LinkedList::new(),
        }
    }

    pub fn insert(&mut self, id: I, value: V, after: Option<I>) -> Option<()> {
        let to_insert = (id, value, false);
        let mut cursor = self.elements.cursor_mut();
        cursor.move_next();
        match after {
            None => {
                self.elements.push_back(to_insert);
                Some(())
            }
            Some(after_id) => {
                let mut found = false;                
                while let Some(element) = cursor.current() {
                    if element.0 == after_id {
                        found = true;
                        cursor.move_next();
                        break;
                    }
                    cursor.move_next(); 
                }

                if !found {
                    return None;
                }

                while let Some(element) = cursor.current() {
                    if element.0 < to_insert.0 {
                        let mut element_temp_list = LinkedList::new();
                        element_temp_list.push_back(to_insert.clone());
                        cursor.splice_before(element_temp_list);
                        return Some(());
                    }
                    cursor.move_next();
                }
                self.elements.push_back(to_insert);
                Some(())
            }
        }
    }

    pub fn delete(&mut self, id: I) -> Option<()> {
        for element in self.elements.iter_mut() {
            if element.0 == id {
                element.2 = true;
                return Some(());
            }
        }
        None
    }

    pub fn get(&self, idx: usize) -> Option<V> {
        let mut count = 0;
        for element in self.elements.iter() {
            if !element.2 {
                if count == idx {
                    return Some(element.1.clone());
                }
                count += 1;
            }
        }
        None
    }

    pub fn get_list(&self) -> Vec<V> {
        self.elements.iter().filter(|e| !e.2).map(|e| e.1.clone()).collect()
    }
}

impl<I: PartialEq + Eq + Hash + Clone + PartialOrd, V: PartialEq + Eq + Hash + Clone> Clone for OrderedList<I, V> {
    fn clone(&self) -> Self {
        OrderedList {
            elements: self.elements.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_insert() {
        let mut list = OrderedList::new();
        list.insert(1, "a", None);
        list.insert(2, "b", Some(1));
        list.insert(3, "c", Some(2));
        
        assert_eq!(list.get_list(), vec!["a", "b", "c"]);
    }

    #[test]
    fn test_delete() {
        let mut list = OrderedList::new();
        list.insert(1, "a", None);
        list.insert(2, "b", Some(1));
        list.insert(3, "c", Some(2));
        list.delete(2);
        assert_eq!(list.get_list(), vec!["a", "c"]);
    }

    #[test]
    fn test_insert_at_the_same_position() {
        let mut list = OrderedList::new();
        list.insert(1, "a", None);
        list.insert(2, "b", Some(1));
        list.insert(3, "c", Some(1));
        assert_eq!(list.get_list(), vec!["a", "c", "b"]);
    }

    #[test]
    fn concurrent_insert_commute() {
        let mut list = OrderedList::new();
        list.insert(1, "a", None);
        list.insert(2, "b", Some(1));
        list.insert(3, "c", Some(1));
        
        let mut list2 = OrderedList::new();
        list2.insert(1, "a", None);
        list2.insert(3, "c", Some(1));
        list2.insert(2, "b", Some(1));

        assert_eq!(list.get_list(), list2.get_list());
    }

    #[test]
    fn concurrent_delete_commute() {
        let mut list = OrderedList::new();
        list.insert(1, "a", None);
        list.insert(2, "b", Some(1));

        let mut list2 = list.clone();

        list.delete(1);
        list.delete(2);
        list2.delete(2);
        list2.delete(1);

        assert_eq!(list.get_list(), list2.get_list());
    }

    #[test]
    fn concurrent_insert_delete_commute() {
        let mut list = OrderedList::new();
        list.insert(1, "a", None);
        list.insert(2, "b", Some(1));
        list.insert(3, "c", Some(2));

        let mut list2 = list.clone();

        list.insert(2, "d", Some(1));
        list.delete(1);

        list2.delete(1);
        list2.insert(2, "d", Some(1));


        assert_eq!(list.get_list(), list2.get_list());
    }
}