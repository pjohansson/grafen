//! Errors when executing the binary.

use grafen::error::GrafenError;

use ansi_term::Colour::{Yellow, Red};
use clap;
use std::error::Error;
use std::fmt;
use std::io;
use std::result;

/// Shorthand for our `Result` class.
pub type Result<T> = result::Result<T, GrafenCliError>;

#[derive(Debug)]
/// Types of user interface errors. These are parsed into a `GrafenCliError::UIError`
/// using a `From` implementation below.
pub enum UIErrorKind {
    /// No selection was made when one was requested.
    NoSelection,
    /// An input value which was requested could not be parsed.
    BadValue(String),
    Abort,
}

#[derive(Debug)]
/// A class for configuration or runtime errors.
pub enum GrafenCliError {
    /// Some command line arguments were bad or non-existant.
    BadArgs(clap::Error),
    /// Something went wrong when reading or writing.
    IoError(io::Error),
    /// Something went wrong when creating the system.
    RunError(String),
    /// Something went wrong when constructing a Residue.
    ConstructError(String),
    /// User interface error.
    UIError(String),
    /// Exit the program without saving any system to disk.
    QuitWithoutSaving,
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

impl fmt::Display for GrafenCliError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let red_error = Red.paint("error:");

        match *self {
            // Clap already colours the `error: ` in red so we do not repeat that
            GrafenCliError::BadArgs(ref err) => {
                write!(f, "{}", err)
            },
            GrafenCliError::IoError(ref err) => {
                write!(f, "{} {}", red_error, err)
            },
            GrafenCliError::RunError(ref err) => {
                write!(f, "{} {}", red_error, err)
            },
            GrafenCliError::ConstructError(ref err) => {
                write!(f, "{}", Yellow.paint(err.as_str()))
            },
            GrafenCliError::UIError(ref err) => {
                write!(f, "{} {}", red_error, err)
            },
            GrafenCliError::QuitWithoutSaving => {
                write!(f, "Exiting without saving system.")
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

impl From<clap::Error> for GrafenCliError {
    fn from(err: clap::Error) -> GrafenCliError {
        GrafenCliError::BadArgs(err)
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
            UIErrorKind::NoSelection => GrafenCliError::UIError("No selection".to_string()),
            UIErrorKind::BadValue(err) => GrafenCliError::UIError(err),
            UIErrorKind::Abort => GrafenCliError::UIError("Discarding changes".to_string()),
        }
    }
}
