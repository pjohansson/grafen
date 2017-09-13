//! Describe objects in an informative way.

use std::fmt::Write;

/// A verbose description of an object. Moreso than `Display` should typically be.
/// Maybe the distinction is unnecessary? If so, change it sometime.
///
/// TODO: Consider adding a `describe_long` or similar method.
pub trait Describe {
    fn describe(&self) -> String;
}

/// Describe a list of items.
pub fn describe_list<T: Describe>(header: &str, items: &[T]) -> String {
    let mut description = String::new();

    write!(description, "[ {} ]\n", header).expect("Could not construct a description string");
    for (i, item) in items.iter().enumerate() {
        write!(description, "{:2}: {}\n", i, item.describe())
            .expect("Could not construct a description string");
    }

    description
}
