use std::fmt;
use std::option::Option;

pub struct LinkedListIter<'a, T> {
    current: &'a Option<Box<Node<T>>>, 
}

pub struct LinkedList<T> {
    head: Option<Box<Node<T>>>,
    size: usize,
}

struct Node<T> {
    value: T,
    next: Option<Box<Node<T>>>,
}

impl<T> Node<T> {
    pub fn new(value: T, next: Option<Box<Node<T>>>) -> Node<T> {
        Node {value: value, next: next}
    }
}

impl<T> LinkedList<T> {
    pub fn new() -> LinkedList<T> {
        LinkedList {head: None, size: 0}
    }
    
    pub fn get_size(&self) -> usize {
        self.size
    }
    
    pub fn is_empty(&self) -> bool {
        self.get_size() == 0
    }
    
    pub fn push_front(&mut self, value: T) {
        let new_node: Box<Node<T>> = Box::new(Node::new(value, self.head.take()));
        self.head = Some(new_node);
        self.size += 1;
    }
    
    pub fn pop_front(&mut self) -> Option<T> {
        let node: Box<Node<T>> = self.head.take()?;
        self.head = node.next;
        self.size -= 1;
        Some(node.value)
    }
}


impl<T> fmt::Display for LinkedList<T> where T: fmt::Display {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut current: &Option<Box<Node<T>>> = &self.head;
        let mut result = String::new();
        loop {
            match current {
                Some(node) => {
                    result = format!("{} {}", result, node.value);
                    current = &node.next;
                },
                None => break,
            }
        }
        write!(f, "{}", result)
    }
}

impl<T> Iterator for LinkedList<T> {
    type Item = T;
    
    fn next(&mut self) -> Option<Self::Item> {
        self.pop_front()
    }
}

impl<'a, T> Iterator for LinkedListIter<'a, T> {
    type Item = &'a T;
    
    fn next(&mut self) -> Option<Self::Item> {
        match self.current {
            Some(node) => {
                self.current = &node.next;
                Some(&node.value)
            },
            None => None,
        }
    }
}

impl<'a, T> IntoIterator for &'a LinkedList<T> {
    type Item = &'a T;
    type IntoIter = LinkedListIter<'a, T>;
    
    fn into_iter(self) -> Self::IntoIter {
        LinkedListIter {current: &self.head}
    }
}

impl<T> Drop for LinkedList<T> {
    fn drop(&mut self) {
        let mut current = self.head.take();
        while let Some(mut node) = current {
            current = node.next.take();
        }
    }
}

impl <T> Clone for LinkedList<T> where T: Clone {
    fn clone(&self) -> Self {
        let mut new_list = LinkedList::new();
        let mut current = &self.head;
        while let Some(node) = current {
            new_list.push_front(node.value.clone());
            current = &node.next;
        }
        new_list
    }
}

impl<T> PartialEq for LinkedList<T> where T: PartialEq {
    fn eq(&self, other: &Self) -> bool {
        let mut current = &self.head;
        let mut other_current = &other.head;
        while let (Some(node), Some(other_node)) = (current, other_current) {
            if node.value != other_node.value {
                return false;
            }
            current = &node.next;
            other_current = &other_node.next;
        }
        current.is_none() && other_current.is_none()
    }
}

pub trait ComputeNorm {
    fn norm(&self) -> f64 {
        0.0
    }
}

impl ComputeNorm for LinkedList<f64> {
    fn norm(&self) -> f64 {
        let mut current = &self.head;
        let mut sum = 0.0;
        while let Some(node) = current {
            sum += node.value * node.value;
            current = &node.next;
        }
        sum.sqrt()
    }
}

// impl<T> Iterator for LinkedList<T> {
//     type Item = T;
//     fn next(&mut self) -> Option<Self::Item> {
//         self.pop_front()
//     }
// }
