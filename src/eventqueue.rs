// EventQueue is a normal binary heap, except that elements are assigned a
// "handle" and can be deleted using this handle.

use slab::Slab;
use std::cmp::Ordering;
use std::ops::{Index, IndexMut};
use crate::Site;
use crate::math_helpers::equals_with_epsilon;
use crate::treeprint::print;
use std::collections::HashMap;

const NULL: usize = !0;

#[derive(Debug)]
pub enum Event {
    Site(Site),
    Vertex(crate::beachline::BeachSegmentHandle, f64, f64, f64)
}

#[derive(Eq, PartialEq, Copy, Clone, Debug, Hash)]
struct Pointer(usize);

#[derive(Eq, PartialEq, Copy, Clone, Debug, Hash)]
pub struct EventHandle(Pointer);

pub struct EventQueue {
    events: Slab<Event>,
    heap: Vec<Pointer>,
    heap_indices_by_events: HashMap<Pointer, usize>
}

// Just for convenience, so that we can type `self[i]` instead of `self.slab[i]`.
impl IndexMut<Pointer> for EventQueue {
    fn index_mut(&mut self, index: Pointer) -> &mut Event {
        &mut self.events[index.0]
    }
}
impl Index<Pointer> for EventQueue {
    type Output = Event;

    fn index(&self, index: Pointer) -> &Event {
        &self.events[index.0]
    }
}

impl Ord for Event {
    fn cmp(&self, other: &Self) -> Ordering {
        let (self_x, self_y) = match self {
            Event::Site(site) => (site.x, site.y),
            Event::Vertex(_, x, y, _) => (*x, *y)
        };

        let (other_x, other_y) = match other {
            Event::Site(site) => (site.x, site.y),
            Event::Vertex(_, x, y, _) => (*x, *y)
        };

        if equals_with_epsilon(self_y, other_y) {
            if equals_with_epsilon(self_x, other_x) {
                return Ordering::Equal;
            }
            if self_x < other_x { Ordering::Less } else { Ordering::Greater }
        } else {
            if self_y < other_y { Ordering::Less } else { Ordering::Greater }
        }
    }
}

impl PartialOrd for Event {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Event {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}
impl Eq for Event {}

impl EventQueue {
    pub fn new() -> EventQueue {
        EventQueue {
            events: Slab::new(),
            heap: Vec::new(),
            heap_indices_by_events: HashMap::new()
        }
    }

    pub fn insert(&mut self, event: Event) -> EventHandle {
        let ptr = self.events.insert(event);

        self.heap.push(Pointer(ptr));
        self.heap_indices_by_events.insert(Pointer(ptr), self.heap.len() - 1);
        self.heapify_up(self.heap.len() - 1);
        EventHandle(Pointer(ptr))
    }

    pub fn pop(&mut self) -> Option<Event> {
        if self.heap.len() < 1 {
            return None;
        }
        let ptr = self.heap[0];
        let last = self.heap.len() - 1;
        self.swap(0, last);
        let event = self.events.remove(ptr.0);
        self.heap_indices_by_events.remove(&ptr);
        self.heap.pop().unwrap();
        if self.heap.len() > 0 {
            self.heapify_down(0);
        }
        return Some(event);
    }

    pub fn len(&self) -> usize {
        self.heap.len()
    }

    pub fn delete(&mut self, handle: EventHandle) -> Option<Event> {
        let ptr = handle.0;
        let index = self.heap_indices_by_events.get(&ptr);
        if index.is_none() { return None; }
        let index = *index.unwrap();
        let last = self.heap.len() - 1;
        self.swap(index, last);
        let event = self.events.remove(ptr.0);
        self.heap_indices_by_events.remove(&ptr);
        self.heap.pop().unwrap();
        if self.heap.len() > index {
            // Element could be either greater than a child or less than its parent
            self.heapify_down(index);
            // At this point, heap[index] could be smaller than its parent
            self.heapify_up(index);
            // Heap property is restored
        }
        return Some(event);
    }

    fn heapify_up(&mut self, at: usize) {
        // Element at i is potentially smaller than element at parent(i)
        let parent = self.parent(at);
        if parent != NULL && self[self.heap[at]] < self[self.heap[parent]] {
            self.swap(at, parent);
            self.heapify_up(parent)
        }
    }

    fn heapify_down(&mut self, at: usize) {
        // Element at i is potentially greater than one of its children
        let left = self.left(at);
        let right = self.right(at);
        let mut smallest = at;
        if left != NULL && self[self.heap[left]] < self[self.heap[smallest]] {
            smallest = left;
        }
        if right != NULL && self[self.heap[right]] < self[self.heap[smallest]] {
            smallest = right;
        }
        if smallest != at {
            self.swap(smallest, at);
            self.heapify_down(smallest)
        }
    }

    fn left(&self, i: usize) -> usize {
        if i == NULL { return NULL; }
        let child = 2*i + 1;
        if self.heap.len() > child {
            child
        } else {
            NULL
        }
    }

    fn right(&self, i: usize) -> usize {
        if i == NULL { return NULL; }
        let child = 2*i + 2;
        if self.heap.len() > child {
            child
        } else {
            NULL
        }
    }

    fn parent(&self, i: usize) -> usize {
        if i == NULL || i == 0 { return NULL; }
        (i - 1) / 2
    }

    fn swap(&mut self, i: usize, j: usize) {
        let temp = self.heap[i];
        self.heap_indices_by_events.insert(self.heap[i], j);
        self.heap_indices_by_events.insert(self.heap[j], i);
        self.heap[i] = self.heap[j];
        self.heap[j] = temp;
    }

    #[allow(dead_code)]
    pub fn print(&self) {
        if self.len() < 1 { return; }
        print(0 as usize, |i| {
            let left = self.left(*i);
            if left == NULL { None } else { Some(left) }
        }, |i| {
            let right = self.right(*i);
            if right == NULL { None } else { Some(right) }
        }, |i| self.index_to_string(*i));
    }

    #[allow(dead_code)]
    pub fn print_simple(&self) {
        for i in 0..self.heap.len() {
            print!("{},", self.index_to_string(i));
        }
        println!("");
    }

    fn index_to_string(&self, i: usize) -> String {
        let event = &self[self.heap[i]];
        match *event {
            Event::Site(site) => format!("S:{}", site.y),
            Event::Vertex(_, _, y, _) => format!("V:{}", y)
        }
    }

    #[allow(dead_code)]
    fn validate(&self, at: usize) {
        let parent = self.parent(at);
        if parent != NULL && self[self.heap[at]] < self[self.heap[parent]] {
            self.print();
            panic!("Invalid heap!");
        }

        let left = self.left(at);
        let right = self.right(at);
        if left != NULL {
            self.validate(left);
        }
        if right != NULL {
            self.validate(right);
        }
    }
}
