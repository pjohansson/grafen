//!  Define and construct 3D volume objects.

mod cuboid;
mod cylinder;
mod sphere;

use coord::{Coord, Direction, Periodic};
use iterator::ResidueIterOut;
use system::{Component};

pub use self::cuboid::Cuboid;
pub use self::cylinder::Cylinder;
pub use self::sphere::Sphere;

/// Volumes can contain coordinates.
pub trait Contains {
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

/// Return residues of an input `Component` which are not contained by a pruning volume.
///
/// Checks all atoms within residues to see if any are contained by the volume.
/// If any are, the residue is filtered from the returned list.
pub fn prune_residues_from_volume<'a, T, V>(component: &'a T, pruning_vol: &V)
     -> Vec<ResidueIterOut>
        where T: Component<'a>, V: ?Sized + Contains {
    let origin = component.get_origin();

    component
        .iter_residues()
        .filter(|res| {
            res.get_atoms()
                .iter()
                .map(|atom| atom.1 + origin)
                .all(|coord| !pruning_vol.contains(coord))
        })
        .collect()
}

/// Return residues of an input `Component` which are contained within an input volume.
///
/// Checks all atoms within residues to see if any are contained by it. If any are,
/// the residue is kept in the returned list.
pub fn keep_residues_within_volume<'a, T, V>(component: &'a T, containing_vol: &V)
     -> Vec<ResidueIterOut>
        where T: Component<'a>, V: ?Sized + Contains {
    let origin = component.get_origin();

    component
        .iter_residues()
        .filter(|res| {
            res.get_atoms()
                .iter()
                .map(|atom| atom.1 + origin)
                .any(|coord| containing_vol.contains(coord))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use system::{Atom, Residue};

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
        let pruning_vol = Cuboid {
            size: Coord::new(1.0, 1.0, 1.0),
            .. Cuboid::default()
        };

        let coords_within = vec![
            Coord::new(0.1, 0.1, 0.1),
            Coord::new(0.5, 0.5, 0.5),
            Coord::new(0.9, 0.9, 0.9)
        ];

        let coord1_without = Coord::new(-0.1, 0.1, 0.1);
        let coord2_without = Coord::new(1.1, 0.9, 0.9);

        let coords_without = vec![
            coord1_without,
            coord2_without
        ];

        let coords = coords_within
            .iter()
            .chain(coords_without.iter())
            .cloned()
            .collect::<Vec<Coord>>();

        let residue = resbase!["RES", ("A", 0.0, 0.0, 0.0)];
        let component = Cuboid {
            residue: Some(residue),
            coords,
            .. Cuboid::default()
        };

        let pruned = prune_residues_from_volume(&component, &pruning_vol);
        let atoms = pruned.iter().map(|res| res.get_atoms()).collect::<Vec<_>>();
        assert_eq!(atoms.len(), 2);
        assert_eq!(atoms[0][0].1, coord1_without);
        assert_eq!(atoms[1][0].1, coord2_without);
    }

    #[test]
    fn component_residues_are_pruned_if_any_atoms_are_inside_the_pruning_volume() {
        let pruning_vol = Cuboid {
            size: Coord::new(1.0, 1.0, 1.0),
            .. Cuboid::default()
        };

        let residue = resbase![
            "RES",
            ("A", 0.0, 0.0, 0.0),
            ("B", 1.0, 0.0, 0.0) // Shifted by 1
        ];

        let coords_within = vec![
            Coord::new(-0.9, 0.1, 0.1), // Atom B within
            Coord::new(-0.5, 0.5, 0.5),
            Coord::new(-0.1, 0.9, 0.9),
            Coord::new(0.1, 0.9, 0.9), // Atom A within
            Coord::new(0.5, 0.9, 0.9),
            Coord::new(0.9, 0.9, 0.9)
        ];

        // Neither of the atoms will be inside the volume for these coords
        let coord1_without = Coord::new(-1.1, 0.1, 0.1);
        let coord2_without = Coord::new(1.1, 0.9, 0.9);

        let coords_without = vec![
            coord1_without,
            coord2_without
        ];

        let coords = coords_within
            .iter()
            .chain(coords_without.iter())
            .cloned()
            .collect::<Vec<Coord>>();

        let component =  Cuboid {
            residue: Some(residue),
            coords,
            .. Cuboid::default()
        };

        let pruned = prune_residues_from_volume(&component, &pruning_vol);
        let atoms = pruned.iter().map(|res| res.get_atoms()).collect::<Vec<_>>();
        assert_eq!(atoms.len(), 2);

        let shift = Coord::new(1.0, 0.0, 0.0);
        assert_eq!(atoms[0][0].1, coord1_without);
        assert_eq!(atoms[0][1].1, coord1_without + shift);
        assert_eq!(atoms[1][0].1, coord2_without);
        assert_eq!(atoms[1][1].1, coord2_without + shift);
    }

    #[test]
    fn component_residues_are_kept_if_any_atoms_are_inside_the_containing_volume() {
        let containing_vol = Cuboid {
            size: Coord::new(1.0, 1.0, 1.0),
            .. Cuboid::default()
        };

        let residue = resbase![
            "RES",
            ("A", 0.0, 0.0, 0.0),
            ("B", 1.0, 0.0, 0.0) // Shifted by 1
        ];

        let coords_within = vec![
            Coord::new(-0.9, 0.1, 0.1), // Atom B within
            Coord::new(-0.5, 0.5, 0.5),
            Coord::new(-0.1, 0.9, 0.9),
            Coord::new(0.5, 0.9, 0.9),
            Coord::new(0.9, 0.9, 0.9)
        ];

        // Neither of the atoms will be inside the volume for these coords
        let coord1_without = Coord::new(-1.1, -0.1, -0.1);
        let coord2_without = Coord::new(1.1, 1.1, 1.1);

        let coords_without = vec![
            coord1_without,
            coord2_without
        ];

        let coords = coords_within
            .iter()
            .chain(coords_without.iter())
            .cloned()
            .collect::<Vec<Coord>>();

        let component =  Cuboid {
            residue: Some(residue),
            coords,
            .. Cuboid::default()
        };

        let contained = keep_residues_within_volume(&component, &containing_vol);
        let atoms = contained.iter().map(|res| res.get_atoms()).collect::<Vec<_>>();
        assert_eq!(atoms.len(), 5);
    }

    #[test]
    fn pruning_accounts_for_the_relative_translation_of_objects() {
        // cuboid from (1.0, 0.0, 0.0) to (3.0, 1.0, 1.0)
        let pruning_vol = Cuboid {
            origin: Coord::new(1.0, 0.0, 0.0),
            size: Coord::new(2.0, 1.0, 1.0),
            .. Cuboid::default()
        };

        let residue = resbase!["RES", ("A", 0.0, 0.0, 0.0)];
        let origin = Coord::new(1.0, 0.0, 0.0);
        let coords = vec![
            Coord::new(0.5, 0.5, 0.5), // at (1.5, 0.5, 0.5): within the other cuboid
            Coord::new(1.5, 0.5, 0.5), // at (2.5, 0.5, 0.5): within the other cuboid
            Coord::new(2.5, 0.5, 0.5)  // at (3.5, 0.5, 0.5): outside the other cuboid
        ];

        let component = Cuboid {
            origin,
            residue: Some(residue),
            coords,
            .. Cuboid::default()
        };

        let pruned = prune_residues_from_volume(&component, &pruning_vol);
        let atoms = pruned.iter().map(|res| res.get_atoms()).collect::<Vec<_>>();
        assert_eq!(atoms.len(), 1);
        assert_eq!(atoms[0][0].1, Coord::new(2.5, 0.5, 0.5));
    }
}
