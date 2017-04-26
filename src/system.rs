//! This module implements the basic structures of an atomic system.
//!
//! A final ```System``` consists of a set of ```Residue```s, which
//! can be moved around and translated with ease. Each ```Residue```
//! in turn consists of some ```Atom```s. These atoms have positions
//! relative to their parent.
//!
//! This somewhat convoluted structure is inherited from molecular
//! simulation packages in which atoms are commonly grouped as such.
//!  A proper physical way to look at is that atoms can be
//! similarly grouped into molecules.

/// A finalized atomic system which consists of a list of residues,
/// each of which contains some atoms.
pub struct System {
    /// System dimensions.
    pub dimensions: Coord,
    /// List of residues.
    pub residues: Vec<Residue>,
}

impl System {
    /// Count and return the number of atoms in the system.
    pub fn num_atoms(&self) -> usize {
        self.residues.iter().map(|r| r.atoms.len()).sum()
    }

    /// Translate all residues within the system and return a copy.
    pub fn translate(&self, add: &Coord) -> System {
        System {
            dimensions: self.dimensions,
            residues: self.residues.iter().map(|r| r.translate(&add)).collect(),
        }
    }
}

/// Every residue has a name and a list of atoms that belong to it
/// with their relative base coordinates. The names are static since
/// they are generated only once from a single source.
pub struct Residue {
    /// Residue code.
    pub code: &'static str,
    /// Position of residue in system.
    pub position: Coord,
    /// List of atoms belonging to the residue. Their positions are relative to the residue.
    pub atoms: Vec<Atom>,
}

impl Residue {
    /// Translate the residue position. Does not alter the atom relative positions.
    fn translate(&self, add: &Coord) -> Residue {
        Residue {
            code: self.code,
            position: self.position.add(&add),
            atoms: self.atoms.clone(),
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

/// A base for generating atoms belonging to a residue.
#[derive(Debug, PartialEq)]
pub struct ResidueBase {
    pub code: &'static str,
    pub atoms: Vec<Atom>,
}

#[macro_export]
/// Construct a ResidueBase with a code and atoms.
///
/// At least one atom has to be present in the base. This is not a limitation
/// when explicitly constructing a residue, but it makes no sense to allow
/// it when invoking a constructor like this.
///
/// # Examples:
/// ```
/// # #[macro_use] extern crate grafen;
/// use grafen::system::{Atom, Coord, ResidueBase};
/// # fn main() {
///
/// let expect = ResidueBase {
///     code: "RES",
///     atoms: vec![
///         Atom { code: "A", position: Coord::new(0.0, 0.0, 0.0) },
///         Atom { code: "B", position: Coord::new(1.0, 2.0, 3.0) }
///     ],
/// };
///
/// let residue = resbase![
///     "RES",
///     ("A", 0.0, 0.0, 0.0),
///     ("B", 1.0, 2.0, 3.0)
/// ];
///
/// assert_eq!(expect, residue);
/// # }
/// ```
macro_rules! resbase {
    (
        $rescode:expr,
        $(($atname:expr, $x:expr, $y:expr, $z:expr)),+
    ) => {
        {
            let mut temp_vec = Vec::new();
            $(
                temp_vec.push(
                    Atom {
                        code: $atname,
                        position: Coord::new($x, $y, $z),
                    }
                );
            )*

            ResidueBase {
                code: $rescode,
                atoms: temp_vec,
            }
        }
    }
}

impl ResidueBase {
    /// Generate a proper residue at the input position.
    pub fn to_residue(&self, position: &Coord) -> Residue {
        Residue {
            code: self.code,
            position: *position,
            atoms: self.atoms.clone(),
        }
    }

    /// Graphene is a single carbon atom at each lattice point.
    pub fn graphene(bond_length: f64) -> ResidueBase {
        let dx = bond_length / 2.0;

        resbase!["GRPH", ("C", dx, dx, 0.0)]
    }

    /// Silica is a rigid SiO2 molecule at each lattice point.
    pub fn silica(bond_length: f64) -> ResidueBase {
        let x0 = bond_length / 4.0;
        let y0 = bond_length / 6.0;
        let dz = 0.151;

        resbase![
            "SIO",
            ("O1", x0, y0, dz),
            ("SI", x0, y0, 0.0),
            ("O2", x0, y0, -dz)
        ]
    }
}

#[derive(Clone, Copy, Debug)]
/// A three-dimensional coordinate.
///
/// # Examples
/// ```
/// use grafen::system::Coord;
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

    #[test]
    fn residue_base_to_residue() {
        let base = ResidueBase {
            code: "RES",
            atoms: vec![
                Atom { code: "A1", position: Coord::new(0.0, 0.0, 0.0) },
                Atom { code: "A2", position: Coord::new(0.0, 1.0, 2.0) }
            ],
        };
        let position = Coord::new(1.0, 1.0, 1.0);

        let residue = base.to_residue(&position);
        assert_eq!("RES", residue.code);
        assert_eq!(position, residue.position);
        assert_eq!(base.atoms, residue.atoms);
    }

    #[test]
    fn create_residue_base_macro() {
        let expect = ResidueBase {
            code: "RES",
            atoms: vec![
                Atom { code: "A1", position: Coord::new(0.0, 0.0, 0.0) },
                Atom { code: "A2", position: Coord::new(0.0, 1.0, 2.0) }
            ],
        };
        let result = resbase![
            "RES",
            ("A1", 0.0, 0.0, 0.0),
            ("A2", 0.0, 1.0, 2.0)
        ];

        assert_eq!(expect, result);
    }
}
