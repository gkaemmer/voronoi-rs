mod math_helpers;
mod beachline;
mod eventqueue;
mod treeprint;

pub use math_helpers::{equals_with_epsilon, breakpoint_between, find_center};
use beachline::{BeachLine, BeachSegmentHandle};
use eventqueue::{Event, EventQueue, EventHandle};
use std::cmp::Ordering;
use std::collections::HashMap;
pub use treeprint::print;

// A site corresponds to an input point. They are given a unique index so that
// they can be uniquely referenced.
#[derive(Clone, Copy, Debug)]
pub struct Site {
    pub x: f32,
    pub y: f32,
    pub id: usize
}

impl PartialEq for Site {
    fn eq(&self, other: &Self) -> bool {
        return equals_with_epsilon(self.x, other.x) && equals_with_epsilon(self.y, other.y);
    }
}

pub struct Voronoi {
    events: EventQueue,
    sites: Vec<Site>,
    beach: BeachLine,
    events_by_beach_segment: HashMap<BeachSegmentHandle, EventHandle>
}

impl Voronoi {
    pub fn new(sites: Vec<Site>) -> Voronoi {
        Voronoi {
            events: EventQueue::new(),
            sites: sites,
            beach: BeachLine::new(),
            events_by_beach_segment: HashMap::new()
        }
    }

    pub fn build(sites: Vec<Site>) {
        let mut voronoi = Voronoi::new(sites);
        voronoi.run();
    }

    pub fn run(&mut self) {
        for site in self.sites.iter() {
            self.events.insert(Event::Site(site.clone()));
        }

        let first_site = self.events.pop();
        if let Some(Event::Site(site)) = first_site {
            self.beach.init(site);
        } else {
            // No points
            return;
        }

        while self.events.len() > 0 {
            match self.events.pop() {
                Some(Event::Site(site)) => {
                    println!("AT SITE {}", site.id);
                    // Find beach segment directly above site.x

                    let x = site.x;
                    let y = site.y;
                    let segment_to_split = self.beach.search(|ptr| {
                        let site = self.beach.get(ptr);
                        let left_ptr = self.beach.predecessor(ptr);
                        let left_breakpoint = if left_ptr.is_null() {
                            -std::f32::MAX
                        } else {
                            let left = self.beach.get(left_ptr);
                            breakpoint_between(left.x, left.y, site.x, site.y, y)
                        };
                        if x < left_breakpoint {
                            return Ordering::Less;
                        }
                        let right_ptr = self.beach.successor(ptr);
                        let right_breakpoint = if right_ptr.is_null() {
                            std::f32::MAX
                        } else {
                            let right = self.beach.get(right_ptr);
                            breakpoint_between(site.x, site.y, right.x, right.y, y)
                        };
                        if x > right_breakpoint {
                            return Ordering::Greater;
                        }
                        Ordering::Equal
                    });

                    println!("Splitting segment {}", self.beach.get(segment_to_split).id);
                    let left_segment = segment_to_split;
                    let middle_segment = self.beach.insert_after(segment_to_split, site);
                    let right_segment = self.beach.insert_after(middle_segment, self.beach.get(segment_to_split).clone());

                    // Re-create vertex events for split segment
                    self.delete_vertex_event(left_segment);
                    self.delete_vertex_event(right_segment);
                    self.create_vertex_event(left_segment);
                    self.create_vertex_event(right_segment);

                    // self.beach.print(|site| format!("{}", site.id));
                },
                Some(Event::Vertex(middle, x, y, rad)) => {
                    let left = self.beach.predecessor(middle);
                    let right = self.beach.successor(middle);
                    println!("AT TRIPLET {} {} {}", if left.is_null() { !0 } else { self.beach.get(left).id }, self.beach.get(middle).id, if right.is_null() { !0 } else { self.beach.get(right).id });
                    self.beach.delete(middle);
                    self.delete_vertex_event(left);
                    self.delete_vertex_event(right);
                    self.create_vertex_event(left);
                    self.create_vertex_event(right);
                    let vertex_x = x;
                    let vertex_y = y-rad;
                    println!("Got vertex at ({}, {})", vertex_x, vertex_y);
                }
                None => {
                    // Impossible
                }
            }
        }
    }

    fn delete_vertex_event(&mut self, segment: BeachSegmentHandle) {
        if let Some(event_handle) = self.events_by_beach_segment.get(&segment) {
            self.events.delete(*event_handle);
            self.events_by_beach_segment.remove(&segment);
        }
    }

    fn create_vertex_event(&mut self, segment: BeachSegmentHandle) {
        if let Some(_) = self.events_by_beach_segment.get(&segment) {
            panic!("Creating an already-existing vertex event");
        }

        let left = self.beach.predecessor(segment);
        let right = self.beach.successor(segment);
        if left.is_null() || right.is_null() {
            return;
        }
        let left_site = self.beach.get(left);
        let middle_site = self.beach.get(segment);
        let right_site = self.beach.get(right);

        // Don't add a vertex event unless these points result in a collapsing
        // segment (i.e. they are clockwise)
        let is_clockwise = (middle_site.y - left_site.y) * (right_site.x - middle_site.x) - (right_site.y - middle_site.y) * (middle_site.x - left_site.x) > 0.0;
        if is_clockwise { return; }

        let center = find_center(left_site.x, left_site.y, middle_site.x, middle_site.y, right_site.x, right_site.y);
        if center.is_none() { return; }
        let (center_x, center_y, rad) = center.unwrap();

        println!("Adding vertex event: {} {} {}", left_site.id, middle_site.id, right_site.id);

        let event_handle = self.events.insert(Event::Vertex(segment, center_x, center_y + rad, rad));
        self.events_by_beach_segment.insert(segment, event_handle);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
