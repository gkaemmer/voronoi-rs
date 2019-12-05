extern crate voronoi;

use voronoi::{Voronoi, Site};

fn main() {
    let sites = vec![
        Site { x: -1.0, y: 0.0, id: 0 },
        Site { x: 0.0, y: 0.0, id: 1 },
        Site { x: 1.0, y: 0.0, id: 2 },
        Site { x: 0.0, y: 1.0, id: 3 },
        Site { x: 0.0, y: -1.0, id: 4 },
    ];

    Voronoi::build(sites);
}
