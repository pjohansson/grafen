//! Define and construct systems used for molecular simulations.

// Needed for the `ResidueIter` object.
// TODO: Figure out how to otherwise solve the lifetimes!
#![feature(nll)]

#[macro_use] extern crate bitflags;
extern crate colored;
extern crate mdio;
extern crate rand;
extern crate serde_json;
#[macro_use] extern crate serde_derive;

#[macro_use] pub mod coord;
#[macro_use] pub mod system;

pub mod describe;
pub mod database;
pub mod error;
pub mod iterator;
pub mod read_conf;
pub mod surface;
pub mod volume;
