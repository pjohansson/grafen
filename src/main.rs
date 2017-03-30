#[macro_use]
extern crate clap;

mod coords;
mod grids;
mod output;
mod substrates;

use coords::Coord;
use substrates::Atom;

use clap::{App, Arg, ArgMatches};
use std::io;
use std::io::Write;

pub struct System {
    title: String,
    atoms: Vec<Atom>,
    dimensions: Coord
}

fn main() {
    let matches = App::new("create_graphene")
        .version("0.2")
        .author("Petter Johansson <pettjoha@kth.se>")
        .about("Create a graphene substrate and output it to a GROMOS formatted file")
        .arg(Arg::with_name("output")
            .help("output .gro file (the extension will be corrected)")
            .value_name("PATH")
            .required(true))
        .arg(Arg::with_name("x")
            .help("size along x")
            .value_name("FLOAT")
            .required(true))
        .arg(Arg::with_name("y")
            .help("size along y")
            .value_name("FLOAT")
            .required(true))
        .arg(Arg::with_name("title")
            .help("title of system (default: \"Graphene substrate\")")
            .short("t")
            .long("title")
            .value_name("STR")
            .takes_value(true)
            .required(false))
        .get_matches();

    match run(matches) {
        Err(val) => std::process::exit(val),
        Ok(_)  => ()
    }
}

fn run(matches: ArgMatches) -> Result<(), i32> {
    let output_file = value_t_or_exit!(matches, "output", String);
    let size_x = value_t_or_exit!(matches, "x", f64);
    let size_y = value_t_or_exit!(matches, "y", f64);
    let title = value_t!(matches, "title", String).unwrap_or("Graphene substrate".to_string());

    let system = match substrates::create_graphene(size_x, size_y) {
        Ok(substrate) => System {
            title: title,
            atoms: substrate.coords,
            dimensions: substrate.dimensions
        },
        Err(e) => {
            writeln!(io::stderr(), "Error: Could not create system ({})", e).unwrap();
            return Err(2);
        }
    };

    match output::write_gromos(system, output_file) {
        Ok(_)  => Ok(()),
        Err(e) => {
            writeln!(io::stderr(), "Error: Could not write system to disk ({})", e).unwrap();
            Err(1)
        }
    }
}
