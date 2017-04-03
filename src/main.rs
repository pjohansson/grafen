#[macro_use]
extern crate clap;

pub mod lattice;
mod config;
mod output;
mod substrates;

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
        writeln!(&mut stderr, "{}", err).expect("could not write to stderr");
        std::process::exit(1)
    });

    if let Err(e) = config::run(config) {
        writeln!(&mut stderr, "{}", e).expect("could not write to stderr");
        std::process::exit(1)
    }
}
