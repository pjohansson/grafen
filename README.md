[![crates.io](https://img.shields.io/crates/v/grafen.svg)](https://crates.io/crates/grafen) [![docs.rs](https://img.shields.io/badge/docs.rs-documentation-orange.svg)](https://docs.rs/crate/grafen) [![Build Status](https://travis-ci.org/pjohansson/grafen.svg?branch=master)](https://travis-ci.org/pjohansson/grafen)

Create graphene and other substrates or system configurations for use in molecular dynamics simulations.

This is a pet project to help me set up simulation systems for my research. It is focused on formats used by [Gromacs](http://www.gromacs.org/).

**As of 0.10 requires Rust Nightly for `nll` and `underscore_lifetimes` features.**

# Usage
```
USAGE:
    grafen [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -d, --database <database>      Path to residue and component database
    -c, --conf <input_confs>...
            Path to input configuration files to add as components

    -o, --output <output>
            Output configuration file [default: conf.gro]

    -t, --title <title>            Title of output system
```

# Available Substrates
Substrate definitions are read from a JSON database. An example is provided
in `assets/database.json`. This database contains a few residue
and the following substrate definitions:

## Graphene
A monolayer of carbon atoms set in a hexagonal honeycomb structure.
The spacing between every atom is 0.142 nm.

## Silica
A monolayer of rigid SiO2 molecules set in a triclinic formation with
with spacing 0.450 nm along both base vectors and an angle of 60 degrees
between them.

## Graphene Nanotube
A cylinder constructed of carbon atoms in the same structure as the above graphene.

# Configuration Files
The program supports reading configurations from disk and manipulating them in some ways. Currently read configurations can be extended by duplicating and cutting them, or cut into cylinders.

Such an example is included in the `database.json` file.

# Database
## Paths to Configuration Files
The database which is saved to disk can contain references to system configurations that it will read data from. If these paths are entered into the database as relative paths, they will be read as relative to the database location.

## Database Location
The program by default tries to read a database from disk. On *Linux* (and other non-OSX *unix* systems) it looks in a subdirectory to the locations specified by the `XDG_DATA_HOME` and `XDG_DATA_DIRS` (read-only) environment variables, or the `$USER/.local/share` directory. On *OSX* it looks in the same `XDG`-spec locations but also in the user and root `Library/Application Support` directories. On *Windows* in the directory set by the `APPDATA` environment variable.

# Development
See the [documentation](https://docs.rs/crate/grafen).

# License
The program is unlicensed. See [unlicense.org](http://unlicense.org) for details.
