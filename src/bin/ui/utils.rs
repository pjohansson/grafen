//! Tools for the user interface.

use error::{GrafenCliError, Result, UIErrorKind, UIResult};

use grafen::system::Coord;
use dialoguer::{Input, Select};
use std::str::FromStr;

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

/// Use a dialogue prompt to select an from a list of corresponding objects..
pub fn select_command<T: Copy>(item_texts: &[&str], commands: &[T]) -> UIResult<T> {
    let index = Select::new()
        .default(0)
        .items(&item_texts[..])
        .interact()?;

    Ok(commands[index])
}

/// Prompt the user to select an item from a list of input strings. The string index is returned.
fn select_string(items: &[String], default: usize) -> UIResult<usize> {
    // Add an option to return from the selection menu.
    let mut select = Select::new();
    for item in items {
        select.item(&item);
    }

    let index = select.default(default).interact()?;

    // Return is at the last index, thus we check is our selected is before that here.
    if index < items.len() - 1 {
        Ok(index)
    } else {
        Err(UIErrorKind::Abort)
    }
}

/// Promp the user to select an item from an input list. The item index is returned.
///
/// Since the items have to be able to be displayed, they need to implement the `Describe`
/// trait. This is meant for the user, rather than the typical `Display`.
pub fn select_item<T: Describe>(items: &[T], default: usize) -> UIResult<usize> {
    let mut item_texts: Vec<_> = items.iter().map(|item| item.describe()).collect();

    // Add an option to return without selecting an index
    // TODO: This should be optional, somehow. Could use a separate function?
    item_texts.push("(Return)".to_string());

    select_string(&item_texts, default)
}

/// Get a `Coord` either from the user or by default at (0, 0, 0)
pub fn get_position_from_user(default: Option<&str>) -> UIResult<Coord> {
    let mut input = Input::new("Position (x y z)");

    if let Some(string) = default {
        input.default(&string);
    }

    Coord::from_str(&input.interact()?).map_err(|err| UIErrorKind::from(&err))
}

/// Prompt the user to remove items from a list.
pub fn remove_items<T: Describe>(item_list: &mut Vec<T>) -> Result<()> {
    loop {
        match select_item(&item_list, 0) {
            Ok(index) => {
                item_list.remove(index);
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
        let mut item_texts: Vec<String> = item_list.iter().map(|item| item.describe()).collect();

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

/// A verbose description of an object. Moreso than `Display` should typically be.
/// Maybe the distinction is unnecessary? If so, change it sometime.
///
/// TODO: Consider adding a `describe_long` or similar method.
pub trait Describe {
    fn describe(&self) -> String;
}

/// Print a group's elements and a header.
pub fn print_group<T: Describe>(title: &str, group: &[T]) {
    eprintln!("[ {} ]", title);

    if group.is_empty() {
        eprintln!("(none)");
    } else {
        for (i, element) in group.iter().enumerate() {
            eprintln!("{}. {}", i, element.describe());
        }
    }

    eprintln!("");
}
