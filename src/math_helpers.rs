const EPSILON: f64 = 1.00e-12_f64;

// Returns the x coordinate of the intersection of two beach segments
//
//                                 *                .
//                                 f2             ..
//    .                                        ...
//     .       f1        .*..             .....
//      ..      *      .. |  .............
//        ....     ....   |
//            .....       |
//                        |
// -----------------------X--------------------------- directrix
//

pub fn equals_with_epsilon(a: f64, b: f64) -> bool {
    return (a - b).abs() < EPSILON;
}

pub fn breakpoint_between(x1: f64, y1: f64, x2: f64, y2: f64, directrix: f64) -> f64 {
    // Credit to:
    // https://www.wolframalpha.com/input/?i=solve+%28x1+-+h%29%5E2+%2B+%28y1-k%29%5E2+%3D+%28k+-+s%29%5E2%2C+%28x2+-+h%29%5E2+%2B+%28y2+-+k%29%5E2+%3D+%28k+-+s%29%5E2+for+h%2C+k
    if equals_with_epsilon(y1, y2) {
        // y's are equal, so just average x's to get x
        return (x1 + x2) / 2.0;
    }
    let s = directrix;
    let sqrt = -((s*s-s*y1-s*y2+y1*y2)*(x1*x1-2.0*x1*x2+x2*x2+y1*y1-2.0*y1*y2+y2*y2)).sqrt();
    return (-sqrt+s*x1-s*x2-x1*y2+x2*y1)/(y1-y2);
}

// Finds the point equidistant to all given points, and also returns the
// distance to that point
pub fn find_center(x1: f64, y1: f64, x2: f64, y2: f64, x3: f64, y3: f64) -> Option<(f64, f64, f64)> {
    let temp = x2 * x2 + y2 * y2;
    let bc = (x1 * x1 + y1 * y1 - temp) / 2.0;
    let cd = (temp - x3 * x3 - y3 * y3) / 2.0;
    let det = (x1 - x2) * (y2 - y3) - (x2 - x3) * (y1 - y2);

    // If determinant is 0, these points are colinear and there is no center
    if det.abs() < EPSILON { return None; }

    let cx = (bc * (y2 - y3) - cd * (y1 - y2)) / det;
    let cy = ((x1 - x2) * cd - (x2 - x3) * bc) / det;
    let dx = cx - x1;
    let dy = cy - y1;
    let rad = (dx*dx + dy*dy).sqrt();
    return Some((cx, cy, rad));
}
