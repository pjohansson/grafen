//! Describe objects in an informative way.

use std::fmt::Write;

/// A verbose description of an object. Moreso than `Display` should typically be.
/// Maybe the distinction is unnecessary? If so, change it sometime.
pub trait Describe {
    /// Return a descriptive `String` of the object. 
    fn describe(&self) -> String;

    /// Return a very short descriptive `String` of the object. Typically just a name or type.
    fn describe_short(&self) -> String;
}

/// Describe a list of items using their `describe_short` method.
pub fn describe_list_short<T: Describe>(header: &str, items: &[T]) -> String {
    let mut description = String::new();

    writeln!(description, "[ {} ]", header).expect("Could not construct a description string");

    for (i, item) in items.iter().enumerate() {
        writeln!(description, "{:2}: {}", i, item.describe_short())
            .expect("Could not construct a description string");
    }

    description
}

/// Describe a list of items using their `describe` method.
pub fn describe_list<T: Describe>(header: &str, items: &[T]) -> String {
    let mut description = String::new();

    writeln!(description, "[ {} ]", header).expect("Could not construct a description string");

    for (i, item) in items.iter().enumerate() {
        writeln!(description, "{:2}: {}", i, item.describe())
            .expect("Could not construct a description string");
    }

    description
}

/// Unwrap an optional name of an object with a default None value.
/// 
/// # Examples
/// ```
/// # use grafen::describe::unwrap_name;
/// assert_eq!("Some name", &unwrap_name(&Some("Some name".to_string())));
/// assert!(!unwrap_name(&None).is_empty());
/// ```
pub fn unwrap_name(name: &Option<String>) -> String {
    name.clone().unwrap_or("(No name)".to_string())
}
