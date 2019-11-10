extern crate voronoi;

use voronoi::{Voronoi, Site};

fn main() {
    let sites = vec![
        Site { x: 50.0, y: 50.0, id: 0 },
        Site { x: 70.0, y: 60.0, id: 1 },
        Site { x: 55.0, y: 70.0, id: 2 },
    ];

    Voronoi::run(sites);
}
