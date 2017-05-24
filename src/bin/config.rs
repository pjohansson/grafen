//! Configure and run the program.

use database::{read_database, write_database, DataBase, SubstrateConfEntry};
use output;

use grafen::error::GrafenError;
use grafen::substrate;
use grafen::system::Coord;

use ansi_term::Colour::{Yellow, Red};
use clap;

use std::error::Error;
use std::fmt;
use std::io;
use std::io::Write;
use std::result;

/// The program run configuration.
pub struct Config {
    /// Title of output system.
    title: String,
    /// Path of output file.
    filename: String,
    /// Substrate configuration.
    substrate_conf: substrate::SubstrateConf,
    /// Substrate position along z.
    z0: f64,
}

impl Config {
    /// Parse the input command line arguments, ask the user to select
    /// a substrate and return the run configuration.
    ///
    /// # Errors
    /// Returns an error if any of the required arguments are missing or invalid,
    /// or if the user did not select a substrate. In the latter case the program
    /// should exit.
    pub fn new(matches: clap::ArgMatches) -> Result<Config> {
        let output_file = value_t!(matches, "output", String)?;
        let size_x = value_t!(matches, "x", f64)?;
        let size_y = value_t!(matches, "y", f64)?;
        let title = value_t!(matches, "title", String).unwrap_or("Substrate".to_string());

        let database = match value_t!(matches, "database", String) {
            Ok(path) => read_database(&path),
            _ => Ok(DataBase::new()),
        }?;

        let substrate_entry = select_substrate(&database.substrate_confs)?;
        let substrate_conf = substrate_entry.to_conf(size_x, size_y);

        Ok(Config {
                title: title,
                filename: output_file,
                substrate_conf: substrate_conf,
                z0: 0.0,
            })
    }

    /// Run the program.
    ///
    /// # Errors
    /// Returns an error if the substrate couldn't be constructed or output to disk.
    pub fn run(&self) -> Result<()> {
        substrate::create_substrate(&self.substrate_conf)
            .map_err(|e| ConfigError::from(e))
            .map(|system| system.translate(&Coord::new(0.0, 0.0, self.z0)))
            .and_then(|system| output::write_gromos(&system, &self.filename, &self.title, 2.0 * self.z0))?;
        Ok(())
    }
}

/// A class for configuration or runtime errors.
pub enum ConfigError {
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

/// Shorthand for our `Result` class.
pub type Result<T> = result::Result<T, ConfigError>;

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let red_error = Red.paint("error:");

        match *self {
            // Clap already colours the `error: ` in red so we do not repeat that
            ConfigError::BadArgs(ref err) => {
                write!(f, "{}", err)
            },
            ConfigError::IoError(ref err) => {
                write!(f, "{} {}", red_error, err)
            },
            ConfigError::RunError(ref err) => {
                write!(f, "{} {}", red_error, err)
            },
            ConfigError::ConstructError(ref err) => {
                write!(f, "{}", Yellow.paint(err.as_str()))
            },
            ConfigError::NoSubstrate => {
                write!(f, "{}", Yellow.paint("No substrate was selected."))
            },
        }
    }
}

impl<'a> From<&'a str> for ConfigError {
    fn from(err: &'a str) -> ConfigError {
        ConfigError::ConstructError(err.to_string())
    }
}

impl From<io::Error> for ConfigError {
    fn from(err: io::Error) -> ConfigError {
        ConfigError::IoError(err)
    }
}

impl From<clap::Error> for ConfigError {
    fn from(err: clap::Error) -> ConfigError {
        ConfigError::BadArgs(err)
    }
}

impl From<GrafenError> for ConfigError {
    fn from(err: GrafenError) -> ConfigError {
        ConfigError::RunError(err.description().to_string())
    }
}

#[derive(Debug, PartialEq)]
/// The user either wants to quit or entered something invalid
enum BadSelection {
    Quit,
    Invalid,
}

/// Parse the input for a positive number or a quit message.
fn parse_selection<'a>(input: &'a str) -> ::std::result::Result<usize, BadSelection> {
    if let Ok(i) = input.parse::<usize>() {
        return Ok(i);
    }

    match input.chars().next() {
        Some('q') | Some('Q') => Err(BadSelection::Quit),
        _ => Err(BadSelection::Invalid),
    }
}

/// Ask the user to select a substrate.
fn select_substrate(substrates: &[SubstrateConfEntry]) -> Result<SubstrateConfEntry> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    println!("Available substrates:");
    for (i, substrate) in substrates.iter().enumerate() {
        println!("{}. {}", i, substrate.name);
    }
    println!("q. Exit program");

    let mut selection = String::new();

    loop {
        print!("Substrate number: ");
        stdout.flush()?;

        selection.clear();
        stdin.read_line(&mut selection)?;

        let parsed = parse_selection(&selection.trim())
            .and_then(|i| {
                substrates.iter().nth(i).ok_or(BadSelection::Invalid)
            });

        // In case the user entered something invalid we keep looping
        match parsed {
            Ok(i) => {
                return Ok(i.clone());
            },
            Err(BadSelection::Quit) => {
                return Err(ConfigError::NoSubstrate)
            },
            Err(BadSelection::Invalid) => {
                stdout.write(b"Not a valid option.\n")?;
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_substrate_selection() {
        assert_eq!(Ok(1), parse_selection("1"));
        assert_eq!(Ok(0), parse_selection("0"));
        assert_eq!(Ok(100), parse_selection("100"));
    }

    #[test]
    fn parse_substrate_badnumber() {
        assert_eq!(Err(BadSelection::Invalid), parse_selection("-1"));
    }

    #[test]
    fn parse_substrate_quit_message() {
        assert_eq!(Err(BadSelection::Quit), parse_selection("q"));
        assert_eq!(Err(BadSelection::Quit), parse_selection("quit"));
        assert_eq!(Err(BadSelection::Quit), parse_selection("Q"));
    }

    #[test]
    fn parse_substrate_no_selection() {
        assert_eq!(Err(BadSelection::Invalid), parse_selection(""));
        assert_eq!(Err(BadSelection::Invalid), parse_selection("v"));
    }
}
