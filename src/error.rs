//! Implements the custom `GrafenError` class for the library.

use std::error;
use std::fmt;
use std::io;
use std::result;

#[derive(Debug)]
/// A class for configuration or runtime errors.
pub enum GrafenError {
    /// Something went wrong when reading or writing.
    IoError(io::Error),
    /// Something went wrong when creating the system.
    RunError(String),
}

/// Shorthand for our `Result` class.
pub type Result<T> = result::Result<T, GrafenError>;

impl fmt::Display for GrafenError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            GrafenError::IoError(ref err) => {
                write!(f, "{}", err)
            },
            GrafenError::RunError(ref err) => {
                write!(f, "{}", err)
            },
        }
    }
}

impl error::Error for GrafenError {
    fn description(&self) -> &str {
        match *self {
            GrafenError::IoError(ref err) => err.description(),
            GrafenError::RunError(ref err) => &err,
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            GrafenError::IoError(ref err) => Some(err),
            GrafenError::RunError(_) => None, // There is no good cause for this currently.
        }
    }
}

impl From<io::Error> for GrafenError {
    fn from(err: io::Error) -> GrafenError {
        GrafenError::IoError(err)
    }
}
