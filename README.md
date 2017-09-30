[![crates.io](https://img.shields.io/crates/v/grafen.svg)](https://crates.io/crates/grafen) [![docs.rs](https://img.shields.io/badge/docs.rs-documentation-orange.svg)](https://docs.rs/crate/grafen) [![Build Status](https://travis-ci.org/pjohansson/grafen.svg?branch=master)](https://travis-ci.org/pjohansson/grafen)

Create graphene and other substrates for use in molecular dynamics simulations.

This is a pet project to help me set up simulation systems for my research. It is focused on formats used by [Gromacs](http://www.gromacs.org/).

# Usage
```
USAGE:
    grafen [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -d, --database <PATH>    Path to database
    -o, --output <PATH>      Output GROMOS configuration file (conf.gro)
    -t, --title <STR>        Title of output system
```

# Available Substrates
Substrate definitions are read from a JSON database. An example is provided
in `include/database.json`. This database contains a few residue
and the following substrate definitions:

## Graphene
A monolayer of carbon atoms set in a hexagonal honeycomb structure.
The spacing between every atom is 0.142 nm.

## Silica
A monolayer of rigid SiO2 molecules set in a triclinic formation with
with spacing 0.450 nm along both base vectors and an angle of 60 degrees
between them.

# Development
See the [documentation](https://docs.rs/crate/grafen).

# License
The program is unlicensed. See [unlicense.org](http://unlicense.org) for details.
