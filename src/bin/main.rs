//! Create graphene and other substrates for use in molecular dynamics simulations.
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

extern crate ansi_term;
#[macro_use]
extern crate clap;
extern crate grafen;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

mod define;
mod error;
mod database;
mod output;

use database::{read_database, write_database, DataBase};
use error::{GrafenCliError, Result};
use define::SystemDefinition;

use grafen::substrate;
use grafen::system::{join_systems, Coord, Residue, System};
use std::cmp;
use std::error::Error;
use std::io;
use std::io::Write;
use std::process;
use std::result;
use std::path::PathBuf;

/// The program run configuration.
pub struct Config {
    /// Title of output system.
    pub title: String,
    /// Path to output file.
    pub output_filename: PathBuf,
    /// Database of residue and substrate definitions.
    pub database: DataBase,
}

impl Config {
    /// Parse the input command line arguments, ask the user to select
    /// a substrate and return the run configuration.
    ///
    /// # Errors
    /// Returns an error if any of the required arguments are missing or invalid,
    /// or if the user did not select a substrate. In the latter case the program
    /// should exit.
    pub fn from_matches(matches: clap::ArgMatches) -> Result<Config> {
        let output_filename = value_t!(matches, "output", String).map(|s| PathBuf::from(s))?;
        let title = value_t!(matches, "title", String)
            .unwrap_or("System created by grafen".to_string());

        let database = match value_t!(matches, "database", String) {
            Ok(path) => read_database(&path),
            _ => Ok(DataBase::new()),
        }?;

        Ok(Config { title, output_filename, database })
    }
}

fn main() {
    let matches = clap_app!(grafen_cli =>
        (version: crate_version!())
        (author: crate_authors!())
        (about: crate_description!())
        (@arg output: <PATH> "Output GROMOS file)")
        (@arg title: -t --title [STR] +takes_value "Title of output system")
        (@arg database: -d --database [STR] +takes_value "Path to database")
    ).get_matches();

    if let Err(err) = Config::from_matches(matches).and_then(|mut conf| run(&mut conf)) {
        let mut stderr = io::stderr();
        writeln!(&mut stderr, "{}", err).expect("could not write to stderr");
        process::exit(1);
    }
}

/// Run the program with the set `Config`.
fn run(config: &mut Config) -> Result<()> {
    let system_defs = define::user_menu(&mut config.database)?;
    let system = construct_system(&system_defs)?;
    output::write_gromos(&system, &config.output_filename, &config.title)
}

/// After the systems have been defined we create them one by one and join them.
fn construct_system(system_defs: &Vec<SystemDefinition>) -> Result<System> {
    let systems = system_defs.iter().map(|def| {
        let (x, y) = def.size;
        substrate::create_substrate(&def.finalized)
            .map(|system| system.translate(&def.position))
            .map_err(|err| GrafenCliError::from(err))
    })
    .collect::<result::Result<Vec<System>, GrafenCliError>>()?;

    Ok(join_systems(systems))
}
