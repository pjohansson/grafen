//! Create graphene and other substrates for use in molecular dynamics simulations.

extern crate ansi_term;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate grafen;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

mod error;
mod database;
mod output;
mod ui;

use database::{read_database, DataBase};
use error::Result;

use std::io;
use std::io::Write;
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
    let matches = clap_app!(grafen_cli =>
        (version: crate_version!())
        (author: crate_authors!())
        (about: crate_description!())
        (@arg output: -o --output [PATH] +takes_value "Output GROMOS configuration file (conf.gro)")
        (@arg title: -t --title [STR] +takes_value "Title of output system")
        (@arg database: -d --database [PATH] +takes_value "Path to database")
    ).get_matches();

    if let Err(err) = Config::from_matches(matches).and_then(|mut conf| ui::user_menu(&mut conf)) {
        let mut stderr = io::stderr();
        writeln!(&mut stderr, "{}", err).expect("could not write to stderr");
        process::exit(1);
    }
}

// Run the program with the set `Config`.
/*
fn run(config: &mut Config) -> Result<()> {
    let system_defs = ui::user_menu(&mut config.database)?;
    let system = construct_system(&system_defs)?;
    output::write_gromos(&system, &config.output_path, &config.title)
}
*/

/*
// After the systems have been defined we create them one by one and join them.
fn construct_system(system_defs: &Vec<SystemDefinition>) -> Result<Component> {
    let components = system_defs.iter().map(|def| {
        substrate::create_substrate(&def.finalized)
            .map(|comp| comp.into_component())
            .map(|comp| comp.translate(&def.position))
            .map_err(|err| GrafenCliError::from(err))
    })
    .collect::<result::Result<Vec<Component>, GrafenCliError>>()?;

    Ok(join_components(components))
}
*/
