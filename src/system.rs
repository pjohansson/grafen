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

/// Trait denoting the ability to `Translate` an object with a `Coord`.
pub trait Translate {
    fn translate(self, &Coord) -> Self;
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

/// A base for generating atoms belonging to a residue.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct ResidueBase {
    pub code: String,
    pub atoms: Vec<Atom>,
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
/// # use grafen::system::{Atom, Coord, ResidueBase};
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

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
/// A three-dimensional coordinate.
///
/// # Examples
/// ```
/// # use grafen::system::Coord;
/// let coord1 = Coord::new(1.0, 0.0, 1.0);
/// let coord2 = Coord::new(0.5, 0.5, 0.5);
///
/// assert_eq!(Coord::new(1.5, 0.5, 1.5), coord1 + coord2);
/// assert_eq!(Coord::new(0.5, -0.5, 0.5), coord1 - coord2);
/// ```
pub struct Coord {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

use std::ops::{Add, Sub};

impl Coord {
    /// Construct a new coordinate.
    pub fn new(x: f64, y: f64, z: f64) -> Coord {
        Coord { x: x, y: y, z: z }
    }

    /// Unpack the coordinate into a tuple.
    pub fn to_tuple(&self) -> (f64, f64, f64) {
        (self.x, self.y, self.z)
    }

    /// Calculate the absolute distance between two coordinates.
    pub fn distance(self, other: Coord) -> f64 {
        let dx = self - other;

        (dx.x * dx.x + dx.y * dx.y + dx.z * dx.z).sqrt()
    }

    /// Return the coordinate with its position adjusted to lie within the input box.
    ///
    /// If an input box size side is 0.0 (or smaller) the coordinate is not changed.
    pub fn with_pbc(self, box_size: Coord) -> Coord {
        let do_pbc = |mut c: f64, size: f64| {
            if size <= 0.0 {
                c
            } else {
                while c < 0.0 {
                    c += size;
                }

                c % size
            }
        };

        let (x, y, z) = self.to_tuple();
        Coord::new(do_pbc(x, box_size.x), do_pbc(y, box_size.y), do_pbc(z, box_size.z))
    }
}

impl Add for Coord {
    type Output = Coord;

    fn add(self, other: Coord) -> Coord {
        Coord::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }

}

impl Sub for Coord {
    type Output = Coord;

    fn sub(self, other: Coord) -> Coord {
        Coord::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }

}

impl PartialEq for Coord {
    fn eq(&self, other: &Coord) -> bool {
        let atol = 1e-9;
        (self.x - other.x).abs() < atol
            && (self.y - other.y).abs() < atol
            && (self.z - other.z).abs() < atol
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coord_addition_and_subtraction() {
        let coord = Coord::new(0.0, 1.0, 2.0);
        assert_eq!(Coord::new(0.0, 2.0, 4.0), coord + coord);
        assert_eq!(Coord::new(0.0, 0.0, 0.0), coord - coord);

    }

    #[test]
    fn coord_eq_tolerance_small_deviation_passes() {
        // Allow for some deviation when testing for equality, since floating point
        // numbers are stupid.
        let coord = Coord::new(0.0, 0.0, 0.0);
        assert_eq!(coord, Coord::new(1e-10, 2e-10, 3e-10));
    }

    #[test]
    #[should_panic]
    fn coord_eq_tolerance_larger_deviation_does_not() {
        let coord = Coord::new(0.0, 0.0, 0.0);
        assert_eq!(coord, Coord::new(1e-9, 2e-9, 3e-9));
    }

    #[test]
    fn coord_distance_calc() {
        let coord1 = Coord::new(1.0, 1.0, 1.0);
        let coord2 = Coord::new(3.0, 3.0, 2.0);

        assert_eq!(3.0, Coord::distance(coord1, coord2));
        assert_eq!(3.0, coord1.distance(coord2));
    }

    #[test]
    fn coord_to_tuple() {
        let coord = Coord::new(1.0, 2.0, 3.0);
        assert_eq!((1.0, 2.0, 3.0), coord.to_tuple());
    }

    #[test]
    fn coord_with_set_pbc() {
        let box_size = Coord::new(2.0, 4.0, 6.0);
        assert_eq!(Coord::new(1.0, 1.0, 1.0), Coord::new(1.0, 1.0, 1.0).with_pbc(box_size));
        assert_eq!(Coord::new(1.0, 1.0, 1.0), Coord::new(3.0, 1.0, 1.0).with_pbc(box_size));
        assert_eq!(Coord::new(1.0, 0.5, 1.0), Coord::new(1.0, 4.5, 1.0).with_pbc(box_size));
        assert_eq!(Coord::new(1.0, 1.0, 1.0), Coord::new(1.0, 1.0, 13.0).with_pbc(box_size));
        assert_eq!(Coord::new(1.0, 1.0, 1.0), Coord::new(-1.0, 1.0, 1.0).with_pbc(box_size));
        assert_eq!(Coord::new(1.0, 3.0, 1.0), Coord::new(1.0, -1.0, 1.0).with_pbc(box_size));
    }

    #[test]
    fn coords_adjusted_by_pbc_with_size_0_does_not_change() {
        let box_size = Coord::new(0.0, 0.0, 0.0);
        let coord = Coord::new(1.0, 2.0, 3.0);
        assert_eq!(coord, coord.with_pbc(box_size));
    }

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
