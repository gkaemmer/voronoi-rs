// Doubly-connected edge list for storing voronoi regions

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
}

#[derive(Debug)]
pub struct Dcel {
    vertices: Vec<Vertex>,
    halfedges: Vec<HalfEdge>,
    faces: Vec<usize> // Array of indices of halfedges that define faces
}

impl HalfEdge {
    fn new() -> HalfEdge {
        HalfEdge { origin: NIL, next: NIL, twin: NIL }
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
            let mut edge = face;
            let mut polygon = Vec::new();
            loop {
                if self.halfedges[*edge].origin == NIL {
                    break;
                }
                let vertex = &self.vertices[self.halfedges[*edge].origin];
                polygon.push((vertex.x, vertex.y));
                edge = &self.halfedges[*edge].next;

                if *edge == NIL || *edge == *face {
                    break;
                }
            }

            if *edge == *face {
                // We made it back to the first point, so we have a full polygon
                polygons.push(polygon);
            } else {
                // We only have a partial polygon, so just push an empty polygon
                polygons.push(Vec::new());
            }
        }
        polygons
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
