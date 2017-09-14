//! Define and construct systems used for molecular simulations.

extern crate rand;
extern crate serde_json;
#[macro_use] extern crate serde_derive;

pub mod describe;
pub mod coord;
pub mod cylinder;
pub mod database;
pub mod error;
pub mod substrate;
pub mod system;
mod volume;
