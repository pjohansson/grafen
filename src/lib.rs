//! Define and construct systems used for molecular simulations.

extern crate rand;
extern crate serde_json;
#[macro_use] extern crate serde_derive;

#[macro_use] pub mod coord;
#[macro_use] pub mod system;

pub mod describe;
pub mod database;
pub mod error;
pub mod iterator;
pub mod surface;
pub mod volume;