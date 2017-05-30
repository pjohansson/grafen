mod edit_database;
mod systemdefinition;
mod tools;

use database::{write_database, DataBase, SubstrateConfEntry};
use error::{GrafenCliError, Result, UIErrorKind};
use ui::tools::{get_input, get_selection_and_tail, parse_tail, print_menu, CommandList};

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

    let commands: CommandList<'static, Command> = vec![
        ("d", Command::DefineSystem, "Define a system to create"),
        ("r", Command::RemoveSystem, "Remove a system from the list"),
        ("s", Command::SwapSystems, "Swap the order of two systems"),
        ("e", Command::EditDatabase, "Edit the substrate and residue database"),
        ("f", Command::QuitAndConstruct, "Finalize and construct systems from list"),
        ("a", Command::QuitWithoutSaving, "Abort and exit without saving")
    ];

    loop {
        describe_system_definitions(&system_defs);
        print_menu(&commands);
        let input = get_input("Selection")?;
        println!("");

        if let Some((cmd, tail)) = get_selection_and_tail(&input, &commands) {
            match cmd {
                Command::DefineSystem => {
                    match systemdefinition::user_menu(&database) {
                        Ok(def) => system_defs.push(def),
                        Err(err) => println!("Could not create definition: {}", err.description()),
                    }
                },
                Command::RemoveSystem => {
                    match remove_system(&mut system_defs, &tail) {
                        Ok(i) => println!("Removed system at index {}.", i),
                        Err(err) => println!("Could not remove system: {}", err.description()),
                    }
                },
                Command::SwapSystems => {
                    match swap_systems(&mut system_defs, &tail) {
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
                    if system_defs.len() > 0 {
                        return Ok(system_defs);
                    } else {
                        println!("No systems are defined, cannot finalize.");
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

/// Remove a system. The index is parsed from the input tail and returned as a result.
fn remove_system<'a, T>(system_defs: &mut Vec<T>, tail: &'a str) -> Result<usize> {
    let parsed = parse_tail(tail)?;

    match parsed.get(0) {
        Some(&i) if i < system_defs.len() => {
            system_defs.remove(i);
            Ok(i)
        },
        Some(&i) => {
            Err(GrafenCliError::UIError(format!("No system with index {} exists", i)))
        },
        None => {
            Err(GrafenCliError::UIError("An index to remove is required".to_string()))
        },
    }
}

/// Swap two systems in-place. The indices are parsed from the input tail
/// and returned as the result.
fn swap_systems<'a, T>(system_defs: &mut Vec<T>, tail: &'a str) -> Result<(usize, usize)> {
    let parsed = parse_tail(tail)?;
    let max_len = system_defs.len();

    match (parsed.get(0), parsed.get(1)) {
        // Assert that neither index is out of bounds with this pattern guard.
        (Some(&i), _) | (_, Some(&i)) if i >= max_len => {
            Err(GrafenCliError::UIError(format!("No system with index {} exists", i)))
        },

        (Some(&i), Some(&j)) => {
            system_defs.swap(i, j);
            Ok((i, j))
        },

        _ => {
            Err(GrafenCliError::UIError("Two indices to swap order for are required".to_string()))
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn swap_parsed_systems() {
        let mut vec = vec![0, 1, 2, 3];
        assert_eq!((1, 2), swap_systems(&mut vec, "1 2").unwrap());
        assert_eq!(vec![0, 2, 1, 3], vec);
        assert_eq!((3, 0), swap_systems(&mut vec, "3 0").unwrap());
        assert_eq!(vec![3, 2, 1, 0], vec);
        assert_eq!((1, 1), swap_systems(&mut vec, "1 1").unwrap());
        assert_eq!(vec![3, 2, 1, 0], vec);
        assert_eq!((1, 2), swap_systems(&mut vec, "1 2 5").unwrap()); // The extra digit is discarded
        assert_eq!(vec![3, 1, 2, 0], vec);
    }

    #[test]
    fn swap_parsed_systems_numbers_out_of_bounds() {
        let mut vec = vec![0, 1];
        assert!(swap_systems(&mut vec, "0 2").is_err());
        assert!(swap_systems(&mut vec, "2 1").is_err());
        assert!(swap_systems(&mut vec, "-1 1").is_err());
        assert!(swap_systems(&mut vec, "a 1").is_err());
        assert!(swap_systems(&mut vec, "0 a").is_err());
    }

    #[test]
    fn swap_parsed_systems_error_messages() {
        let mut vec = vec![0, 1, 2];

        // Invalid characters
        let err = swap_systems(&mut vec, "a 0").unwrap_err().description().to_string();
        assert!(err.contains("'a' is not a valid index"));
        let err = swap_systems(&mut vec, "-1 0").unwrap_err().description().to_string();
        assert!(err.contains("'-1' is not a valid index"));

        // Too few indices
        let err = swap_systems(&mut vec, "0").unwrap_err().description().to_string();
        assert!(err.contains("Two indices to swap order for are required"));

        // First index is out of bounds
        let err = swap_systems(&mut vec, "3 0").unwrap_err().description().to_string();
        assert!(err.contains("No system with index 3 exists"));

        // Second index
        let err = swap_systems(&mut vec, "0 4").unwrap_err().description().to_string();
        assert!(err.contains("No system with index 4 exists"));

    }

    #[test]
    fn remove_a_parsed_system() {
        let mut vec = vec![1, 2, 3, 4];
        assert_eq!(1, remove_system(&mut vec, "1").unwrap());
        assert_eq!(vec![1, 3, 4], vec);
        assert_eq!(2, remove_system(&mut vec, "2").unwrap());
        assert_eq!(vec![1, 3], vec);
        assert_eq!(0, remove_system(&mut vec, "0").unwrap());
        assert_eq!(vec![3], vec);
        assert_eq!(0, remove_system(&mut vec, "0").unwrap());
        assert!(vec.is_empty());
    }

    #[test]
    fn remove_a_parsed_system_out_of_bounds() {
        let mut vec = vec![0, 1];
        let err = remove_system(&mut vec, "2").unwrap_err().description().to_string();
        assert!(err.contains("No system with index 2 exists"));
        let err = remove_system(&mut vec, "-1").unwrap_err().description().to_string();
        assert!(err.contains("'-1' is not a valid index"));
    }

    #[test]
    fn remove_a_parsed_system_empty_string() {
        let mut vec = vec![1];
        let err = remove_system(&mut vec, "\t").unwrap_err().description().to_string();
        assert!(err.contains("An index to remove is required"))
    }
}
