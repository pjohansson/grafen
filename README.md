[![Build Status](https://travis-ci.org/pjohansson/create_system.svg?branch=master)](https://travis-ci.org/pjohansson/create_system)

Construct substrates for use in molecular dynamics simulations.
Writes to GROMOS formatted files.

# Usage
```
USAGE:
    create_system [OPTIONS] <PATH> <X> <Y>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -t, --title <STR>    Title of system

ARGS:
    <PATH>    Output .gro file (the extension will be corrected)
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