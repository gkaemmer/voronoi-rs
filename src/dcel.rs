// Doubly-connected edge list for storing voronoi regions
use crate::math_helpers::{equals_with_epsilon};

const NIL: usize = !0;

#[derive(Debug)]
struct Vertex {
    x: f64,
    y: f64
}

#[derive(Debug)]
struct HalfEdge {
    origin: usize, // Index of vertex point
    next: usize, // Index of next half edge
    twin: usize, // Index of twin half edge
    active: bool
}

#[derive(Debug)]
pub struct Dcel {
    vertices: Vec<Vertex>,
    halfedges: Vec<HalfEdge>,
    faces: Vec<usize> // Array of indices of halfedges that define faces
}

impl HalfEdge {
    fn new() -> HalfEdge {
        HalfEdge { origin: NIL, next: NIL, twin: NIL, active: true }
    }
}

impl Dcel {
    pub fn new(face_count: usize) -> Dcel {
        Dcel {
            vertices: Vec::new(),
            halfedges: Vec::new(),
            faces: vec![NIL; face_count]
        }
    }

    pub fn ensure_face(&mut self, face_id: usize, halfedge: usize) {
        if self.faces[face_id] == NIL {
            self.faces[face_id] = halfedge;
        }
    }

    pub fn create_twins(&mut self) -> (usize, usize) {
        let index = self.halfedges.len();
        let twin_index = index + 1;

        let mut edge = HalfEdge::new();
        let mut twin = HalfEdge::new();

        twin.twin = index;
        edge.twin = twin_index;

        self.halfedges.push(edge);
        self.halfedges.push(twin);

        (index, twin_index)
    }

    pub fn create_vertex(&mut self, x: f64, y: f64) -> usize {
        let index = self.vertices.len();
        self.vertices.push(Vertex { x, y });
        index
    }

    pub fn get_twin(&self, halfedge: usize) -> usize {
        self.halfedges[halfedge].twin
    }

    pub fn set_origin(&mut self, halfedge: usize, origin: usize) {
        self.halfedges[halfedge].origin = origin;
    }

    pub fn set_next(&mut self, halfedge: usize, next: usize) {
        self.halfedges[halfedge].next = next;
    }

    pub fn get_edges(&self) -> Vec<(f64, f64, f64, f64)> {
        let mut edges = Vec::new();
        for i in 0..(self.halfedges.len() / 2) {
            let edge = i * 2;
            let twin = i * 2 + 1;
            if self.halfedges[edge].origin != NIL && self.halfedges[twin].origin != NIL {
                let from = &self.vertices[self.halfedges[edge].origin];
                let to = &self.vertices[self.halfedges[twin].origin];
                edges.push((from.x, from.y, to.x, to.y));
            }
        }
        edges
    }

    pub fn get_polygons(&self) -> Vec<Vec<(f64, f64)>> {
        let mut polygons = Vec::with_capacity(self.faces.len());
        for face in &self.faces {
            let mut edge = *face;
            let mut polygon = Vec::new();
            loop {
                if edge == NIL || self.halfedges[edge].origin == NIL {
                    break;
                }
                let vertex = &self.vertices[self.halfedges[edge].origin];
                polygon.push((vertex.x, vertex.y));
                edge = self.halfedges[edge].next;

                // println!("Building {} {} ", edge, face);
                if edge == *face {
                    break;
                }
            }

            if edge == *face {
                // We made it back to the first point, so we have a full polygon
                polygons.push(polygon);
            } else {
                // We only have a partial polygon, so just push an empty polygon
                polygons.push(Vec::new());
            }
        }
        polygons
    }

