mod edit_database;
mod systemdefinition;
mod utils;

use database::{write_database, DataBase, SubstrateConfEntry};
use error::{GrafenCliError, Result, UIErrorKind};
use ui::utils::{get_input, CommandList, CommandParser};

use grafen::substrate::SubstrateConf;
use grafen::system::{Coord, System};
use std::error::Error;
use std::io;
use std::io::Write;
use std::result;

#[derive(Debug, PartialEq)]
/// One system is defined by these attributes.
pub struct SystemDefinition {
    pub config: SubstrateConfEntry,
    pub position: Coord,
    pub size: (f64, f64),
    pub finalized: SubstrateConf,
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
        let input = get_input("Selection")?;
        println!("");

        if let Some((cmd, tail)) = commands.get_selection_and_tail(&input) {
            match cmd {
                Command::DefineSystem => {
                    match systemdefinition::user_menu(&database) {
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
                        Ok(_) => println!("Finished editing database."),
                        Err(err) => println!("Error: {}", err.description()),
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
            println!("{}. {} of size {:.1} x {:.1} nm^2 at position ({:.1}, {:.1}, {:.1})",
                     i, def.config.name, dx, dy, x, y, z);
        }
    }

    println!("");
}
