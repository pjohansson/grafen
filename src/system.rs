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
use iterator::{ResidueIter, ResidueIterOut};

use colored::*;
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

impl<'a> System {
    /// Calculate the total box size of the system as the maximum size along each axis
    /// from all components.
    pub fn box_size(&self) -> Coord {
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

    /// Print the system state to standard error.
    pub fn print_state(&self) {
        let (dx, dy, dz) = self.box_size().to_tuple();

        eprintln!("{}", "System".underline().color("yellow"));
        eprintln!("Title       '{}'", self.title);
        eprintln!("Output path  {}", self.output_path.to_str().unwrap_or("(Not set)"));
        eprintln!("Box size     ({:.8}, {:.8}, {:.8})", dx, dy, dz);
        eprintln!("");

        if self.components.len() > 0 {
            eprintln!("{}", describe_list("Components", &self.components));
        } else {
            eprintln!("(no constructed components)\n");
        }
    }

    /// Calculate the total number of atoms in the system.
    pub fn num_atoms(&self) -> u64 {
        self.components.iter().map(|object| object.num_atoms()).sum()
    }
}

/// Methods for yielding atoms and output information from constructed objects.
pub trait Component<'a> {
    /// Assign some residues from an `iter_residues()` call to the component, presumably
    /// after modifying it.
    ///
    /// # Note
    /// It is assumed that the assignment is to an object of the same type that created
    /// the residue collection. Any differing information will be thrown away.
    fn assign_residues(&mut self, residues: &[ResidueIterOut]);

    /// Return the size of the object's bounding box seen from origo.
    ///
    /// That is, for a component of size (1, 1, 1) with origin (1, 1, 1)
    /// this returns (2, 2, 2).
    fn box_size(&self) -> Coord;

    /// Return the origin of the component.
    fn get_origin(&self) -> Coord;

    /// Return an `Iterator` over all residues of an object.
    fn iter_residues(&self) -> ResidueIter;

    /// Return the number of atoms in the object.
    fn num_atoms(&self) -> u64;

    /// Return the component with its coordinates adjusted to lie within its box.
    fn with_pbc(self) -> Self;
}

