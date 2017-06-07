//! Create a cylinder from a sheet.

use system::*;

struct Cylinder;

impl Cylinder {
    pub fn from_sheet<'a>(sheet: &'a Component) -> Component<'a> {
        let length = sheet.dimensions.x;
        let radius = length / (2.0 * ::std::f64::consts::PI);
        let origin = sheet.origin;
        let (x0, _, z0) = origin.to_tuple();

        let residues = sheet.residues
            .iter()
            .map(|res| {
                let (x, y, _) = res.position.to_tuple();
                let angle = ((x - x0) * 360.0 / length).to_radians();

                let x = radius * angle.sin();
                let z = -radius * angle.cos();

                Residue {
                    base: res.base,
                    position: Coord::new(x + x0, y, z + z0),
                }
            }).collect();

        Component {
            origin: origin,
            dimensions: sheet.dimensions,
            residues: residues,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::f64::consts::PI;
    use super::*;

    fn setup_residue() -> ResidueBase {
        ResidueBase {
            code: "RES".to_string(),
            atoms: vec![
                Atom { code: "A".to_string(), position: Coord::new(0.0, 0.0, 0.0) }
            ],
        }
    }

    fn setup_sheet<'a>(residue: &'a ResidueBase) -> Component<'a> {
        // A sheet with four positions along x will result in a cylinder in the x-z plane
        // with points at 0, 90, 180 and 270 degrees and its center at the first position.
        // Positions along y will be unchanged.
        Component {
            origin: Coord::new(0.0, 0.0, 0.0),
            dimensions: Coord::new(4.0, 0.0, 0.0),
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
        assert_eq!(Coord::new(0.0, 0.0, -radius) + shift, cylinder.residues[0].position);
        assert_eq!(Coord::new(radius, 0.5, 0.0) + shift, cylinder.residues[1].position);
        assert_eq!(Coord::new(0.0, 1.0, radius) + shift, cylinder.residues[2].position);
        assert_eq!(Coord::new(-radius, 1.5, 0.0) + shift, cylinder.residues[3].position);
    }

    #[test]
    fn cylinder_from_empty_sheet_works() {
        let sheet = Component {
            origin: Coord::new(0.0, 0.0, 0.0),
            dimensions: Coord::new(1.0, 2.0, 3.0),
            residues: vec![],
        };

        let cylinder = Cylinder::from_sheet(&sheet);
        assert_eq!(0, cylinder.num_atoms());
    }
}
