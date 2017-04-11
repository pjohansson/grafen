//! Create graphene and other substrates for use in molecular dynamics simulations.

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
        (@arg std_z: --std [Z] +takes_value "\
            Uniformly distribute positions along z. This value is the deviation range \
            (in nm) from the original position of each residue.")
    ).get_matches();

    if let Err(err) = Config::new(matches).and_then(|conf| conf.run()) {
        let mut stderr = io::stderr();
        writeln!(&mut stderr, "{}", err).expect("could not write to stderr");
        process::exit(1);
    }
}
