//! Define a system components to create.
//!
//! This interface could use a lot of improvement.

use database::{AvailableComponents, DataBase, SheetConfEntry};
use error::{GrafenCliError, Result, UIErrorKind};
use ui::utils;
use ui::utils::{CommandList, CommandParser};

use grafen::system::{Coord, Component};
use std::error::Error;

#[derive(Clone, Copy, Debug)]
/// User commands for defining the system.
enum Command {
    DefineSystem,
    RemoveSystem,
    SwapSystems,
    QuitAndSave,
    QuitWithoutSaving,
}

/// Edit the list of system definitions to construct from.
pub fn user_menu(database: &DataBase, mut system_defs: &mut Vec<AvailableComponents>)
        -> Result<()> {
    let command_list: CommandList<Command> = vec![
        ("d", Command::DefineSystem, "Define a system to create"),
        ("r", Command::RemoveSystem, "Remove a system from the list"),
        ("s", Command::SwapSystems, "Swap the order of two systems"),
        ("f", Command::QuitAndSave, "Finalize editing and return"),
        ("a", Command::QuitWithoutSaving, "Abort and discard changes to list")
    ];
    let commands = CommandParser::from_list(command_list);

    let backup = system_defs.clone();

    loop {
        describe_system_definitions(&system_defs);
        commands.print_menu();
        let input = utils::get_input_string("Selection")?;
        println!("");

        if let Some((cmd, tail)) = commands.get_selection_and_tail(&input) {
            match cmd {
                Command::DefineSystem => {
                    match create_definition(&database) {
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
                Command::QuitAndSave => {
                    return Ok(());
                },
                Command::QuitWithoutSaving => {
                    system_defs.clear();
                    system_defs.extend_from_slice(&backup);

                    return Ok(());
                },
            }
        } else {
            println!("Not a valid selection.");
        }

        println!("");
    }
}

/// Print the current system definitions to stdout.
pub fn describe_system_definitions(system_defs: &[AvailableComponents]) {
    if system_defs.is_empty() {
        println!("(No systems have been defined)");
    } else {
        println!("System definitions:");
        for (i, def) in system_defs.iter().enumerate() {
            println!("{}. {}", i, def.describe_long());
        }
    }

    println!("");
}

fn create_definition(database: &DataBase) -> Result<AvailableComponents> {
    let mut definition = select_substrate(&database).map(|def| def.clone())?;
    let position = select_position()?;
    let size = select_size()?;

    definition.position = Some(position);
    definition.size = Some(size);

    Ok(AvailableComponents::Sheet(definition))
}

fn select_substrate(database: &DataBase) -> Result<SheetConfEntry> {
    let available_substrates: Vec<SheetConfEntry> = database.component_defs
        .iter()
        .filter_map(|ref def| {
            match def {
                &&AvailableComponents::Sheet(ref conf) => Some(conf.clone()),
                _ => None,
            }
        })
        .collect();

    println!("Available substrates:");
    for (i, sub) in available_substrates.iter().enumerate() {
        println!("{}. {}", i, sub.name);
    }
    println!("");

    let selection = utils::get_input_string("Select substrate")?;
    selection
        .parse::<usize>()
        .map_err(|_| UIErrorKind::BadValue(format!("'{}' is not a valid index", &selection)))
        .and_then(|n| {
            available_substrates
                .get(n)
                .map(|def| def.clone())
                .ok_or(UIErrorKind::BadValue(format!("No substrate with index {} exists", n)))
        })
        .map_err(|err| GrafenCliError::from(err))
}

fn select_position() -> Result<Coord> {
    let selection = utils::get_input_string("Change position (default: (0.0, 0.0, 0.0))")?;
    if selection.is_empty() {
        return Ok(Coord::new(0.0, 0.0, 0.0));
    }

    let coords = utils::parse_string(&selection)?;
    let &x = coords.get(0).ok_or(UIErrorKind::BadValue("3 positions are required".to_string()))?;
    let &y = coords.get(1).ok_or(UIErrorKind::BadValue("3 positions are required".to_string()))?;
    let &z = coords.get(2).ok_or(UIErrorKind::BadValue("3 positions are required".to_string()))?;

    Ok(Coord::new(x, y, z))
}

fn select_size() -> Result<(f64, f64)> {
    let selection = utils::get_input_string("Set size")?;

    let size = utils::parse_string(&selection)?;
    let &dx = size.get(0).ok_or(UIErrorKind::BadValue("2 values are required".to_string()))?;
    let &dy = size.get(1).ok_or(UIErrorKind::BadValue("2 values are required".to_string()))?;

    Ok((dx, dy))
}
