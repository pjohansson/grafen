//! Create graphene and other substrates for use in molecular dynamics simulations.

mod error;
mod output;
mod ui;

use crate::{
    error::{GrafenCliError, Result},
    ui::read_configuration,
};

use grafen::{
    database::{read_database, ComponentEntry, DataBase},
    read_conf::ReadConf,
};

use colored::*;
use dirs;
use std::{
    env::current_dir,
    fs::DirBuilder,
    path::PathBuf,
    process,
};
use structopt::StructOpt;

const DEFAULT_DBNAME: &str = "database.json";

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

        Ok(Config {
            title,
            output_path,
            components,
            database,
        })
    }
}

#[derive(StructOpt, Debug)]
#[structopt(raw(setting = "structopt::clap::AppSettings::ColoredHelp"))]
/// Command line options
struct CliOptions {
    #[structopt(short = "t", long = "title")]
    /// Title of output system
    title: Option<String>,
    #[structopt(
        short = "o",
        long = "output",
        default_value = "conf.gro",
        parse(from_os_str)
    )]
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
        match read_configuration(&path) {
            Ok(conf) => configurations.push(conf),
            Err(err) => eprintln!("{}", err),
        }
    }

    eprint!("\n");

    let current_dir = current_dir().unwrap_or(PathBuf::new());
    let entries = configurations
        .iter()
        .map(|conf| ReadConf {
            conf: None,
            path: current_dir.join(&conf.path),
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

fn read_or_create_default_database() -> Result<DataBase> {
    let path = match get_default_database_path() {
        Some(path) => path,
        None => {
            eprintln!(
                "{}",
                format!(
                    "Could not find a location for the default database. \
                    Opening a temporary database which cannot be saved.",
                )
                .color("yellow")
            );

            return Ok(DataBase::new());
        },
    };

    // See if a database can be found before creating a new one.
    if path.is_file() {
        read_database(&path).map_err(|err| GrafenCliError::from(err))
    } else {
        let mut default_database = DataBase::new();
        let default_path = path;

        // Try to create the database directory and update the path
        if let Some(parent_dir) = default_path.parent() {
            match DirBuilder::new().recursive(true).create(&parent_dir) {
                Ok(_) => default_database.set_path(&default_path).unwrap(),
                Err(err) => {
                    eprintln!(
                        "{}",
                        format!(
                            "Warning: Could not create a folder for a default database at '{}' ({}). \
                            Opening a database which cannot be saved.",
                            default_path.display(),
                            err
                        )
                        .color("yellow")
                    );
                }
            }
        }

        Ok(default_database)
    }
}

fn get_default_database_path() -> Option<PathBuf> {
    dirs::data_local_dir().map(|dir| dir.join("grafen").join(DEFAULT_DBNAME))
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use std::env::set_var;
    use std::path::Component;
    #[test]
    #[cfg(any(unix, windows))]
    fn default_database_path_adds_grafen_directory_and_database_path() {
        let path = get_default_database_path().unwrap();
        let mut iter = path.components().rev();

        assert_eq!(
            iter.next().unwrap(),
            Component::Normal(DEFAULT_DBNAME.as_ref())
        );

        assert_eq!(iter.next().unwrap(), Component::Normal("grafen".as_ref()));
    }
}
