[![crates.io](https://img.shields.io/crates/v/grafen.svg)](https://crates.io/crates/grafen) [![docs.rs](https://img.shields.io/badge/docs.rs-documentation-orange.svg)](https://docs.rs/crate/grafen) [![Build Status](https://travis-ci.org/pjohansson/grafen.svg?branch=master)](https://travis-ci.org/pjohansson/grafen)

Create graphene and other substrates for use in molecular dynamics simulations. A binary CLI utility `grafen_cli` and the library `grafen` are both available for use.

This is a pet project to help me set up simulation systems for my research. It is focused on formats used by [Gromacs](http://www.gromacs.org/). 

# Usage
```
USAGE:
    grafen_cli [OPTIONS] <PATH> <X> <Y>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
        --std <Z>        Uniformly distribution positions along z (nm)
    -t, --title <STR>    Title of system
        --z0 <Z>         Substrate position along z (nm)

ARGS:
    <PATH>    Output GROMOS file (the extension will be set to .gro)
    <X>       Size of system along the x axis (nm)
    <Y>       Size of system along the y axis (nm)
```

# Library
See the [documentation](https://docs.rs/crate/grafen) for usage examples.

# Available Substrates
Spacings and translations for all substrates is currently hard-coded.
Preferably this should be set in some configuration files or as an option
input by the user during runtime.

## Graphene
A monolayer of carbon atoms set in a hexagonal honeycomb structure.
The spacing between every atom is 0.142 nm.

## Silica
A monolayer of rigid SiO2 molecules set in a triclinic formation with
with spacing 0.450 nm along both base vectors and an angle of 60 degrees
between them.

# License
The program is unlicensed. See [unlicense.org](http://unlicense.org) for details.
