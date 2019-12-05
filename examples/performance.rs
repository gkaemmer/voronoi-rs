extern crate voronoi;
extern crate rand;

use rand::prelude::*;
use std::time::Instant;
use voronoi::{Voronoi, InputSite};

fn main() {
    let count = 10000;
    let mut rng = rand::thread_rng();
    let sites: Vec<InputSite> = (0..count).map(|_i| InputSite {
        x: rng.gen(),
        y: rng.gen()
    }).collect();

    let now = Instant::now();
    Voronoi::build(sites);
    println!("Finding voronoi diagram of {} points took {}ms", count, now.elapsed().as_millis());
}
