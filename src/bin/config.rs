//! Configure and run the program.

use database::{read_database, write_database, DataBase, SubstrateConfEntry};
use error::{GrafenCliError, Result};
use output;

use grafen::substrate;
use grafen::system::Coord;

use clap;
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

        let substrate_entry = select_substrate(&database.substrate_defs)?;
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
            .map_err(|e| GrafenCliError::from(e))
            .map(|system| system.translate(&Coord::new(0.0, 0.0, self.z0)))
            .and_then(|system| output::write_gromos(&system, &self.filename, &self.title, 2.0 * self.z0))?;
        Ok(())
    }
}

#[derive(Debug, PartialEq)]
/// The user either wants to quit or entered something invalid
enum BadSelection {
    Quit,
    Invalid,
}

/// Parse the input for a positive number or a quit message.
fn parse_selection<'a>(input: &'a str) -> result::Result<usize, BadSelection> {
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
                return Err(GrafenCliError::NoSubstrate)
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
