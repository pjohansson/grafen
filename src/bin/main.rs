//! Create graphene and other substrates for use in molecular dynamics simulations.

extern crate colored;
extern crate dialoguer;
extern crate serde;
extern crate serde_json;
extern crate structopt;
#[macro_use] extern crate structopt_derive;

extern crate grafen;

mod error;
mod output;
mod ui;

use error::Result;

use grafen::database::{read_database, DataBase};

use std::process;
use std::path::PathBuf;
use structopt::StructOpt;

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
    /// Returns an error if the `DataBase` (if given as an input) could not be read.
    fn new() -> Result<Config> {
        let options = CliOptions::from_args();

        let output_path = options.output;
        let title = options.title.unwrap_or("System created by grafen".into());

        let database = match options.database {
            Some(path) => read_database(&path),
            None => Ok(DataBase::new()),
        }?;

        eprintln!("{} {}\n", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));

        Ok(Config { title, output_path, database })
    }
}

#[derive(StructOpt, Debug)]
/// Command line options
struct CliOptions {
    #[structopt(short = "t", long = "title")]
    /// Title of output system
    title: Option<String>,
    #[structopt(short = "o", long = "output", default_value = "conf.gro", parse(from_os_str))]
    /// Output configuration file
    output: PathBuf,
    #[structopt(short = "d", long = "database", parse(from_os_str))]
    /// Path to residue and component database
    database: Option<PathBuf>,
}

fn main() {
    if let Err(err) = Config::new().and_then(|conf| ui::user_menu(conf)) {
        eprintln!("{}", err);
        process::exit(1);
    }
}
