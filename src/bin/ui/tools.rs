//! Tools for the user interface.

use error::{GrafenCliError, Result, UIErrorKind};

use std::io;
use std::io::Write;
use std::result;
use std::str::FromStr;

/// Define commands with a selection string, enum and information string.
type CommandArg<'a, T> = (&'a str, T, &'a str);

/// These are put into a list. Unfortunately it is difficult to use
/// a const array for this since we need to send a Sized type to functions.
///
/// # Examples:
/// ```
/// # use ui::tools::CommandList;
/// // enum Command { First, Second }
/// let commands: CommandList<'static, Command> = vec![
///     ("a", Command::First, "Select option `First` by inputting "a")
///     ("bad", Command::Second, "and `Second` by inputting "bad")
/// ];
///
/// print_menu(&commands);
/// if let Some(cmd) = get_selection("bad", &commands) {
///     // Do something
/// }
/// ```
pub type CommandList<'a, T> = Vec<CommandArg<'a, T>>;

/// Read and trim a string from stdin.
pub fn get_input(query: &'static str) -> Result<String> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut selection = String::new();

    print!("{}: ", query);
    stdout.flush()?;
    stdin.read_line(&mut selection)?;

    Ok(selection.trim().to_string())
}

/// Parse a string for a command from the input list.
pub fn get_selection<'a, 'b, T: Copy>(input: &'a str, commands: &CommandList<'b, T>) -> Option<T> {
    if let Some(needle) = input.trim().to_lowercase().split_whitespace().next() {
        for &(ident, cmd, _) in commands.iter() {
            if needle == ident {
                return Some(cmd);
            }
        }
    }

    None
}

/// Parse a string for a command from the input list and return along with the remaining string.
pub fn get_selection_and_tail<'a, 'b, T: Copy>(input: &'a str, commands: &CommandList<'b, T>)
        -> Option<(T, String)> {
    let mut iter = input.split_whitespace();

    iter.next()
        .and_then(|needle| get_selection(needle, &commands))
        .and_then(|cmd| {
            let tail = iter.collect::<Vec<&str>>().join(" ");
            Some((cmd, tail))
        })
}

/// Parse an input string for values. Values are whitespace separated.
/// Return an Error if any value of the string could not be parsed.
pub fn parse_string<'a, T: FromStr>(tail: &'a str) -> result::Result<Vec<T>, UIErrorKind> {
    // Note to self: This uses `FromIterator` to turn a Vec<Result> into Result<Vec>. Neat!
    tail.split_whitespace()
        .map(|s| s.parse::<T>().map_err(|_| {
            UIErrorKind::BadNumber(format!("'{}' is not a valid index", s))
        }))
        .collect()
}

/// Print a menu for the input commands.
pub fn print_menu<'a, T>(commands: &CommandList<'a, T>) {
    println!("Commands:");
    for &(ref c, _, ref info) in commands.iter() {
        println!("{:>4}{:4}{}", c, ' ', info);
    }
    println!("");
}

