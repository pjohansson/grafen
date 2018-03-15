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

use error::{GrafenCliError, Result};
use ui::read_configuration;

use grafen::database::{read_database, ComponentEntry, DataBase};
use grafen::read_conf::ReadConf;

use colored::*;
use std::env::{home_dir, var_os};
use std::fs::DirBuilder;
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
            Some(path) => read_database(&path).map_err(|err| GrafenCliError::from(err)),
            None => read_or_create_default_database(),
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

fn read_or_create_default_database() -> Result<DataBase> {
    match get_default_database() {
        Some(default_path) => {
            if default_path.is_file() {
                read_database(&default_path).map_err(|err| GrafenCliError::from(err))
            } else {
                let mut default_database = DataBase::new();

                if let Some(parent_dir) = default_path.parent() {
                    match DirBuilder::new().recursive(true).create(&parent_dir) {
                        Ok(_) => default_database.set_path(&default_path).unwrap(),
                        Err(err) => {
                            eprintln!("{}", format!(
                                "Warning: Could not create a folder for the \
                                default database at '{}' ({}). \
                                Opening a directory-local database.",
                                default_path.to_str().unwrap(), err
                            ).color("yellow"));
                        },
                    }
                }

                Ok(default_database)
            }
        },
        None => {
            eprintln!("{}", format!(
                "Could not find a location for the default database. \
                Opening a directory-local database."
            ).color("yellow"));

            Ok(DataBase::new())
        },
    }
}

fn read_input_configurations(confs: Vec<PathBuf>) -> (Vec<ComponentEntry>, Vec<ComponentEntry>) {
    let mut configurations = Vec::new();

    for path in confs {
        match read_configuration(&path) {
            Ok(conf) => configurations.push(conf),
            Err(err) => eprintln!("{}", err),
        }
    }

    eprint!("\n");

    let entries = configurations
        .iter()
        .map(|conf| ReadConf {
            conf: None,
            path: conf.path.clone(),
            backup_conf: None,
            description: conf.description.clone(),
            volume_type: conf.volume_type.clone(),
        })
        .map(|conf| ComponentEntry::ConfigurationFile(conf))
        .collect::<Vec<_>>();

    let components = configurations
        .into_iter()
        .map(|conf| ComponentEntry::ConfigurationFile(conf))
        .collect::<Vec<_>>();

    (components, entries)
}

fn get_default_database() -> Option<PathBuf> {
    const DEFAULT_DBNAME: &str = "database.json";
    get_platform_dependent_data_dir().map(|dir| dir.join(DEFAULT_DBNAME))
}

fn get_platform_dependent_data_dir() -> Option<PathBuf> {
    if cfg!(target_os = "linux") {
        var_os("XDG_DATA_HOME").map(|dir| PathBuf::from(dir))
            .or(home_dir().map(|dir| dir.join(".local").join("share")))
    } else if cfg!(target_os = "macos") {
        home_dir().map(|dir| dir.join("Library").join("Application Support"))
    } else if cfg!(target_os = "windows") {
        var_os("APPDATA").map(|dir| PathBuf::from(dir))
    } else {
        None
    }.map(|dir| dir.join("grafen"))
}