#[macro_export]
/// Macro to implement `Component` for an object.
///
/// The object has to contain the fields
/// {
///     residue: Option<Residue>,
///     origin: Coord,
///     coords: [Coord]
/// }
/// and the method `calc_box_size`.
macro_rules! impl_component {
    ( $( $class:path ),+ ) => {
        $(
            impl<'a> Component<'a> for $class {
                /// Assign a set of input residues to the component.
                ///
                /// # Downcasting information
                /// Note that some information may be downcast in the particular implementation
                /// for this `Component`. This is due to the base of it saving only as single
                /// coordinate for every `Residue` position: the residue atoms are exactly
                /// relative to this position, not set explicitly.
                ///
                /// However, the residues which result from calling eg. `iter_residues()`
                /// on components results in objects which carry information about atoms
                /// with all of their positions set explicitly (but relative to the containing
                /// `Component`). This explicit information will be lost as it is downcast
                /// to a single position per residue, with residue-relative positions that are
                /// shared by all atoms.
                ///
                /// The component-relative position for the iterated residues is taken as
                /// the first atom of every object. That position is subtracted by
                /// the residue-relative position of the atom of the `Residue` that is set
                /// to the `Component`, to find the component-relative position of the residue.
                ///
                /// Furthermore, note that any information about several different residue
                /// types in the iterating object is lost. This object assumes that the
                /// current set `Residue` is the only existing residue in the iterator.
                ///
                /// # Panics
                /// Panics if no `Residue` is set to the `Component`, if the `Residue`
                /// contains no `Atom`s or if any residue in the iterating object contains
                /// no atoms. This *should* never happen since we should always assign
                /// residues to complete objects of the same type, but we could consider
                /// this to returning a `Result` to guard against it.
                fn assign_residues(&mut self, residues: &[ResidueIterOut]) {
                    let residue = self.residue.clone().unwrap();

                    self.coords = residues.iter()
                        .map(|res| res.get_atoms()[0].1 - residue.atoms[0].position)
                        .collect::<Vec<_>>();
                }

                fn box_size(&self) -> Coord {
                    self.calc_box_size() + self.origin
                }

                fn get_origin(&self) -> Coord {
                    self.origin
                }

                fn iter_residues(&self) -> ResidueIter {
                    match self.residue {
                        None => ResidueIter::None,
                        Some(ref code) => ResidueIter::Component(code, self.coords.iter()),
                    }
                }

                fn num_atoms(&self) -> u64 {
                    let residue_len = self.residue
                        .as_ref()
                        .map(|res| res.atoms.len())
                        .unwrap_or(0);

                    (residue_len * self.coords.len()) as u64
                }

                fn with_pbc(mut self) -> Self {
                    let box_size = self.calc_box_size();

                    self.coords
                        .iter_mut()
                        .for_each(|c| *c = c.with_pbc(box_size));

                    self
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

    #[derive(Debug, Deserialize, Serialize)]
    struct TestObject { residue: Option<Residue>, origin: Coord, coords: Vec<Coord> }
    impl Describe for TestObject {
        fn describe(&self) -> String { "Doesn't matter".to_string() }
        fn describe_short(&self) -> String { "Doesn't matter".to_string() }
    }
    impl TestObject { fn calc_box_size(&self) -> Coord { Coord::ORIGO } }

    impl_translate![TestObject];
    impl_component![TestObject];

    #[test]
    fn iterate_over_residues_in_macro_generated_impl_object_works_and_ignores_origin() {
        let residue = resbase!["RES", ("A", 0.0, 0.1, 0.2), ("B", 0.3, 0.4, 0.5)];
        let constructed = TestObject {
            residue: Some(residue.clone()),
            origin: Coord::new(10.0, 20.0, 30.0), // Will be ignored
            coords: vec![Coord::new(0.0, 2.0, 4.0), Coord::new(1.0, 3.0, 5.0)],
        };

        let mut iter = constructed.iter_residues();

        // Check the last residue.
        assert!(iter.next().is_some());
        let res = iter.next().unwrap();

        let res_name = res.get_residue();
        assert_eq!(*res_name.borrow(), "RES");

        let atoms = res.get_atoms();
        assert_eq!(atoms.len(), 2);
        assert_eq!(*atoms[0].0.borrow(), "A");
        assert_eq!(atoms[0].1, Coord::new(1.0, 3.1, 5.2));
        assert_eq!(*atoms[1].0.borrow(), "B");
        assert_eq!(atoms[1].1, Coord::new(1.3, 3.4, 5.5));

        assert!(iter.next().is_none());
    }

    #[test]
    fn iterate_over_residues_in_either_empty_or_without_residue_returns_none() {
        let residue = resbase!["RES", ("A", 0.0, 0.1, 0.2), ("B", 0.3, 0.4, 0.5)];
        let empty_component = TestObject {
            residue: Some(residue.clone()),
            origin: Coord::ORIGO,
            coords: Vec::new(),
        };
        assert!(empty_component.iter_residues().next().is_none());

        let without_residue = TestObject {
            residue: None,
            origin: Coord::ORIGO,
            coords: Vec::new(),
        };
        assert!(without_residue.iter_residues().next().is_none());
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
    fn origin_in_macro_generated_impl_objects() {
        let origin = Coord::new(1.0, 2.0, 3.0);
        let component = TestObject {
            residue: None,
            origin,
            coords: Vec::new(),
        };

        assert_eq!(component.get_origin(), origin);
    }

    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn assigning_residues_in_macro_generated_impl_objects() {
        // The residue of the component has two atoms with these positions relative to the itself.
        let atom1_relative = Coord::new(1.0, 2.0, 3.0);
        let atom2_relative = Coord::new(1.1, 2.2, 3.3);

        let residue = resbase![
            "RES",
            ("A", atom1_relative.x, atom1_relative.y, atom1_relative.z),
            ("B", atom2_relative.x, atom2_relative.y, atom2_relative.z)
        ];

        let res_name = Rc::new(RefCell::new(residue.code.to_string()));
        let atom1_name = Rc::new(RefCell::new(residue.atoms[0].code.to_string()));
        let atom2_name = Rc::new(RefCell::new(residue.atoms[1].code.to_string()));

        let mut cuboid = Cuboid {
            residue: Some(residue.clone()),
            .. Cuboid::default()
        };

        // The residue iterator has atom positions which are component-relative,
        // not residue-relative. These are their positions in the component.
        let res1_position = Coord::new(0.0, 10.0, 20.0);
        let res2_position = Coord::new(30.0, 40.0, 50.0);

        // In this list, the component-relative atom positions are used.
        let residues = vec![
            ResidueIterOut::FromComp(Rc::clone(&res_name), vec![
                (Rc::clone(&atom1_name), res1_position + atom1_relative),
                (Rc::clone(&atom2_name), res1_position + atom2_relative),
            ]),
            ResidueIterOut::FromComp(Rc::clone(&res_name), vec![
                (Rc::clone(&atom1_name), res2_position + atom1_relative),
                (Rc::clone(&atom2_name), res2_position + atom2_relative),
            ])
        ];

        // Assign the residues and check that the main residue was not modified.
        cuboid.assign_residues(&residues);
        assert_eq!(cuboid.residue.unwrap(), residue);

        // Assert that the coordinate vector contains the component-relative positions.
        assert_eq!(cuboid.coords.len(), 2);
        assert_eq!(cuboid.coords[0], res1_position);
        assert_eq!(cuboid.coords[1], res2_position);
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
            density: None,
            coords: vec![],
        };

        assert_eq!(origin + size, cuboid.box_size());
    }

    #[test]
    fn with_pbc_in_macro_generated_impls_works_locally() {
        let origin = Coord::new(10.0, 20.0, 30.0);
        let size = Coord::new(1.0, 2.0, 3.0);

        let cuboid = Cuboid {
            origin,
            size,
            coords: vec![
                Coord::new(-0.9, 0.1, 0.1), // outside locally
                Coord::new(0.1, 0.1, 0.1),  // inside locally
                Coord::new(1.1, 0.1, 0.1),  // outside locally
            ],
            .. Cuboid::default()
        }.with_pbc();

        let expected = vec![
            Coord::new(0.1, 0.1, 0.1), // moved inside
            Coord::new(0.1, 0.1, 0.1), // unmoved
            Coord::new(0.1, 0.1, 0.1), // moved inside
        ];

        assert_eq!(cuboid.coords, expected);
    }
    //
    // #[test]
    // fn iterate_over_atoms_in_whole_system_gives_correct_results() {
    //     // The first component has 3 residues * 2 atoms / residue = 6 atoms
    //     let residue1 = resbase!["ONE", ("A", 1.0, 1.0, 1.0), ("B", 2.0, 2.0, 2.0)];
    //     let component1 = Cuboid {
    //         name: None,
    //         residue: Some(residue1.clone()),
    //         origin: Coord::default(),
    //         size: Coord::default(),
    //         density: None,
    //         coords: vec![Coord::default(), Coord::default(), Coord::default()],
    //     };
    //
    //     let origin = Coord::new(10.0, 20.0, 30.0);
    //     let position = Coord::new(1.0, 2.0, 3.0);
    //
    //     // The second component has 2 residues * 1 atom / residue = 2 atoms
    //     let residue2 = resbase!["TWO", ("C", 5.0, 6.0, 7.0)];
    //     let component2 = Cuboid {
    //         name: None,
    //         residue: Some(residue2.clone()),
    //         origin: origin,
    //         size: Coord::default(),
    //         density: None,
    //         coords: vec![position, Coord::default()],
    //     };
    //
    //     let system = System {
    //         title: String::new(),
    //         output_path: PathBuf::new(),
    //         database: DataBase::new(),
    //         components: vec![
    //             ComponentEntry::VolumeCuboid(component1),
    //             ComponentEntry::VolumeCuboid(component2)
    //         ],
    //     };
    //
    //     let iter = system.iter_atoms();
    //
    //     // Inspect the seventh atom: the first in the second component
    //     let atom = iter.skip(6).next().unwrap();
    //
    //     assert_eq!(6, atom.atom_index);
    //     assert_eq!(3, atom.residue_index);
    //     assert_eq!(&residue2.atoms[0], atom.atom);
    //     assert_eq!(&residue2, atom.residue);
    //     assert_eq!(origin + position + residue2.atoms[0].position, atom.position);
    // }

    #[test]
    fn num_atoms_in_system() {
        // 2 components * 3 residues / component * 2 atoms / residue = 12 atoms
        let residue = resbase!["RES", ("A", 0.0, 0.0, 0.0), ("B", 1.0, 0.0, 0.0)];
        let component = ComponentEntry::VolumeCuboid(Cuboid {
            name: None,
            residue: Some(residue.clone()),
            origin: Coord::default(),
            size: Coord::default(),
            density: None,
            coords: vec![Coord::default(), Coord::default(), Coord::default()],
        });

        let system = System {
            title: String::new(),
            output_path: PathBuf::new(),
            database: DataBase::new(),
            components: vec![
                component.clone(), component.clone()
            ],
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
            density: None,
            coords: vec![],
        });

        let component2 = ComponentEntry::VolumeCuboid(Cuboid {
            name: None,
            residue: None,
            origin: Coord::new(3.0, 3.0, 3.0),
            size: Coord::new(3.0, 2.0, 1.0),
            density: None,
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
