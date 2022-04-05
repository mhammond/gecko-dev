/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;

// https://rust-lang.github.io/rust-clippy/master/index.html#float_cmp
// Silence, clippy!
const EPSILON: f64 = 0.0001f64;

#[derive(Debug, Clone)]
pub struct Point {
    coord_x: f64,
    coord_y: f64,
    description: String,
}

#[derive(Debug, Clone)]
pub struct Line {
    start: Point,
    end: Point,
}

#[derive(Debug, thiserror::Error)]
pub enum GeometryError {
    #[error("The gradient is undefined")]
    UndefinedGradient,
}

#[derive(Debug, thiserror::Error)]
pub enum ComplexGeometryError {
    #[error("No Intersection because {reason}")]
    NoIntersection { reason: String, code: u32 },
}

pub fn gradient(ln: Line) -> Result<f64, GeometryError> {
    let rise = ln.end.coord_y - ln.start.coord_y;
    let run = ln.end.coord_x - ln.start.coord_x;
    if run == 0f64 {
        return Err(GeometryError::UndefinedGradient);
    }
    Ok(rise / run)
}

pub fn intersection(ln1: Line, ln2: Line) -> Result<Point, ComplexGeometryError> {
    // TODO: yuck, should be able to take &Line as argument here
    // and have rust figure it out with a bunch of annotations...
    let g1 = gradient(ln1.clone()).map_err(|_| ComplexGeometryError::NoIntersection {
        reason: "Line has no gradient".to_string(),
        code: 1,
    })?;
    let z1 = ln1.start.coord_y - g1 * ln1.start.coord_x;
    let g2 = gradient(ln2.clone()).map_err(|_| ComplexGeometryError::NoIntersection {
        reason: "Line has no gradient".to_string(),
        code: 1,
    })?;
    let z2 = ln2.start.coord_y - g2 * ln2.start.coord_x;
    // Parallel lines do not intersect.
    if (g1 - g2).abs() < EPSILON {
        return Err(ComplexGeometryError::NoIntersection {
            reason: "Parallel lines do not intersect".to_string(),
            code: 2,
        });
    }
    // Otherwise, they intersect at this fancy calculation that
    // I found on wikipedia.
    let x = (z2 - z1) / (g1 - g2);
    Ok(Point {
        coord_x: x,
        coord_y: g1 * x + z1,
        description: Default::default(),
    })
}

pub fn string_round(s: String) -> String {
    std::str::from_utf8(s.as_bytes()).unwrap();
    s
}

pub fn string_record_round(p: Point) -> Point {
    std::str::from_utf8(p.description.as_bytes()).unwrap();
    p
}

pub fn arr_round(arr: Vec<String>, size: u32) -> Vec<String> {
    assert_eq!(arr.len(), size as usize);
    for s in arr.clone() {
        std::str::from_utf8(s.as_bytes()).unwrap();
    }
    arr
}

pub fn map_round(map: HashMap<String, String>, size: u32) -> HashMap<String, String> {
    assert_eq!(map.len(), size as usize);
    for (key, value) in map.clone() {
        std::str::from_utf8(key.as_bytes()).unwrap();
        std::str::from_utf8(value.as_bytes()).unwrap();
    }
    map
}

include!(concat!(env!("OUT_DIR"), "/geometry.uniffi.rs"));
