//! Create graphene and other substrates for use in molecular dynamics simulations.

extern crate ansi_term;
#[macro_use]
extern crate clap;
extern crate grafen;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

mod config;
mod error;
mod database;
mod output;

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
        (@arg title: -t --title [STR] +takes_value "Title of output system")
        (@arg database: -d --database [STR] +takes_value "Path to database")
    ).get_matches();

    if let Err(err) = Config::new(matches).and_then(|conf| conf.run()) {
        let mut stderr = io::stderr();
        writeln!(&mut stderr, "{}", err).expect("could not write to stderr");
        process::exit(1);
    }
}
