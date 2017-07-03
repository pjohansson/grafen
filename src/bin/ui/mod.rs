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

use database::{DataBase, SheetConfEntry};
use error::{GrafenCliError, Result};
use ui::utils::{CommandList, CommandParser};

use grafen::substrate::SheetConf;
use grafen::system::Coord;
use std::error::Error;

#[derive(Debug, PartialEq)]
/// One system is defined by these attributes.
pub struct SystemDefinition {
    pub config: SheetConfEntry,
    pub position: Coord,
    pub size: (f64, f64),
    pub finalized: SheetConf,
}

#[derive(Clone, Copy, Debug)]
/// User commands for defining the system.
enum Command {
    DefineSystem,
    RemoveSystem,
    SwapSystems,
    EditDatabase,
    QuitAndConstruct,
    QuitWithoutSaving,
}

/// Loop over a menu in which the user can define the system which will be created, etc.
pub fn user_menu(mut database: &mut DataBase) -> Result<Vec<SystemDefinition>> {
    let mut system_defs: Vec<SystemDefinition> = Vec::new();

    let command_list: CommandList<Command> = vec![
        ("d", Command::DefineSystem, "Define a system to create"),
        ("r", Command::RemoveSystem, "Remove a system from the list"),
        ("s", Command::SwapSystems, "Swap the order of two systems"),
        ("e", Command::EditDatabase, "Edit the substrate and residue database"),
        ("f", Command::QuitAndConstruct, "Finalize and construct systems from list"),
        ("a", Command::QuitWithoutSaving, "Abort and exit without saving")
    ];
    let commands = CommandParser::from_list(command_list);

    loop {
        describe_system_definitions(&system_defs);
        commands.print_menu();
        let input = utils::get_input_string("Selection")?;
        println!("");

        if let Some((cmd, tail)) = commands.get_selection_and_tail(&input) {
            match cmd {
                Command::DefineSystem => {
                    match define_system::user_menu(&database) {
                        Ok(def) => system_defs.push(def),
                        Err(err) => println!("Could not create definition: {}", err.description()),
                    }
                },
                Command::RemoveSystem => {
                    match utils::remove_item(&mut system_defs, &tail) {
                        Ok(i) => println!("Removed system at index {}.", i),
                        Err(err) => println!("Could not remove system: {}", err.description()),
                    }
                },
                Command::SwapSystems => {
                    match utils::swap_items(&mut system_defs, &tail) {
                        Ok((i, j)) => println!("Swapped system at index {} with system at {}.",
                                               i, j),
                        Err(err) => println!("Could not swap systems: {}", err.description()),
                    }
                },
                Command::EditDatabase => {
                    match edit_database::user_menu(&mut database) {
                        Ok(msg) => println!("{}", msg),
                        Err(err) => println!("Error when editing database: {}", err.description()),
                    }
                },
                Command::QuitAndConstruct => {
                    if system_defs.is_empty() {
                        println!("No systems are defined, cannot finalize.");
                    } else {
                        return Ok(system_defs);
                    }
                },
                Command::QuitWithoutSaving => {
                    return Err(GrafenCliError::QuitWithoutSaving);
                },
            }
        } else {
            println!("Not a valid selection.");
        }

        println!("");
    }
}

/// Print the current system definitions to stdout.
fn describe_system_definitions(system_defs: &[SystemDefinition]) {
    if system_defs.is_empty() {
        println!("(No systems defined)");
    } else {
        for (i, def) in system_defs.iter().enumerate() {
            let (dx, dy) = def.size;
            let (x, y, z) = def.position.to_tuple();
            println!("{}. {} of size ({:.1}, {:.1}) at position ({:.1}, {:.1}, {:.1})",
                     i, def.config.name, dx, dy, x, y, z);
        }
    }

    println!("");
}
