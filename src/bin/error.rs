//! Errors when executing the binary.

use grafen::error::GrafenError;

use colored::*;
use std::error::Error;
use std::fmt;
use std::fmt::Display;
use std::io;
use std::num::{ParseFloatError, ParseIntError};
use std::result;

/// Shorthand for our `Result` class.
pub type Result<T> = result::Result<T, GrafenCliError>;

#[derive(Debug)]
/// Types of user interface errors. These are parsed into a `GrafenCliError::UIError`
/// using a `From` implementation below.
pub enum UIErrorKind {
    /// An input value which was requested could not be parsed.
    BadValue(String),
    /// User aborted a process.
    Abort,
}

pub type UIResult<T> = result::Result<T, UIErrorKind>;

impl From<io::Error> for UIErrorKind {
    fn from(_: io::Error) -> UIErrorKind {
        UIErrorKind::BadValue("could not parse a value".to_string())
    }
}

impl From<ParseFloatError> for UIErrorKind {
    fn from(_: ParseFloatError) -> UIErrorKind {
        UIErrorKind::BadValue("could not parse a value".to_string())
    }
}

impl From<ParseIntError> for UIErrorKind {
    fn from(_: ParseIntError) -> UIErrorKind {
        UIErrorKind::BadValue("could not parse a value".to_string())
    }
}

impl<'a, T: Display + ?Sized> From<&'a T> for UIErrorKind {
    fn from(err: &'a T) -> UIErrorKind {
        UIErrorKind::BadValue(err.to_string())
    }
}

#[derive(Debug)]
/// A class for configuration or runtime errors.
pub enum GrafenCliError {
    /// Something went wrong when reading or writing.
    IoError(io::Error),
    /// Something went wrong when creating the system.
    RunError(String),
    /// Something went wrong when constructing a Residue.
    ConstructError(String),
    /// User interface error.
    UIError(String),
}

impl Error for GrafenCliError {
    fn description(&self) -> &str {
        match *self {
            GrafenCliError::UIError(ref err) => err,
            _ => "Unknown error",
        }
    }

    /// I don't know how to use this yet, so all cause's are None.
    fn cause(&self) -> Option<&Error> {
        None
    }
}

impl Display for GrafenCliError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let red_error = "error:".color("red");

        match *self {
            GrafenCliError::IoError(ref err) => {
                write!(f, "{} {}", red_error, err)
            },
            GrafenCliError::RunError(ref err) => {
                write!(f, "{} {}", red_error, err)
            },
            GrafenCliError::ConstructError(ref err) => {
                write!(f, "{}", err.as_str().color("yellow"))
            },
            GrafenCliError::UIError(ref err) => {
                write!(f, "{} {}", red_error, err)
            },
        }
    }
}

impl<'a> From<&'a str> for GrafenCliError {
    fn from(err: &'a str) -> GrafenCliError {
        GrafenCliError::ConstructError(err.to_string())
    }
}

impl From<io::Error> for GrafenCliError {
    fn from(err: io::Error) -> GrafenCliError {
        GrafenCliError::IoError(err)
    }
}

impl From<GrafenError> for GrafenCliError {
    fn from(err: GrafenError) -> GrafenCliError {
        GrafenCliError::RunError(err.description().to_string())
    }
}

impl From<UIErrorKind> for GrafenCliError {
    fn from(err: UIErrorKind) -> GrafenCliError {
        match err {
            UIErrorKind::BadValue(err) => GrafenCliError::UIError(err),
            UIErrorKind::Abort => GrafenCliError::UIError("Discarding changes".to_string()),
        }
    }
}
