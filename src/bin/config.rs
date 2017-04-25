//! Configure and run the program.

use grafen::error::GrafenError;
use grafen::output;
use grafen::substrates;
use grafen::substrates::LatticeType;
use grafen::system::ResidueBase;

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
    substrate_conf: substrates::SubstrateConf,
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
        let (lattice, residue) = select_substrate()?;

        // z0 has some default values depending on the chosen substrate
        //let z0 = match value_t!(matches, "z0", f64).ok() {
            //Some(v) => v,
            //None => match substrate_type {
                //SubstrateType::Graphene => 0.10,
                //SubstrateType::Silica => 0.30,
            //},
        //};
        //let std_z = value_t!(matches, "std_z", f64).ok();

        let substrate_conf = substrates::SubstrateConf {
            lattice: lattice,
            residue: residue,
            size: (size_x, size_y),
            std_z: None,
        };

        Ok(Config {
                title: title,
                filename: output_file,
                substrate_conf: substrate_conf,
            })
    }

    /// Run the program.
    ///
    /// # Errors
    /// Returns an error if the substrate couldn't be constructed or output to disk.
    pub fn run(&self) -> Result<()> {
        substrates::create_substrate(&self.substrate_conf)
            .and_then(|system| output::write_gromos(&system, &self.filename, &self.title))?;
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
            ConfigError::NoSubstrate => {
                write!(f, "{}", Yellow.paint("No substrate was selected."))
            },
        }
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

/// Ask the user to select a substrate.
fn select_substrate() -> Result<(LatticeType, ResidueBase)> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    const PROMPT: &'static [u8] = b"\
Available substrates:
0. Graphene
1. Silica
q. Exit program
";
    stdout.write(PROMPT)?;
    let mut selection = String::new();

    loop {
        stdout.write(b"Substrate number: ")?;
        stdout.flush()?;

        selection.clear();
        stdin.read_line(&mut selection)?;
        let value = selection.trim().chars().next();

        match value {
            Some('0') => {
                let spacing = 0.142;
                return Ok((
                    LatticeType::Hexagonal { a: spacing },
                    ResidueBase::graphene(spacing)
                ))
            },
            Some('1') => {
                let spacing = 0.45;
                return Ok((
                    LatticeType::Triclinic { a: spacing, b: spacing, gamma: 60.0 },
                    ResidueBase::silica(spacing)
                ))
            },
            Some('q') => return Err(ConfigError::NoSubstrate),
            _ => stdout.write(b"Not a valid option.\n")?,
        };
    }
}
