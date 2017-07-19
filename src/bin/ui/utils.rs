//! Tools for the user interface.

use error::{GrafenCliError, Result, UIErrorKind};

use std::io;
use std::io::Write;
use std::result;
use std::str::FromStr;

/// Define commands with a selection string, enum and information string.
type CommandArg<T> = (&'static str, T, &'static str);

/// These are put into a list. Unfortunately it is difficult to use
/// a const array for this since we need to send a Sized type to functions.
pub type CommandList<T> = Vec<CommandArg<T>>;

/// Finally a `CommandParser` is used to parse input strings for commands.
/// Commands are any unit type which implements `Copy`.
///
/// # Examples:
/// ```
/// # use ui::utils::{CommandList, CommandParser};
/// // enum Command { First, Second }
/// let command_list: CommandList<Command> = vec![
///     ("a", Command::First, "Select option `First` by inputting "a")
///     ("bad", Command::Second, "and `Second` by inputting "bad")
/// ];
/// let commands = CommandParser::from_list(command_list);
///
/// commands.print_menu();
/// if let Some(cmd) = commands.get_selection("bad") {
///     // Do something
/// }
/// ```
pub struct CommandParser<T: Copy> {
    commands: CommandList<T>,
}

impl<T: Copy> CommandParser<T> {
    /// Create a `CommandParser` from an input `CommandList`.
    pub fn from_list(commands: CommandList<T>) -> CommandParser<T> {
        CommandParser { commands }
    }

    /// Parse a string for a command from the input list.
    pub fn get_selection<'a>(&self, input: &'a str) -> Option<T> {
        if let Some(needle) = input.trim().to_lowercase().split_whitespace().next() {
            for &(ident, cmd, _) in self.commands.iter() {
                if needle == ident {
                    return Some(cmd);
                }
            }
        }

        None
    }

    /// Parse a string for a command from the input list and return along
    /// with the remaining string ("tail").
    pub fn get_selection_and_tail<'a>(&self, input: &'a str) -> Option<(T, String)> {
        let mut iter = input.split_whitespace();

        iter.next()
            .and_then(|needle| self.get_selection(needle))
            .and_then(|cmd| {
                let tail = iter.collect::<Vec<&str>>().join(" ");
                Some((cmd, tail))
            })
    }

    /// Print a menu of the available commands.
    pub fn print_menu(&self) {
        println!("Commands:");
        for &(ref c, _, ref info) in self.commands.iter() {
            println!("{:>4}{:4}{}", c, ' ', info);
        }
        println!("");
    }
}

/// Macro for creating a `CommandParser` object.
///
/// See the tests below for an up-to-date example of use. The general idea
/// is to just do:
/// ```
/// #[derive(Clone, Copy)] enum Commands { One, Two }
/// let commands = command_parser!(
///     ("a", Commands::One, "Description"),
///     ("b", Commands::Two, "Description")
/// );
/// ```
macro_rules! command_parser {
    (
        $(($short:expr, $command:path, $description:expr)),+
    ) => {
        {
            let mut temp_command_list = Vec::new();
            $(
                temp_command_list.push(($short, $command, $description));
            )*
            CommandParser::from_list(temp_command_list)
        }
    }
}

/// Read and trim a string from stdin.
pub fn get_input_string(query: &'static str) -> Result<String> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut selection = String::new();

    print!("{}: ", query);
    stdout.flush()?;
    stdin.read_line(&mut selection)?;

    Ok(selection.trim().to_string())
}

/// Parse an input string for values. Values are whitespace separated.
/// Return an Error if any value of the string could not be parsed.
pub fn parse_string<'a, T: FromStr>(tail: &'a str) -> result::Result<Vec<T>, UIErrorKind> {
    // Note to self: This uses `FromIterator` to turn a Vec<Result> into Result<Vec>. Neat!
    tail.split_whitespace()
        .map(|s| s.parse::<T>().map_err(|_| {
            UIErrorKind::BadValue(format!("'{}' is not a valid value", s))
        }))
        .collect()
}

/// Parse an input string for one value. Return an Error if a value could not be parsed.
pub fn parse_string_single<'a, T: FromStr>(tail: &'a str) -> result::Result<T, UIErrorKind> {
    let string = tail.split_whitespace().next()
        .ok_or(UIErrorKind::BadValue("Could not parse a value".to_string()))?;

    string.parse::<T>().map_err(|_| {
            UIErrorKind::BadValue(format!("'{}' is not a valid value", string))
        })
}

/// Parse an input string for an index and ensure that it exists in the input list.
/// An error is raised if it is out of bounds.
pub fn parse_string_for_index<T>(input: &str, list: &Vec<T>) -> Result<usize> {
    parse_string_single::<usize>(input)
        .and_then(|i| {
            if i < list.len() {
                Ok(i)
            } else {
                Err(UIErrorKind::BadValue(format!("No item with index {} exists", i)))
            }
        })
        .map_err(|err| GrafenCliError::from(err))
}

