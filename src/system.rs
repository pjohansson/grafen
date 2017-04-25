pub struct Residue {
    pub code: &'static str,
    pub position: Coord,
    pub atoms: Vec<Atom>,
}

pub struct System {
    /// System dimensions.
    pub dimensions: Coord,
    /// List of residues.
    pub residues: Vec<Residue>,
}

impl System {
    pub fn num_atoms(&self) -> usize {
        self.residues.iter().map(|r| r.atoms.len()).sum()
    }

    pub fn translate(&self, add: &Coord) -> System {
        let residues = self.residues.iter().map(|r| {
            Residue {
                code: r.code,
                position: r.position.add(&add),
                atoms: r.atoms.clone(),
            }
        })
        .collect();

        System {
            dimensions: self.dimensions,
            residues: residues,
        }
    }
}

/// A base for generating atoms belonging to a residue.
/// Every residue has a name and a list of atoms that belong to it
/// with their relative base coordinates. The names are static since
/// they are generated only once from a single source.
pub struct ResidueBase {
    /// Residue code.
    pub code: &'static str,
    /// List of atoms belonging to the residue.
    pub atoms: Vec<Atom>,
}

impl ResidueBase {
    /// Graphene is a single carbon atom at each lattice point.
    pub fn graphene(bond_length: f64) -> ResidueBase {
        let dx = bond_length / 2.0;
        ResidueBase {
            code: "GRPH",
            atoms: vec![Atom {
                            code: "C",
                            position: Coord::new(dx, dx, dx),
                        }],
        }
    }

    /// Silica is a rigid SiO2 molecule at each lattice point.
    pub fn silica(bond_length: f64) -> ResidueBase {
        let z0 = 0.000;
        let dz = 0.151;
        let base_coord = Coord::new(bond_length / 4.0, bond_length / 6.0, z0);

        ResidueBase {
            code: "SIO",
            atoms: vec![Atom {
                            code: "O1",
                            position: base_coord.add(&Coord::new(0.0, 0.0, dz)),
                        },
                        Atom {
                            code: "SI",
                            position: base_coord,
                        },
                        Atom {
                            code: "O2",
                            position: base_coord.add(&Coord::new(0.0, 0.0, -dz)),
                        }],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
/// Every atom in a residue has their own code and relative
/// position some base coordinate.
pub struct Atom {
    /// Atom code.
    pub code: &'static str,
    /// Relative position.
    pub position: Coord,
}

#[derive(Clone, Copy, Debug)]
/// A three-dimensional coordinate.
///
/// # Examples
/// ```
/// use grafen::lattice::Coord;
///
/// let coord1 = Coord::new(1.0, 0.0, 1.0);
/// let coord2 = Coord::new(0.5, 0.5, 0.5);
///
/// assert_eq!(Coord::new(1.5, 0.5, 1.5), coord1.add(&coord2));
/// ```
pub struct Coord {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Coord {
    /// Construct a new coordinate.
    pub fn new(x: f64, y: f64, z: f64) -> Coord {
        Coord { x: x, y: y, z: z }
    }

    /// Add a coordinate to another.
    pub fn add(&self, other: &Coord) -> Coord {
        Coord::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }
}

impl PartialEq for Coord {
    fn eq(&self, other: &Coord) -> bool {
        let atol = 1e-9;
        (self.x - other.x).abs() < atol
            && (self.y - other.y).abs() < atol
            && (self.z - other.z).abs() < atol
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64;

    #[test]
    fn coord_translations() {
        let coord = Coord::new(0.0, 1.0, 2.0);
        let coord_add = coord.add(&coord);
        let expected = Coord::new(0.0, 2.0, 4.0);
        assert_eq!(expected, coord_add);
    }

    #[test]
    fn coord_eq_tolerance_small_deviation_passes() {
        // Allow for some deviation when testing for equality, since floating point
        // numbers are stupid.
        let coord = Coord::new(0.0, 0.0, 0.0);
        assert_eq!(coord, Coord::new(1e-10, 2e-10, 3e-10));
    }

    #[test]
    #[should_panic]
    fn coord_eq_tolerance_larger_deviation_does_not() {
        let coord = Coord::new(0.0, 0.0, 0.0);
        assert_eq!(coord, Coord::new(1e-9, 2e-9, 3e-9));
    }

    // A simple system with two different residues and five atoms
    fn setup_system() -> System {
        let coord0 = Coord::new(0.0, 0.0, 0.0);

        let residue_one = Residue {
            code: "R1",
            position: Coord::new(0.0, 0.0, 0.0),
            atoms: vec![
                Atom { code: "A1", position: coord0, },
                Atom { code: "A2", position: coord0, },
                Atom { code: "A3", position: coord0, },
            ]
        };
        let residue_two = Residue {
            code: "R2",
            position: Coord::new(1.0, 1.0, 1.0),
            atoms: vec![
                Atom { code: "B1", position: coord0, },
                Atom { code: "B2", position: coord0, },
            ]
        };

        System {
            dimensions: Coord::new(0.0, 0.0, 0.0),
            residues: vec![
                residue_one,
                residue_two
            ]
        }
    }

    #[test]
    fn count_atoms_in_system() {
        let system = setup_system();
        assert_eq!(5, system.num_atoms());
    }

    #[test]
    fn translate_a_system() {
        let system = setup_system();
        let translate = Coord::new(0.0, 1.0, 2.0);

        let translated_system = system.translate(&translate);
        for (orig, updated) in system.residues.iter().zip(translated_system.residues.iter()) {
            assert_eq!(orig.position.add(&translate), updated.position);
        }
    }
}
