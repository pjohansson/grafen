//! The main user interface from which the user will define systems to create.
//! They can also access and modify the `DataBase` of components to use in their
//! systems.
//!
//! This is implemented as a *very* basic text interface. This could be improved
//! greatly by knowing more about human interface design. In particular the systems
//! for creating `SheetConfEntry` and `SystemDefinition` are in need of improvement.

mod edit_database;
mod define_system;
mod utils;

use super::Config;
use database::{DataBase, SheetConfEntry};
use error::{GrafenCliError, Result};
use output;
use ui::utils::{CommandList, CommandParser};

use grafen::cylinder::Cylinder;
use grafen::substrate::{create_substrate, Sheet, SheetConf};
use grafen::system::{Component, Coord, IntoComponent};
use std::error::Error;

#[derive(Clone, Debug, PartialEq)]
/// List of components that can be constructed.
pub enum AvailableComponents {
    Sheet { conf: SheetConfEntry, size: (f64, f64) },
}

#[derive(Clone, Debug, PartialEq)]
/// One system is defined by these attributes.
pub struct ComponentDefinition {
    pub definition: AvailableComponents,
    pub position: Coord,
}

#[derive(Debug)]
pub struct ConstructedComponent {
    description: String,
    pub component: Component,
}

#[derive(Debug)]
pub struct System {
    /// System box size. Either set by the user or calculated from the components.
    box_size: Option<Coord>,
    /// All components belonging to the system.
    components: Vec<ConstructedComponent>,
}

impl System {
    /// Calculate the box size from the system components. The largest extension along each
    /// direction is used for the final box size.
    ///
    /// Returns None if no components exist in the system.
    fn calc_box_size(&self) -> Option<Coord> {
        unimplemented!();
    }

    /// Calculate the number of atoms in the system.
    fn num_atoms(&self) -> usize {
        unimplemented!();
    }
}

impl ComponentDefinition {
    /// Return a description of the component that is to be created.
    fn describe(&self) -> String {
        let (x0, y0, z0) = self.position.to_tuple();

        match self.definition {
            AvailableComponents::Sheet { conf: ref conf, size: (dx, dy) } => {
                format!("Sheet of {} and size ({:.2}, {:.2}) at position ({:.2}, {:.2}, {:.2})",
                        conf.residue.code, dx, dy, x0, y0, z0)
            }
        }
    }

    /// Construct the component from the definition.
    fn into_component(self) -> Result<Component> {
        match self.definition {
            AvailableComponents::Sheet { conf: ref conf, size: (dx, dy) } => {
                let sheet = conf.to_conf(dx, dy);
                let component = create_substrate(&sheet)?;
                Ok(component.into_component())
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
    let mut system_defs: Vec<ComponentDefinition> = Vec::new();
    let mut system_components: Vec<ConstructedComponent> = Vec::new();
    //let mut system_components: Vec<Box<IntoComponent>> = Vec::new();

    let command_list: CommandList<Command> = vec![
        ("de", Command::DefineComponents, "Define the list of components to construct"),
        ("co", Command::ConstructComponents, "Construct components from all definitions"),
        ("db", Command::EditDatabase, "Edit the database of residue and object definitions"),
        ("save", Command::SaveSystem, "Save the constructed components to disk as a system"),
        ("quit", Command::Quit, "Quit the program"),
    ];
    let commands = CommandParser::from_list(command_list);

    loop {
        define_system::describe_system_definitions(&system_defs);
        describe_created_components(&system_components);

        commands.print_menu();
        let input = utils::get_input_string("Selection")?;
        println!("");

        if let Some((cmd, tail)) = commands.get_selection_and_tail(&input) {
            match cmd {
                Command::DefineComponents => {
                    match define_system::user_menu(&config.database, &mut system_defs) {
                        Ok(_) => println!("Finished editing list of definitions."),
                        Err(err) => println!("Could not create definition: {}", err.description()),
                    }
                },
                Command::ConstructComponents => {
                    match construct_components(&mut system_defs, &mut system_components) {
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
                    match output::write_gromos(&system_components, &config) {
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

fn construct_components(definitions: &mut Vec<ComponentDefinition>, components: &mut Vec<ConstructedComponent>) -> Result<()> {
    for def in definitions.drain(..) {
        let description = def.describe();
        let component = def.into_component()?;
        components.push(ConstructedComponent{ description, component });
    }

    Ok(())
}

fn describe_created_components(components: &Vec<ConstructedComponent>) {
    if components.is_empty() {
        println!("(No components have been created)");
    } else {
        println!("Constructed system components:");
        for (i, def) in components.iter().enumerate() {
            println!("{}. {}", i, &def.description);
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
        let base_one = resbase!["R1", ("A1", 0.0, 1.0, 2.0), ("A2", 0.0, 2.0, 1.0)];

        System {
            box_size: None,
            components: vec![
                ConstructedComponent {
                    description: "None".to_string(),
                    component: Component {
                        box_size: Coord::new(10.0, 1.0, 0.0),
                        origin: Coord::new(0.0, 0.0, 0.0),
                        residue_base: base_one.clone(),
                        residue_coords: vec![],
                    }
                }
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
            components: vec![],
        };

        assert!(system.calc_box_size().is_none());
    }
}