/// Remove an item from a list. The index is parsed from the input tail and returned as a result.
pub fn remove_item<'a, T>(item_list: &mut Vec<T>, tail: &'a str) -> Result<usize> {
    let index = parse_string_for_index(&tail, &item_list)?;
    item_list.remove(index);

    Ok(index)
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
        let commands = CommandParser::from_list(command_list);

        assert_eq!(Some(TestCommands::One), commands.get_selection("a"));
        assert_eq!(Some(TestCommands::Two), commands.get_selection("b"));
        assert_eq!(None, commands.get_selection("nope"));
    }

    #[test]
    fn select_a_command_trims_and_lowercases() {
        let command_list = vec![
            ("a", TestCommands::One, "one"),
            ("b", TestCommands::Two, "two")
        ];
        let commands = CommandParser::from_list(command_list);

        assert_eq!(Some(TestCommands::One), commands.get_selection("\n\ta\n"));
        assert_eq!(Some(TestCommands::Two), commands.get_selection("  B  "));
    }

    #[test]
    fn select_a_command_checks_first_word() {
        let command_list = vec![
            ("a", TestCommands::One, "one"),
            ("b", TestCommands::Two, "two")
        ];
        let commands = CommandParser::from_list(command_list);

        assert_eq!(Some(TestCommands::One), commands.get_selection("a ignore"));
        assert_eq!(None, commands.get_selection("aignore"));
    }

    #[test]
    fn select_a_command_differentiates_between_similar() {
        let command_list = vec![
            ("at", TestCommands::One, "one"),
            ("ar", TestCommands::Two, "two")
        ];
        let commands = CommandParser::from_list(command_list);

        assert_eq!(None, commands.get_selection("a"));
        assert_eq!(Some(TestCommands::One), commands.get_selection("at"));
        assert_eq!(Some(TestCommands::Two), commands.get_selection("ar"));
    }

    #[test]
    fn select_a_command_and_tail() {
        let command_list = vec![
            ("a", TestCommands::One, "one"),
            ("b", TestCommands::Two, "two")
        ];
        let commands = CommandParser::from_list(command_list);

        let (cmd, tail) = commands.get_selection_and_tail("a and a tail").unwrap();
        assert_eq!(TestCommands::One, cmd);
        assert_eq!("and a tail", &tail);

        let (cmd, tail) = commands.get_selection_and_tail("b\twith trimming\n").unwrap();
        assert_eq!(TestCommands::Two, cmd);
        assert_eq!("with trimming", &tail);

        assert_eq!(None, commands.get_selection_and_tail(""));
        assert_eq!(None, commands.get_selection_and_tail("tail"));
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
    fn parse_string_for_one_value() {
        assert_eq!(0, parse_string_single::<usize>("0").unwrap());
        assert_eq!(0, parse_string_single::<usize>("\t0\n").unwrap());
        assert_eq!(0, parse_string_single::<usize>("0 1").unwrap());
        assert!(parse_string_single::<usize>("a").is_err());
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
        assert!(err.contains("'a' is not a valid value"));
        let err = swap_items(&mut vec, "-1 0").unwrap_err().description().to_string();
        assert!(err.contains("'-1' is not a valid value"));

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
        assert!(err.contains("No item with index 2 exists"));
        let err = remove_item(&mut vec, "-1").unwrap_err().description().to_string();
        assert!(err.contains("'-1' is not a valid value"));
    }

    #[test]
    fn parse_and_remove_vector_without_input_is_error() {
        let mut vec = vec![1];
        assert!(remove_item(&mut vec, "\t").is_err());
    }

    #[test]
    fn parse_index_inside_vector() {
        assert_eq!(2, parse_string_for_index("2", &vec![0, 1, 2]).unwrap());
        assert_eq!(0, parse_string_for_index("0", &vec![2, 1, 0]).unwrap());
        assert!(parse_string_for_index("1", &vec![0]).is_err());
        assert!(parse_string_for_index("a", &vec![0]).is_err());
        assert!(parse_string_for_index("\n", &vec![0]).is_err());
    }

    #[test]
    fn command_list_macro_is_consistent() {
        #[derive(Clone, Copy, Debug, PartialEq)]
        enum TestCommand { One, Two }

        let commands_macro = command_parser!(
            ("a", TestCommand::One, "First command"),
            ("b", TestCommand::Two, "Second command")
        );

        let command_list_full: CommandList<TestCommand> = vec![
            ("a", TestCommand::One, "First command"),
            ("b", TestCommand::Two, "Second command")
        ];
        let commands_full = CommandParser::from_list(command_list_full);

        assert_eq!(2, commands_macro.commands.len());

        let mut iter_macro = commands_macro.commands.iter();
        let mut iter_full = commands_full.commands.iter();
        for (&cmd_macro, &cmd_full) in iter_macro.zip(iter_full) {
            assert_eq!(cmd_macro, cmd_full);
        }
    }
}