    pub fn bound(&mut self, bbox: &BoundingBox) {
        let mut faces_to_remove = Vec::new();
        let face_count = self.faces.len();
        for i in 0..face_count {
            let face = self.faces[i];
            // Find a edge whose origin is inside bbox -- if one doesn't exist, mark this face for removal
            // Set STATE to INSIDE
            // For each edge.next until we're back:
            //   If STATE is INSIDE:
            //     If segment intersects EXITING_SIDE of bounding box:
            //       Change STATE to OUTSIDE
            //       Add new vertex at intersect's x,y and name it OUT_VERTEX
            //       Store EXITING_SIDE
            //       Store current halfedge as EXITING_EDGE
            //   If STATE is OUTSIDE:
            //     Mark current halfedge as inactive
            //     If segment intersects with ENTERING_SIDE of bounding box:
            //       If ENTERING_SIDE is not equal to EXITING_SIDE:
            //         For each corner between EXITING_SIDE and ENTERING_SIDE:
            //           Add new vertex at corner and name it CORNER
            //           Add a new halfedge CORNER_EDGE with origin OUT_VERTEX
            //           Set EXITING_EDGE.next to CORNER_EDGE
            //           Set OUT_VERTEX to CORNER
            //           Set EXITING_EDGE to CORNER_EDGE
            //       Add new vertex at intersect's x,y and name it IN_VERTEX
            //       Add a new halfedge BBOX_EDGE with origin OUT_VERTEX
            //       Add a new halfedge ENTERING_EDGE with origin IN_VERTEX
            //       Set EXITING_EDGE.next to BBOX_EDGE
            //       Set BBOX_EDGE.next to ENTERING_EDGE
            //       Set ENTERING_EDGE.next to current edge's next
            //       Break out of the loop
            //
            let mut starting_edge = face;
            let mut should_remove_face = false;

            loop {
                if self.halfedges[starting_edge].origin == NIL {
                    // Only bound faces that are complete
                    should_remove_face = true;
                    break;
                }
                let vertex = &self.vertices[self.halfedges[starting_edge].origin];
                if is_inside(vertex.x, vertex.y, bbox) {
                    // This edge is inside, we can use it as a starting edge
                    break;
                }
                starting_edge = self.halfedges[starting_edge].next;
                if starting_edge == face || starting_edge == NIL {
                    // We're back to the beginning and we didn't find a vertex inside
                    should_remove_face = true;
                    break;
                }
                println!("Trying again to find starting edge");
            }

            if should_remove_face {
                faces_to_remove.push(i);
                continue;
            }

            // We now have an edge to start at
            let mut state = 0; // Inside bbox
            let mut prev_edge = NIL;
            let mut edge = starting_edge;
            let mut exiting_edge = NIL;
            let mut out_vertex = NIL;
            let mut exiting_side = BoundSide::Left;
            loop {
                if self.halfedges[edge].origin == NIL || self.halfedges[edge].next == NIL {
                    // Only bound faces that are complete
                    should_remove_face = true;
                    break;
                }
                let vertex = &self.vertices[self.halfedges[edge].origin];
                let next = self.halfedges[edge].next;
                let next_vertex = &self.vertices[self.halfedges[next].origin];
                // Note: next's origin must not be NIL because it has a previous halfedge (unless DCEL is malformed)

                // Bound this line segment
                let segment = LineSegment {
                    start_x: vertex.x,
                    start_y: vertex.y,
                    end_x: next_vertex.x,
                    end_y: next_vertex.y
                };
                let result = bound(segment, bbox);

                if state == 0 {
                    // State is INSIDE
                    if let BoundResult::Intersect { x, y, side } = result {
                        state = 1; // Outside
                        println!("Leaving at {} {}", x, y);
                        out_vertex = self.create_vertex(x, y);
                        exiting_edge = edge;
                        exiting_side = side;
                    }
                    if result == BoundResult::Outside {
                        println!("Inside to outside");
                        state = 1;
                        self.halfedges[edge].active = false;
                        out_vertex = self.halfedges[edge].origin;
                        exiting_edge = prev_edge;
                        let Vertex { x, y } = self.vertices[out_vertex];
                        exiting_side = if equals_with_epsilon(x, bbox.max_x) {
                            BoundSide::Right
                        } else if equals_with_epsilon(x, bbox.min_x) {
                            BoundSide::Left
                        } else if equals_with_epsilon(y, bbox.min_y) {
                            BoundSide::Top
                        } else {
                            BoundSide::Bottom
                        };
                    }
                } else {
                    // State is OUTSIDE
                    println!("Looking for entrance: {:?}", result);
                    self.halfedges[edge].active = false;
                    if let BoundResult::Intersect { x, y, side } = result {
                        if side != exiting_side {
                            for (corner_x, corner_y) in corners_between(exiting_side, side, &bbox) {
                                let corner = self.create_vertex(corner_x, corner_y);
                                let (corner_edge, _) = self.create_twins();
                                let exiting_edge_twin = self.halfedges[exiting_edge].twin;
                                self.halfedges[corner_edge].origin = out_vertex;
                                self.halfedges[exiting_edge_twin].origin = corner;
                                self.halfedges[exiting_edge].next = corner_edge;
                                out_vertex = corner;
                                exiting_edge = corner_edge;
                            }
                        }
                        println!("Entering at {} {}", x, y);
                        let in_vertex = self.create_vertex(x, y);
                        let (bbox_edge, bbox_edge_twin) = self.create_twins();
                        let (entering_edge, entering_edge_twin) = self.create_twins();
                        let exiting_edge_twin = self.halfedges[exiting_edge].twin;
                        self.halfedges[bbox_edge].origin = out_vertex;
                        self.halfedges[bbox_edge_twin].origin = in_vertex;
                        self.halfedges[bbox_edge].next = entering_edge;
                        self.halfedges[exiting_edge].next = bbox_edge;
                        self.halfedges[exiting_edge_twin].origin = out_vertex;
                        self.halfedges[entering_edge].origin = in_vertex;
                        self.halfedges[entering_edge].next = next;
                        self.halfedges[entering_edge_twin].origin = self.halfedges[next].origin;
                        self.faces[i] = entering_edge;
                        break;
                    }

                    if result == BoundResult::Inside {
                        println!("Outside to inside");
                        // Jumped inside, this is actually an easier case
                        let in_vertex = self.halfedges[edge].origin;
                        let Vertex { x, y } = self.vertices[in_vertex];
                        let side = if equals_with_epsilon(x, bbox.max_x) {
                            BoundSide::Right
                        } else if equals_with_epsilon(x, bbox.min_x) {
                            BoundSide::Left
                        } else if equals_with_epsilon(y, bbox.min_y) {
                            BoundSide::Top
                        } else {
                            BoundSide::Bottom
                        };
                        if side != exiting_side {
                            for (corner_x, corner_y) in corners_between(exiting_side, side, &bbox) {
                                let corner = self.create_vertex(corner_x, corner_y);
                                let (corner_edge, _) = self.create_twins();
                                let exiting_edge_twin = self.halfedges[exiting_edge].twin;
                                self.halfedges[corner_edge].origin = out_vertex;
                                self.halfedges[exiting_edge_twin].origin = corner;
                                self.halfedges[exiting_edge].next = corner_edge;
                                out_vertex = corner;
                                exiting_edge = corner_edge;
                            }
                        }
                        let (bbox_edge, bbox_edge_twin) = self.create_twins();
                        let exiting_edge_twin = self.halfedges[exiting_edge].twin;
                        self.halfedges[bbox_edge].origin = out_vertex;
                        self.halfedges[bbox_edge_twin].origin = self.halfedges[edge].origin;
                        self.halfedges[bbox_edge].next = edge;
                        self.halfedges[exiting_edge].next = bbox_edge;
                        self.halfedges[exiting_edge_twin].origin = out_vertex;
                        self.faces[i] = edge;
                        break;
                    }
                }

                prev_edge = edge;
                edge = next;
                if edge == starting_edge {
                    // We're back to the beginning, we're done
                    println!("Back to beginning");
                    break;
                } else {
                    println!("Looking at edge {}", edge);
                }
            }

            if should_remove_face {
                faces_to_remove.push(i);
                continue;
            }
        }
        for face in faces_to_remove {
            self.faces[face] = NIL;
        }
    }

