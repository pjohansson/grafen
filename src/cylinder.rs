//! A cylinder structure.

use substrate::Sheet;
use system::{Coord, Component, IntoComponent, Residue, Translate};

/// A `Cylinder` of some `Residue`s.
struct Cylinder<'a> {
    /// `Cylinder` origin, positioned in the middle point of one of the cylinder edges.
    /// `Residue` positions are relative to this.
    origin: Coord,
    /// Cylinder radius.
    radius: f64,
    /// Cylinder height.
    height: f64,
    /// `Residue`s belonging to the `Cylinder`.
    residues: Vec<Residue<'a>>,
}

impl<'a> Cylinder<'a> {
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
    pub fn from_sheet(sheet: &'a Sheet) -> Cylinder<'a> {
        let (length, height, _) = sheet.size.to_tuple();

        let radius = length / (2.0 * ::std::f64::consts::PI);
        let origin = sheet.origin;

        let residues = sheet.residues
            .iter()
            .map(|res| {
                let (x, y, _) = res.position.to_tuple();
                let angle = (x * 360.0 / length).to_radians();

                let x = radius * angle.sin();
                let z = -radius * angle.cos();

                Residue {
                    base: res.base,
                    position: Coord::new(x, y, z),
                }
            }).collect();

        Cylinder { origin, radius, height, residues }
    }
}

impl<'a> IntoComponent<'a> for Cylinder<'a> {
    fn into_component(self) -> Component<'a> {
        Component {
            origin: self.origin,
            dimensions: Coord::new(0.0, 0.0, 0.0),
            residues: self.residues,
        }
    }

    fn num_atoms(&self) -> usize {
        self.residues.iter().map(|r| r.base.atoms.len()).sum()
    }
}

impl<'a> Translate for Cylinder<'a> {
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

    fn setup_sheet<'a>(residue: &'a ResidueBase) -> Sheet<'a> {
        // A sheet with four positions along x will result in a cylinder in the x-z plane
        // with points at 0, 90, 180 and 270 degrees and its center at the first position.
        // Positions along y will be unchanged.
        Sheet {
            origin: Coord::new(0.0, 0.0, 0.0),
            size: Coord::new(4.0, 1.0, 0.0),
            residues: vec![
                Residue { base: &residue, position: Coord::new(0.0, 0.0, 0.0) },
                Residue { base: &residue, position: Coord::new(1.0, 0.5, 0.0) },
                Residue { base: &residue, position: Coord::new(2.0, 1.0, 0.0) },
                Residue { base: &residue, position: Coord::new(3.0, 1.5, 0.0) },
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

        assert_eq!(Coord::new(0.0, 0.0, -radius), cylinder.residues[0].position);
        assert_eq!(Coord::new(radius, 0.5, 0.0), cylinder.residues[1].position);
        assert_eq!(Coord::new(0.0, 1.0, radius), cylinder.residues[2].position);
        assert_eq!(Coord::new(-radius, 1.5, 0.0), cylinder.residues[3].position);
    }

    #[test]
    fn offset_sheet_origin_gives_offset_cylinder() {
        let residue = setup_residue();
        let shift = Coord::new(1.0, 2.0, 3.0);
        let sheet = setup_sheet(&residue).translate(&shift);

        let cylinder = Cylinder::from_sheet(&sheet);
        assert_eq!(cylinder.origin, sheet.origin);

        let radius = 4.0 / (2.0 * PI);
        assert_eq!(Coord::new(0.0, 0.0, -radius), cylinder.residues[0].position);
        assert_eq!(Coord::new(radius, 0.5, 0.0), cylinder.residues[1].position);
        assert_eq!(Coord::new(0.0, 1.0, radius), cylinder.residues[2].position);
        assert_eq!(Coord::new(-radius, 1.5, 0.0), cylinder.residues[3].position);
    }

    #[test]
    fn cylinder_from_empty_sheet_works() {
        let sheet = Sheet {
            origin: Coord::new(0.0, 0.0, 0.0),
            size: Coord::new(1.0, 2.0, 3.0),
            residues: vec![],
        };

        let cylinder = Cylinder::from_sheet(&sheet);
        assert_eq!(0, cylinder.num_atoms());
    }
}
