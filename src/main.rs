//! Construct substrates for use in molecular dynamics simulations.
//! Writes to GROMOS formatted files.
//!
//! # Usage
//! ```bash
//! USAGE:
//!     create_system [OPTIONS] <PATH> <X> <Y>
//!
//! FLAGS:
//!     -h, --help       Prints help information
//!     -V, --version    Prints version information
//!
//! OPTIONS:
//!     -t, --title <STR>    title of system
//!
//! ARGS:
//!     <PATH>    output .gro file (the extension will be corrected)
//!     <X>       size along x
//!     <Y>       size along y
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

#[macro_use]
extern crate clap;

pub mod lattice;
pub mod config;
pub mod output;
pub mod substrates;

use std::io::prelude::*;

fn main() {
    let mut stderr = std::io::stderr();

    let matches = clap_app!(create_system =>
        (version: crate_version!())
        (author: crate_authors!())
        (about: crate_description!())
        (@arg output: <PATH> "output .gro file (the extension will be corrected)")
        (@arg x: <X> "size along x")
        (@arg y: <Y> "size along y")
        (@arg title: -t --title [STR] +takes_value "title of system")
    ).get_matches();

    let config = config::Config::new(matches).unwrap_or_else(|err| {
        writeln!(&mut stderr, "error: {}", err).expect("could not write to stderr");
        std::process::exit(1)
    });

    if let Err(e) = config::run(config) {
        writeln!(&mut stderr, "error: {}", e).expect("could not write to stderr");
        std::process::exit(1)
    }
}
