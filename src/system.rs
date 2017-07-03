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

/// A system component which consists of a list of residues,
/// each of which contains some atoms.
pub struct Component<'a> {
    /// Component origin position.
    pub origin: Coord,
    /// Component dimensions.
    pub dimensions: Coord,
    /// List of residues.
    pub residues: Vec<Residue<'a>>,
}

impl<'a> Component<'a> {
    /// Count and return the number of atoms in the component.
    pub fn num_atoms(&self) -> usize {
        self.residues.iter().map(|r| r.base.atoms.len()).sum()
    }

    /// Translate all residues within the component and return a copy.
    pub fn translate(&self, add: &Coord) -> Component<'a> {
        Component {
            origin: self.origin + *add,
            dimensions: self.dimensions,
            residues: self.residues.iter().map(|r| r.translate(*add)).collect(),
        }
    }
}

pub trait IntoComponent<'a> {
    fn into_component(self) -> Component<'a>;
}

/// Join a list of `Component`s into a single `Component`. The output `Component` dimensions
/// is the maximum for all individual `Component`s along all axes. `Residue`s are
/// added in order to the list.
pub fn join_components<'a>(components: Vec<Component<'a>>) -> Component<'a> {
    components.into_iter()
        .fold(Component { origin: Coord::new(0.0, 0.0, 0.0), dimensions: Coord::new(0.0, 0.0, 0.0), residues: vec![] },
            |acc, comp| {
                let (x0, y0, z0) = acc.dimensions.to_tuple();
                let (x1, y1, z1) = comp.dimensions.to_tuple();

                let dimensions = Coord::new(x0.max(x1), y0.max(y1), z0.max(z1));

                let mut residues = acc.residues;
                for residue in comp.residues {
                    residues.push(residue);
                }

                Component { origin: Coord::new(0.0, 0.0, 0.0), dimensions, residues }
            }
        )
}

/// Every residue has a reference to their base and a position.
pub struct Residue<'a> {
    /// Residue base.
    pub base: &'a ResidueBase,
    /// Absolute position of residue in system.
    pub position: Coord,
}

impl<'a> Residue<'a> {
    /// Translate the residue position. Does not alter the atom relative positions.
    fn translate(&self, add: Coord) -> Residue<'a> {
        Residue {
            base: self.base,
            position: self.position + add,
        }
    }
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

