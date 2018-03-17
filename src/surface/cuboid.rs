//! Construct planar sheets.

use surface::LatticeType;
use surface::Sheet;

use coord::{Coord, Direction, Translate};
use describe::{unwrap_name, Describe};
use error::Result;
use iterator::{ResidueIter, ResidueIterOut};
use system::*;

use std::fmt;
use std::fmt::{Display, Formatter};

impl_component![Cuboid];
impl_translate![Cuboid];

bitflags! {
    #[derive(Deserialize, Serialize)]
    /// Sides of a cuboid box.
    pub struct Sides: u8 {
        const X0 = 0b00000001;
        const X1 = 0b00000010;
        const Y0 = 0b00000100;
        const Y1 = 0b00001000;
        const Z0 = 0b00010000;
        const Z1 = 0b00100000;
    }
}

impl Display for Sides {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        if self.contains(Sides::X0) {
            write!(f, "X0")?;
        }
        if self.contains(Sides::X1) {
            write!(f, "X1")?;
        }
        if self.contains(Sides::Y0) {
            write!(f, "Y0")?;
        }
        if self.contains(Sides::Y1) {
            write!(f, "Y1")?;
        }
        if self.contains(Sides::Z0) {
            write!(f, "Z0")?;
        }
        if self.contains(Sides::Z1) {
            write!(f, "Z1")?;
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
/// A cuboid surface. All sides will be created to exactly match multiples of the lattice spacing.
pub struct Cuboid {
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
    /// Size of cuboid box.
    pub size: Coord,
    /// Sides that are added for the box. Is a `bitflag` struct.
    pub sides: Sides,
    #[serde(skip)]
    /// List of coordinates belonging to the sheet. Relative to the `origin`.
    pub coords: Vec<Coord>,
}

fn translate_coordinate_list(coords: &[Coord], translate: Coord) -> Vec<Coord> {
    coords.iter().map(|&c| c + translate).collect()
}

impl Cuboid {
    /// Construct the cuboid coordinates and return the object.
    ///
    /// # Errors
    /// Returns an error if either the length or width is non-positive.
    pub fn construct(self) -> Result<Cuboid> {
        let sheet_base = Sheet {
            name: None,
            residue: None,
            lattice: self.lattice.clone(),
            std_z: self.std_z,
            origin: Coord::ORIGO,
            normal: Direction::X,
            length: 0.0,
            width: 0.0,
            coords: Vec::new(),
        };

        let mut coords: Vec<Coord> = Vec::new();
        let (dx_target, dy_target, dz_target) = self.size.to_tuple();

        let sheet_yz = Sheet {
            normal: Direction::X,
            length: dz_target,
            width: dy_target,
            .. sheet_base.clone()
        }.construct()?.with_pbc();

        let sheet_xz = Sheet {
            normal: Direction::Y,
            length: dx_target,
            width: dz_target,
            .. sheet_base.clone()
        }.construct()?.with_pbc();

        let sheet_xy = Sheet {
            normal: Direction::Z,
            length: dx_target,
            width: dy_target,
            .. sheet_base.clone()
        }.construct()?.with_pbc();

        let (dx, dy, dz) = (sheet_xy.length, sheet_xy.width, sheet_yz.length);

        if self.sides.contains(Sides::X0) {
            coords.extend_from_slice(&sheet_yz.coords);
        }

        if self.sides.contains(Sides::X1) {
            let dr = Coord::new(dx, 0.0, 0.0);
            coords.extend_from_slice(&translate_coordinate_list(&sheet_yz.coords, dr));
        }

        if self.sides.contains(Sides::Y0) {
            coords.extend_from_slice(&sheet_xz.coords);
        }

        if self.sides.contains(Sides::Y1) {
            let dr = Coord::new(0.0, dy, 0.0);
            coords.extend_from_slice(&translate_coordinate_list(&sheet_xz.coords, dr));
        }

        if self.sides.contains(Sides::Z0) {
            coords.extend_from_slice(&sheet_xy.coords);
        }

        if self.sides.contains(Sides::Z1) {
            let dr = Coord::new(0.0, 0.0, dz);
            coords.extend_from_slice(&translate_coordinate_list(&sheet_xy.coords, dr));
        }

        Ok(Cuboid {
            coords,
            size: Coord::new(dx, dy, dz),
            .. self
        })
    }

    /// Calculate the box size.
    fn calc_box_size(&self) -> Coord {
        self.size
    }
}

impl Describe for Cuboid {
    fn describe(&self) -> String {
        format!("{} (Surface box of size {})",
            unwrap_name(&self.name), self.size)
    }

    fn describe_short(&self) -> String {
        format!("{} (Surface box)", unwrap_name(&self.name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_sheets_and_cuboid_base(dx: f64, dy: f64, dz: f64, lattice: LatticeType)
        -> (Sheet, Sheet, Sheet, Cuboid) {
        let size = Coord::new(dx, dy, dz);

        // Create sheets of the box in each direction to compare against.
        let sheet_base = Sheet {
            name: None,
            residue: None,
            std_z: None,
            origin: Coord::ORIGO,
            lattice: lattice.clone(),
            normal: Direction::X,
            length: 0.0,
            width: 0.0,
            coords: Vec::new(),
        };

        let sheet_xy = Sheet {
            normal: Direction::Z,
            length: dx,
            width: dy,
            .. sheet_base.clone()
        }.construct().unwrap().with_pbc();
        assert!(!sheet_xy.coords.is_empty());

        let sheet_xz = Sheet {
            normal: Direction::Y,
            length: dx,
            width: dz,
            .. sheet_base.clone()
        }.construct().unwrap().with_pbc();
        assert!(!sheet_xz.coords.is_empty());

        let sheet_yz = Sheet {
            normal: Direction::X,
            length: dz,
            width: dy,
            .. sheet_base.clone()
        }.construct().unwrap().with_pbc();
        assert!(!sheet_yz.coords.is_empty());

        // Now create the cuboids with the six different faces and ensure that the coordinates
        // match the sheets.
        let cuboid_base = Cuboid {
            name: None,
            residue: None,
            lattice: lattice.clone(),
            std_z: None,
            origin: Coord::ORIGO,
            size: size,
            sides: Sides::empty(),
            coords: Vec::new(),
        };

        (sheet_xy, sheet_xz, sheet_yz, cuboid_base)
    }

    #[test]
    fn box_is_created_with_set_sides_that_are_translated() {
        let (dx, dy, dz) = (3.0, 5.0, 7.0);
        let lattice = LatticeType::Hexagonal { a: 0.57 };

        let (sheet_xy, sheet_xz, sheet_yz, cuboid_base) = setup_sheets_and_cuboid_base(
            dx, dy, dz, lattice);

        let cuboid_x0 = Cuboid {
            sides: Sides::X0,
            .. cuboid_base.clone()
        }.construct().unwrap();

        assert_eq!(cuboid_x0.coords, sheet_yz.coords);

        let cuboid_x1 = Cuboid {
            sides: Sides::X1,
            .. cuboid_base.clone()
        }.construct().unwrap();

        // Compare the further away side by translating the sheet coordinates accordingly.
        // Note that they are translated using the *output* side size, since this may not
        // match the input due to the lattice construction!
        let dr = Coord::new(sheet_xy.length, 0.0, 0.0);
        for (&c0, &c1) in cuboid_x1.coords.iter().zip(sheet_yz.coords.iter()) {
            assert_eq!(c0, c1 + dr);
        }

        let cuboid_y0 = Cuboid {
            sides: Sides::Y0,
            .. cuboid_base.clone()
        }.construct().unwrap();

        assert_eq!(cuboid_y0.coords, sheet_xz.coords);

        let cuboid_y1 = Cuboid {
            sides: Sides::Y1,
            .. cuboid_base.clone()
        }.construct().unwrap();

        let dr = Coord::new(0.0, sheet_xy.width, 0.0);
        for (&c0, &c1) in cuboid_y1.coords.iter().zip(sheet_xz.coords.iter()) {
            assert_eq!(c0, c1 + dr);
        }

        let cuboid_z0 = Cuboid {
            sides: Sides::Z0,
            .. cuboid_base.clone()
        }.construct().unwrap();

        assert_eq!(cuboid_z0.coords, sheet_xy.coords);

        let cuboid_z1 = Cuboid {
            sides: Sides::Z1,
            .. cuboid_base.clone()
        }.construct().unwrap();

        let dr = Coord::new(0.0, 0.0, sheet_yz.length);
        for (&c0, &c1) in cuboid_z1.coords.iter().zip(sheet_xy.coords.iter()) {
            assert_eq!(c0, c1 + dr);
        }
    }

    #[test]
    fn box_with_several_set_sides_has_matching_number_of_coordinates() {
        let (dx, dy, dz) = (3.0, 5.0, 7.0);
        let lattice = LatticeType::Hexagonal { a: 0.57 };

        let (_, sheet_xz, sheet_yz, cuboid_base)
            = setup_sheets_and_cuboid_base(dx, dy, dz, lattice);

        let cuboid = Cuboid {
            sides: Sides::X0 | Sides::X1 | Sides::Y0,
            .. cuboid_base
        }.construct().unwrap();

        assert_eq!(cuboid.coords.len(), 2 * sheet_yz.coords.len() + sheet_xz.coords.len());
    }

    #[test]
    fn box_from_sheets_has_box_size_matching_the_lattice() {
        let (dx_target, dy_target, dz_target) = (3.0, 5.0, 7.0);
        let lattice = LatticeType::Hexagonal { a: 0.57 };

        let (sheet_xy, _, sheet_yz, cuboid_base) = setup_sheets_and_cuboid_base(
            dx_target, dy_target, dz_target, lattice);

        // The final box size will be that of the sheets
        let size = Coord::new(sheet_xy.length, sheet_xy.width, sheet_yz.length);

        let cuboid = cuboid_base.construct().unwrap();

        assert_eq!(cuboid.size, size);
        assert_eq!(cuboid.calc_box_size(), size);
    }

    #[test]
    fn creating_a_box_respects_the_set_stdz_value() {
        let (dx_target, dy_target, dz_target) = (3.0, 5.0, 7.0);
        let lattice = LatticeType::Hexagonal { a: 0.57 };

        let (sheet_xy, _, _, cuboid_base) = setup_sheets_and_cuboid_base(
            dx_target, dy_target, dz_target, lattice);

        // Create a cuboid with a side in the (lower) xy plane
        let cuboid_z0 = Cuboid {
            std_z: Some(2.0),
            sides: Sides::Z0,
            .. cuboid_base
        }.construct().unwrap();

        assert!(!cuboid_z0.coords
            .iter()
            .zip(sheet_xy.coords.iter())
            .all(|(&c0, &c1)| c0.z == c1.z)
        );
    }
}
