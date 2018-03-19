//! Spherical objects.

use coord::Coord;

#[allow(dead_code)]
/// A spherical volume.
pub struct Sphere {
    pub origin: Coord,
    pub radius: f64,
    pub coords: Vec<Coord>,
}
