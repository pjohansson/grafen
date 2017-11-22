//! Create graphene and other substrates for use in molecular dynamics simulations.

extern crate ansi_term;
#[macro_use] extern crate clap;
extern crate dialoguer;
extern crate serde;
extern crate serde_json;

extern crate grafen;

mod error;
mod output;
mod ui;

use error::Result;

use grafen::database::{read_database, DataBase};

use std::process;
use std::path::PathBuf;

/// The program run configuration.
pub struct Config {
    /// Title of output system.
    pub title: String,
    /// Path to output file.
    pub output_path: PathBuf,
    /// Database of residue and substrate definitions.
    pub database: DataBase,
}

impl Config {
    /// Parse the input command line arguments and read the `DataBase`.
    ///
    /// # Errors
    /// Returns an error if any of the arguments could not be properly parsed
    /// or the `DataBase` (if input) not opened.
    pub fn from_matches(matches: clap::ArgMatches) -> Result<Config> {
        let output_path = PathBuf::from(
            value_t!(matches, "output", String).unwrap_or("conf.gro".to_string())
        );
        let title = value_t!(matches, "title", String)
            .unwrap_or("System created by grafen".to_string()
        );

        let database = match value_t!(matches, "database", String) {
            Ok(path) => read_database(&path),
            Err(_) => Ok(DataBase::new()),
        }?;

        Ok(Config { title, output_path, database })
    }
}

fn main() {
    let matches = clap_app!(grafen =>
        (version: crate_version!())
        (author: crate_authors!())
        (about: crate_description!())
        (@arg output: -o --output [PATH] +takes_value "Output GROMOS configuration file (conf.gro)")
        (@arg title: -t --title [STR] +takes_value "Title of output system")
        (@arg database: -d --database [PATH] +takes_value "Path to database")
    ).get_matches();

    eprintln!("{} {}\n", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));

    if let Err(err) = Config::from_matches(matches).and_then(|conf| ui::user_menu(conf)) {
        eprintln!("{}", err);
        process::exit(1);
    }
}
