#[macro_use]
extern crate clap;

mod coords;
mod grids;
mod output;
mod substrates;

use coords::Coord;
use substrates::{Atom, Substrate};

use std::error::Error;
use std::io;
use std::io::Write;

pub struct System {
    title: String,
    atoms: Vec<Atom>,
    dimensions: Coord
}

pub struct Config {
    title: String,
    filename: String,
    size: (f64, f64),
}

impl Config {
    pub fn new(matches: clap::ArgMatches) -> Result<Config, Box<Error>> {
        let output_file = value_t!(matches, "output", String)?;
        let size_x = value_t!(matches, "x", f64)?;
        let size_y = value_t!(matches, "y", f64)?;
        let title = value_t!(matches, "title", String).unwrap_or("Graphene substrate".to_string());

        Ok(Config {
            title: title,
            filename: output_file,
            size: (size_x, size_y)
        })
    }
}

pub fn run(config: Config) -> Result<(), String> {
    let substrate = match select_substrate() {
        Ok(result) => result,
        Err(e) => {
            return Err(format!("error: {}", e));
        }
    };
    let system = match substrates::create_substrate(config.size, substrate) {
        Ok(sub) => System {
            title: config.title,
            atoms: sub.coords,
            dimensions: sub.dimensions
        },
        Err(e) => {
            return Err(format!("error: Could not create system ({})", e));
        }
    };
    if let Err(e) = output::write_gromos(system, config.filename) {
        return Err(format!("error: Could not write system to disk ({})", e))
    }

    Ok(())
}

fn select_substrate() -> Result<Substrate, io::Error> {
    let io_other = io::ErrorKind::Other;

    println!("Available substrates:");
    println!("0. Graphene");
    println!("1. Silica");
    print!("Substrate number: ");
    io::stdout().flush()?;

    let mut selection = String::new();
    io::stdin().read_line(&mut selection)?;
    let num = selection
        .trim()
        .parse::<i64>().map_err(|e|
            io::Error::new(io_other, format!("'{}' is not a valid number", selection.trim()))
        );

    match num {
        Ok(0) => Ok(Substrate::Graphene),
        Ok(1) => Ok(Substrate::Silica),
        Ok(_) => Err(io::Error::new(io_other, "No substrate was selected")),
        Err(e) => Err(e)
    }
}
