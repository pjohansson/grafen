//! The main user interface from which the user will define systems to create.
//! They can also access and modify the `DataBase` of components to use in their
//! systems.
//!
//! This is implemented as a *very* basic text interface. This could be improved
//! greatly by knowing more about human interface design. In particular the systems
//! for creating `SheetConfEntry` and `SystemDefinition` are in need of improvement.

#[macro_use]
pub mod utils;
mod edit_database;
mod define_components;

use super::Config;
use database::AvailableComponents;
use error::{GrafenCliError, Result};
use output;
use ui::utils::{CommandParser, Describe};

use grafen::system::{Component, Coord, ResidueBase};
use std::error::Error;

#[derive(Debug)]
/// A `Component` which has been constructed along with a descriptive string.
pub struct ConstructedComponent {
    /// Description of the component.
    description: String,
    /// The component.
    pub component: Component,
}

impl utils::Describe for ConstructedComponent {
    fn describe(&self) -> String {
        format!("{} ({} residues at {})", self.description,
                                          self.component.num_residues(),
                                          self.component.origin)
    }
}

#[derive(Debug)]
/// All `ConstructedComponent`s and `ComponentDefinition`s are kept in this
/// `System` which keeps track of those and other meta information.
pub struct System {
    /// System box size. Either set by the user or calculated from the components.
    pub box_size: Option<Coord>,
    /// Component definitions.
    pub definitions: Vec<AvailableComponents>,
    /// Components and their descriptions belonging to the system.
    pub constructed: Vec<ConstructedComponent>,
}

impl System {
    /// Calculate the box size from the system components. The largest extension along each
    /// direction is used for the final box size.
    ///
    /// Returns None if no components exist in the system.
    pub fn calc_box_size(&self) -> Option<Coord> {
        if self.constructed.is_empty() {
            return None;
        }

        let box_size = self.constructed.iter().fold(
            Coord::new(0.0, 0.0, 0.0), |acc, conf| {
                let (x1, y1, z1) = acc.to_tuple();
                let (x2, y2, z2) = conf.component.box_size.to_tuple();

                Coord::new(x1.max(x2), y1.max(y2), z1.max(z2))
            }
        );

        Some(box_size)
    }

    fn describe(&self) {
        //define_components::describe_system_definitions(&self.definitions);
        utils::print_group("Defined components", &self.definitions);
        utils::print_group("Constructed components", &self.constructed);
    }

    /// Iterate over the residues in a `System`.
    ///
    /// Yields the `Coord` and `ResidueBase` for each residue, as the tuple `CoordAndResidue`.
    pub fn iter_residues(&self) -> ResidueIter {
        CoordsAndResiduesIterator::new(&self.constructed)
    }

    /// Calculate the number of atoms in the system.
    pub fn num_atoms(&self) -> usize {
        self.constructed.iter().map(|conf| {
            conf.component.residue_base.atoms.len() * conf.component.residue_coords.len()
        }).sum()
    }
}

/// We want to be able to iterate over all residues in a `System`.
/// To do this we use as custom `Iterator` which yields every residue's
/// `Coord` and `ResidueBase`.
struct CoordsAndResiduesIterator<'a> {
    components: &'a [ConstructedComponent],
    current_component: usize,
    current_coord: usize,
}

/// Construct it from a list of components.
impl<'a> CoordsAndResiduesIterator<'a> {
    fn new(components: &[ConstructedComponent]) -> Box<CoordsAndResiduesIterator> {
        Box::new(CoordsAndResiduesIterator {
            components: &components,
            current_component: 0,
            current_coord: 0,
        })
    }
}

// Newtypes for the iterator.
type CoordAndResidue<'a> = (Coord, &'a ResidueBase);
// TODO: This should use `impl Trait` once that is in Rust stable.
type ResidueIter<'a> = Box<Iterator<Item = CoordAndResidue<'a>> + 'a>;

