//!  Define and construct 3D volume objects.

mod cuboid;
mod cylinder;
mod sphere;

use coord::{Coord, Direction, Periodic, Translate};
use describe::{unwrap_name, Describe};
use iterator::{AtomIterator, AtomIterItem};
use system::{Component, Residue};

use rand;
use rand::distributions::IndependentSample;
use std::f64::consts::PI;

pub use self::cuboid::Cuboid;
pub use self::cylinder::Cylinder;
pub use self::sphere::Sphere;

/// Volumes can contain coordinates.
pub trait Contains: Describe {
    /// Whether a coordinate is contained within the volume's space.
    fn contains(&self, coord: Coord) -> bool;
}

/// Traits for volume objects.
pub trait Volume: Contains {
    /// Fill the object with (roughly) uniformly distributed coordinates and return it.
    fn fill(self, num_coords: u64) -> Self;

    /// Return the object volume in units cubed.
    fn volume(&self) -> f64;
}

#[allow(dead_code)]
/// Helper function to cut a set of coordinates into a cylinder around a center point.
fn cut_to_cylinder(coords: &[Coord], bottom_center: Coord, alignment: Direction,
        radius: f64, height: f64) -> Vec<Coord> {
    coords.iter()
        .filter(|&c| {
            let (dr, dh) = bottom_center.distance_cylindrical(*c, alignment);
            dr <= radius && dh >= 0.0 && dh <= height
        })
        .map(|&c| c - bottom_center)
        .collect()
}

#[allow(dead_code)]
/// Helper function to cut a set of coordinates into a sphere around a center point.
fn cut_to_sphere(coords: &[Coord], center: Coord, radius: f64) -> Vec<Coord> {
    coords.iter()
        .filter(|c| c.distance(center) <= radius)
        .map(|&c| c - center)
        .collect()
}

/// Helper function to periodically replicate a set of coordinates for a volume object.
pub fn pbc_multiply_volume(coords: &[Coord], size: Coord, nx: usize, ny: usize, nz: usize)
        -> Vec<Coord> {
    let capacity = (nx * ny * nz) as usize * coords.len();

    // Only go through the loop if we have periodic multiples, otherwise just clone it.
    // Even if the compiler is good at rolling out the loops there are extra operations.
    match capacity {
        1 => {
            let mut new_coords = Vec::new();
            new_coords.extend_from_slice(coords);
            new_coords
        },
        _ => {
            let mut new_coords = Vec::with_capacity(capacity);

            for i in 0..nx {
                for j in 0..ny {
                    for k in 0..nz {
                        let pbc_add = size.pbc_multiply(i, j, k);

                        for &coord in coords {
                            new_coords.push(coord + pbc_add);
                        }
                    }
                }
            }

            new_coords
        },
    }
}

/// Return residue coordinates which are not contained by a volume.
///
/// Checks all atoms within the input residue to see if any are contained by the volume.
/// If any are, the residue coordinate is kept in the returned list.
pub fn prune_residues_from_volume<T: ?Sized>(coords: &[Coord], residue: &Residue, volume: &T)
        -> Vec<Coord> where T: Contains {
    coords.iter()
          .filter(|&c0| {
              residue.atoms
                .iter()
                .map(|ref atom| atom.position + *c0)
                .all(|c1| !volume.contains(c1))
          })
          .cloned()
          .collect::<Vec<_>>()
}
