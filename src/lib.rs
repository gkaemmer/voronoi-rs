mod math_helpers;
mod beachline;
mod eventqueue;
mod treeprint;

pub use math_helpers::{equals_with_epsilon, breakpoint_between, find_center};
use beachline::{BeachLine, BeachSegmentHandle};
use eventqueue::{Event, EventQueue, EventHandle};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

// A site corresponds to an input point. They are given a unique index so that
// they can be uniquely referenced.
#[derive(Clone, Copy, Debug)]
pub struct Site {
    pub x: f64,
    pub y: f64,
    pub id: usize
}

pub struct Diagram {
    edges: Vec<Edge>
}

struct SitePair(usize, usize);
impl Hash for SitePair {
    fn hash<H: Hasher>(&self, state: &mut H) {
        if self.0 < self.1 {
            self.0.hash(state);
            self.1.hash(state);
        } else {
            self.1.hash(state);
            self.0.hash(state);
        }
    }
}
impl PartialEq for SitePair {
    fn eq(&self, other: &Self) -> bool {
        (self.0 == other.0 && self.1 == other.1) ||
        (self.0 == other.1 && self.1 == other.0)
    }
}
impl Eq for SitePair {}

#[derive(Debug)]
pub enum Edge {
    Half(f64, f64, f64, f64), // A point and a direction (a ray)
    Full(f64, f64, f64, f64) // Two points (x1, y1, x2, y2)
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
    events_by_beach_segment: HashMap<BeachSegmentHandle, EventHandle>,
    edges_by_site_pair: HashMap<SitePair, Edge>
}

impl Voronoi {
    pub fn new(sites: Vec<Site>) -> Voronoi {
        Voronoi {
            events: EventQueue::new(),
            sites: sites,
            beach: BeachLine::new(),
            events_by_beach_segment: HashMap::new(),
            edges_by_site_pair: HashMap::new()
        }
    }

    pub fn build(sites: Vec<Site>) {
        let mut voronoi = Voronoi::new(sites);
        voronoi.run();
    }

    pub fn run(&mut self) -> Diagram {
        for site in self.sites.iter() {
            self.events.insert(Event::Site(site.clone()));
        }

        let first_site = self.events.pop();
        if let Some(Event::Site(site)) = first_site {
            self.beach.init(site);
        } else {
            // No points
            return Diagram { edges: vec![] };
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
                            -std::f64::MAX
                        } else {
                            let left = self.beach.get(left_ptr);
                            breakpoint_between(left.x, left.y, site.x, site.y, y)
                        };
                        if x < left_breakpoint {
                            return Ordering::Less;
                        }
                        let right_ptr = self.beach.successor(ptr);
                        let right_breakpoint = if right_ptr.is_null() {
                            std::f64::MAX
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
                    self.print_beach();

                    // Re-create vertex events for split segment
                    self.delete_vertex_event(left_segment);
                    self.delete_vertex_event(right_segment);
                    self.create_vertex_event(left_segment);
                    self.create_vertex_event(right_segment);

                    // self.events.print_simple();

                    // self.beach.print(|site| format!("{}", site.id));
                },
                Some(Event::Vertex(middle, x, y, rad)) => {
                    // We're at this vertex event, make sure we don't reference it again
                    self.events_by_beach_segment.remove(&middle);

                    let left = self.beach.predecessor(middle);
                    let right = self.beach.successor(middle);
                    println!("AT TRIPLET {} {} {}", if left.is_null() { !0 } else { self.beach.get(left).id }, self.beach.get(middle).id, if right.is_null() { !0 } else { self.beach.get(right).id });
                    let middle_site = self.beach.delete(middle).unwrap();
                    self.print_beach();
                    self.delete_vertex_event(left);
                    self.delete_vertex_event(right);
                    self.create_vertex_event(left);
                    self.create_vertex_event(right);
                    // self.events.print_simple();
                    let vertex_x = x;
                    let vertex_y = y-rad;
                    println!("Got vertex at ({}, {})", vertex_x, vertex_y);

                    let left_site = self.beach.get(left).clone();
                    let right_site = self.beach.get(right).clone();

                    // Add vertex to edges
                    self.add_vertex_to_edge(left_site, right_site, vertex_x, vertex_y);
                    self.add_vertex_to_edge(left_site, middle_site, vertex_x, vertex_y);
                    self.add_vertex_to_edge(middle_site, right_site, vertex_x, vertex_y);
                }
                None => {
                    // Impossible
                }
            }
        }

        let mut edges = Vec::new();
        for (_, value) in self.edges_by_site_pair.drain() {
            edges.push(value);
        }
        println!("{:?}", edges);

        Diagram {
            edges: edges
        }
    }

    fn delete_vertex_event(&mut self, segment: BeachSegmentHandle) {
        if let Some(event_handle) = self.events_by_beach_segment.get(&segment) {
            let middle_site = self.beach.get(segment);
            println!("Deleting vertex event around {}", middle_site.id);
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

    fn add_vertex_to_edge(&mut self, a: Site, b: Site, x: f64, y: f64) {
        let new_edge = match self.edges_by_site_pair.remove(&SitePair(a.id, b.id)) {
            Some(Edge::Half(other_x, other_y, _, _)) => {
                Edge::Full(other_x, other_y, x, y)
            },
            Some(edge) => {
                // Impossible
                edge
            },
            None => {
                let dx = b.y - a.y;
                let dy = a.x - b.x;
                Edge::Half(x, y, dx, dy)
            }
        };
        self.edges_by_site_pair.insert(SitePair(a.id, b.id), new_edge);
    }

    fn print_beach(&self) {
        self.beach.in_order(|x| print!("{},", x.id));
        println!("");
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
