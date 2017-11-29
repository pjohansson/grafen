//! Construct planar sheets.

use surface::distribution::PoissonDistribution;
use surface::lattice::Lattice;
use surface::LatticeType;
use surface::LatticeType::*;

use coord::{Coord, Direction, Periodic, Translate,
    rotate_planar_coords_to_alignment};
use describe::{unwrap_name, Describe};
use error::{GrafenError, Result};
use iterator::{AtomIterator, AtomIterItem};
use system::*;
use volume::pbc_multiply_volume;

impl_component![Sheet];
impl_translate![Circle, Sheet];

#[derive(Clone, Debug, Deserialize, Serialize)]
/// A rectangular sheet.
pub struct Sheet {
    /// Name of component.
    pub name: Option<String>,
    /// Optional residue placed at each coordinate. If not set the sheet describes
    ///  a general collection of coordinates.
    pub residue: Option<Residue>,
    /// Lattice type used to construct the surface structure.
    pub lattice: LatticeType,
    /// Standard deviation along z of coordinates. Added to the coordinates when `construct`
    /// is called.
    pub std_z: Option<f64>,
    #[serde(skip)]
    /// Origin of the sheet. Located in the lower-left position of it.
    pub origin: Coord,
    /// Normal vector of the sheet.
    pub normal: Direction,
    #[serde(skip)]
    /// Length of the sheet along the x axis.
    pub length: f64,
    #[serde(skip)]
    /// Length of the sheet along the y axis.
    pub width: f64,
    #[serde(skip)]
    /// List of coordinates belonging to the sheet. Relative to the `origin.
    pub coords: Vec<Coord>,
}

impl Sheet {
    /// Construct the sheet coordinates and return the object.
    ///
    /// # Errors
    /// Returns an error if either the length or width is non-positive.
    pub fn construct(self) -> Result<Sheet> {
        if self.length <= 0.0 || self.width <= 0.0 {
            return Err(
                GrafenError::RunError("cannot create a substrate of negative size".to_string())
            );
        }

        let mut coords_lattice = match self.lattice {
            Hexagonal { a } => {
                Lattice::hexagonal(a)
                    .with_size(self.length, self.width)
                    .finalize()
            },
            Triclinic { a, b, gamma } => {
                Lattice::triclinic(a, b, gamma.to_radians())
                    .with_size(self.length, self.width)
                    .finalize()
            },
            PoissonDisc { density } => {
                // The factor 1/sqrt(pi) comes from the area and the factor sqrt(2)
                // is a magic number which roughly gives the correct density. It works!
                use std::f64::consts::PI;
                let rmin = (2.0 / (PI * density)).sqrt();

                PoissonDistribution::new(rmin, self.length, self.width)
            },
        };

        if let Some(std) = self.std_z {
            coords_lattice = coords_lattice.uniform_distribution(std);
        };

        let (length, width, _) = coords_lattice.box_size.to_tuple();

        let coords = match self.normal {
            Direction::Z => coords_lattice.coords,
            direction => {
                rotate_planar_coords_to_alignment(&coords_lattice.coords, Direction::Z, direction)
            },
        };

        Ok(Sheet {
            length,
            width,
            coords,
            .. self
        })
    }

    /// Calculate the box size. The height of a pure sheet is set to 0.1 (nm)
    /// as a lower limit of the system.
    fn calc_box_size(&self) -> Coord {
        let margin = 0.1;

        match self.normal {
            Direction::X => Coord::new(margin, self.width, self.length),
            Direction::Y => Coord::new(self.length, margin, self.width),
            Direction::Z => Coord::new(self.length, self.width, margin),
        }
    }

