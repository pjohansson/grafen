//! Construct substrates for use in molecular dynamics simulations.
//! Writes to GROMOS formatted files.
//!
//! # Usage
//! ```
//! USAGE:
//!     create_system [OPTIONS] <PATH> <X> <Y>
//!
//! FLAGS:
//!     -h, --help       Prints help information
//!     -V, --version    Prints version information
//!
//! OPTIONS:
//!     -t, --title <STR>    Title of system
//!
//! ARGS:
//!     <PATH>    Output .gro file (the extension will be corrected)
//!     <X>       Size along x
//!     <Y>       Size along y
//! ```
//!
//! # Available Substrates
//! Spacings and translations for all substrates is currently hard-coded.
//! Preferably this should be set in some configuration files or as an option
//! input by the user during runtime.
//!
//! ## Graphene
//! A monolayer of carbon atoms set in a hexagonal honeycomb structure.
//! The spacing between every atom is 0.142 nm.
//!
//! ## Silica
//! A monolayer of rigid SiO2 molecules set in a triclinic formation with
//! with spacing 0.450 nm along both base vectors and an angle of 60 degrees
//! between them.

extern crate ansi_term;
#[macro_use]
extern crate clap;

pub mod lattice;
pub mod config;
pub mod output;
pub mod substrates;

use config::Config;

use std::io;
use std::io::Write;
use std::process;

fn main() {
    let matches = clap_app!(create_system =>
        (version: crate_version!())
        (author: crate_authors!())
        (about: crate_description!())
        (@arg output: <PATH> "Output .gro file (the extension will be corrected)")
        (@arg x: <X> "Size along x")
        (@arg y: <Y> "Size along y")
        (@arg title: -t --title [STR] +takes_value "Title of system")
    ).get_matches();

    if let Err(err) = Config::new(matches).and_then(|conf| conf.run()) {
        let mut stderr = io::stderr();
        writeln!(&mut stderr, "{}", err).expect("could not write to stderr");
        process::exit(1);
    }
}
