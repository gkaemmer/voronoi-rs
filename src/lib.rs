mod math_helpers;
mod beachline;
mod eventqueue;

pub use math_helpers::{equals_with_epsilon, breakpoint_between};
use beachline::{BeachLine, Pointer};
use eventqueue::{Event, EventQueue, EventHandle};
use std::cmp::Ordering;

// A site corresponds to an input point. They are given a unique index so that
// they can be uniquely referenced.
#[derive(Clone, Copy, Debug)]
pub struct Site {
    pub x: f32,
    pub y: f32,
    pub id: usize
}

// A BeachLineSegment is one piece of the current beachline, referencing a
// single site that defines it. It may not be the only beach line segment
// referencing that particular site. The left and right boundaries of the
// segment are defined by its intersection with its neighboring segments, and
// are calculated on the fly, only when needed.
// Note that the boundaries of the segment change as the sweep line moves
struct BeachLineSegment {
    site: Site,
    event_id: EventHandle,
}

impl PartialEq for Site {
    fn eq(&self, other: &Self) -> bool {
        return equals_with_epsilon(self.x, other.x) && equals_with_epsilon(self.y, other.y);
    }
}

pub struct Voronoi {
    events: EventQueue,
    sites: Vec<Site>,
    beach: BeachLine<Site>
}

impl Voronoi {
    fn new(sites: Vec<Site>) -> Voronoi {
        Voronoi {
            events: EventQueue::new(),
            sites: sites,
            beach: BeachLine::new()
        }
    }

    pub fn run(sites: Vec<Site>) {
        let mut voronoi = Voronoi::new(sites);

        for site in voronoi.sites.iter() {
            voronoi.events.insert(Event::Site(site.clone()));
        }

        let first_site = voronoi.events.pop();
        if let Some(Event::Site(site)) = first_site {
            voronoi.beach.init(site);
        } else {
            // No points;
            return;
        }

        while voronoi.events.len() > 0 {
            match voronoi.events.pop() {
                Some(Event::Site(site)) => {
                    println!("Got site: {:?}", site);
                    // Find beach segment directly above site.x

                    let x = site.x;
                    let y = site.y;
                    let segment_to_split = voronoi.beach.search(|ptr| {
                        let site = voronoi.beach.get(ptr);
                        let left_ptr = voronoi.beach.predecessor(ptr);
                        let left_breakpoint = if left_ptr.is_null() {
                            -std::f32::MAX
                        } else {
                            let left = voronoi.beach.get(left_ptr);
                            breakpoint_between(left.x, left.y, site.x, site.y, y)
                        };
                        if x < left_breakpoint {
                            return Ordering::Less;
                        }
                        let right_ptr = voronoi.beach.successor(ptr);
                        let right_breakpoint = if right_ptr.is_null() {
                            std::f32::MAX
                        } else {
                            let right = voronoi.beach.get(right_ptr);
                            breakpoint_between(site.x, site.y, right.x, right.y, y)
                        };
                        if x > right_breakpoint {
                            return Ordering::Greater;
                        }
                        Ordering::Equal
                    });

                    println!("Splitting segment {:?} ({:?})", segment_to_split, voronoi.beach.get(segment_to_split));
                    let left_segment = segment_to_split;
                    let middle_segment = voronoi.beach.insert_after(segment_to_split, site);
                    let right_segment = voronoi.beach.insert_after(middle_segment, voronoi.beach.get(segment_to_split).clone());
                    voronoi.beach.print(|site| format!("{}", site.id));
                },
                Some(Event::Vertex(pointer, x, y)) => {
                    // Do nothing for now
                }
                None => {
                    // Impossible
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
