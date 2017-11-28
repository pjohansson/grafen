//!  Define and construct 3D volume objects.

mod cuboid;
mod cylinder;
mod sphere;

use coord::{Coord, Direction, Periodic};
use describe::Describe;
use system::Residue;

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
    fn fill(self, fill_type: FillType) -> Self;

    /// Return the object volume in units cubed.
    fn volume(&self) -> f64;
}

#[derive(Clone, Copy, Debug)]
/// Variants for how a volume can be filled.
pub enum FillType {
    /// An input density from which a number of coordinates to fill with is calculated.
    Density(f64),
    /// An absolute number of coordinates.
    NumCoords(u64),
}

impl FillType {
    /// Unwrap the number of coordinates by either calculating it using the density and volume
    /// of the input object, or return it.
    fn to_num_coords<T: Volume>(&self, volume: &T) -> u64 {
        match *self {
            FillType::Density(density) => (volume.volume() * density).round() as u64,
            FillType::NumCoords(num) => num,
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use system::Atom;

    #[test]
    fn fill_type_returns_correct_numbers() {
        let size = Coord::new(1.0, 2.0, 3.0);
        let cuboid = Cuboid {
            size,
            .. Cuboid::default()
        };

        let density = 15.6;
        let expected_num_coords = (cuboid.volume() * density).round() as u64;

        assert_eq!(FillType::Density(density).to_num_coords(&cuboid), expected_num_coords);

        let num = 11;
        assert_eq!(FillType::NumCoords(num).to_num_coords(&cuboid), num);
    }

    #[test]
    fn coordinates_within_cuboid_are_pruned() {
        let residue = resbase!["RES", ("A", 0.0, 0.0, 0.0)];
        let cuboid = Cuboid {
            size: Coord::new(1.0, 1.0, 1.0),
            .. Cuboid::default()
        };

        let coords_within = vec![
            Coord::new(0.1, 0.1, 0.1),
            Coord::new(0.5, 0.5, 0.5),
            Coord::new(0.9, 0.9, 0.9)
        ];

        let coords_without = vec![
            Coord::new(-0.1, 0.1, 0.1),
            Coord::new(1.1, 0.9, 0.9)
        ];

        let coords = coords_within
            .iter()
            .chain(coords_without.iter())
            .cloned()
            .collect::<Vec<Coord>>();

        let pruned = prune_residues_from_volume(&coords, &residue, &cuboid);

        assert_eq!(coords_without, pruned);
    }

    #[test]
    fn coordinates_within_cuboid_prune_with_respect_to_residue_atoms() {
        let residue = resbase![
            "RES",
            ("A", 0.0, 0.0, 0.0),
            ("B", 1.0, 0.0, 0.0) // Shifted by 1
        ];
        let cuboid = Cuboid {
            size: Coord::new(1.0, 1.0, 1.0),
            .. Cuboid::default()
        };

        let coords_within = vec![
            Coord::new(-0.9, 0.1, 0.1), // Atom B within
            Coord::new(-0.5, 0.5, 0.5),
            Coord::new(-0.1, 0.9, 0.9),
            Coord::new(0.1, 0.9, 0.9), // Atom A within
            Coord::new(0.5, 0.9, 0.9),
            Coord::new(0.9, 0.9, 0.9)
        ];

        let coords_without = vec![
            Coord::new(-1.1, 0.1, 0.1),
            Coord::new(1.1, 0.9, 0.9)
        ];

        let coords = coords_within
            .iter()
            .chain(coords_without.iter())
            .cloned()
            .collect::<Vec<Coord>>();

        let pruned = prune_residues_from_volume(&coords, &residue, &cuboid);

        assert_eq!(coords_without, pruned);
    }
}
