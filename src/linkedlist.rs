// The doubly linked list is designed to have the forward link as a strong/owning link
// and the backward link as weak/referencing link.
//
//                       [PTR, ptr]
//                         ^    ^
//   +---------------------+    +-------------------+
//   v                                              v
// (nil, x1, PTR) <-> (ptr, x2, PTR) <-> (ptr, x3, NIL)

use std::{fmt::Debug, ptr::NonNull};

struct Node<T> {
    value: T,
    next: Option<Box<Node<T>>>,
    prev: Option<NonNull<Node<T>>>,
}

/// A doubly linked list of elements of type `T`.
pub struct LinkedList<T> {
    len: usize,
    head: Option<Box<Node<T>>>,
    tail: Option<NonNull<Node<T>>>,
}

/// An referencing iterator over a doubly linked list.
pub struct Iter<'a, T> {
    len: usize,
    head: Option<&'a Node<T>>,
    tail: Option<&'a Node<T>>,
}

/// An borrowing iterator over a doubly linked list.
pub struct IterMut<'a, T> {
    len: usize,
    head: Option<&'a mut Node<T>>,
    tail: Option<NonNull<Node<T>>>,
}

/// An owning iterator over a doubly linked list.
pub struct IntoIter<T> {
    inner: LinkedList<T>,
}

impl<T> Node<T> {
    fn new(value: T) -> Self {
        Self {
            value,
            next: None,
            prev: None,
        }
    }

    fn to_boxed(self) -> Box<Self> {
        Box::new(self)
    }

    fn as_ptr(&self) -> NonNull<Self> {
        NonNull::from_ref(self)
    }

    fn push(&mut self, mut other: Box<Self>) {
        other.prev = Some(self.as_ptr());
        self.next = Some(other);
    }

    fn pop(&mut self) -> Option<Box<Self>> {
        self.next.take().map(|mut next| {
            next.prev = None;
            next
        })
    }
}

impl<T> LinkedList<T> {
    /// Creates an empty doubly linked list.
    pub fn new() -> Self {
        Self {
            len: 0,
            head: None,
            tail: None,
        }
    }

    /// Reads the current amount of elements in the list.
    /// This operation is O(1) in time.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Inserts the given `value` at the beginning of the list.
    pub fn push_front(&mut self, value: T) {
        self.len += 1;
        let mut node = Node::new(value).to_boxed();

        if let Some(head) = self.head.take() {
            node.push(head);
        } else {
            self.tail = Some(node.as_ptr());
        }
        self.head = Some(node);
    }

    /// Removes the first element in the list.
    /// `None` is returned if the list is empty.
    pub fn pop_front(&mut self) -> Option<T> {
        self.head.take().map(|mut head| {
            self.len -= 1;
            self.head = head.pop();

            if self.head.is_none() {
                self.tail = None;
            }

            head.value
        })
    }

    /// Inserts the given `value` at the end of the list.
    pub fn push_back(&mut self, value: T) {
        self.len += 1;
        let node = Node::new(value).to_boxed();

        let tail = self.tail.take();
        self.tail = Some(node.as_ptr());

        if let Some(mut tail) = tail {
            // SAFETY: Weak pointers within the linked list are always valid.
            let tail = unsafe { tail.as_mut() };
            tail.push(node);
        } else {
            self.head = Some(node);
        }
    }

    /// Removes the last element in the list.
    /// `None` is returned if the list is empty.
    pub fn pop_back(&mut self) -> Option<T> {
        self.tail.take().map(|tail| {
            self.len -= 1;
            // SAFETY: Weak pointers within the linked list are always valid.
            let tail = unsafe { tail.as_ref() };

            let tail = if let Some(mut prev) = tail.prev {
                self.tail = Some(prev);
                // SAFETY: Weak pointers within the linked list are always valid.
                let prev = unsafe { prev.as_mut() };
                prev.pop()
                    .expect("Previous node must point to the tail node")
            } else {
                self.head.take().expect(
                    "Head must point to the tail node since it is the only node in the list",
                )
            };

            tail.value
        })
    }
}

impl<T> FromIterator<T> for LinkedList<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut list = Self::new();
        for x in iter {
            list.push_back(x);
        }
        list
    }
}

impl<T> LinkedList<T> {
    /// Creates a referencing iterator.
    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            len: self.len,
            head: self.head.as_ref().map(Box::as_ref),
            // SAFETY: Weak pointers within the linked list are always valid.
            tail: self.tail.as_ref().map(|x| unsafe { x.as_ref() }),
        }
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(head) = self.head.take()
            && self.len > 0
        {
            self.len -= 1;
            self.head = head.next.as_ref().map(Box::as_ref);
            Some(&head.value)
        } else {
            None
        }
    }
}

