//! Configure and run the program.

use output;
use substrates;
use substrates::SubstrateType;

use clap;
use std::error::Error;
use std::io;
use std::io::Write;

/// Program configuration.
pub struct Config {
    /// Title of output system.
    title: String,
    /// Path of output file.
    filename: String,
    /// Create a system of this size, please.
    size: InputSize,
}

#[derive(Clone, Copy)]
/// Input system size along x and y.
pub struct InputSize(pub f64, pub f64);

impl Config {
    /// Parse the input command line arguments and return the run configuration.
    ///
    /// # Errors
    /// Returns an error if any of the required arguments are missing or invalid.
    pub fn new(matches: clap::ArgMatches) -> Result<Config, Box<Error>> {
        let output_file = value_t!(matches, "output", String)?;
        let size_x = value_t!(matches, "x", f64)?;
        let size_y = value_t!(matches, "y", f64)?;
        let title = value_t!(matches, "title", String)
            .unwrap_or("Substrate".to_string());

        Ok(Config {
            title: title,
            filename: output_file,
            size: InputSize(size_x, size_y),
        })
    }
}

/// Run the program with a given configuration.
///
/// # Errors
/// Returns an Error if the substrate couldn't be selected,
/// constructed or output to disk.
pub fn run(config: Config) -> Result<(), Box<Error>> {
    let substrate_type = select_substrate()?;
    let system = substrates::create_substrate(config.size, substrate_type)?;
    output::write_gromos(&system, &config.filename, &config.title)
}

/// Ask the user to select a substrate.
fn select_substrate() -> Result<SubstrateType, io::Error> {
    let io_other = io::ErrorKind::Other;

    println!("Available substrates:");
    println!("0. Graphene");
    println!("1. Silica");
    print!("Substrate number: ");
    io::stdout().flush()?;

    let mut selection = String::new();
    io::stdin().read_line(&mut selection)?;
    let num = selection
        .trim()
        .parse::<i64>().map_err(|_|
            io::Error::new(io_other, format!("'{}' is not a valid number", selection.trim()))
        );

    match num {
        Ok(0) => Ok(SubstrateType::Graphene),
        Ok(1) => Ok(SubstrateType::Silica),
        Ok(_) => Err(io::Error::new(io_other, "No substrate was selected")),
        Err(e) => Err(e)
    }
}