    // // Repairs edges that are missing origins or nexts (i.e. the rays)
    // pub fn repair_edges(&mut self) {
    //     let mut to_repair = vec![];
    //     for (i, halfedge) in self.halfedges.iter().enumerate() {
    //         if halfedge.origin == NIL || halfedge.next == NIL {
    //             to_repair.push(i);
    //         }
    //     }

    //     for i in to_repair {
    //         if self.halfedges[i].next == NIL {

    //         }
    //     }
    // }
}

pub struct BoundingBox {
    min_x: f64,
    min_y: f64,
    max_x: f64,
    max_y: f64
}

impl BoundingBox {
    pub fn new(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> BoundingBox {
        BoundingBox {
            min_x,
            min_y,
            max_x,
            max_y
        }
    }

    pub fn width(&self) -> f64 {
        self.max_x - self.min_x
    }

    pub fn height(&self) -> f64 {
        self.max_y - self.min_y
    }

    pub fn mid_x(&self) -> f64 {
        (self.max_x + self.min_x) / 2.
    }

    pub fn mid_y(&self) -> f64 {
        (self.max_y + self.min_y) / 2.
    }
}

#[derive(Debug)]
struct LineSegment {
    start_x: f64,
    start_y: f64,
    end_x: f64,
    end_y: f64
}

fn is_inside(x: f64, y: f64, bbox: &BoundingBox) -> bool {
    if x < bbox.min_x || x > bbox.max_x || y < bbox.min_y || y > bbox.max_y {
        false
    } else {
        true
    }
}

#[derive(PartialEq, Debug)]
enum BoundResult {
    Inside,
    Outside,
    Intersect { x: f64, y: f64, side: BoundSide }
}
#[derive(PartialEq, Clone, Copy, Debug)]
enum BoundSide {
    Left,
    Right,
    Top,
    Bottom
}

fn corners_between(side1: BoundSide, side2: BoundSide, bbox: &BoundingBox) -> Vec<(f64, f64)> {
    let bottom_left = (bbox.min_x, bbox.min_y);
    let bottom_right = (bbox.max_x, bbox.min_y);
    let top_right = (bbox.max_x, bbox.max_y);
    let top_left = (bbox.min_x, bbox.max_y);
    match (side1, side2) {
        (BoundSide::Top, BoundSide::Left) => vec![top_left],
        (BoundSide::Top, BoundSide::Bottom) => vec![top_left, bottom_left],
        (BoundSide::Top, BoundSide::Right) => vec![top_left, bottom_left, bottom_right],
        (BoundSide::Left, BoundSide::Bottom) => vec![bottom_left],
        (BoundSide::Left, BoundSide::Right) => vec![bottom_left, bottom_right],
        (BoundSide::Left, BoundSide::Top) => vec![bottom_left, bottom_right, top_right],
        (BoundSide::Bottom, BoundSide::Right) => vec![bottom_right],
        (BoundSide::Bottom, BoundSide::Top) => vec![bottom_right, top_right],
        (BoundSide::Bottom, BoundSide::Left) => vec![bottom_right, top_right, top_left],
        (BoundSide::Right, BoundSide::Top) => vec![top_right],
        (BoundSide::Right, BoundSide::Left) => vec![top_right, top_left],
        (BoundSide::Right, BoundSide::Bottom) => vec![top_right, top_left, bottom_left],
        _ => vec![]
    }
}

fn bound(segment: LineSegment, bbox: &BoundingBox) -> BoundResult {
    // Case 1: Returns Some(segment) if segment is entirely inside box
    // Case 2: Returns None if segment is entirely outside box
    // Cases 3 and 4: Returns Some(new_segment) where new_segment is the portion of
    // segment that is inside the box



    let starts_inside = is_inside(segment.start_x, segment.start_y, bbox);
    let ends_inside = is_inside(segment.end_x, segment.end_y, bbox);
    println!("{} {} Starts: {}, {} {} Ends: {}", segment.start_x, segment.start_y, starts_inside, segment.end_x, segment.end_y, ends_inside);

    if starts_inside && ends_inside {
        // Case 1: completely inside
        BoundResult::Inside
    } else if !starts_inside && !ends_inside {
        // Case 2: completely outside
        BoundResult::Outside
    } else if starts_inside && !ends_inside {
        // Case 3: starts inside and ends outside
        let dx = segment.end_x - segment.start_x;
        let dy = segment.end_y - segment.start_y;

        // Find the bounds to test against
        let test_x = if dx < 0. { bbox.min_x } else { bbox.max_x };
        let test_y = if dy < 0. { bbox.min_y } else { bbox.max_y };

        let tx = if equals_with_epsilon(dx, 0.) { std::f64::MAX } else { (test_x - segment.start_x) / dx };
        let ty = if equals_with_epsilon(dy, 0.) { std::f64::MAX } else { (test_y - segment.start_y) / dy };

        let tmin = if tx < ty { tx } else { ty };
        let end_x = segment.start_x + dx * tmin;
        let end_y = segment.start_y + dy * tmin;

        let side = if dx < 0. && dy < 0. {
            if tx < ty { BoundSide::Left } else { BoundSide::Bottom }
        } else if dx < 0. && dy > 0. {
            if tx < ty { BoundSide::Left } else { BoundSide::Top }
        } else if dx > 0. && dy > 0. {
            if tx < ty { BoundSide::Right } else { BoundSide::Top }
        } else /*if dx > 0. && dy < 0.*/ {
            if tx < ty { BoundSide::Right } else { BoundSide::Bottom }
        };

        BoundResult::Intersect { x: end_x, y: end_y, side }
    } else if !starts_inside && ends_inside {
        // Case 4: starts outside and ends inside
        // Just do case 3 backward
        let dx = segment.start_x - segment.end_x;
        let dy = segment.start_y - segment.end_y;

        // Find the bounds to test against
        let test_x = if dx < 0. { bbox.min_x } else { bbox.max_x };
        let test_y = if dy < 0. { bbox.min_y } else { bbox.max_y };

        let tx = if equals_with_epsilon(dx, 0.) { std::f64::MAX } else { (test_x - segment.end_x) / dx };
        let ty = if equals_with_epsilon(dy, 0.) { std::f64::MAX } else { (test_y - segment.end_y) / dy };

        let tmax = if tx < ty { tx } else { ty };
        let start_x = segment.end_x + dx * tmax;
        let start_y = segment.end_y + dy * tmax;

        let side = if dx < 0. && dy < 0. {
            if tx < ty { BoundSide::Left } else { BoundSide::Bottom }
        } else if dx < 0. && dy > 0. {
            if tx < ty { BoundSide::Left } else { BoundSide::Top }
        } else if dx > 0. && dy > 0. {
            if tx < ty { BoundSide::Right } else { BoundSide::Top }
        } else /*if dx > 0. && dy < 0.*/ {
            if tx < ty { BoundSide::Right } else { BoundSide::Bottom }
        };

        BoundResult::Intersect { x: start_x, y: start_y, side }
    } else {
        // Impossible
        BoundResult::Outside
    }
}

#[cfg(test)]
mod tests {
    use crate::dcel::{Dcel, bound, LineSegment, BoundingBox, BoundResult, BoundSide};
    use crate::math_helpers::{equals_with_epsilon};

