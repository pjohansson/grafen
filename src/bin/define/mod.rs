mod systemdefinition;

use database::{write_database, DataBase, SubstrateConfEntry};
use error::{GrafenCliError, Result, UIErrorKind};

use grafen::substrate::SubstrateConf;
use grafen::system::{Coord, System};
use std::error::Error;
use std::io;
use std::io::Write;
use std::result;

#[derive(Debug, PartialEq)]
pub struct SystemDefinition {
    pub config: SubstrateConfEntry,
    pub position: Coord,
    pub size: (f64, f64),
    pub finalized: SubstrateConf,
}

/// Loop over a menu in which the user can define the system which will be created, etc.
pub fn user_menu(mut database: &mut DataBase) -> Result<Vec<SystemDefinition>> {
    let mut system_defs: Vec<SystemDefinition> = Vec::new();

    loop {
        print_menu(&system_defs)?;
        let input = get_input("Selection")?;

        match parse_selection_and_tail(&input) {
            Ok(('d', _)) => {
                match systemdefinition::user_menu(&database) {
                    Ok(def) => system_defs.push(def),
                    Err(err) => println!("Could not create definition: {}", err.description()),
                }
            },
            Ok(('r', tail)) => {
                match remove_system(&mut system_defs, &tail) {
                    Ok(i) => println!("Removed system at index {}.", i),
                    Err(err) => println!("Could not remove system: {}", err.description()),
                }
            },
            Ok(('s', tail)) => {
                match swap_systems(&mut system_defs, &tail) {
                    Ok((i, j)) => println!("Swapped system at index {} with system at {}.", i, j),
                    Err(err) => println!("Could not swap systems: {}", err.description()),
                }
            },
            Ok(('w', _)) => {
                match write_database(&database) {
                    Ok(_) => println!("Saved database to disk."),
                    Err(err) => println!("Error: {}", err.description()),
                }
            },
            Ok(('f', _)) => {
                if system_defs.len() > 0 {
                    return Ok(system_defs);
                } else {
                    println!("No systems are defined, cannot finalize.");
                }
            },
            Ok(('a', _)) => {
                return Err(GrafenCliError::QuitWithoutSaving);
            },
            _ => {
                println!("Not a valid selection.");
            },
        }

        println!("");
    }
}

/// Print the user menu for system construction.
fn print_menu(system_defs: &[SystemDefinition]) -> Result<()> {
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
    println!("[D]efine system      [R]emove system                    [S]wap systems");
    println!("[W]rite database     [F]inish and construct systems     [A]bort and exit");
    println!();

    Ok(())
}

/// Parse a string for the character selection and an optional tail.
fn parse_selection_and_tail<'a>(input: &'a str) -> Result<(char, String)> {
    let lowercase_input = input.to_lowercase();
    let mut iter = lowercase_input.trim().splitn(2, ' ');

    let selection = iter.next().and_then(|s| s.chars().next()).ok_or(UIErrorKind::NoSelection)?;
    let tail = iter.next().map(|s| s.to_string()).unwrap_or(String::new());

    Ok((selection, tail))
}

/// Parse an input tail for values. The input tail is a string with white space
/// separated values. Return an Error if any value of the string could not be parsed.
fn parse_tail<'a>(tail: &'a str) -> result::Result<Vec<usize>, UIErrorKind> {
    // Note to self: This uses FromIterator to turn a Vec<Result> into Result<Vec>! Neat!
    tail.split_whitespace()
        .map(|s| s.parse::<usize>().map_err(|_| {
            UIErrorKind::BadNumber(format!("'{}' is not a valid index", s))
        }))
        .collect()
}

/// Read and trim a string from stdin..
pub fn get_input(query: &'static str) -> Result<String> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut selection = String::new();

    print!("{}: ", query);
    stdout.flush()?;
    stdin.read_line(&mut selection)?;

    Ok(selection.trim().to_string())
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
pub mod tests {
    use super::*;

    #[test]
    fn parse_tail_for_usize() {
        assert_eq!(vec![3, 2, 1, 100], parse_tail("3 2 1 100").unwrap());
        assert_eq!(vec![3, 2, 1, 0], parse_tail("3\t2\t1 0").unwrap());
        assert!(parse_tail("").unwrap().is_empty());
    }

    #[test]
    fn parse_tail_for_usize_bad_numbers() {
        assert!(parse_tail("3a").is_err());
        assert!(parse_tail("3 1 2 a 3").is_err());
    }

    #[test]
    fn parse_character_selection() {
        assert_eq!(('a', String::new()), parse_selection_and_tail("a\n").unwrap());
        assert_eq!(('a', String::new()), parse_selection_and_tail("a").unwrap());
        assert_eq!(('a', String::new()), parse_selection_and_tail("\t   a").unwrap());
        assert_eq!(('a', String::new()), parse_selection_and_tail("ABORT").unwrap());
        assert_eq!(('a', String::new()), parse_selection_and_tail("\n\nABORT").unwrap());
    }

    #[test]
    fn parse_character_empty_string() {
        assert!(parse_selection_and_tail("").is_err());
        assert!(parse_selection_and_tail(" ").is_err());
        assert!(parse_selection_and_tail("\n").is_err());
    }

    #[test]
    fn parse_character_selection_and_option() {
        assert_eq!(('a', "option".to_string()), parse_selection_and_tail("a option\n").unwrap());
        assert_eq!(('a', "first second".to_string()),
                   parse_selection_and_tail("abort first second\n").unwrap());
    }

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
