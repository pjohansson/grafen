//! Construct substrates of given types.

use config::InputSize;
use lattice::{Coord, Lattice};

/// A system with a list of atoms belonging to it.
pub struct System {
    /// System dimensions.
    pub dimensions: Coord,
    /// List of atoms.
    pub atoms: Vec<Atom>
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
    pub position: Coord
}

/// Substrate types.
pub enum SubstrateType {
    Graphene,
    Silica
}
use self::SubstrateType::*;

/// A base for generating atoms belonging to a residue.
/// Every residue has a name and a list of atoms that belong to it
/// with their relative base coordinates. The names are static since
/// they are generated only once from a single source.
struct ResidueBase {
    /// Residue code.
    code: &'static str,
    /// List of atoms belonging to the residue.
    atoms: Vec<ResidueAtom>
}

impl ResidueBase {
    /// Graphene is a single carbon atom at each lattice point.
    fn graphene(bond_length: f64) -> ResidueBase {
        ResidueBase {
            code: "GRPH",
            atoms: vec![
                ResidueAtom {
                    code: "C",
                    position: Coord::new(bond_length/2.0, bond_length/2.0, bond_length/2.0)
                }
            ]
        }
    }

    /// Silica is a rigid SiO2 molecule at each lattice point.
    fn silica(bond_length: f64) -> ResidueBase {
        let z0 = 0.000;
        let dz = 0.151;
        let base_coord = Coord::new(bond_length/4.0, bond_length/6.0, z0);

        ResidueBase {
            code: "SIO",
            atoms: vec![
                ResidueAtom { code: "O1", position: base_coord.add(&Coord::new(0.0, 0.0,  dz)) },
                ResidueAtom { code: "SI", position: base_coord },
                ResidueAtom { code: "O2", position: base_coord.add(&Coord::new(0.0, 0.0, -dz)) }
            ]
        }
    }
}

#[derive(Clone, Copy)]
/// Every atom in a residue has their own code and relative
/// position some base coordinate.
struct ResidueAtom  {
    /// Atom code.
    code: &'static str,
    /// Relative position.
    position: Coord
}

/// Create a substrate of desired input size and type. The returned system's
/// size will be adjusted to a multiple of the substrate spacing along both
/// directions. Thus the system can be periodically replicated along x and y.
///
/// # Examples
/// Create a graphene substrate:
///
/// ```
/// let graphene = create_substrate(InputSize(5.0, 4.0), SubstrateType::Graphene);
/// ```
///
/// # Errors
/// Returns an Error if the either of the input size are non-positive.
pub fn create_substrate(size: InputSize,
                        substrate_type: SubstrateType)
                        -> Result<System, String> {
    let InputSize(size_x, size_y) = size;
    if size_x <= 0.0 || size_y <= 0.0 {
        return Err("input sizes of the system have to be positive".to_string());
    }

    let substrate = match substrate_type {
        Graphene => create_graphene(size),
        Silica => create_silica(size),
    };

    Ok(substrate)
}

/// Create a graphene layer of desired size.
///
/// The layer consists of a hexagonal lattice of carbon atoms
/// which is created with a bond length of 0.142 nm. To ensure
/// that the system can be periodically replicated along x and y
/// the dimensions are trimmed to the closest possible size
/// that fits an even number of replicas.
fn create_graphene(InputSize(size_x, size_y): InputSize) -> System {
    let bond_length = 0.142;
    let z0 = bond_length;
    let residue_base = ResidueBase::graphene(bond_length);

    let lattice = Lattice::hexagonal(bond_length)
                          .from_size(size_x, size_y)
                          .finalize()
                          .translate(&Coord::new(0.0, 0.0, z0));

    let atoms = broadcast_residue_onto_coords(&lattice.coords, residue_base);

    System {
        dimensions: lattice.box_size.add(&Coord::new(0.0, 0.0, 2.0*z0)),
        atoms: atoms
    }
}

/// Create a silica layer of desired size.
///
/// The layer consists of a triclinic lattice where the spacing
/// is 0.45 along both vectors and the angle between them
/// is 60 degrees. At each lattice point an SiO2 molecule is placed.
fn create_silica(InputSize(size_x, size_y): InputSize) -> System {
    let bond_length = 0.450;
    let z0 = 0.30;
    let residue_base = ResidueBase::silica(bond_length);

    let lattice = Lattice::triclinic(bond_length, bond_length, 60f64.to_radians())
                          .from_size(size_x, size_y)
                          .finalize()
                          .translate(&Coord::new(0.0, 0.0, z0));

    let atoms = broadcast_residue_onto_coords(&lattice.coords, residue_base);

    System {
        dimensions: lattice.box_size.add(&Coord::new(0.0, 0.0, 2.0*z0)),
        atoms: atoms
    }
}

/// Use a constructed lattice and generate atoms from a residue
/// at their relative positions to this lattice.
fn broadcast_residue_onto_coords(coords: &Vec<Coord>,
                                 residue: ResidueBase)
                                 -> Vec<Atom> {
    let mut atoms: Vec<Atom> = Vec::new();

    for (residue_number, lattice_point) in coords.iter().enumerate() {
        for (atom_number, residue_atom) in residue.atoms.iter().enumerate() {
            let atom = Atom {
                residue_name: residue.code.to_string(),
                residue_number: residue_number as u64,
                atom_name: residue_atom.code.to_string(),
                atom_number: (residue.atoms.len()*residue_number) as u64
                             + (atom_number as u64),
                position: lattice_point.add(&residue_atom.position)
            };
            atoms.push(atom);
        }
    }

    atoms
}

#[cfg(test)]
mod tests {
    use std::f64;
    use super::*;

    #[test]
    fn graphene_layer() {
        let desired_size = InputSize(1.0, 1.0);
        let graphene = create_graphene(desired_size);

        // Assert that we get the expected dimensions which create
        // perfect PBC replicability
        let bond_length = 0.142;
        let spacing = (2.0*bond_length*f64::sqrt(3.0)/2.0, 3.0*bond_length);
        let dimensions = Coord::new(
            f64::round(desired_size.0/spacing.0) * spacing.0,
            f64::round(desired_size.1/spacing.1) * spacing.1,
            0.0);

        assert_eq!(dimensions, graphene.dimensions);

        // We expect 32 atoms to exist in the grid
        assert_eq!(32, graphene.atoms.len());

        // Verify the first atom
        let mut atoms = graphene.atoms.iter();
        let first_atom = Atom {
            residue_name: "GRPH".to_string(),
            residue_number: 0,
            atom_name: "C".to_string(),
            atom_number: 0,
            position: Coord::new(bond_length/2.0, bond_length/2.0, bond_length/2.0)
        };
        assert_eq!(Some(&first_atom), atoms.next());
    }
}
