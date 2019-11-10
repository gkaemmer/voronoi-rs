const EPSILON: f32 = 1.00e-9_f32;

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

pub fn equals_with_epsilon(a: f32, b: f32) -> bool {
    return (a - b).abs() < EPSILON;
}

pub fn breakpoint_between(x1: f32, y1: f32, x2: f32, y2: f32, directrix: f32) -> f32 {
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
