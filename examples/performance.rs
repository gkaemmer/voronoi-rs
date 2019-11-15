extern crate voronoi;
extern crate rand;

use rand::prelude::*;
use std::time::Instant;
use voronoi::{Voronoi, Site};

fn main() {
    let count = 10000;
    let mut rng = rand::thread_rng();
    let sites: Vec<Site> = (1..count).map(|i| Site {
        x: rng.gen(),
        y: rng.gen(),
        id: i
    }).collect();

    let now = Instant::now();
    Voronoi::build(sites);
    println!("Finding voronoi diagram of {} points took {}ms", count, now.elapsed().as_millis());
}
