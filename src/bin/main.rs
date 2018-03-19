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
use std::env::{current_dir, home_dir, var, var_os};
use std::fs::DirBuilder;
use std::process;
use std::path::PathBuf;
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
    let default_database_paths = get_default_database_paths();

    if default_database_paths.is_empty() {
        eprintln!("{}", format!(
            "Could not find a location for the default database. \
            Opening a database which cannot be saved.",
        ).color("yellow"));

        return Ok(DataBase::new());
    }

    // See if a default database can be found at any path before creating a new one.
    for path in &default_database_paths {
        if path.is_file() {
            return read_database(&path).map_err(|err| GrafenCliError::from(err))
        }
    }

    let mut default_database = DataBase::new();
    let default_path = &default_database_paths[0];

    if let Some(parent_dir) = default_path.parent() {
        match DirBuilder::new().recursive(true).create(&parent_dir) {
            Ok(_) => default_database.set_path(&default_path).unwrap(),
            Err(err) => {
                eprintln!("{}", format!(
                    "Warning: Could not create a folder for a default database at '{}' ({}). \
                    Opening a database which cannot be saved.",
                    default_path.display(), err
                ).color("yellow"));
            },
        }
    }

    Ok(default_database)
}

fn get_default_database_paths() -> Vec<PathBuf> {
    get_platform_dependent_data_dirs()
        .into_iter()
        .map(|dir| dir.join("grafen").join(DEFAULT_DBNAME))
        .collect()
}

fn get_platform_dependent_data_dirs() -> Vec<PathBuf> {
    let xdg_data_dirs_variable = var("XDG_DATA_DIRS")
        .unwrap_or(String::from("/usr/local/share:/usr/local"));
    let xdg_dirs_iter = xdg_data_dirs_variable.split(':').map(|s| Some(PathBuf::from(s)));

    let dirs = if cfg!(target_os = "macos") {
        vec![
            var_os("XDG_DATA_HOME").map(|dir| PathBuf::from(dir)),
            home_dir().map(|dir| dir.join("Library").join("Application Support"))
        ].into_iter()
         .chain(xdg_dirs_iter)
         .chain(vec![Some(PathBuf::from("/").join("Library").join("Application Support"))])
         .collect()
    } else if cfg!(target_os = "linux") {
        vec![
            var_os("XDG_DATA_HOME").map(|dir| PathBuf::from(dir)),
            home_dir().map(|dir| dir.join(".local").join("share"))
        ].into_iter()
         .chain(xdg_dirs_iter)
         .collect()
    } else if cfg!(target_os = "windows") {
        vec![var_os("APPDATA").map(|dir| PathBuf::from(dir))]
    } else {
        Vec::new()
    };

    dirs.into_iter().filter_map(|dir| dir).collect()
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use std::env::set_var;
    use std::path::Component;

    #[test]
    #[cfg(target_os = "macos")]
    fn default_database_dirs_on_macos_lead_with_xdg_dirs_then_application_support() {
        let xdg_data_home = "data_home";
        set_var("XDG_DATA_HOME", xdg_data_home);

        let xdg_data_directories = vec!["data_dir1", "data_dir2"];
        let xdg_data_dirs = format!("{}:{}", xdg_data_directories[0], xdg_data_directories[1]);
        set_var("XDG_DATA_DIRS", xdg_data_dirs);

        let user_appsupport = home_dir().unwrap().join("Library").join("Application Support");
        let root_appsupport = PathBuf::from("/").join("Library").join("Application Support");

        let result = get_platform_dependent_data_dirs();
        let priority_list = vec![
            PathBuf::from(xdg_data_home),
            user_appsupport,
            PathBuf::from(xdg_data_directories[0]),
            PathBuf::from(xdg_data_directories[1]),
            root_appsupport
        ];

        assert_eq!(result, priority_list);
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn default_database_dirs_on_linux_lead_with_xdg_dirs_then_local_share() {
        let xdg_data_home = "data_home";
        set_var("XDG_DATA_HOME", xdg_data_home);

        let xdg_data_directories = vec!["data_dir1", "data_dir2"];
        let xdg_data_dirs = format!("{}:{}", xdg_data_directories[0], xdg_data_directories[1]);
        set_var("XDG_DATA_DIRS", xdg_data_dirs);

        let user_local_share = home_dir().unwrap().join(".local").join("share");

        let result = get_platform_dependent_data_dirs();
        let priority_list = vec![
            PathBuf::from(xdg_data_home),
            user_local_share,
            PathBuf::from(xdg_data_directories[0]),
            PathBuf::from(xdg_data_directories[1])
        ];

        assert_eq!(result, priority_list);
    }

    #[test]
    #[cfg(any(unix, windows))]
    fn default_database_path_adds_grafen_directory_and_database_path() {
        let dirs = get_default_database_paths();
        assert!(!dirs.is_empty());

        for path in dirs {
            let mut iter = path.components().rev();
            assert_eq!(iter.next().unwrap(), Component::Normal(DEFAULT_DBNAME.as_ref()));
            assert_eq!(iter.next().unwrap(), Component::Normal("grafen".as_ref()));
        }
    }
}
