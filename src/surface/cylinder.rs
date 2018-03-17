//! Construct cylinders that are curved sheets, not volumes.

use surface::{Sheet, LatticeType};

use coord::{Coord, Direction, Translate,
    rotate_coords, rotate_planar_coords_to_alignment};
use describe::{unwrap_name, Describe};
use error::Result;
use iterator::{ResidueIter, ResidueIterOut};
use system::*;

use std::f64::consts::PI;
use std::fmt;
use std::fmt::{Display, Formatter};


impl_component![Cylinder];
impl_translate![Cylinder];

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
            normal: Direction::Z,
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

#[cfg(test)]
mod tests {
    use super::*;
    use surface::LatticeType::*;

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
