//! Construct substrates of given types.

use error::{GrafenError, Result};
use lattice::Lattice;
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

/// Lattice types.
pub enum LatticeType {
    Hexagonal { a: f64 },
    Triclinic { a: f64, b: f64, gamma: f64 },
}

/// Create a substrate of input configuration. The returned system's
/// size will be adjusted to a multiple of the substrate spacing along both
/// directions. Thus the system can be periodically replicated along x and y.
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

    #[test]
    fn negative_sizes_return_error() {
        let mut conf = SubstrateConf {
            lattice: LatticeType::Hexagonal { a: 1.0 },
            residue: ResidueBase::graphene(1.0),
            size: (1.0, 1.0),
            std_z: None,
        };

        assert!(create_substrate(&conf).is_ok());

        conf.size = (-1.0, 1.0);
        assert!(create_substrate(&conf).is_err());

        conf.size = (1.0, -1.0);
        assert!(create_substrate(&conf).is_err());
    }
}
