extern crate voronoi;

use voronoi::{Voronoi, InputSite};

fn main() {
    let sites = vec![
        InputSite { x: -1.0, y: 0.0 },
        InputSite { x: 0.0, y: 0.0 },
        InputSite { x: 1.0, y: 0.0 },
        InputSite { x: 0.0, y: 1.0 },
        InputSite { x: 0.0, y: -1.0 },
    ];

    Voronoi::build(sites, -10., -10., 10., 10.).get_polygons();
}