    #[test]
    fn it_bounds_segments() {
        fn result_equal(s1: BoundResult, s2: BoundResult) -> bool {
            if let BoundResult::Intersect { x: x1, y: y1, side: side1 } = s1 {
                if let BoundResult::Intersect { x: x2, y: y2, side: side2 } = s2 {
                    return equals_with_epsilon(x1, x2) && equals_with_epsilon(y1, y2) && side1 == side2;
                }
            }
            s1 == s2
        }

        let segment1 = LineSegment { start_x: 0., start_y: 0., end_x: 100., end_y: -200. };
        let segment2 = LineSegment { start_x: 0., start_y: 0., end_x: 5., end_y: -2. };
        let segment3 = LineSegment { start_x: 100., start_y: 50., end_x: 0., end_y: 0. };
        let segment4 = LineSegment { start_x: 100., start_y: 50., end_x: 20., end_y: 20. };
        let bbox = BoundingBox { min_x: -10., min_y: -10., max_x: 10., max_y: 10. };

        assert!(result_equal(bound(segment1, &bbox), BoundResult::Intersect {
            x: 5.,
            y: -10.,
            side: BoundSide::Bottom
        }));

        assert!(result_equal(bound(segment2, &bbox), BoundResult::Inside));

        assert!(result_equal(bound(segment3, &bbox), BoundResult::Intersect {
            x: 10.,
            y: 5.,
            side: BoundSide::Right
        }));

        assert!(result_equal(bound(segment4, &bbox), BoundResult::Outside));
    }

    #[test]
    fn it_bounds_faces() {
        // This test tries to bound a triangle to a bounding box

        let mut dcel = Dcel::new(1);
        let (edge1, edge1_twin) = dcel.create_twins();
        let (edge2, edge2_twin) = dcel.create_twins();
        let (edge3, edge3_twin) = dcel.create_twins();

        let a = dcel.create_vertex(0., 0.);
        let b = dcel.create_vertex(2., 1.);
        let c = dcel.create_vertex(2., 0.);

        dcel.set_origin(edge1, a);
        dcel.set_origin(edge1_twin, b);
        dcel.set_origin(edge2, b);
        dcel.set_origin(edge2_twin, c);
        dcel.set_origin(edge3, c);
        dcel.set_origin(edge3_twin, a);

        dcel.set_next(edge1, edge2);
        dcel.set_next(edge2, edge3);
        dcel.set_next(edge3, edge1);

        dcel.ensure_face(0, 0);

        dcel.bound(&BoundingBox {
            min_x: -1.,
            min_y: -1.,
            max_x: 1.,
            max_y: 1.
        });

        println!("DCEL: {:?}", dcel.get_polygons());
    }
}
