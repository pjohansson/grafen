//! Create graphene and other substrates for use in molecular dynamics simulations.

extern crate colored;
extern crate dialoguer;
extern crate serde;
extern crate serde_json;
extern crate structopt;
#[macro_use] extern crate structopt_derive;

extern crate grafen;
extern crate mdio;

mod error;
mod output;
mod ui;

use error::Result;

use grafen::database::{read_database, ComponentEntry, DataBase};
use grafen::read_conf::ReadConf;
use grafen::system::Component;

use colored::*;

use std::process;
use std::path::PathBuf;
use structopt::StructOpt;

/// The program run configuration.
pub struct Config {
    /// Title of output system.
    pub title: String,
    /// Path to output file.
    pub output_path: PathBuf,
    /// Input components that were read from the command line.
    pub components: Vec<ComponentEntry>,
    /// Database of residue and substrate definitions.
    pub database: DataBase,
}

impl Config {
    /// Parse the input command line arguments and read the `DataBase`.
    ///
    /// # Errors
    /// Returns an error if the `DataBase` (if given as an input) could not be read.
    fn new() -> Result<Config> {
        eprintln!("{} {}\n", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));

        let options = CliOptions::from_args();

        let output_path = options.output;
        let title = options.title.unwrap_or("System created by grafen".into());

        let mut database = match options.database {
            Some(path) => read_database(&path),
            None => Ok(DataBase::new()),
        }?;

        let (components, mut entries) = read_input_configurations(options.input_confs);
        database.component_defs.append(&mut entries);

        Ok(Config { title, output_path, components, database })
    }
}

#[derive(StructOpt, Debug)]
#[structopt(raw(setting = "structopt::clap::AppSettings::ColoredHelp"))]
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
    #[structopt(short = "c", long = "conf", parse(from_os_str))]
    /// Path to input configuration files to add as components
    input_confs: Vec<PathBuf>,
}

fn main() {
    if let Err(err) = Config::new().and_then(|conf| ui::user_menu(conf)) {
        eprintln!("{}", err);
        process::exit(1);
    }
}

fn read_input_configurations(confs: Vec<PathBuf>) -> (Vec<ComponentEntry>, Vec<ComponentEntry>) {
    let mut configurations = Vec::new();

    for path in confs {
        match path.to_str() {
            Some(p) => eprint!("Reading configuration at '{}' ... ", p),
            None => eprint!("Reading configuration with a non-utf8 path ... "),
        }

        match ReadConf::from_gromos87(&path) {
            Ok(conf) => {
                eprint!("Done! Read {} atoms.", conf.num_atoms());
                configurations.push(conf);
            },
            Err(err) => eprint!("{}", format!("Failed! {}.", err).color("yellow")),
        }

        eprint!("\n");
    }

    eprint!("\n");

    let entries = configurations
        .iter()
        .map(|conf| ReadConf {
            conf: None,
            path: conf.path.clone(),
            description: conf.description.clone(),
        })
        .map(|conf| ComponentEntry::Conf(conf))
        .collect::<Vec<_>>();

    let components = configurations
        .into_iter()
        .map(|conf| ComponentEntry::Conf(conf))
        .collect::<Vec<_>>();

    (components, entries)
}
