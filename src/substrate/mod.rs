//! Define and construct substrates.
//!
//! # Examples
//! Construct a triclinic lattice of hydrogen molecules.
//!
//! ```
//! use grafen::substrate::{create_substrate, LatticeType, SubstrateConf};
//! use grafen::system::{Atom, Coord, ResidueBase};
//!
//! // Define the molecule as a Residue.
//! let residue_base = ResidueBase {
//!     code: "HMOL",
//!     atoms: vec![
//!         Atom { code: "H1", position: Coord::new(0.0, 0.0, 0.5), },
//!         Atom { code: "H2", position: Coord::new(0.0, 0.0, -0.5), }
//!     ],
//! };
//!
//! // Define the substrate
//! let conf = SubstrateConf {
//!     lattice: LatticeType::Triclinic { a: 1.0, b: 0.5, gamma: 60.0 },
//!     residue: residue_base,
//!     size: (5.0, 10.0),
//!     std_z: None,
//! };
//!
//! // ... and create it!
//! let substrate = create_substrate(&conf).unwrap();
//! ```

mod lattice;

use error::{GrafenError, Result};
use substrate::lattice::Lattice;
use system::*;

/// Configuration for constructing a substrate.
pub struct SubstrateConf {
    /// The type of lattice which will be generated.
    pub lattice: LatticeType,
    /// Base residue to generate coordinates for.
    pub residue: ResidueBase,
    /// Desired size of substrate along x and y.
    pub size: (f64, f64),
    /// Optionally use a random uniform distribution with this
    /// deviation to shift residue positions along z. The
    /// positions are shifted with the range (-std_z, +std_z)
    /// where std_z is the input devation.
    pub std_z: Option<f64>,
}

/// Lattice types which a substrate can be constructed from.
pub enum LatticeType {
    /// A hexagonal (honey comb) lattice with bond spacing `a`.
    Hexagonal { a: f64 },
    /// A triclinic lattice with base vectors of length `a` and `b`.
    /// Vector `a` is directed along the x axis and vector `b` is separated
    /// to it by the input angle `gamma` in degrees.
    Triclinic { a: f64, b: f64, gamma: f64 },
}

/// Create a substrate of input configuration and return as a `System`.
///
/// The returned system's size will be adjusted to a multiple of the
/// substrate spacing along both directions. Thus the system can be
/// periodically replicated along x and y.
///
/// # Errors
/// Returns an Error if the either of the input size are non-positive.
pub fn create_substrate(conf: &SubstrateConf) -> Result<System> {
    let (dx, dy) = conf.size;
    if dx < 0.0 || dy < 0.0 {
        return Err(
            GrafenError::RunError("cannot create a substrate of negative size".to_string())
        );
    }

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

    Ok(System {
        dimensions: lattice.box_size,
        residues: lattice.coords.iter().map(|&coord| conf.residue.to_residue(&coord)).collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_conf() -> SubstrateConf {
        SubstrateConf {
            lattice: LatticeType::Hexagonal { a: 1.0 },
            residue: ResidueBase {
                code: "GRPH",
                atoms: vec![Atom { code: "C", position: Coord::new(0.0, 0.0, 0.0) }],
            },
            size: (10.0, 10.0),
            std_z: None,
        }
    }

    #[test]
    fn negative_sizes_return_error() {
        let mut conf = setup_conf();
        assert!(create_substrate(&conf).is_ok());

        conf.size = (-1.0, 1.0);
        assert!(create_substrate(&conf).is_err());

        conf.size = (1.0, -1.0);
        assert!(create_substrate(&conf).is_err());
    }

    #[test]
    fn uniform_distribution_is_set() {
        // The graphene is ordinarily positioned at z = 0.0
        let mut conf = setup_conf();
        let regular = create_substrate(&conf).unwrap();
        assert!(regular.residues.iter().all(|r| r.position.z == 0.0));

        conf.std_z = Some(1.0);
        let uniform = create_substrate(&conf).unwrap();

        // Non-zero variance This can fail, but it should not be common!
        // How else to assert that a distribution has been applied, though?
        assert!(uniform.residues.iter().map(|r| r.position.z).all(|z| z == 0.0) == false);

        // But no positions should exceed the input distribution max
        assert!(uniform.residues.iter().all(|r| r.position.z.abs() <= 1.0));
    }
}
