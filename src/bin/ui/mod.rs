//! The main user interface from which the user will define systems to create.
//! They can also access and modify the `DataBase` of components to use in their
//! systems.
//!
//! This is implemented as a *very* basic text interface. This could be improved
//! greatly by knowing more about human interface design. In particular the systems
//! for creating `SheetConfEntry` and `SystemDefinition` are in need of improvement.

mod edit_database;
mod define_components;
mod utils;

use super::Config;
use database::{AvailableComponents, CylinderConfEntry, SheetConfEntry};
use error::{GrafenCliError, Result};
use output;
use ui::utils::{CommandList, CommandParser};

use grafen::cylinder::Cylinder;
use grafen::substrate::{create_substrate, SheetConf};
use grafen::system::{Component, Coord, IntoComponent, Translate};
use std::error::Error;

#[derive(Debug)]
/// A `Component` which has been constructed along with a descriptive string.
pub struct ConstructedComponent {
    /// Description of the component.
    description: String,
    /// The component.
    pub component: Component,
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

    /// Calculate the number of atoms in the system.
    pub fn num_atoms(&self) -> usize {
        self.constructed.iter().map(|conf| {
            conf.component.residue_base.atoms.len() * conf.component.residue_coords.len()
        }).sum()
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

    let command_list: CommandList<Command> = vec![
        ("de", Command::DefineComponents, "Define the list of components to construct"),
        ("co", Command::ConstructComponents, "Construct components from all definitions"),
        ("db", Command::EditDatabase, "Edit the database of residue and object definitions"),
        ("save", Command::SaveSystem, "Save the constructed components to disk as a system"),
        ("quit", Command::Quit, "Quit the program"),
    ];
    let commands = CommandParser::from_list(command_list);

    loop {
        define_components::describe_system_definitions(&system.definitions);
        describe_created_components(&system.constructed);

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
        let description = def.describe_long();
        let component = def.into_component()?;
        components.push(ConstructedComponent{ description, component });
    }

    Ok(())
}

fn describe_created_components(constructed: &Vec<ConstructedComponent>) {
    if constructed.is_empty() {
        println!("(No components have been created)");
    } else {
        println!("Constructed system components:");
        for (i, conf) in constructed.iter().enumerate() {
            println!("{}. {}", i, &conf.description);
        }
    }

    println!("");
}

#[cfg(test)]
mod tests {
    use super::*;
    use grafen::system::{Atom, ResidueBase};

    /// Setup a system of three components and in total seven atoms
    fn setup_system() -> System {
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
}
