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
    //pub finalized: SheetConf,
}

impl ComponentDefinition {
    fn describe(&self) -> String {
        let (x0, y0, z0) = self.position.to_tuple();

        match self.definition {
            AvailableComponents::Sheet { conf: ref conf, size: (dx, dy) } => {
                format!("Sheet of {} and size ({:.2}, {:.2}) at position ({:.2}, {:.2}, {:.2})",
                        conf.residue.code, dx, dy, x0, y0, z0)
            }
        }
    }

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
    let mut system_components: Vec<Box<IntoComponent>> = Vec::new();

    let command_list: CommandList<Command> = vec![
        ("define", Command::DefineComponents, "Define the list of components to construct"),
        ("construct", Command::ConstructComponents, "Construct components from all definitions"),
        ("db", Command::EditDatabase, "Edit the database of residue and object definitions"),
        ("save", Command::SaveSystem, "Save the constructed components to disk as a system"),
        ("quit", Command::Quit, "Quit the program"),
    ];
    let commands = CommandParser::from_list(command_list);

    loop {
        define_system::describe_system_definitions(&system_defs);
        //describe_created_components(&system_components);
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
                    //match construct_components()
                },
                Command::EditDatabase => {
                    match edit_database::user_menu(&mut config.database) {
                        Ok(msg) => println!("{}", msg),
                        Err(err) => println!("Error when editing database: {}", err.description()),
                    }
                },
                Command::SaveSystem => {
                    match save_system(vec![], &config) {
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

fn save_system(sub_components: Vec<Box<IntoComponent>>, config: &Config) -> Result<()> {
    // Unwrap the `Box`ed sub-components and copy them into proper `Component`s for output.
    // This means that a clone is made of every `Residue` vec, which is inefficient since
    // they are later again copied in `merge_components`. I need a better grasp of using
    // `Box`es to fix this. One method would be that `merge_components` takes a vector
    // of references instead of the actual objects but this is all turning very ugly, very fast.
    //
    // TODO: Hopefully I will change the structure of how these components interlock sometime,
    // this currently feels a bit too much like forcing square pegs into round holes.
    //
    // IDEA: `merge_components` could take Vec<Box<IntoComponent>> and perform the clone directly.
    // Somewhat ugly to have several so similar functions though.
    //let finished_components = sub_components
        //.iter()
        //.map(|sub| sub.to_component())
        //.collect::<Vec<_>>();

    // TODO: Merge in output!!!
    unimplemented!();
    //let system = merge_components(&finished_components);
    //output::write_gromos(&system, &config.output_path, &config.title)
}