impl<'a, T> DoubleEndedIterator for Iter<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if let Some(tail) = self.tail.take()
            && self.len > 0
        {
            self.len -= 1;
            // SAFETY: Weak pointers within the linked list are always valid.
            self.tail = tail.prev.as_ref().map(|x| unsafe { x.as_ref() });
            Some(&tail.value)
        } else {
            None
        }
    }
}

impl<T> LinkedList<T> {
    /// Creates a borrowing iterator.
    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        IterMut {
            len: self.len,
            head: self.head.as_mut().map(Box::as_mut),
            tail: self.tail,
        }
    }
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(head) = self.head.take()
            && self.len > 0
        {
            self.len -= 1;
            self.head = head.next.as_mut().map(Box::as_mut);
            Some(&mut head.value)
        } else {
            None
        }
    }
}

impl<'a, T> DoubleEndedIterator for IterMut<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if let Some(mut tail) = self.tail.take()
            && self.len > 0
        {
            self.len -= 1;
            // SAFETY: Weak pointers within the linked list are always valid.
            let tail = unsafe { tail.as_mut() };
            self.tail = tail.prev;
            Some(&mut tail.value)
        } else {
            None
        }
    }
}

impl<T> IntoIterator for LinkedList<T> {
    type Item = T;

    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter { inner: self }
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.pop_front()
    }
}

impl<T> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.pop_back()
    }
}

impl<T: Debug> Debug for LinkedList<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_pop() {
        let mut list = LinkedList::new();
        assert_eq!(list.len(), 0);

        // Pop empty list
        assert_eq!(list.pop_front(), None);
        assert_eq!(list.pop_back(), None);
        assert_eq!(list.len(), 0);

        // Push front
        list.push_front(1);
        assert_eq!(list.len(), 1);
        assert_eq!(list.pop_front(), Some(1));
        assert_eq!(list.len(), 0);

        list.push_front(2);
        assert_eq!(list.len(), 1);
        assert_eq!(list.pop_back(), Some(2));
        assert_eq!(list.len(), 0);

        // Push back
        list.push_back(3);
        assert_eq!(list.len(), 1);
        assert_eq!(list.pop_front(), Some(3));
        assert_eq!(list.len(), 0);

        list.push_back(4);
        assert_eq!(list.len(), 1);
        assert_eq!(list.pop_back(), Some(4));
        assert_eq!(list.len(), 0);

        list.push_front(5);
        list.push_back(6);
        list.push_front(7);
        assert_eq!(list.len(), 3);
        assert_eq!(list.pop_back(), Some(6));
        assert_eq!(list.len(), 2);
        assert_eq!(list.pop_front(), Some(7));
        assert_eq!(list.len(), 1);
        assert_eq!(list.pop_back(), Some(5));
        assert_eq!(list.len(), 0);
    }

    #[test]
    fn test_from_iterator() {
        let v = vec![1, 2, 3];
        let mut list = LinkedList::from_iter(v);

        assert_eq!(list.len(), 3);
        assert_eq!(list.pop_front(), Some(1));
        assert_eq!(list.pop_front(), Some(2));
        assert_eq!(list.pop_front(), Some(3));
    }

    #[test]
    fn test_iter() {
        let list = LinkedList::from_iter(vec![1, 2, 3, 4, 5]);
        let mut iter = list.iter();

        assert_eq!(iter.next(), Some(&1));
        assert_eq!(iter.next_back(), Some(&5));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next_back(), Some(&4));
        assert_eq!(iter.next(), Some(&3));

        assert_eq!(iter.next(), None);
        assert_eq!(iter.next_back(), None);
    }

    #[test]
    fn test_iter_mut() {
        let mut list = LinkedList::from_iter(vec![1, 2, 3, 4, 5]);
        let mut iter = list.iter_mut();

        assert_eq!(iter.next(), Some(&mut 1));
        assert_eq!(iter.next_back(), Some(&mut 5));
        assert_eq!(iter.next(), Some(&mut 2));
        assert_eq!(iter.next_back(), Some(&mut 4));
        assert_eq!(iter.next(), Some(&mut 3));

        assert_eq!(iter.next(), None);
        assert_eq!(iter.next_back(), None);
    }

    #[test]
    fn test_into_iter() {
        let list = LinkedList::from_iter(vec![1, 2, 3, 4, 5]);
        let mut iter = list.into_iter();

        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next_back(), Some(5));
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next_back(), Some(4));
        assert_eq!(iter.next(), Some(3));

        assert_eq!(iter.next(), None);
        assert_eq!(iter.next_back(), None);
    }

    #[test]
    fn test_debug() {
        let list = LinkedList::from_iter(vec![1, 2, 3, 4, 5]);
        assert_eq!(format!("{:?}", list), "[1, 2, 3, 4, 5]");
    }
}