    /// Cut a circle out of coordinates in the sheet.
    pub fn to_circle(&self, radius: f64) -> Circle {
        let center = Coord::new(radius, radius, 0.0);
        let box_size = Coord::new(self.length, self.width, 0.0);

        // Assert that we have a large enough sheet to cut a circle from
        let pbc_multiples = (
            (2.0 * radius / self.length).ceil() as usize,
            (2.0 * radius / self.width).ceil() as usize
        );

        let coords = match pbc_multiples {
            (1, 1) => {
                cut_circle(&self.coords, center, box_size, radius)
            },
            (nx, ny) => {
                let box_size = box_size.pbc_multiply(nx, ny, 1);
                let coords = self.pbc_multiply(nx, ny, 1).coords;

                cut_circle(&coords, center, box_size, radius)
            },
        };

        Circle {
            residue: self.residue.clone(),
            origin: self.origin,
            radius,
            coords,
        }
    }
}

impl Describe for Sheet {
    fn describe(&self) -> String {
        format!("{} (Rectangular sheet of size ({:.2}, {:.2}) at {})",
            unwrap_name(&self.name), self.length, self.width, self.origin)
    }

    fn describe_short(&self) -> String {
        format!("{} (Sheet)", unwrap_name(&self.name))
    }
}

impl Periodic for Sheet {
    /// Clone sheet coordinates into PBC multiples.
    fn pbc_multiply(&self, nx: usize, ny: usize, _: usize) -> Sheet {
        let current_size = Coord::new(self.length, self.width, 0.0);
        let coords = pbc_multiply_volume(&self.coords, current_size, nx, ny, 1);

        Sheet {
            length: nx as f64 * self.length,
            width: ny as f64 * self.width,
            coords,
            .. self.clone()
        }

    }
}

#[derive(Clone, Debug)]
/// A 2D circular sheet. For now used only to cut caps for the `Cylinder` construction.
pub struct Circle {
    residue: Option<Residue>,
    origin: Coord,
    radius: f64,
    pub coords: Vec<Coord>,
}

