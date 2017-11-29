//! Define and construct 2D surface objects.

mod distribution;
mod lattice;
mod points;

use coord::{Coord, Direction, Periodic, Translate,
    rotate_coords, rotate_planar_coords_to_alignment};
use error::{GrafenError, Result};
use self::distribution::PoissonDistribution;
use self::lattice::Lattice;
use system::*;

use std::f64::consts::PI;
use std::fmt;
use std::fmt::{Display, Formatter};

use describe::{unwrap_name, Describe};
use volume::pbc_multiply_volume;
use iterator::{AtomIterator, AtomIterItem};
use system::Component;

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
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
use self::LatticeType::*;

impl_component![Cylinder, Sheet];
impl_translate![Circle, Cylinder, Sheet];

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

        let mut coords = match self.lattice {
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
            coords = coords.uniform_distribution(std);
        };

        let (length, width, _) = coords.box_size.to_tuple();

        Ok(Sheet {
            length,
            width,
            coords: coords.coords,
            .. self
        })
    }

    /// Calculate the box size. The height of a pure sheet is set to 0.1 (nm)
    /// as a lower limit of the system.
    fn calc_box_size(&self) -> Coord {
        Coord::new(self.length, self.width, 0.1)
    }

    /// Cut a circle out of coordinates in the sheet.
    fn to_circle(&self, radius: f64) -> Circle {
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
struct Circle {
    residue: Option<Residue>,
    origin: Coord,
    radius: f64,
    coords: Vec<Coord>,
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
/// Cylinders can be capped in either or both ends.
pub enum CylinderCap {
    Top,
    Bottom,
    Both,
}

impl Display for CylinderCap {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            CylinderCap::Top => write!(f, "Top"),
            CylinderCap::Bottom => write!(f, "Bottom"),
            CylinderCap::Both => write!(f, "Both"),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
/// A 2D cylindrical surface.
pub struct Cylinder {
    /// Name of cylinder in database.
    pub name: Option<String>,
    /// Optional residue placed at each coordinate. If not set the cylinder describes
    /// a general collection of coordinates.
    pub residue: Option<Residue>,
    /// lattice type used to construct the cylinder surface structure.
    pub lattice: LatticeType,
    /// The axis along which the cylinder is aligned.
    pub alignment: Direction,
    /// Cylinders can be capped at its ends.
    pub cap: Option<CylinderCap>,
    #[serde(skip)]
    /// Origin of the cylinder. Located in the center of the bottom.
    pub origin: Coord,
    #[serde(skip)]
    /// Radius of cylinder.
    pub radius: f64,
    #[serde(skip)]
    /// Height of cylinder.
    pub height: f64,
    #[serde(skip)]
    /// List of coordinates belonging to the cylinder. Relative to the `origin.
    pub coords: Vec<Coord>,
}

impl Cylinder {
    /// Construct the cylinder coordinates and return the object.
    ///
    /// # Errors
    /// Returns an error if either the radius or height is non-positive.
    pub fn construct(self) -> Result<Cylinder> {
        // Bend a `Sheet` of the chosen lattice type into the cylinder.
        let length = 2.0 * PI * self.radius;
        let width = self.height;

        let sheet = Sheet {
            name: None,
            residue: None,
            lattice: self.lattice.clone(),
            std_z: None,
            origin: Coord::default(),
            length,
            width,
            coords: vec![],
        }.construct()?;

        let final_radius = sheet.length / (2.0 * PI);
        let final_height = sheet.width;

        // The cylinder will be created aligned to the Y axis
        let mut coords: Vec<_> = sheet.coords
            .iter()
            .map(|coord| {
                let (x0, y, _) = coord.to_tuple();

                let angle = (x0 * 360.0 / sheet.length).to_radians();

                let x = final_radius * angle.sin();
                let z = -final_radius * angle.cos();

                Coord::new(x, y, z)
            })
            .collect();

        if let Some(cap) = self.cap {
            // The cylinder is aligned along the y axis. Construct a cap from
            // the same sheet and rotate it to match.

            let mut bottom = sheet.to_circle(final_radius); //.rotate(Direction::X);
            bottom.coords = rotate_planar_coords_to_alignment(&bottom.coords,
                Direction::Z, Direction::Y);

            // Get the top cap coordinates by shifting the bottom ones, not just the origin.
            let top_coords: Vec<_> = bottom.coords
                .iter()
                .map(|&coord| coord + Coord::new(0.0, final_height, 0.0))
                .collect();

            match cap {
                CylinderCap::Bottom => coords.extend_from_slice(&bottom.coords),
                CylinderCap::Top => coords.extend_from_slice(&top_coords),
                CylinderCap::Both => {
                    coords.extend_from_slice(&bottom.coords);
                    coords.extend_from_slice(&top_coords);
                }
            }
        }

        // Rotate the cylinder once along the x-axis to align them to the z-axis.
        Ok(Cylinder {
            alignment: Direction::Z,
            radius: final_radius,
            height: final_height,
            coords: rotate_coords(&coords, Direction::X),
            .. self
        })
    }

    /// Calculate the box size.
    fn calc_box_size(&self) -> Coord {
        let diameter = 2.0 * self.radius;

        match self.alignment {
            Direction::X => Coord::new(self.height, diameter, diameter),
            Direction::Y => Coord::new(diameter, self.height, diameter),
            Direction::Z => Coord::new(diameter, diameter, self.height),
        }
    }
}

impl Describe for Cylinder {
    fn describe(&self) -> String {
        format!("{} (Cylinder surface of radius {:.2} and height {:.2} at {})",
            unwrap_name(&self.name), self.radius, self.height, self.origin)
    }

    fn describe_short(&self) -> String {
        format!("{} (Cylinder)", unwrap_name(&self.name))
    }
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

    fn setup_sheet(length: f64, width: f64, lattice: &LatticeType) -> Sheet {
        Sheet {
            name: None,
            residue: None,
            lattice: lattice.clone(),
            std_z: None,
            origin: Coord::default(),
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
    fn calc_box_size_of_sheet() {
        let length = 5.0;
        let width = 4.0;
        let lattice = Hexagonal { a: 0.1 };

        let sheet = setup_sheet(length, width, &lattice).construct().unwrap();

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

    fn setup_cylinder(radius: f64, height: f64, lattice: &LatticeType) -> Cylinder {
        Cylinder {
            name: None,
            residue: None,
            lattice: lattice.clone(),
            alignment: Direction::Z,
            cap: None,
            origin: Coord::default(),
            radius,
            height,
            coords: vec![],
        }
    }

    #[test]
    fn cylinder_is_bent_from_sheet_as_expected() {
        let radius = 2.0;
        let height = 5.0;
        let density = 10.0;
        let lattice = PoissonDisc { density };

        let cylinder = setup_cylinder(radius, height, &lattice).construct().unwrap();

        // We should have a rough surface density match
        let expected = 2.0 * PI * radius * height * density;
        assert!((expected - cylinder.coords.len() as f64).abs() / expected < 0.1);

        // Not all coords should be at z = 0, ie. not still a sheet
        let sum_z = cylinder.coords.iter().map(|&Coord { x: _, y: _, z }| z.abs()).sum::<f64>();
        assert!(sum_z > 0.0);

        // Currently the alignment should be along Z
        assert_eq!(Direction::Z, cylinder.alignment);

        // Rigorous test of coordinate structure
        for coord in cylinder.coords {
            let (r, h) = Coord::ORIGO.distance_cylindrical(coord, Direction::Z);
            assert!(r <= cylinder.radius);
            assert!(h >= 0.0 && h <= cylinder.height);
        }
    }

    #[test]
    fn cylinder_corrects_radius_and_height_to_match_lattice_spacing() {
        let radius = 1.0; // should give circumference = 2 * PI
        let height = 5.0;

        let a = 1.0; // not a match to the circumference
        let b = 1.1; // not a match to the height
        let lattice = Triclinic { a, b, gamma: 90.0 };

        let cylinder = setup_cylinder(radius, height, &lattice).construct().unwrap();

        assert_ne!(radius, cylinder.radius);
        assert_ne!(height, cylinder.height);

        // The best match to the circumference 2 * PI is the multiple 6 * a.
        assert_eq!(6.0 * a / (2.0 * PI), cylinder.radius);
        assert_eq!(5.0 * b, cylinder.height);
    }

    #[test]
    fn constructing_cylinder_with_negative_radius_or_height_returns_error() {
        let lattice = PoissonDisc { density: 10.0 };

        assert!(setup_cylinder(-1.0, 1.0, &lattice).construct().is_err());
        assert!(setup_cylinder(1.0, -1.0, &lattice).construct().is_err());
        assert!(setup_cylinder(1.0, 1.0, &lattice).construct().is_ok());
    }

    #[test]
    fn add_caps_to_cylinder() {
        let radius = 2.0;
        let height = 5.0;
        let lattice = Hexagonal { a: 0.1 };

        let mut conf = setup_cylinder(radius, height, &lattice);

        // Without caps
        let cylinder = conf.clone().construct().unwrap();
        let num_coords = cylinder.coords.len();

        // With a bottom cap
        conf.cap = Some(CylinderCap::Bottom);
        let cylinder_cap = conf.clone().construct().unwrap();

        // The first coordinates should be the original cylinder
        let (original, bottom) = cylinder_cap.coords.split_at(num_coords);
        assert_eq!(&original, &cylinder.coords.as_slice());
        assert!(bottom.len() > 0);

        // All the bottom coordinates should be at z = 0
        for coord in bottom {
            assert_eq!(coord.z, 0.0);
        }

        // A top cap
        conf.cap = Some(CylinderCap::Top);
        let cylinder_cap = conf.clone().construct().unwrap();

        let (original, top) = cylinder_cap.coords.split_at(num_coords);
        assert_eq!(&original, &cylinder.coords.as_slice());
        assert_eq!(top.len(), bottom.len());

        // All the top coordinates should be at the cylinder height
        for coord in top {
            assert_eq!(coord.z, cylinder.height);
        }

        // Both caps
        conf.cap = Some(CylinderCap::Both);
        let cylinder_cap = conf.clone().construct().unwrap();

        let (original, bottom_and_top) = cylinder_cap.coords.split_at(num_coords);
        assert_eq!(&original, &cylinder.coords.as_slice());

        let (bottom_from_both, top_from_both) = bottom_and_top.split_at(bottom.len());
        assert_eq!(bottom, bottom_from_both);
        assert_eq!(top, top_from_both);
    }

    #[test]
    fn calc_box_size_of_cylinder() {
        let radius = 2.0;
        let height = 5.0;
        let lattice = Hexagonal { a: 0.1 };

        // Check each direction
        let mut cylinder = Cylinder {
            alignment: Direction::X,
            .. setup_cylinder(radius, height, &lattice)
        };

        let diameter = 2.0 * radius;

        assert_eq!(Coord::new(height, diameter, diameter), cylinder.calc_box_size());

        cylinder.alignment = Direction::Y;
        assert_eq!(Coord::new(diameter, height, diameter), cylinder.calc_box_size());

        cylinder.alignment = Direction::Z;
        assert_eq!(Coord::new(diameter, diameter, height), cylinder.calc_box_size());
    }
}
