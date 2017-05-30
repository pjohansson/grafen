//! Tools for the user interface.

use error::UIErrorKind;

use error::Result;
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
/// # use define::uitools::CommandList;
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

/// Parse an input tail for values. The input tail is a string with white space
/// separated values. Return an Error if any value of the string could not be parsed.
pub fn parse_tail<'a, T: FromStr>(tail: &'a str) -> result::Result<Vec<T>, UIErrorKind> {
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

#[cfg(test)]
mod tests {
    use super::*;

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
    fn parse_tail_for_usize() {
        assert_eq!(vec![3, 2, 1, 100], parse_tail::<usize>("3 2 1 100").unwrap());
        assert_eq!(vec![3, 2, 1, 0], parse_tail::<usize>("3\t2\t1 0").unwrap());
        assert!(parse_tail::<usize>("").unwrap().is_empty());
    }

    #[test]
    fn parse_tail_for_usize_bad_numbers() {
        assert!(parse_tail::<usize>("3a").is_err());
        assert!(parse_tail::<usize>("3 1 2 a 3").is_err());
    }
}