/// Cut a set of coordinates into a circle with input radius in the x-y plane.
fn cut_circle(coords: &[Coord], center: Coord, box_size: Coord, radius: f64) -> Vec<Coord> {
    coords.iter()
        .map(|&coord| coord.with_pbc(box_size) - center)
        .filter(|coord| {
            let (dr, _) = coord.distance_cylindrical(Coord::ORIGO, Direction::Z);
            dr <= radius
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    fn setup_sheet(length: f64, width: f64, lattice: &LatticeType) -> Sheet {
        Sheet {
            name: None,
            residue: None,
            lattice: lattice.clone(),
            std_z: None,
            origin: Coord::default(),
            normal: Direction::Z,
            length,
            width,
            coords: vec![],
        }
    }

    #[test]
    fn create_sheet_with_density() {
        let length = 10.0;
        let width = 5.0;
        let density = 10.0;

        let sheet = setup_sheet(length, width, &PoissonDisc { density }).construct().unwrap();

        // There should be almost no risk of the density being off by more than 10%
        // since 500 points were created.
        let output_density = sheet.coords.len() as f64 / (length * width);
        assert!((output_density - density).abs() / density < 0.1);
    }

    #[test]
    fn create_regular_sheets_updates_sheet_size_to_match_pbc() {
        let a = 1.1;
        let b = 1.4;
        let lattice = Triclinic { a, b, gamma: 90.0 };

        // The length and width do not perfectly match the lattice constants
        let length = 2.0;
        let width = 1.0;

        let sheet = setup_sheet(length, width, &lattice).construct().unwrap();

        // The final size has been rounded to the closest multiple of the spacing
        assert_eq!(2.0 * a, sheet.length);
        assert_eq!(1.0 * b, sheet.width);
    }

    #[test]
    fn create_sheets_with_negative_input_size_returns_error() {
        let lattice = PoissonDisc { density: 10.0 };

        assert!(setup_sheet(-1.0, 1.0, &lattice).construct().is_err());
        assert!(setup_sheet(1.0, -1.0, &lattice).construct().is_err());
        assert!(setup_sheet(1.0, 1.0, &lattice).construct().is_ok());
    }

    #[test]
    fn variance_is_added_if_requested() {
        let length = 10.0;
        let width = 5.0;
        let lattice = PoissonDisc { density: 10.0 };
        let sheet = Sheet {
            std_z: Some(1.0),
            .. setup_sheet(length, width, &lattice)
        }.construct().unwrap();

        let num_coords = sheet.coords.len();
        let var = sheet.coords
            .iter()
            .map(|&Coord { x: _, y: _, z }| z.abs())
            .sum::<f64>() / (num_coords as f64);

        assert!(var > 0.0);
    }

    #[test]
    fn calc_box_size_of_sheet_accounts_for_sheet_normal() {
        let length = 5.0;
        let width = 4.0;
        let lattice = Hexagonal { a: 0.1 };

        let margin = 0.1;

        let sheet = Sheet {
            normal: Direction::X,
            .. setup_sheet(length, width, &lattice)
        }.construct().unwrap();

        assert_eq!(Coord::new(margin, sheet.width, sheet.length), sheet.calc_box_size());

        let sheet = Sheet {
            normal: Direction::Y,
            .. setup_sheet(length, width, &lattice)
        }.construct().unwrap();

        assert_eq!(Coord::new(sheet.length, margin, sheet.width), sheet.calc_box_size());

        let sheet = Sheet {
            normal: Direction::Z,
            .. setup_sheet(length, width, &lattice)
        }.construct().unwrap();

        assert_eq!(Coord::new(sheet.length, sheet.width, 0.1), sheet.calc_box_size());
    }

    #[test]
    fn cut_a_sheet_into_a_circle() {
        let radius = 4.0;

        let density = 10.0;
        let lattice = PoissonDisc { density };

        let circle = setup_sheet(10.0, 10.0, &lattice)
            .construct()
            .unwrap()
            .to_circle(radius);

        // Our density should match within 10%.
        let expected = PI * radius.powi(2) * density;
        assert!((circle.coords.len() as f64 - expected).abs() / expected < 0.1);

        // The coordinates are centered around origo.
        for coord in circle.coords {
            let (x, y, _) = coord.to_tuple();

            assert!(x.powi(2) + y.powi(2) <= radius.powi(2));
        }
    }

    #[test]
    fn small_sheets_extend_periodically_if_the_circle_is_too_large() {
        let radius = 4.0;

        let density = 10.0;
        let lattice = PoissonDisc { density };

        // This sheet will not contain the entire circle
        let circle = setup_sheet(6.0, 6.0, &lattice)
            .construct()
            .unwrap()
            .to_circle(radius);

        // Our density should match within 10%.
        let expected = PI * radius.powi(2) * density;
        assert!((circle.coords.len() as f64 - expected).abs() / expected < 0.1);
    }

    #[test]
    fn sheet_can_be_constructed_along_x_and_y() {
        let length = 10.0;
        let density = 10.0;
        let min_expected_coords = (length * length * density) as usize / 2;

        let sheet_x = Sheet {
            normal: Direction::X,
            .. setup_sheet(length, length, &PoissonDisc { density })
        }.construct().unwrap();

        assert!(sheet_x.coords.len() > min_expected_coords);
        for coord in sheet_x.coords {
            assert_eq!(coord.x, 0.0);
        }

        let sheet_y = Sheet {
            normal: Direction::Y,
            .. setup_sheet(length, length, &PoissonDisc { density })
        }.construct().unwrap();

        assert!(sheet_y.coords.len() > min_expected_coords);
        for coord in sheet_y.coords {
            assert_eq!(coord.y, 0.0);
        }

        let sheet_z = Sheet {
            normal: Direction::Z,
            .. setup_sheet(length, length, &PoissonDisc { density })
        }.construct().unwrap();

        assert!(sheet_z.coords.len() > min_expected_coords);
        for coord in sheet_z.coords {
            assert_eq!(coord.z, 0.0);
        }
    }
}
