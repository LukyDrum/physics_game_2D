use std::collections::{linked_list::Iter, LinkedList};

pub struct LinkedLinkedList<'a, T>(LinkedList<&'a LinkedList<T>>);

impl<T> Default for LinkedLinkedList<'_, T> {
    fn default() -> Self {
        LinkedLinkedList(LinkedList::new())
    }
}

impl<'a, T> LinkedLinkedList<'a, T> {
    pub fn push_back(&mut self, list: &'a LinkedList<T>) {
        self.0.push_back(list);
    }

    pub fn iter(&self) -> LLIterator<T> {
        LLIterator {
            outer: self.0.iter(),
            inner: None,
        }
    }
}

pub struct LLIterator<'a, T> {
    outer: Iter<'a, &'a LinkedList<T>>,
    inner: Option<Iter<'a, T>>,
}

impl<'a, T> Iterator for LLIterator<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(inner_iter) = &mut self.inner {
            if let Some(item) = inner_iter.next() {
                return Some(item);
            }
        }

        if let Some(list) = self.outer.next() {
            self.inner = Some(list.iter());
            return self.next();
        }

        None
    }
}
