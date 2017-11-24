//! Tools for the user interface.

use error::{GrafenCliError, Result, UIErrorKind, UIResult};

use grafen::coord::Coord;
use grafen::describe::{describe_list_short, describe_list, Describe};

use dialoguer::{Input, Select};
use std::str::FromStr;

pub type MenuResult = Result<Option<String>>;

/// Parse a value from the user.
pub fn get_value_from_user<T: FromStr>(description: &str) -> UIResult<T> {
    Input::new(description)
        .interact()?
        .trim()
        .parse::<T>()
        .map_err(|_| UIErrorKind::from("could not parse a value"))
}

/// Get a `Coord` either from the user or by default at (0, 0, 0)
pub fn get_position_from_user(default: Option<&str>) -> UIResult<Coord> {
    let mut input = Input::new("Position (x y z nm)");

    if let Some(string) = default {
        input.default(&string);
    }

    Coord::from_str(&input.interact()?).map_err(|err| UIErrorKind::from(&err))
}

/// Macro to create a consistent user menu.
///
/// The menu will loop until it is broken, either by returning from the function
/// or breaking it.
///
/// It takes a closure of commands to perform before every menu loop, then for each menu command:
///
/// 1. A unique identifier (enum type) for the command.
/// 2. A text description for the command which will be shown to the user.
/// 3. A closure that performs the desired actions and returns a `MenuResult`.
///
/// Note that the macro defines an enum for the menu selection, which the input command
/// identifiers are added to. This enum has the identifier `_ScopedMenu`.
///
/// # Examples
/// ```
/// # use ui::utils::MenuResult;
/// let mut string = String::new();
///
/// create_menu![
///     @pre: { println!("Current: {:?}", string); };
///     Set, "Set the string" => {
///         string = String::from("set");
///         Ok(Some("String set".to_string()))
///     },
///     Clear, "Clear the string" => {
///         string.clear();
///         Ok(Some("String cleared".to_string()))
///     },
///     Nothing, "Do nothing" => { Ok(None) }
/// ];
/// ```
macro_rules! create_menu {
    (
        @pre: $topmatter:tt;
        $( $command:ident, $text:expr  => $closure:block ),+
    ) => {
        #[derive(Clone, Copy, Debug)]
        enum _ScopedMenu { $( $command ),* }

        let (commands, item_texts) = create_menu_items![
            $( (_ScopedMenu::$command, $text) ),*
        ];

        loop {
            $topmatter
            let command = select_command(item_texts, commands)?;
            let result: MenuResult = match command { $( _ScopedMenu::$command => $closure ),* };

            match result {
                Ok(Some(msg)) => eprintln!("{}", msg),
                Ok(None) => (),
                Err(msg) => eprintln!("{}", msg),
            }

            eprintln!("");
        }
    }
}

/// Macro for constructing and returning a tuple of matching menu commands and descriptions.
///
/// They are yielded as a (&[Commands], &[Descriptions]) tuple.
macro_rules! create_menu_items {
    (
        $(($command:path, $text:expr)),+
    ) => {
        (
            &[
                $( $command ),*
            ],
            &[
                $( $text ),*
            ]
        )
    }
}

/// Use a dialogue prompt to select a command from a list.
pub fn select_command<T: Copy>(item_texts: &[&str], commands: &[T]) -> UIResult<T> {
    let index = Select::new()
        .default(0)
        .items(&item_texts[..])
        .interact()?;

    Ok(commands[index])
}

/// Promp the user to select an item from an input list. Return as a reference
/// to the object.
///
/// Optionally print a header description of the choices to standard error.
pub fn select_item<'a, T: Describe>(items: &'a [T], header: Option<&str>) -> UIResult<&'a T> {
    if let Some(h) = header {
        eprintln!("{}:", h);
    }

    select_item_index(items, 0).map(|index| &items[index])
}

/// Promp the user to select an item from an input list. Return as a mutable
/// reference to the object.
///
/// Optionally print a header description of the choices to standard error.
pub fn select_item_mut<'a, T: Describe>(items: &'a mut [T], header: Option<&str>) -> UIResult<&'a mut T> {
    if let Some(h) = header {
        eprintln!("{}:", h);
    }

    select_item_index(items, 0).map(move |index| &mut items[index])
}

/// Promp the user to select an item from an input list. The item index is returned.
///
/// Helper function for quick item selection in other functions.
pub fn select_item_index<T: Describe>(items: &[T], default: usize) -> UIResult<usize> {
    let item_texts: Vec<_> = items.iter().map(|item| item.describe_short()).collect();

    select_string(&item_texts, default)
}

/// Prompt the user to select an item from a list of input strings. The string index is returned.
///
/// Helper function for quick item selection in other functions.
fn select_string(items: &[String], default: usize) -> UIResult<usize> {
    // Add an option to return from the selection menu.
    let mut select = Select::new();
    for item in items {
        select.item(&item);
    }

    // Add an option to return without selecting an index
    // TODO: This should be optional, somehow. Could use a separate function?
    select.item("(Return)");

    let index = select.default(default).interact()?;

    if index < items.len() {
        Ok(index)
    } else {
        Err(UIErrorKind::Abort)
    }
}

/// Prompt the user to remove items from a list.
pub fn remove_items<T: Describe>(item_list: &mut Vec<T>) -> Result<()> {
    let mut last_index = 0;

    loop {
        match select_item_index(&item_list, last_index) {
            Ok(index) => {
                item_list.remove(index);
                last_index = index;
            },
            Err(UIErrorKind::Abort) => {
                return Ok(());
            },
            Err(err) => {
                return Err(GrafenCliError::from(err));
            },
        }
    }
}

/// Prompt the user to reorder a list in-place.
pub fn reorder_list<T: Describe>(item_list: &mut Vec<T>) -> Result<()> {
    let mut last_index = 0;

    loop {
        let mut item_texts: Vec<_> = item_list.iter().map(|item| item.describe_short()).collect();

        match select_string(&item_texts, last_index) {
            Ok(i) => {
                item_texts[i].insert_str(0, " *");

                match select_string(&item_texts, i) {
                    Ok(j) => {
                        item_list.swap(i, j);
                        last_index = j;
                    },
                    Err(UIErrorKind::Abort) => {
                        last_index = 0;
                    },
                    Err(err) => {
                        return Err(GrafenCliError::from(err));
                    },
                }
            },
            Err(UIErrorKind::Abort) => {
                return Ok(());
            },
            Err(err) => {
                return Err(GrafenCliError::from(err));
            },
        }
    }
}

/// Print a description of an object using its `describe` method to standard error.
pub fn print_description<T: Describe>(object: &T) {
    eprintln!("{}", object.describe());
}

/// Print a description of an input list using their `describe_short` method to standard error.
pub fn print_list_description_short<T: Describe>(description: &str, list: &[T]) {
    eprintln!("{}", describe_list_short(description, list));
}

/// Print a description of an input list using their `describe` method to standard error.
pub fn print_list_description<T: Describe>(description: &str, list: &[T]) {
    eprintln!("{}", describe_list(description, list));
}
