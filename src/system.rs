//! Contains the basic structures of an atomic system.
//!
//! A final system consists of a set of `Component`s. Each `Component`
//! in turn consists of a collection of `Residue`s, which
//! can be moved around and translated with ease. Each `Residue`
//! consists of some `Atom`s. These atoms have positions
//! relative to their parent.
//!
//! This somewhat convoluted structure is inherited from molecular
//! simulation packages in which atoms are commonly grouped as such.
//! A proper physical way to look at is that atoms can be
//! similarly grouped into molecules.

use coord::Coord;
use describe::Describe;

#[derive(Clone, Debug)]
/// A system component which consists of a list of residues,
/// each of which contains some atoms.
pub struct Component {
    /// Component origin position. All residue positions are relative to this.
    pub origin: Coord,
    /// Component boundary box size.
    pub box_size: Coord,
    /// Residue base of component.
    pub residue_base: ResidueBase,
    /// List of residue positions.
    pub residue_coords: Vec<Coord>,
}

impl Component {
    /// Count and return the number of atoms in the component.
    pub fn num_atoms(&self) -> usize {
        self.residue_base.atoms.len() * self.residue_coords.len()
    }

    /// Count and return the number of residues in the component.
    pub fn num_residues(&self) -> usize {
        self.residue_coords.len()
    }

    /// Translate all residues within the component.
    pub fn translate(mut self, add: &Coord) -> Self {
        self.origin = self.origin + *add;
        self
    }

    /// Extend the component with coordinates from another, translating them by
    /// the relative difference of their origins.
    pub fn extend(&mut self, other: Component) {
        let difference = other.origin - self.origin;
        for coord in other.residue_coords {
            self.residue_coords.push(coord + difference);
        }
    }

    /// Rotate all coordinates along the x axis by 90 degrees, counter-clockwise.
    pub fn rotate_x(mut self) -> Self {
        for coord in self.residue_coords.iter_mut() {
            let y = coord.y;
            coord.y = -coord.z;
            coord.z = y;
        }

        let box_y = self.box_size.y;
        self.box_size.y = self.box_size.z;
        self.box_size.z = box_y;

        self
    }

    /// Rotate all coordinates along the y axis by 90 degrees, counter-clockwise.
    pub fn rotate_y(mut self) -> Self {
        for coord in self.residue_coords.iter_mut() {
            let x = coord.x;
            coord.x = -coord.z;
            coord.z = x;
        }

        let box_x = self.box_size.x;
        self.box_size.x = self.box_size.z;
        self.box_size.z = box_x;

        self
    }

    /// Rotate all coordinates along the z axis by 90 degrees, counter-clockwise.
    pub fn rotate_z(mut self) -> Self {
        for coord in self.residue_coords.iter_mut() {
            let x = coord.x;
            coord.x = -coord.y;
            coord.y = x;
        }

        let box_x = self.box_size.x;
        self.box_size.x = self.box_size.y;
        self.box_size.y = box_x;

        self
    }
}

/// Components (eg. `Sheet`, `Cylinder`) use this trait to define
/// common behaviour and conversion into a proper `Component` object.
pub trait IntoComponent {
    /// Copy residues to create a `Component` from the sub-component.
    fn to_component(&self) -> Component;

    /// Transform the sub-component into a `Component`.
    fn into_component(self) -> Component;

    /// Return the number of atoms of component.
    fn num_atoms(&self) -> usize;
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
/// Every atom in a residue has their own code and relative
/// position some base coordinate.
pub struct Atom {
    /// Atom code.
    pub code: String,
    /// Relative position.
    pub position: Coord,
}

impl Describe for Atom {
    fn describe(&self) -> String {
        format!("{}", self.code)
    }
}

/// A base for generating atoms belonging to a residue.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct ResidueBase {
    pub code: String,
    pub atoms: Vec<Atom>,
}

impl Describe for ResidueBase {
    fn describe(&self) -> String {
        format!("{} ({} atoms)", self.code, self.atoms.len())
    }
}

