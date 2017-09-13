//! Describe objects in an informative way.

/// A verbose description of an object. Moreso than `Display` should typically be.
/// Maybe the distinction is unnecessary? If so, change it sometime.
///
/// TODO: Consider adding a `describe_long` or similar method.
pub trait Describe {
    fn describe(&self) -> String;
}

/// Print a group's elements and a header.
/// TODO: This is now temporary a duplicate of `utils::print_group`. Fix this.
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
