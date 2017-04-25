//! Construct substrates of given types.

use error::{GrafenError, Result};
use lattice::{Coord, Lattice};

/// A system with a list of atoms belonging to it.
pub struct System {
    /// System dimensions.
    pub dimensions: Coord,
    /// List of atoms.
    pub atoms: Vec<Atom>,
}

impl System {
    pub fn translate(&self, add: &Coord) -> System {
        let atoms = self.atoms
            .iter()
            .map(|a| {
                Atom {
                    residue_name: a.residue_name.clone(),
                    residue_number: a.residue_number,
                    atom_name: a.atom_name.clone(),
                    atom_number: a.atom_number,
                    position: a.position.add(&add),
                }
            })
            .collect();

        System {
            dimensions: self.dimensions,
            atoms: atoms,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
/// Every atom in a system has some information connected to it
/// which is used when writing the output.
pub struct Atom {
    /// Code for the parent residue name.
    pub residue_name: String,
    /// Number of residue (0-indexed).
    pub residue_number: u64,
    /// Code for the atom name.
    pub atom_name: String,
    /// Number of the atom (0-indexed).
    pub atom_number: u64,
    /// Absolute atom position in a system.
    pub position: Coord,
}

#[derive(Clone, Copy)]
/// Substrate types.
pub enum SubstrateType {
    Graphene,
    Silica,
}
use self::SubstrateType::*;

#[derive(Clone, Copy, Debug)]
/// Configuration used for substrate construction.
pub struct Config {
    /// Desired size of substrate along x and y.
    pub size: (f64, f64),
    /// Substrate position along z.
    pub z0: f64,
    /// Optionally use a random uniform distribution with this
    /// deviation to shift residue positions along z. The
    /// positions are shifted with the range (-std_z, +std_z)
    /// where std_z is the input devation.
    pub std_z: Option<f64>,
}

/// A base for generating atoms belonging to a residue.
/// Every residue has a name and a list of atoms that belong to it
/// with their relative base coordinates. The names are static since
/// they are generated only once from a single source.
pub struct ResidueBase {
    /// Residue code.
    pub code: &'static str,
    /// List of atoms belonging to the residue.
    pub atoms: Vec<ResidueAtom>,
}

impl ResidueBase {
    /// Graphene is a single carbon atom at each lattice point.
    pub fn graphene(bond_length: f64) -> ResidueBase {
        let dx = bond_length / 2.0;
        ResidueBase {
            code: "GRPH",
            atoms: vec![ResidueAtom {
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
            atoms: vec![ResidueAtom {
                            code: "O1",
                            position: base_coord.add(&Coord::new(0.0, 0.0, dz)),
                        },
                        ResidueAtom {
                            code: "SI",
                            position: base_coord,
                        },
                        ResidueAtom {
                            code: "O2",
                            position: base_coord.add(&Coord::new(0.0, 0.0, -dz)),
                        }],
        }
    }
}

#[derive(Clone, Copy)]
/// Every atom in a residue has their own code and relative
/// position some base coordinate.
pub struct ResidueAtom {
    /// Atom code.
    pub code: &'static str,
    /// Relative position.
    pub position: Coord,
}

pub struct SubstrateConf {
    pub lattice: LatticeType,
    pub residue: ResidueBase,
    pub size: (f64, f64),
    pub std_z: Option<f64>,
}

pub enum LatticeType {
    Hexagonal { a: f64 },
    Triclinic { a: f64, b: f64, gamma: f64 },
}

/// Create a substrate of desired input size and type. The returned system's
/// size will be adjusted to a multiple of the substrate spacing along both
/// directions. Thus the system can be periodically replicated along x and y.
///
/// # Examples
/// Create a graphene substrate:
///
/// ```
/// use grafen::substrates::{create_substrate, Config, SubstrateType};
/// let conf = Config {
///     size: (5.0, 4.0),
///     z0: 0.10,
///     std_z: None,
/// };
/// let graphene = create_substrate(&conf, SubstrateType::Graphene);
/// ```
///
/// # Errors
/// Returns an Error if the either of the input size are non-positive.
pub fn create_substrate(conf: &SubstrateConf) -> Result<System> {
    let (dx, dy) = conf.size;

    let mut lattice = match conf.lattice {
        LatticeType::Hexagonal { a } => {
            Lattice::hexagonal(a)
        },
        LatticeType::Triclinic { a, b, gamma } => {
            Lattice::triclinic(a, b, gamma.to_radians())
        },
    }.with_size(dx, dy).finalize();

    if let Some(std) = conf.std_z {
        lattice = lattice.uniform_distribution(std);
    };

    let atoms = broadcast_residue_onto_coords(&lattice.coords, &conf.residue);

    Ok(System {
        dimensions: lattice.box_size,
        atoms: atoms,
    })
}

/// Use a constructed lattice and generate atoms from a residue
/// at their relative positions to this lattice.
fn broadcast_residue_onto_coords(coords: &Vec<Coord>, residue: &ResidueBase) -> Vec<Atom> {
    let mut atoms: Vec<Atom> = Vec::new();

    for (residue_number, lattice_point) in coords.iter().enumerate() {
        for (atom_number, residue_atom) in residue.atoms.iter().enumerate() {
            let atom = Atom {
                residue_name: residue.code.to_string(),
                residue_number: residue_number as u64,
                atom_name: residue_atom.code.to_string(),
                atom_number: (residue.atoms.len() * residue_number) as u64 + (atom_number as u64),
                position: lattice_point.add(&residue_atom.position),
            };
            atoms.push(atom);
        }
    }

    atoms
}
