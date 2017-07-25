//! Define and construct substrates.
//!
//! # Examples
//! Construct a triclinic lattice of hydrogen molecules.
//!
//! ```
//! # use grafen::substrate::{create_substrate, LatticeType, SheetConf};
//! # use grafen::system::{Atom, Coord, ResidueBase};
//! // Define the molecule as a Residue.
//! let residue_base = ResidueBase {
//!     code: "HMOL".to_string(),
//!     atoms: vec![
//!         Atom { code: "H1".to_string(), position: Coord::new(0.0, 0.0, 0.5), },
//!         Atom { code: "H2".to_string(), position: Coord::new(0.0, 0.0, -0.5), }
//!     ],
//! };
//!
//! // Define the substrate
//! let conf = SheetConf {
//!     lattice: LatticeType::Triclinic { a: 1.0, b: 0.5, gamma: 60.0 },
//!     residue: residue_base,
//!     size: (5.0, 10.0),
//!     std_z: None,
//! };
//!
//! // ... and create it!
//! let substrate = create_substrate(&conf).unwrap();
//! ```

mod distribution;
mod lattice;
mod points;

use error::{GrafenError, Result};
use substrate::distribution::PoissonDistribution;
use substrate::lattice::Lattice;
use system::*;

#[derive(Clone, Debug, PartialEq)]
/// Configuration for constructing a substrate.
pub struct SheetConf {
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

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
/// Lattice types which a substrate can be constructed from.
pub enum LatticeType {
    /// A hexagonal (honey comb) lattice with bond spacing `a`.
    Hexagonal { a: f64 },
    /// A triclinic lattice with base vectors of length `a` and `b`.
    /// Vector `a` is directed along the x axis and vector `b` is separated
    /// to it by the input angle `gamma` in degrees.
    Triclinic { a: f64, b: f64, gamma: f64 },
    /// A Poisson disc distribution of points with an input `density` in number
    /// of points per unit area. It is implemented using Bridson's algorithm
    /// which ensures that no points are within sqrt(2 / (pi * density)) of
    /// each other. This creates a good match to the input density.
    ///
    /// *Fast Poisson disk sampling in arbitrary dimensions*,
    ///  R. Bridson, ACM SIGGRAPH 2007 Sketches Program,
    ///  http://www.cs.ubc.ca/~rbridson/docs/bridson-siggraph07-poissondisk.pdf
    PoissonDisc { density: f64 },
}

#[derive(Clone, Debug)]
/// A `Sheet` of `Residue`s in some points.
pub struct Sheet {
    /// Sheet origin position. Residue coordinates are relative to this.
    pub origin: Coord,
    /// Sheet size.
    pub size: Coord,
    /// Residue base.
    pub residue_base: ResidueBase,
    /// `Residue`s belonging to the sheet.
    pub residue_coords: Vec<Coord>,
}

impl Sheet {
    /// Cut the `Sheet` into a circle of input radius.
    ///
    /// This method assumes that the sheet is aligned in the xy plane and that
    /// coordinates are set at z = 0.0. Furthermore, it cuts the circle around
    /// the center point (radius, radius, 0).
    ///
    /// The final circle will be centered around (0, 0, 0).
    fn into_circle(self, radius: f64) -> Self {
        const ORIGO: Coord = Coord { x: 0.0, y: 0.0, z: 0.0 };
        let center = Coord::new(radius, radius, 0.0);

        let residue_coords = self.residue_coords
            .iter()
            .inspect(|coord| println!("{:?}", coord))
            .map(|&coord| coord.with_pbc(self.size) - center)
            .filter(|coord| coord.distance(ORIGO) <= radius)
            .collect();

        Sheet {
            origin: Coord::new(0.0, 0.0, 0.0),
            size: Coord::new(2.0 * radius, 2.0 * radius, self.size.z),
            residue_base: self.residue_base,
            residue_coords,
        }
    }
}

impl IntoComponent for Sheet {
    fn to_component(&self) -> Component {
        Component {
            origin: self.origin,
            box_size: self.size,
            residue_base: self.residue_base.clone(),
            residue_coords: self.residue_coords.clone(),
        }
    }

    fn into_component(self) -> Component {
        Component {
            origin: self.origin,
            box_size: self.size,
            residue_base: self.residue_base,
            residue_coords: self.residue_coords,
        }
    }