impl<'a> Iterator for CoordsAndResiduesIterator<'a> {
    type Item = CoordAndResidue<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(cons_component) = self.components.get(self.current_component) {
                let component = &cons_component.component;

                if let Some(&coord) = component.residue_coords.get(self.current_coord) {
                    self.current_coord += 1;
                    return Some((component.origin + coord, &component.residue_base));
                } else {
                    self.current_component += 1;
                    self.current_coord = 0;
                }
            } else {
                return None;
            }
        }
    }
}


#[derive(Clone, Copy, Debug)]
/// User commands for defining the system.
enum Command {
    DefineComponents,
    ConstructComponents,
    EditDatabase,
    SaveSystem,
    Quit,
}

/// Loop over a menu in which the user can define the system which will be created, etc.
///
/// The idea of this interface is relatively simple:
///
/// 1. The user reads or constructs a `DataBase` of residues (`ResidueBase`).
/// 2. Then constructs definitions of substrates or objects to be created (can also be
///    saved to and read from the `DataBase`).
/// 3. Processes these definitions to create the actual components which make up the system.
/// 4. Modifies or transforms these components by copying, translating, rotating etc.
/// 5. Finally saves the full system to disk.
///
/// Modifying the `DataBase`, setting the definitions `SheetConf` and editing the
/// `Component`s each require a separate menu, accessed from this super menu.
/// This menu should also allow the user to save the system to disk, set its name
/// and file path and any other possible options.
pub fn user_menu(mut config: &mut Config) -> Result<()> {
    let mut system = System { box_size: None, constructed: vec![], definitions: vec![] };

    let commands = command_parser!(
        ("de", Command::DefineComponents, "Define the list of components to construct"),
        ("co", Command::ConstructComponents, "Construct components from all definitions"),
        ("db", Command::EditDatabase, "Edit the database of residue and object definitions"),
        ("save", Command::SaveSystem, "Save the constructed components to disk as a system"),
        ("quit", Command::Quit, "Quit the program")
    );

    loop {
        system.describe();

        commands.print_menu();
        let input = utils::get_input_string("Selection")?;
        println!("");

        if let Some((cmd, _)) = commands.get_selection_and_tail(&input) {
            match cmd {
                Command::DefineComponents => {
                    match define_components::user_menu(&config.database, &mut system.definitions) {
                        Ok(_) => println!("Finished editing list of definitions."),
                        Err(err) => println!("Could not create definition: {}", err.description()),
                    }
                },
                Command::ConstructComponents => {
                    match construct_components(&mut system) {
                        Ok(_) => println!("Successfully constructed all components."),
                        Err(err) => println!("Could not construct all components: {}", err.description()),
                    }
                },
                Command::EditDatabase => {
                    match edit_database::user_menu(&mut config.database) {
                        Ok(msg) => println!("{}", msg),
                        Err(err) => println!("Error when editing database: {}", err.description()),
                    }
                },
                Command::SaveSystem => {
                    match output::write_gromos(&system, &config) {
                        Ok(()) => println!("Saved system to disk."),
                        Err(msg) => println!("Error when saving system: {}", msg),
                    }
                },
                Command::Quit => {
                    return Err(GrafenCliError::QuitWithoutSaving);
                },
            }
        } else {
            println!("Not a valid selection.");
        }

        println!("");
    }
}