#[macro_export]
/// Construct a ResidueBase with a code and atoms.
///
/// At least one atom has to be present in the base. This is not a limitation
/// when explicitly constructing a residue, but it makes no sense to allow
/// it when invoking a constructor like this.
///
/// # Examples
/// ```
/// # #[macro_use] extern crate grafen;
/// # use grafen::coord::Coord;
/// # use grafen::system::{Atom, ResidueBase};
/// # fn main() {
/// let expect = ResidueBase {
///     code: "RES".to_string(),
///     atoms: vec![
///         Atom { code: "A".to_string(), position: Coord::new(0.0, 0.0, 0.0) },
///         Atom { code: "B".to_string(), position: Coord::new(1.0, 2.0, 3.0) }
///     ],
/// };
///
/// let residue = resbase![
///     "RES",
///     ("A", 0.0, 0.0, 0.0),
///     ("B", 1.0, 2.0, 3.0)
/// ];
///
/// assert_eq!(expect, residue);
/// # }
/// ```
macro_rules! resbase {
    (
        $rescode:expr,
        $(($atname:expr, $x:expr, $y:expr, $z:expr)),+
    ) => {
        {
            let mut temp_vec = Vec::new();
            $(
                temp_vec.push(
                    Atom {
                        code: $atname.to_string(),
                        position: Coord::new($x, $y, $z),
                    }
                );
            )*

            ResidueBase {
                code: $rescode.to_string(),
                atoms: temp_vec,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_component(base: &ResidueBase, num: usize) -> Component {
        Component {
            origin: Coord::new(0.0, 0.0, 0.0),
            box_size: Coord::new(0.0, 0.0, 0.0),
            residue_base: base.clone(),
            residue_coords: vec![Coord::new(0.0, 0.0, 0.0); num],
        }
    }

    #[test]
    fn count_atoms_in_component() {
        // A residue with three atoms duplicated twice
        let coord0 = Coord::new(0.0, 1.0, 2.0);
        let residue_base = ResidueBase {
            code: "R1".to_string(),
            atoms: vec![
                Atom { code: "A1".to_string(), position: coord0, },
                Atom { code: "A2".to_string(), position: coord0, },
                Atom { code: "A3".to_string(), position: coord0, },
            ]
        };
        let component = setup_component(&residue_base, 2);

        assert_eq!(3 * 2, component.num_atoms());
    }

    #[test]
    fn count_residues_in_component() {
        // A residue duplicated twice
        let coord0 = Coord::new(0.0, 1.0, 2.0);
        let residue_base = ResidueBase {
            code: "R1".to_string(),
            atoms: vec![
                Atom { code: "A1".to_string(), position: coord0, },
                Atom { code: "A2".to_string(), position: coord0, },
                Atom { code: "A3".to_string(), position: coord0, },
            ]
        };
        let component = setup_component(&residue_base, 3);

        assert_eq!(3, component.num_residues());
    }

    #[test]
    fn translate_a_component() {
        let coord0 = Coord::new(0.0, 1.0, 2.0);
        let residue_base = ResidueBase {
            code: "R1".to_string(),
            atoms: vec![
                Atom { code: "A1".to_string(), position: coord0, },
                Atom { code: "A2".to_string(), position: coord0, },
                Atom { code: "A3".to_string(), position: coord0, },
            ]
        };

        let component = setup_component(&residue_base, 1);
        let shift = Coord::new(0.0, 1.0, 2.0);

        let trans_component = component.clone().translate(&shift);
        for (orig, updated) in component.residue_coords.iter().zip(trans_component.residue_coords.iter()) {
            assert_eq!(orig, updated);
        }
        assert_eq!(component.origin + shift, trans_component.origin);
    }

    #[test]
    fn create_residue_base_macro() {
        let expect = ResidueBase {
            code: "RES".to_string(),
            atoms: vec![
                Atom { code: "A1".to_string(), position: Coord::new(0.0, 0.0, 0.0) },
                Atom { code: "A2".to_string(), position: Coord::new(0.0, 1.0, 2.0) }
            ],
        };
        let result = resbase![
            "RES",
            ("A1", 0.0, 0.0, 0.0),
            ("A2", 0.0, 1.0, 2.0)
        ];

        assert_eq!(expect, result);
    }

    #[test]
    fn rotate_component_around_yxz() {
        let residue = resbase![
            "RES",
            ("A", 0.0, 0.0, 0.0)
        ];

        let origin = Coord::new(1.0, 1.0, 1.0);

        let component = Component {
            origin,
            box_size: Coord::new(2.0, 5.0, 8.0),
            residue_base: residue,
            residue_coords: vec![
                Coord::new(0.0, 3.0, 6.0),
                Coord::new(1.0, 4.0, 7.0),
                Coord::new(2.0, 5.0, 8.0),
            ],
        };

        let rotated_y = component.rotate_y();

        {
            assert_eq!(origin, rotated_y.origin);
            assert_eq!(Coord::new(8.0, 5.0, 2.0), rotated_y.box_size);

            // Our 90 degree rotation is counter-clockwise.
            let mut iter = rotated_y.residue_coords.iter();
            assert_eq!(&Coord::new(-6.0, 3.0, 0.0), iter.next().unwrap());
            assert_eq!(&Coord::new(-7.0, 4.0, 1.0), iter.next().unwrap());
            assert_eq!(&Coord::new(-8.0, 5.0, 2.0), iter.next().unwrap());
            assert_eq!(None, iter.next());
        }

        let rotated_yx = rotated_y.rotate_x();
        {
            assert_eq!(origin, rotated_yx.origin);
            assert_eq!(Coord::new(8.0, 2.0, 5.0), rotated_yx.box_size);
            let mut iter = rotated_yx.residue_coords.iter();
            assert_eq!(&Coord::new(-6.0, 0.0, 3.0), iter.next().unwrap());
            assert_eq!(&Coord::new(-7.0, -1.0, 4.0), iter.next().unwrap());
            assert_eq!(&Coord::new(-8.0, -2.0, 5.0), iter.next().unwrap());
            assert_eq!(None, iter.next());
        }

        let rotated_yxz = rotated_yx.rotate_z();
        {
            assert_eq!(origin, rotated_yxz.origin);
            assert_eq!(Coord::new(2.0, 8.0, 5.0), rotated_yxz.box_size);
            let mut iter = rotated_yxz.residue_coords.iter();
            assert_eq!(&Coord::new(0.0, -6.0, 3.0), iter.next().unwrap());
            assert_eq!(&Coord::new(1.0, -7.0, 4.0), iter.next().unwrap());
            assert_eq!(&Coord::new(2.0, -8.0, 5.0), iter.next().unwrap());
            assert_eq!(None, iter.next());
        }
    }

    #[test]
    fn extend_component_with_more_coordinates_using_their_relative_position() {
        let origin = Coord::new(0.0, 0.0, 1.0);
        let mut component = Component {
            origin: origin,
            box_size: Coord::new(0.0, 0.0, 0.0),
            residue_base: resbase!["RES", ("A", 0.0, 0.0, 0.0)],
            residue_coords: vec![
                Coord::new(0.0, 0.0, 0.0),
                Coord::new(1.0, 0.0, 0.0)
            ],
        };

        // Extend the component by one that is translated by 5 along z.
        let translate = Coord::new(0.0, 0.0, 5.0);
        let extension = component.clone().translate(&translate);
        component.extend(extension);

        assert_eq!(Coord::new(0.0, 0.0, 0.0), component.residue_coords[0]);
        assert_eq!(Coord::new(1.0, 0.0, 0.0), component.residue_coords[1]);
        assert_eq!(Coord::new(0.0, 0.0, 5.0), component.residue_coords[2]);
        assert_eq!(Coord::new(1.0, 0.0, 5.0), component.residue_coords[3]);
    }
}