    fn num_atoms(&self) -> usize {
        self.residue_base.atoms.len() * self.residue_coords.len()
    }
}

impl Translate for Sheet {
    fn translate(mut self, trans: &Coord) -> Sheet {
        self.origin = self.origin + *trans;
        self
    }
}

/// Create a substrate of input configuration and return as a `Component`.
///
/// The returned component's size will be adjusted to a multiple of the
/// substrate spacing along both directions. Thus the substrate can be
/// periodically replicated along x and y.
///
/// # Errors
/// Returns an Error if the either of the input size are non-positive.
pub fn create_substrate(conf: &SheetConf) -> Result<Sheet> {
    let (dx, dy) = conf.size;
    if dx <= 0.0 || dy <= 0.0 {
        return Err(
            GrafenError::RunError("cannot create a substrate of negative size".to_string())
        );
    }

    let mut points = match conf.lattice {
        LatticeType::Hexagonal { a } => {
            Lattice::hexagonal(a).with_size(dx, dy).finalize()
        },
        LatticeType::Triclinic { a, b, gamma } => {
            Lattice::triclinic(a, b, gamma.to_radians()).with_size(dx, dy).finalize()
        },
        LatticeType::PoissonDisc { density } => {
            // The factor 1/sqrt(pi) comes from the area and the factor sqrt(2)
            // is a magic number which roughly gives the correct density. It works!
            use std::f64::consts::PI;
            let rmin = (2.0 / (PI * density)).sqrt();
            PoissonDistribution::new(rmin, dx, dy)
        },
    };

    if let Some(std) = conf.std_z {
        points = points.uniform_distribution(std);
    };

    Ok(Sheet {
        origin: Coord::new(0.0, 0.0, 0.0),
        size: points.box_size,
        residue_base: conf.residue.clone(),
        residue_coords: points.coords,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_conf() -> SheetConf {
        SheetConf {
            lattice: LatticeType::Hexagonal { a: 1.0 },
            residue: ResidueBase {
                code: "GRPH".to_string(),
                atoms: vec![Atom { code: "C".to_string(), position: Coord::new(0.0, 0.0, 0.0) }],
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
        {
            let regular = create_substrate(&conf).unwrap();
            assert!(regular.residue_coords.iter().any(|r| r.z != 0.0) == false);
        }

        let std_z = 1.0;
        conf.std_z = Some(std_z);
        let uniform = create_substrate(&conf).unwrap();

        // Non-zero variance: This *can* fail, but it should not be common!
        // How else to assert that a distribution has been applied, though?
        assert!(uniform.residue_coords.iter().map(|r| r.z).any(|z| z != 0.0));

        // No positions should exceed the input distribution max
        assert!(uniform.residue_coords.iter().all(|r| r.z.abs() <= std_z));
    }

    #[test]
    fn sheet_into_component() {
        let origin = Coord::new(0.0, 0.0, 0.0);
        let size = Coord::new(1.0, 2.0, 3.0);
        let residue_base = ResidueBase {
            code: "RES".to_string(),
            atoms: vec![
                Atom { code: "A1".to_string(), position: Coord::new(0.0, 0.0, 0.0) },
            ],
        };
        let residue_coords = vec![Coord::new(0.0, 0.0, 0.0), Coord::new(1.0, 0.0, 0.0)];

        let sheet = Sheet { origin, size, residue_base, residue_coords: residue_coords.clone() };
        let component = sheet.into_component();

        assert_eq!(origin, component.origin);
        assert_eq!(size, component.box_size);
        assert_eq!(residue_coords, component.residue_coords);
    }

    #[test]
    fn translate_a_sheet() {
        let residue_base = ResidueBase {
            code: "GRPH".to_string(),
            atoms: vec![Atom {
                code: "C".to_string(),
                position: Coord::new(0.0, 0.0, 0.0) }],
        };
        let origin = Coord::new(-1.0, 0.0, 1.0);
        let size = Coord::new(1.0, 2.0, 1.0);
        let residue_coords = vec![Coord::new(0.0, 0.0, 0.0)];

        let translate = Coord::new(1.0, 2.0, 3.0);

        let sheet = Sheet { origin, size, residue_base, residue_coords }
            .translate(&Coord::new(1.0, 2.0, 3.0));

        assert_eq!(size, sheet.size);
        assert_eq!(origin + translate, sheet.origin);
        assert_eq!(Coord::new(0.0, 0.0, 0.0), sheet.residue_coords[0]);
    }

    #[test]
    fn cut_a_circle_from_a_sheet() {
        let conf = setup_conf();
        let sheet = create_substrate(&conf).unwrap();

        let radius = 2.5;
        let origin = Coord::new(0.0, 0.0, 0.0);

        let circle = sheet.into_circle(radius);
        assert_eq!(origin, circle.origin);
        assert!(circle.num_atoms() > 0);

        for coord in circle.residue_coords {
            assert!(coord.distance(origin) <= radius);
        }
    }

    #[test]
    fn circles_center_coords_around_origo() {
        let origin = Coord::new(0.0, 0.0, 0.0);
        let size = Coord::new(4.0, 4.0, 1.0);
        let residue_base = ResidueBase {
            code: "RES".to_string(),
            atoms: vec![
                Atom { code: "A1".to_string(), position: Coord::new(0.0, 0.0, 0.0) },
            ],
        };

        // The coordinates are offset from (0, 0, 0) but will be centered around it
        let radius = 2.0;
        let residue_coords = vec![
            Coord::new(0.1, radius, 0.0), // After: (-1.9, radius, 0.0)
            Coord::new(3.9, radius, 0.0)  // After: (1.9, radius, 0.0)
        ];

        let sheet = Sheet { origin, size, residue_base, residue_coords };
        let circle = sheet.into_circle(radius);

        assert_eq!(2, circle.num_atoms());
        assert_eq!(Coord::new(-1.9, 0.0, 0.0), circle.residue_coords[0]);
        assert_eq!(Coord::new(1.9, 0.0, 0.0), circle.residue_coords[1]);
    }

    #[test]
    fn circles_check_coordinates_after_pbc_adjustment() {
        let origin = Coord::new(0.0, 0.0, 0.0);
        let size = Coord::new(1.0, 1.0, 1.0);
        let residue_base = ResidueBase {
            code: "RES".to_string(),
            atoms: vec![
                Atom { code: "A1".to_string(), position: Coord::new(0.0, 0.0, 0.0) },
            ],
        };

        // The coordinates are outside the box, but will be adjusted to be in the center.
        let radius = 0.5;
        let residue_coords = vec![
            Coord::new(-0.6, radius, 0.0), // In box: (0.4, radius, 0.0)
            Coord::new(2.6, radius, 0.0)   // In box: (0.6, radius, 0.0)
        ];

        let sheet = Sheet { origin, size, residue_base, residue_coords };
        let circle = sheet.into_circle(radius);

        assert_eq!(2, circle.num_atoms());
        assert_eq!(Coord::new(-0.1, 0.0, 0.0), circle.residue_coords[0]);
        assert_eq!(Coord::new(0.1, 0.0, 0.0), circle.residue_coords[1]);
    }
}
