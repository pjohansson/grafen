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

/// A class for configuration or runtime errors.
pub enum GrafenCliError {
    /// No substrate was selected. The program should exit.
    NoSubstrate,
    /// Some command line arguments were bad or non-existant.
    BadArgs(clap::Error),
    /// Something went wrong when reading or writing.
    IoError(io::Error),
    /// Something went wrong when creating the system.
    RunError(String),
    /// Something went wrong when constructing a Residue.
    ConstructError(String),
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
            GrafenCliError::NoSubstrate => {
                write!(f, "{}", Yellow.paint("No substrate was selected."))
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
