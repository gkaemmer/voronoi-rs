mod math_helpers;
mod beachline;
mod eventqueue;
mod treeprint;
mod dcel;

pub use math_helpers::{equals_with_epsilon, breakpoint_between, find_center};
use beachline::{BeachLine, BeachSegmentHandle};
use eventqueue::{Event, EventQueue, EventHandle};
use std::cmp::Ordering;
use std::collections::HashMap;
use dcel::Dcel;

pub struct InputSite {
    pub x: f64,
    pub y: f64
}

// A site corresponds to an input point. They are given a unique index so that
// they can be uniquely referenced.
#[derive(Clone, Copy, Debug)]
pub struct Site {
    pub x: f64,
    pub y: f64,
    pub id: usize
}

pub struct Diagram {
    pub edges: Vec<Edge>
}

#[derive(Hash, PartialEq, Eq)]
struct SitePair(usize, usize);

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
    edges_by_site_pair: HashMap<SitePair, Edge>,
    dcel: Dcel,
    halfedges_by_site_pair: HashMap<SitePair, usize>
}

impl Voronoi {
    pub fn new(sites: Vec<InputSite>) -> Voronoi {
        let len = sites.len();
        let sites = sites.iter().enumerate().map(|(i, s)| Site { x: s.x, y: s.y, id: i }).collect();
        Voronoi {
            events: EventQueue::new(),
            sites,
            beach: BeachLine::new(),
            events_by_beach_segment: HashMap::new(),
            edges_by_site_pair: HashMap::new(),
            dcel: Dcel::new(len),
            halfedges_by_site_pair: HashMap::new()
        }
    }

    pub fn build(sites: Vec<InputSite>) -> Dcel {
        let voronoi = Voronoi::new(sites);
        voronoi.run()
    }

    pub fn run(mut self) -> Dcel {
        for site in self.sites.iter() {
            self.events.insert(Event::Site(site.clone()));
        }

        let first_site = self.events.pop();
        if let Some(Event::Site(site)) = first_site {
            self.beach.init(site);
        } else {
            // No points
            return self.dcel;
        }

        while self.events.len() > 0 {
            match self.events.pop() {
                Some(Event::Site(site)) => {
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

                    self.delete_vertex_event(segment_to_split);
                    let left_segment = segment_to_split;
                    let middle_segment = self.beach.insert_after(segment_to_split, site);
                    let right_segment = self.beach.insert_after(middle_segment, self.beach.get(segment_to_split).clone());

                    // Re-create vertex events for split segment
                    self.create_vertex_event(left_segment);
                    self.create_vertex_event(right_segment);

                    self.create_halfedges(site, self.beach.get(segment_to_split).clone());
                },
                Some(Event::Vertex(middle, x, y, rad)) => {
                    // We're at this vertex event, make sure we don't reference it again
                    self.events_by_beach_segment.remove(&middle);

                    let left = self.beach.predecessor(middle);
                    let right = self.beach.successor(middle);
                    let middle_site = self.beach.delete(middle).unwrap();
                    self.delete_vertex_event(left);
                    self.delete_vertex_event(right);
                    self.create_vertex_event(left);
                    self.create_vertex_event(right);
                    let vertex_x = x;
                    let vertex_y = y-rad;

                    let left_site = self.beach.get(left).clone();
                    let right_site = self.beach.get(right).clone();

                    // Add vertex to edges
                    // Get edge id of (left, middle) edge LM and
                    //  set LM origin to V
                    //  set LM's twin's next to MR
                    let lm = self.get_halfedge(left_site, middle_site);
                    let mr = self.get_halfedge(middle_site, right_site);

                    let lm_twin = self.dcel.get_twin(lm);
                    let mr_twin = self.dcel.get_twin(mr);

                    let vertex = self.dcel.create_vertex(vertex_x, vertex_y);

                    let (rl, rl_twin) = self.create_halfedges(right_site, left_site);

                    self.dcel.set_origin(lm_twin, vertex);
                    self.dcel.set_origin(mr_twin, vertex);
                    self.dcel.set_origin(rl_twin, vertex);

                    self.dcel.set_next(lm, rl_twin);
                    self.dcel.set_next(mr, lm_twin);
                    self.dcel.set_next(rl, mr_twin);

                    // Get edge id of (middle, right) edge MR and set its next to LM
                    // Create new (right, left) edge

                    self.add_vertex_to_edge(left_site, middle_site, vertex_x, vertex_y);
                    self.add_vertex_to_edge(middle_site, right_site, vertex_x, vertex_y);
                    self.add_vertex_to_edge(right_site, left_site, vertex_x, vertex_y);
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

        // println!("{:?}", self.dcel);
        // println!("{:?}", self.dcel.get_polygons());

        self.dcel
    }

    fn create_halfedges(&mut self, left: Site, right: Site) -> (usize, usize) {
        if let Some(_) = self.halfedges_by_site_pair.get(&SitePair(left.id, right.id)) {
            panic!(format!("Edge already exists: {} {}", left.id, right.id));
        }
        let (edge, twin) = self.dcel.create_twins();
        self.halfedges_by_site_pair.insert(SitePair(left.id, right.id), edge);
        self.halfedges_by_site_pair.insert(SitePair(right.id, left.id), twin);
        self.dcel.ensure_face(left.id, edge);
        self.dcel.ensure_face(right.id, twin);
        (edge, twin)
    }

    fn get_halfedge(&self, left: Site, right: Site) -> usize {
        if let Some(halfedge) = self.halfedges_by_site_pair.get(&SitePair(left.id, right.id)) {
            *halfedge
        } else {
            panic!("Tried getting a non-existant halfedge");
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
}