/// Remove an item from a list. The index is parsed from the input tail and returned as a result.
pub fn remove_item<'a, T>(item_list: &mut Vec<T>, tail: &'a str) -> Result<usize> {
    let parsed = parse_string(tail)?;

    match parsed.get(0) {
        Some(&i) if i < item_list.len() => {
            item_list.remove(i);
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

/// Swap two items of a list in-place. The indices are parsed from the input string
/// and returned as the result.
pub fn swap_items<'a, T>(item_list: &mut Vec<T>, tail: &'a str) -> Result<(usize, usize)> {
    let parsed = parse_string(tail)?;
    let max_len = item_list.len();

    match (parsed.get(0), parsed.get(1)) {
        // Assert that neither index is out of bounds with this pattern guard.
        (Some(&i), _) | (_, Some(&i)) if i >= max_len => {
            Err(GrafenCliError::UIError(format!("No system with index {} exists", i)))
        },

        (Some(&i), Some(&j)) => {
            item_list.swap(i, j);
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
    use std::error::Error;

    #[derive(Clone, Copy, Debug, PartialEq)]
    enum TestCommands {
        One,
        Two,
    }

    #[test]
    fn select_a_command() {
        let command_list = vec![
            ("a", TestCommands::One, "one"),
            ("b", TestCommands::Two, "two")
        ];

        assert_eq!(Some(TestCommands::One), get_selection("a", &command_list));
        assert_eq!(Some(TestCommands::Two), get_selection("b", &command_list));
        assert_eq!(None, get_selection("nope", &command_list));
    }

    #[test]
    fn select_a_command_trims_and_lowercases() {
        let command_list = vec![
            ("a", TestCommands::One, "one"),
            ("b", TestCommands::Two, "two")
        ];

        assert_eq!(Some(TestCommands::One), get_selection("\n\ta\n", &command_list));
        assert_eq!(Some(TestCommands::Two), get_selection("  B  ", &command_list));
    }

    #[test]
    fn select_a_command_checks_first_word() {
        let command_list = vec![
            ("a", TestCommands::One, "one"),
            ("b", TestCommands::Two, "two")
        ];

        assert_eq!(Some(TestCommands::One), get_selection("a ignore", &command_list));
        assert_eq!(None, get_selection("aignore", &command_list));
    }

    #[test]
    fn select_a_command_differentiates_between_similar() {
        let command_list = vec![
            ("at", TestCommands::One, "one"),
            ("ar", TestCommands::Two, "two")
        ];

        assert_eq!(None, get_selection("a", &command_list));
        assert_eq!(Some(TestCommands::One), get_selection("at", &command_list));
        assert_eq!(Some(TestCommands::Two), get_selection("ar", &command_list));
    }

    #[test]
    fn select_a_command_and_tail() {
        let command_list = vec![
            ("a", TestCommands::One, "one"),
            ("b", TestCommands::Two, "two")
        ];

        let (cmd, tail) = get_selection_and_tail("a and a tail", &command_list).unwrap();
        assert_eq!(TestCommands::One, cmd);
        assert_eq!("and a tail", &tail);

        let (cmd, tail) = get_selection_and_tail("b\twith trimming\n", &command_list).unwrap();
        assert_eq!(TestCommands::Two, cmd);
        assert_eq!("with trimming", &tail);

        assert_eq!(None, get_selection_and_tail("", &command_list));
        assert_eq!(None, get_selection_and_tail("tail", &command_list));
    }

    #[test]
    fn parse_string_for_usize() {
        assert_eq!(vec![3, 2, 1, 100], parse_string::<usize>("3 2 1 100").unwrap());
        assert_eq!(vec![3, 2, 1, 0], parse_string::<usize>("3\t2\t1 0").unwrap());
        assert!(parse_string::<usize>("").unwrap().is_empty());
    }

    #[test]
    fn parse_string_for_usize_bad_numbers() {
        assert!(parse_string::<usize>("3a").is_err());
        assert!(parse_string::<usize>("3 1 2 a 3").is_err());
    }

    #[test]
    fn parse_and_swap_vector_order() {
        let mut vec = vec![0, 1, 2, 3];
        assert_eq!((1, 2), swap_items(&mut vec, "1 2").unwrap());
        assert_eq!(vec![0, 2, 1, 3], vec);
        assert_eq!((3, 0), swap_items(&mut vec, "3 0").unwrap());
        assert_eq!(vec![3, 2, 1, 0], vec);
        assert_eq!((1, 1), swap_items(&mut vec, "1 1").unwrap());
        assert_eq!(vec![3, 2, 1, 0], vec);
        assert_eq!((1, 2), swap_items(&mut vec, "1 2 5").unwrap()); // The extra digit is discarded
        assert_eq!(vec![3, 1, 2, 0], vec);
    }

    #[test]
    fn parse_and_swap_out_of_bounds_is_error() {
        let mut vec = vec![0, 1];
        assert!(swap_items(&mut vec, "0 2").is_err());
        assert!(swap_items(&mut vec, "2 1").is_err());
        assert!(swap_items(&mut vec, "-1 1").is_err());
        assert!(swap_items(&mut vec, "a 1").is_err());
        assert!(swap_items(&mut vec, "0 a").is_err());
    }

    #[test]
    fn parse_and_swap_error_messages_are_correct() {
        let mut vec = vec![0, 1, 2];

        // Invalid characters
        let err = swap_items(&mut vec, "a 0").unwrap_err().description().to_string();
        assert!(err.contains("'a' is not a valid index"));
        let err = swap_items(&mut vec, "-1 0").unwrap_err().description().to_string();
        assert!(err.contains("'-1' is not a valid index"));

        // Too few indices
        let err = swap_items(&mut vec, "0").unwrap_err().description().to_string();
        assert!(err.contains("Two indices to swap order for are required"));

        // First index is out of bounds
        let err = swap_items(&mut vec, "3 0").unwrap_err().description().to_string();
        assert!(err.contains("No system with index 3 exists"));

        // Second index
        let err = swap_items(&mut vec, "0 4").unwrap_err().description().to_string();
        assert!(err.contains("No system with index 4 exists"));

    }

    #[test]
    fn parse_and_remove_vector_item() {
        let mut vec = vec![1, 2, 3, 4];
        assert_eq!(1, remove_item(&mut vec, "1").unwrap());
        assert_eq!(vec![1, 3, 4], vec);
        assert_eq!(2, remove_item(&mut vec, "2").unwrap());
        assert_eq!(vec![1, 3], vec);
        assert_eq!(0, remove_item(&mut vec, "0").unwrap());
        assert_eq!(vec![3], vec);
        assert_eq!(0, remove_item(&mut vec, "0").unwrap());
        assert!(vec.is_empty());
    }

    #[test]
    fn parse_and_remove_vector_out_of_bounds_is_error() {
        let mut vec = vec![0, 1];
        let err = remove_item(&mut vec, "2").unwrap_err().description().to_string();
        assert!(err.contains("No system with index 2 exists"));
        let err = remove_item(&mut vec, "-1").unwrap_err().description().to_string();
        assert!(err.contains("'-1' is not a valid index"));
    }

    #[test]
    fn parse_and_remove_vector_without_input_is_error() {
        let mut vec = vec![1];
        let err = remove_item(&mut vec, "\t").unwrap_err().description().to_string();
        assert!(err.contains("An index to remove is required"))
    }
}