impl ResidueBase {
    /// Generate a proper residue at the input position.
    pub fn to_residue(&self, position: &Coord) -> Residue {
        Residue {
            base: &self,
            position: *position,
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

    #[allow(non_upper_case_globals)]
    const origin: Coord = Coord { x: 0.0, y: 0.0, z: 0.0 };

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

    fn setup_residues() -> (ResidueBase, ResidueBase) {
        let coord0 = Coord::new(0.0, 0.0, 0.0);
        let residue_one = ResidueBase {
            code: "R1".to_string(),
            atoms: vec![
                Atom { code: "A1".to_string(), position: coord0, },
                Atom { code: "A2".to_string(), position: coord0, },
                Atom { code: "A3".to_string(), position: coord0, },
            ]
        };
        let residue_two = ResidueBase {
            code: "R2".to_string(),
            atoms: vec![
                Atom { code: "B1".to_string(), position: coord0, },
                Atom { code: "B2".to_string(), position: coord0, },
            ]
        };

        (residue_one, residue_two)
    }

    // A simple component with two different residues and five atoms
    fn setup_component<'a>(base_one: &'a ResidueBase, base_two: &'a ResidueBase) -> Component<'a> {
        let residue_one = Residue {
            base: &base_one,
            position: Coord::new(0.0, 0.0, 0.0),
        };
        let residue_two = Residue {
            base: &base_two,
            position: Coord::new(1.0, 1.0, 1.0),
        };

        Component {
            origin: Coord::new(0.0, 0.0, 0.0),
            dimensions: Coord::new(0.0, 0.0, 0.0),
            residues: vec![
                residue_one,
                residue_two
            ]
        }
    }

    #[test]
    fn count_atoms_in_component() {
        let (res_one, res_two) = setup_residues();
        let component = setup_component(&res_one, &res_two);
        assert_eq!(5, component.num_atoms());
    }

    #[test]
    fn translate_a_component() {
        let (res_one, res_two) = setup_residues();
        let component = setup_component(&res_one, &res_two);
        let shift = Coord::new(0.0, 1.0, 2.0);

        let trans_component = component.translate(&shift);
        for (orig, updated) in component.residues.iter().zip(trans_component.residues.iter()) {
            assert_eq!(orig.position + shift, updated.position);
        }
        assert_eq!(shift, trans_component.origin);
    }

    #[test]
    fn residue_base_to_residue() {
        let base = ResidueBase {
            code: "RES".to_string(),
            atoms: vec![
                Atom { code: "A1".to_string(), position: Coord::new(0.0, 0.0, 0.0) },
                Atom { code: "A2".to_string(), position: Coord::new(0.0, 1.0, 2.0) }
            ],
        };
        let position = Coord::new(1.0, 1.0, 1.0);

        let residue = base.to_residue(&position);
        assert_eq!("RES", residue.base.code);
        assert_eq!(position, residue.position);
        assert_eq!(base.atoms, residue.base.atoms);
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
    fn joined_system_dimensions() {
        let components = vec![
            Component { origin, dimensions: Coord::new(1.0, 2.0, 3.0), residues: vec![] },
            Component { origin, dimensions: Coord::new(3.0, 1.0, 1.0), residues: vec![] },
            Component { origin, dimensions: Coord::new(1.0, 0.0, 4.0), residues: vec![] }
        ];

        let system = join_components(components);
        assert_eq!(Coord::new(3.0, 2.0, 4.0), system.dimensions);
    }

    #[test]
    fn joined_system_residues() {
        let (res_one, res_two) = setup_residues();

        let components = vec![
            Component {
                origin,
                dimensions: Coord::new(1.0, 0.0, 0.0),
                residues: vec![
                    Residue {
                        base: &res_one,
                        position: Coord::new(1.0, 0.0, 0.0),
                    },
                    Residue {
                        base: &res_one,
                        position: Coord::new(2.0, 0.0, 0.0),
                    },
                    Residue {
                        base: &res_one,
                        position: Coord::new(3.0, 0.0, 0.0),
                    }
                ]
            },
            Component {
                origin,
                dimensions: Coord::new(0.0, 0.0, 0.0),
                residues: vec![
                    Residue {
                        base: &res_two,
                        position: Coord::new(0.0, 1.0, 0.0),
                    },
                    Residue {
                        base: &res_two,
                        position: Coord::new(0.0, 2.0, 0.0),
                    }
                ]
            }
        ];

        let system = join_components(components);
        assert_eq!(5, system.residues.len());

        // The components should be flattened in the correct order from above,
        // check both ResidueBase and Coord.
        let mut iter = system.residues.iter();
        assert_eq!(&res_one, iter.next().unwrap().base);
        assert_eq!(&res_one, iter.next().unwrap().base);
        assert_eq!(&res_one, iter.next().unwrap().base);
        assert_eq!(&res_two, iter.next().unwrap().base);
        assert_eq!(&res_two, iter.next().unwrap().base);
        assert!(iter.next().is_none());

        let mut iter = system.residues.iter();
        assert_eq!(Coord::new(1.0, 0.0, 0.0), iter.next().unwrap().position);
        assert_eq!(Coord::new(2.0, 0.0, 0.0), iter.next().unwrap().position);
        assert_eq!(Coord::new(3.0, 0.0, 0.0), iter.next().unwrap().position);
        assert_eq!(Coord::new(0.0, 1.0, 0.0), iter.next().unwrap().position);
        assert_eq!(Coord::new(0.0, 2.0, 0.0), iter.next().unwrap().position);
        assert!(iter.next().is_none());
    }
}