fn construct_components(system: &mut System) -> Result<()> {
    let ref mut definitions = system.definitions;
    let ref mut components = system.constructed;

    for def in definitions.drain(..) {
        let description = def.describe();
        let component = def.into_component()?;
        components.push(ConstructedComponent{ description, component });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use grafen::system::{Atom, ResidueBase};

    /// Setup the base residues.
    /// base_one: two atoms
    /// base_two: one atom
    fn setup_base_residues() -> (ResidueBase, ResidueBase) {
        let base_one = ResidueBase {
            code: "R1".to_string(),
            atoms: vec![
                Atom { code: "A1".to_string(), position: Coord::new(0.0, 1.0, 2.0) },
                Atom { code: "A2".to_string(), position: Coord::new(0.0, 2.0, 1.0) }
                ],
            };
        let base_two = ResidueBase {
            code: "R2".to_string(),
            atoms: vec![
                Atom { code: "B".to_string(), position: Coord::new(0.0, 1.0, 2.0) },
                ],
            };

        (base_one, base_two)
    }

    /// Setup a system of three components and in total four residues / seven atoms:
    ///     0. base_one, 2 atoms
    ///     1. base_two, 1 atom
    ///     2. base_one, 2 atoms
    ///     3. base_one, 2 atoms
    fn setup_system() -> System {
        let (base_one, base_two) = setup_base_residues();

        System {
            box_size: None,
            definitions: vec![],
            constructed: vec![
                ConstructedComponent {
                    description: "None".to_string(),
                    component: Component {
                        // Largest along x
                        box_size: Coord::new(10.0, 1.0, 0.0),
                        origin: Coord::new(0.0, 0.0, 0.0),
                        residue_base: base_one.clone(),
                        // 1 residue * 2 atoms per residue
                        residue_coords: vec![Coord::new(0.0, 0.0, 0.0)],
                    },
                },
                ConstructedComponent {
                    description: "None".to_string(),
                    component: Component {
                        // Largest along z
                        box_size: Coord::new(1.0, 1.0, 7.0),
                        origin: Coord::new(0.0, 0.0, 0.0),
                        residue_base: base_two.clone(),
                        // 1 * 1 atoms
                        residue_coords: vec![Coord::new(0.0, 0.0, 0.0)],
                    },
                },
                ConstructedComponent {
                    description: "None".to_string(),
                    component: Component {
                        // Largest along y
                        box_size: Coord::new(1.0, 5.0, 0.0),
                        origin: Coord::new(0.0, 0.0, 0.0),
                        residue_base: base_one.clone(),
                        // 2 * 2 atoms
                        residue_coords: vec![Coord::new(0.0, 0.0, 0.0), Coord::new(1.0, 0.0, 0.0)],
                    },
                },
            ]
        }
    }

    #[test]
    fn all_atoms_are_counted_in_system() {
        let system = setup_system();

        assert_eq!(7, system.num_atoms());
    }

    #[test]
    fn box_size_is_largest_in_each_direction() {
        let system = setup_system();
        assert_eq!(Some(Coord::new(10.0, 5.0, 7.0)), system.calc_box_size());
    }

    #[test]
    fn box_size_is_none_if_no_components() {
        let system = System {
            box_size: None,
            definitions: vec![],
            constructed: vec![],
        };

        assert!(system.calc_box_size().is_none());
    }

    #[test]
    fn iterate_over_system_residues() {
        let (base_one, base_two) = setup_base_residues();
        let system = setup_system();

        let mut iter = system.iter_residues();
        assert_eq!(Some((Coord::new(0.0, 0.0, 0.0), &base_one)), iter.next());
        assert_eq!(Some((Coord::new(0.0, 0.0, 0.0), &base_two)), iter.next());
        assert_eq!(Some((Coord::new(0.0, 0.0, 0.0), &base_one)), iter.next());
        assert_eq!(Some((Coord::new(1.0, 0.0, 0.0), &base_one)), iter.next());
        assert_eq!(None, iter.next());
    }

    #[test]
    fn iter_residues_adds_component_origin() {
        let origin = Coord::new(-5.0, 10.0, 5.0);
        let coord1 = Coord::new(0.0, 0.0, 0.0);
        let coord2 = Coord::new(1.0, 1.0, 1.0);

        let (base_one, _) = setup_base_residues();
        let system = System {
            box_size: None,
            definitions: vec![],
            constructed: vec![
                ConstructedComponent {
                    description: "None".to_string(),
                    component: Component {
                        box_size: Coord::ORIGO,
                        origin: origin,
                        residue_base: base_one.clone(),
                        residue_coords: vec![coord1, coord2],
                    },
                },
            ]
        };

        let mut iter = system.iter_residues();
        assert_eq!(Some((origin + coord1, &base_one)), iter.next());
        assert_eq!(Some((origin + coord2, &base_one)), iter.next());
        assert_eq!(None, iter.next());
    }
}
