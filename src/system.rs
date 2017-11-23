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
use describe::{describe_list, Describe};
use database::{ComponentEntry, DataBase};
use iterator::AtomIterItem;

use std::fmt::Write;
use std::path::PathBuf;

/// Main structure of a constructed system with several components.
pub struct System {
    /// Title of system.
    pub title: String,
    /// Path to which the system will be written.
    pub output_path: PathBuf,
    /// Database with component and residue definitions.
    pub database: DataBase,
    /// List of constructed components.
    pub components: Vec<ComponentEntry>,
}

impl<'a> Component<'a> for System {
    /// Calculate the total box size of the system as the maximum size along each axis
    /// from all components.
    fn box_size(&self) -> Coord {
        self.components
            .iter()
            .map(|object| object.box_size())
            .fold(Coord::new(0.0, 0.0, 0.0), |max_size, current| {
                Coord {
                    x: max_size.x.max(current.x),
                    y: max_size.y.max(current.y),
                    z: max_size.z.max(current.z),
                }
            })
    }

    /// Return an `Iterator` over all atoms in the whole system as `CurrentAtom` objects.
    ///
    /// Corrects residue and atom index numbers to be system-absolute instead
    /// of for each component.
    fn iter_atoms(&'a self) -> AtomIterItem {
        // We want to return system-wide atom and residue indices. The atom index
        // is easy to increase by one for each iterated atom, but to update the residue
        // index we have to see if it has changed from the previous iteration.
        struct Indices { atom: u64, residue: u64, last_residue: u64 }

        Box::new(self.components
            .iter()
            .flat_map(|object| object.iter_atoms())
            .scan(Indices { atom: 0, residue: 0, last_residue: 0 }, |state, mut current| {
                // Find out if the component residue number has increased, if so update it
                if current.residue_index != state.last_residue {
                    state.last_residue = current.residue_index;
                    state.residue += 1;
                }

                // Set the absolute atom and residue indices to the object
                current.atom_index = state.atom;
                current.residue_index = state.residue;

                state.atom += 1;

                Some(current)
            })
        )
    }

    /// Calculate the total number of atoms in the system.
    fn num_atoms(&self) -> u64 {
        self.components.iter().map(|object| object.num_atoms()).sum()
    }
}

impl Describe for System {
    fn describe(&self) -> String {
        let mut description = String::new();

        writeln!(description, "Title: '{}'", self.title).unwrap();
        writeln!(description, "Output path: {}", self.output_path.to_str().unwrap_or("(Not set)")).unwrap();
        writeln!(description, "").unwrap();
        writeln!(description, "{}", describe_list("Components", &self.components)).unwrap();

        description
    }

    fn describe_short(&self) -> String {
        self.describe()
    }
}

/// Methods for yielding atoms and output information from constructed objects.
pub trait Component<'a> {
    /// Return the size of the object's bounding box seen from origo.
    ///
    /// That is, for a component of size (1, 1, 1) with origin (1, 1, 1)
    /// this returns (2, 2, 2).
    fn box_size(&self) -> Coord;

    /// Return an `Iterator` over all atoms in the object as `CurrentAtom` objects.
    fn iter_atoms(&'a self) -> AtomIterItem<'a>;

    /// Return the number of atoms in the object.
    fn num_atoms(&self) -> u64;
}

