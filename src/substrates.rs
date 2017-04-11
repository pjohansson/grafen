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

#[derive(Debug, PartialEq)]
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
    /// standard deviation to shift residue positions along z.
    pub std_z: Option<f64>,
}

/// A base for generating atoms belonging to a residue.
/// Every residue has a name and a list of atoms that belong to it
/// with their relative base coordinates. The names are static since
/// they are generated only once from a single source.
struct ResidueBase {
    /// Residue code.
    code: &'static str,
    /// List of atoms belonging to the residue.
    atoms: Vec<ResidueAtom>,
}

impl ResidueBase {
    /// Graphene is a single carbon atom at each lattice point.
    fn graphene(bond_length: f64) -> ResidueBase {
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
    fn silica(bond_length: f64) -> ResidueBase {
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
struct ResidueAtom {
    /// Atom code.
    code: &'static str,
    /// Relative position.
    position: Coord,
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
pub fn create_substrate(conf: &Config, substrate_type: SubstrateType) -> Result<System> {
    let (size_x, size_y) = conf.size;
    if size_x <= 0.0 || size_y <= 0.0 {
        return Err(
            GrafenError::RunError(
                "input sizes of the system have to be positive".to_string()
            ));
    }

    let substrate = match substrate_type {
        Graphene => {
            create_graphene(&conf)
        },
        Silica => {
            create_silica(&conf)
        },
    };

    Ok(substrate)
}

/// Create a graphene layer.
///
/// The layer consists of a hexagonal lattice of carbon atoms
/// which is created with a bond length of 0.142 nm. To ensure
/// that the system can be periodically replicated along x and y
/// the dimensions are trimmed to the closest possible size
/// that fits an even number of replicas.
fn create_graphene(conf: &Config) -> System {
    let bond_length = 0.142;
    let residue_base = ResidueBase::graphene(bond_length);

    let mut lattice = Lattice::hexagonal(bond_length)
        .from_size(conf.size.0, conf.size.1)
        .finalize()
        .translate(&Coord::new(0.0, 0.0, conf.z0));

    if let Some(std_z) = conf.std_z {
        lattice = lattice.uniform_distribution(std_z);
    }

    let atoms = broadcast_residue_onto_coords(&lattice.coords, residue_base);

    System {
        dimensions: lattice.box_size.add(&Coord::new(0.0, 0.0, 2.0 * conf.z0)),
        atoms: atoms,
    }
}

/// Create a silica layer of desired size.
///
/// The layer consists of a triclinic lattice where the spacing
/// is 0.45 along both vectors and the angle between them
/// is 60 degrees. At each lattice point an SiO2 molecule is placed.
fn create_silica(conf: &Config) -> System {
    let bond_length = 0.450;
    let residue_base = ResidueBase::silica(bond_length);

    let mut lattice = Lattice::triclinic(bond_length, bond_length, 60f64.to_radians())
        .from_size(conf.size.0, conf.size.1)
        .finalize()
        .translate(&Coord::new(0.0, 0.0, conf.z0));

    if let Some(std_z) = conf.std_z {
        lattice = lattice.uniform_distribution(std_z);
    }

    let atoms = broadcast_residue_onto_coords(&lattice.coords, residue_base);

    System {
        dimensions: lattice.box_size.add(&Coord::new(0.0, 0.0, 2.0 * conf.z0)),
        atoms: atoms,
    }
}

/// Use a constructed lattice and generate atoms from a residue
/// at their relative positions to this lattice.
fn broadcast_residue_onto_coords(coords: &Vec<Coord>, residue: ResidueBase) -> Vec<Atom> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn graphene_layer() {
        let conf = Config {
            size: (1.0, 1.0),
            z0: 0.1,
            std_z: None,
        };

        let graphene = create_graphene(&conf);

        // Verify the first atom
        let mut atoms = graphene.atoms.iter();
        let bond_length = 0.142;
        let first_atom = Atom {
            residue_name: "GRPH".to_string(),
            residue_number: 0,
            atom_name: "C".to_string(),
            atom_number: 0,
            position: Coord::new(
                bond_length / 2.0,
                bond_length / 2.0,
                conf.z0 + bond_length / 2.0
            ),
        };
        assert_eq!(Some(&first_atom), atoms.next());
    }
}
