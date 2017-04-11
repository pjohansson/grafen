//! Create graphene and other substrates for use in molecular dynamics simulations.
//!
//! # Usage
//! ```
//! USAGE:
//!     grafen_cli [OPTIONS] <PATH> <X> <Y>
//!
//! FLAGS:
//!     -h, --help       Prints help information
//!     -V, --version    Prints version information
//!
//! OPTIONS:
//!        --std <Z>        Uniformly distribution positions along z (nm)
//!    -t, --title <STR>    Title of system
//!        --z0 <Z>         Substrate position along z (nm)
//!
//! ARGS:
//!     <PATH>    Output GROMOS file (the extension will be set to .gro)
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
extern crate grafen;

mod config;
use config::Config;

use std::io;
use std::io::Write;
use std::process;

fn main() {
    let matches = clap_app!(grafen_cli =>
        (version: crate_version!())
        (author: crate_authors!())
        (about: crate_description!())
        (@arg output: <PATH> "Output GROMOS file (the extension will be set to .gro)")
        (@arg x: <X> "Size of system along the x axis (nm)")
        (@arg y: <Y> "Size of system along the y axis (nm)")
        (@arg title: -t --title [STR] +takes_value "Title of system")
        (@arg z0: --z0 [Z] +takes_value "Substrate position along z (nm)")
        (@arg std_z: --std [Z] +takes_value "Uniformly distribution positions along z (nm)")
    ).get_matches();

    if let Err(err) = Config::new(matches).and_then(|conf| conf.run()) {
        let mut stderr = io::stderr();
        writeln!(&mut stderr, "{}", err).expect("could not write to stderr");
        process::exit(1);
    }
}
