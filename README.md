[![crates.io](https://img.shields.io/crates/v/grafen.svg)](https://crates.io/crates/grafen) [![Build Status](https://travis-ci.org/pjohansson/grafen.svg?branch=master)](https://travis-ci.org/pjohansson/grafen)

Create graphene and other substrates for use in molecular dynamics simulations.

# Usage
```
USAGE:
    grafen [OPTIONS] <PATH> <X> <Y>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -t, --title <STR>    Title of system

ARGS:
    <PATH>    Output GROMOS file (the extension will be set to .gro)
    <X>       Size along x
    <Y>       Size along y
```

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