#[macro_export]
/// Macro to implement `Component` for an object.
///
/// The object has to contain the fields
/// ```
/// {
///     residue: Option<Residue>,
///     origin: Coord,
///     coords: [Coord]
/// }
/// ```
/// and the method `calc_box_size`.
macro_rules! impl_component {
    ( $( $class:path ),+ ) => {
        $(
            impl<'a> Component<'a> for $class {
                fn box_size(&self) -> Coord {
                    self.calc_box_size() + self.origin
                }

                fn iter_atoms(&self) -> AtomIterItem {
                    Box::new(
                        AtomIterator::new(self.residue.as_ref(), &self.coords, self.origin)
                    )
                }

                fn num_atoms(&self) -> u64 {
                    let residue_len = self.residue
                        .as_ref()
                        .map(|res| res.atoms.len())
                        .unwrap_or(0);

                    (residue_len * self.coords.len()) as u64
                }
            }
        )*
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

impl Describe for Atom {
    fn describe(&self) -> String {
        format!("{} {}", self.code, self.position)
    }

    fn describe_short(&self) -> String {
        format!("{}", self.code)
    }
}

/// A base for generating atoms belonging to a residue.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Residue {
    pub code: String,
    pub atoms: Vec<Atom>,
}

impl Describe for Residue {
    fn describe(&self) -> String {
        format!("{} ({} atoms)", self.code, self.atoms.len())
    }

    fn describe_short(&self) -> String {
        format!("{}", self.code)
    }
}

#[macro_export]
/// Construct a Residue with a code and atoms.
///
/// At least one atom has to be present in the base. This is not a limitation
/// when explicitly constructing a residue, but it makes no sense to allow
/// it when invoking a constructor like this.
///
/// # Examples
/// ```
/// # #[macro_use] extern crate grafen;
/// # use grafen::coord::Coord;
/// # use grafen::system::{Atom, Residue};
/// # fn main() {
/// let expect = Residue {
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

            Residue {
                code: $rescode.to_string(),
                atoms: temp_vec,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use coord::Translate;
    use iterator::AtomIterator;
    use volume::Cuboid;

    #[test]
    fn create_residue_base_macro() {
        let expect = Residue {
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

    // Verify that the macro generated `iter_atoms` methods work as expected.

    #[derive(Debug, Deserialize, Serialize)]
    struct TestObject { residue: Option<Residue>, origin: Coord, coords: Vec<Coord> }
    impl Describe for TestObject {
        fn describe(&self) -> String { unimplemented!(); }
        fn describe_short(&self) -> String { unimplemented!(); }
    }
    impl TestObject { fn calc_box_size(&self) -> Coord { unimplemented!(); } }

    impl_translate![TestObject];
    impl_component![TestObject];

    #[test]
    fn iterate_over_atoms_in_macro_generated_impl_object() {
        let residue = resbase!["RES", ("A", 0.0, 0.1, 0.2), ("B", 0.3, 0.4, 0.5)];
        let constructed = TestObject {
            residue: Some(residue.clone()),
            origin: Coord::ORIGO,
            coords: vec![Coord::new(0.0, 2.0, 4.0), Coord::new(1.0, 3.0, 5.0)],
        };

        // Skip to and compare against the last atom
        let mut iter = constructed.iter_atoms().skip(3);

        let atom = iter.next().unwrap();
        assert_eq!(3, atom.atom_index);
        assert_eq!(1, atom.residue_index);
        assert_eq!(&residue.atoms[1], atom.atom);
        assert_eq!(&residue, atom.residue);
        assert_eq!(residue.atoms[1].position + constructed.coords[1], atom.position);

        assert_eq!(None, iter.next());
    }

    #[test]
    fn num_atoms_in_macro_generated_impl_objects() {
        // An object with 2 residues * 2 atoms / residue = 4 atoms
        let residue = resbase!["RES", ("A", 0.0, 0.1, 0.2), ("B", 0.3, 0.4, 0.5)];
        let mut constructed = TestObject {
            residue: Some(residue.clone()),
            origin: Coord::ORIGO,
            coords: vec![Coord::new(0.0, 2.0, 4.0), Coord::new(1.0, 3.0, 5.0)],
        };

        assert_eq!(4, constructed.num_atoms());

        // With no residue set, no atoms
        constructed.residue = None;
        assert_eq!(0, constructed.num_atoms());
    }

    #[test]
    fn box_size_in_macro_generated_impls_adds_origin() {
        let origin = Coord::new(0.0, 1.0, 2.0);
        let size = Coord::new(5.0, 6.0, 7.0);

        let cuboid = Cuboid {
            name: None,
            residue: None,
            size,
            origin,
            coords: vec![],
        };

        assert_eq!(origin + size, cuboid.box_size());
    }

    #[test]
    fn iterate_over_atoms_in_macro_generated_impl_without_residue_returns_empty_iterator() {
        let constructed = TestObject {
            residue: None,
            origin: Coord::ORIGO,
            coords: vec![Coord::new(0.0, 2.0, 4.0), Coord::new(1.0, 3.0, 5.0)],
        };

        let mut iter = constructed.iter_atoms();
        assert_eq!(None, iter.next());
    }

    #[test]
    fn iterate_over_atoms_in_whole_system_gives_correct_results() {
        // The first component has 3 residues * 2 atoms / residue = 6 atoms
        let residue1 = resbase!["ONE", ("A", 1.0, 1.0, 1.0), ("B", 2.0, 2.0, 2.0)];
        let component1 = Cuboid {
            name: None,
            residue: Some(residue1.clone()),
            origin: Coord::default(),
            size: Coord::default(),
            coords: vec![Coord::default(), Coord::default(), Coord::default()],
        };

        let origin = Coord::new(10.0, 20.0, 30.0);
        let position = Coord::new(1.0, 2.0, 3.0);

        // The second component has 2 residues * 1 atom / residue = 2 atoms
        let residue2 = resbase!["TWO", ("C", 5.0, 6.0, 7.0)];
        let component2 = Cuboid {
            name: None,
            residue: Some(residue2.clone()),
            origin: origin,
            size: Coord::default(),
            coords: vec![position, Coord::default()],
        };

        let system = System {
            title: String::new(),
            output_path: PathBuf::new(),
            database: DataBase::new(),
            components: vec![
                ComponentEntry::VolumeCuboid(component1),
                ComponentEntry::VolumeCuboid(component2)
            ],
        };

        let iter = system.iter_atoms();

        // Inspect the seventh atom: the first in the second component
        let atom = iter.skip(6).next().unwrap();

        assert_eq!(6, atom.atom_index);
        assert_eq!(3, atom.residue_index);
        assert_eq!(&residue2.atoms[0], atom.atom);
        assert_eq!(&residue2, atom.residue);
        assert_eq!(origin + position + residue2.atoms[0].position, atom.position);
    }

    #[test]
    fn num_atoms_in_system() {
        // 2 components * 3 residues / component * 2 atoms / residue = 12 atoms
        let residue = resbase!["RES", ("A", 0.0, 0.0, 0.0), ("B", 1.0, 0.0, 0.0)];
        let component = ComponentEntry::VolumeCuboid(Cuboid {
            name: None,
            residue: Some(residue.clone()),
            origin: Coord::default(),
            size: Coord::default(),
            coords: vec![Coord::default(), Coord::default(), Coord::default()],
        });

        let system = System {
            title: String::new(),
            output_path: PathBuf::new(),
            database: DataBase::new(),
            components: vec![
                component.clone(), component.clone()],
        };

        assert_eq!(12, system.num_atoms());
    }

    #[test]
    fn box_size_of_system_adds_origin() {
        let component1 = ComponentEntry::VolumeCuboid(Cuboid {
            name: None,
            residue: None,
            origin: Coord::new(0.0, 0.0, 0.0),
            size: Coord::new(5.0, 5.0, 5.0),
            coords: vec![],
        });

        let component2 = ComponentEntry::VolumeCuboid(Cuboid {
            name: None,
            residue: None,
            origin: Coord::new(3.0, 3.0, 3.0),
            size: Coord::new(3.0, 2.0, 1.0),
            coords: vec![],
        });

        let system = System {
            title: String::new(),
            output_path: PathBuf::new(),
            database: DataBase::new(),
            components: vec![
                component1.clone(),
                component2.clone()
            ],
        };

        assert_eq!(Coord::new(6.0, 5.0, 5.0), system.box_size());
    }
}
