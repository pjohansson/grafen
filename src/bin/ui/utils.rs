//! Tools for the user interface.

use error::{GrafenCliError, Result, UIErrorKind, UIResult};

use grafen::coord::Coord;
use grafen::describe::Describe;

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

/// Promp the user to select an item from an input list. The item index is returned.
///
/// Since the items have to be able to be displayed, they need to implement the `Describe`
/// trait. This is meant for the user, rather than the typical `Display`.
pub fn select_item<T: Describe>(items: &[T], default: usize) -> UIResult<usize> {
    let item_texts: Vec<_> = items.iter().map(|item| item.describe()).collect();

    select_string(&item_texts, default)
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

/// Get a position either from the user. Optionally with a default.
pub fn get_position_from_user(default: Option<&str>) -> UIResult<Coord> {
    get_coord_from_user("Position", default)
}

/// Get a `Coord` either from the user. Optionally with a default.
pub fn get_coord_from_user(description: &str, default: Option<&str>) -> UIResult<Coord> {
    let mut input = Input::new(&format!("{} (x y z nm)", description));

    if let Some(string) = default {
        input.default(&string);
    }

    Coord::from_str(&input.interact()?).map_err(|err| UIErrorKind::from(&err))
}
