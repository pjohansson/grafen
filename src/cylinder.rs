//! A cylinder structure.

use substrate::Sheet;
use system::{Coord, Component, IntoComponent, ResidueBase, Translate};

#[derive(Clone, Debug)]
/// A cylinder of some residues.
pub struct Cylinder {
    /// `Cylinder` origin, positioned in the middle point of one of the cylinder edges.
    /// Residue positions are relative to this.
    pub origin: Coord,
    /// Cylinder radius.
    pub radius: f64,
    /// Cylinder height.
    pub height: f64,
    /// Residue base.
    pub residue_base: ResidueBase,
    /// Residue positions of the `Cylinder`.
    pub residues: Vec<Coord>,
}

impl Cylinder {
    /// Fold a `Sheet` into a `Cylinder`.
    ///
    /// The `Sheet` is folded along the x axis using its `size` as the full length.
    /// Its length along y becomes the cylinder height and its radius calculated from
    /// the length along x.
    ///
    /// # Bugs:
    /// It is assumed that the `Sheet` consists of a single layer only. All information
    /// of positions along z is discarded to create the cylinder. It would be preferable
    /// for this to be accounted for in some way, but it would require knowledge of the
    /// number of layers.
    pub fn from_sheet(sheet: &Sheet) -> Cylinder {
        let (length, height, _) = sheet.size.to_tuple();

        let radius = length / (2.0 * ::std::f64::consts::PI);
        let origin = sheet.origin;

        let residues = sheet.residue_coords
            .iter()
            .map(|res| {
                let (x, y, _) = res.to_tuple();
                let angle = (x * 360.0 / length).to_radians();

                let x = radius * angle.sin();
                let z = -radius * angle.cos();

                Coord::new(x, y, z)
            }).collect();

        Cylinder { origin, radius, height, residue_base: sheet.residue_base.clone(), residues }
    }
}

impl IntoComponent for Cylinder {
    fn to_component(&self) -> Component {
        let radius = self.radius;
        let height = self.height;

        Component {
            origin: self.origin,
            box_size: Coord::new(2.0 * radius, height, 2.0 * radius),
            residue_base: self.residue_base.clone(),
            residue_coords: self.residues.clone(),
        }
    }

    fn into_component(self) -> Component {
        let radius = self.radius;
        let height = self.height;

        Component {
            origin: self.origin,
            box_size: Coord::new(2.0 * radius, height, 2.0 * radius),
            residue_base: self.residue_base,
            residue_coords: self.residues,
        }
    }

    fn num_atoms(&self) -> usize {
        self.residue_base.atoms.len() * self.residues.len()
    }
}

impl Translate for Cylinder {
    fn translate(mut self, trans: &Coord) -> Self {
        self.origin = self.origin + *trans;
        self
    }
}

#[cfg(test)]
mod tests {
    use std::f64::consts::PI;
    use substrate::Sheet;
    use system::*;
    use super::*;

    fn setup_residue() -> ResidueBase {
        ResidueBase {
            code: "RES".to_string(),
            atoms: vec![
                Atom { code: "A".to_string(), position: Coord::new(0.0, 0.0, 0.0) }
            ],
        }
    }

    fn setup_sheet(residue: &ResidueBase) -> Sheet {
        // A sheet with four positions along x will result in a cylinder in the x-z plane
        // with points at 0, 90, 180 and 270 degrees and its center at the first position.
        // Positions along y will be unchanged.
        Sheet {
            origin: Coord::new(0.0, 0.0, 0.0),
            size: Coord::new(4.0, 1.0, 0.0),
            residue_base: residue.clone(),
            residue_coords: vec![
                Coord::new(0.0, 0.0, 0.0),
                Coord::new(1.0, 0.5, 0.0),
                Coord::new(2.0, 1.0, 0.0),
                Coord::new(3.0, 1.5, 0.0),
            ]
        }
    }

    #[test]
    fn sheet_to_cylinder_positions_are_circle() {
        let residue = setup_residue();
        let sheet = setup_sheet(&residue);

        let cylinder = Cylinder::from_sheet(&sheet);
        assert_eq!(cylinder.num_atoms(), sheet.num_atoms());
        assert_eq!(cylinder.origin, sheet.origin);

        let radius = 4.0 / (2.0 * PI);
        assert_eq!(radius, cylinder.radius);
        assert_eq!(1.0, cylinder.height); // The size along y

        assert_eq!(Coord::new(0.0, 0.0, -radius), cylinder.residues[0]);
        assert_eq!(Coord::new(radius, 0.5, 0.0), cylinder.residues[1]);
        assert_eq!(Coord::new(0.0, 1.0, radius), cylinder.residues[2]);
        assert_eq!(Coord::new(-radius, 1.5, 0.0), cylinder.residues[3]);
    }

    #[test]
    fn offset_sheet_origin_gives_offset_cylinder() {
        let residue = setup_residue();
        let shift = Coord::new(1.0, 2.0, 3.0);
        let sheet = setup_sheet(&residue).translate(&shift);

        let cylinder = Cylinder::from_sheet(&sheet);
        assert_eq!(cylinder.origin, sheet.origin);

        let radius = 4.0 / (2.0 * PI);
        assert_eq!(Coord::new(0.0, 0.0, -radius), cylinder.residues[0]);
        assert_eq!(Coord::new(radius, 0.5, 0.0), cylinder.residues[1]);
        assert_eq!(Coord::new(0.0, 1.0, radius), cylinder.residues[2]);
        assert_eq!(Coord::new(-radius, 1.5, 0.0), cylinder.residues[3]);
    }

    #[test]
    fn cylinder_from_empty_sheet_works() {
        let residue = setup_residue();
        let sheet = Sheet {
            origin: Coord::new(0.0, 0.0, 0.0),
            size: Coord::new(1.0, 2.0, 3.0),
            residue_base: residue,
            residue_coords: vec![],
        };

        let cylinder = Cylinder::from_sheet(&sheet);
        assert_eq!(0, cylinder.num_atoms());
    }

    #[test]
    fn cylinder_into_component_gives_correct_dimensions() {
        let residue = setup_residue();
        let sheet = setup_sheet(&residue);

        let cylinder = Cylinder::from_sheet(&sheet);
        let radius = cylinder.radius;
        let height = cylinder.height;

        // The cylinder is still directed along the y axis.
        let size = Coord::new(2.0 * radius, height, 2.0 * radius);

        let component = cylinder.into_component();
        assert_eq!(size, component.box_size);
    }
}
