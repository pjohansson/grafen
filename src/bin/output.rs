//! Write systems to disk.

use error::Result;
use grafen::system::Coord;
use super::Config;
use ui::System;

use std::fs::File;
use std::io::{BufWriter, Write};

/// Output a system to disk as a GROMOS formatted file.
/// The filename extension is adjusted to .gro.
///
/// # Errors
/// Returns an error if the file could not be written to.
pub fn write_gromos(system: &System, config: &Config) -> Result<()> {
    let path = config.output_path.with_extension("gro");
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    writer.write_fmt(format_args!("{}\n", config.title))?;
    writer.write_fmt(format_args!("{}\n", system.num_atoms()))?;

    // Absolute atom numbering.
    let mut i = 0;
    let mut j = 0;

    // TODO: Ideally, we would call an iterator (eg. system.iter_residues()) here
    // which would in order yield all `Coord`s with their `ResidueBase`s.
    // It would then be trivial to iterate over all `Atom`s of each base
    // and more flexible if we implement more output file types in the future.
    //
    // Alas, Rust cannot currently just yield types like that.
    // Look out for future improvements.
    for conf in system.constructed.iter() {
        let coords = &conf.component.residue_coords;
        let base = &conf.component.residue_base;

        for &coord in coords {
            // GROMOS files wrap atom and residue numbering after five digits
            // so we must output at most that. We also switch to indexing the
            // numbers from 1 instead of from 0.
            let residue_number = (i + 1) % 100_000;

            for atom in &base.atoms {
                let atom_number = (j + 1) % 100_000;
                let position = conf.component.origin + coord + atom.position;
                let (x, y, z) = position.to_tuple();

                writer.write_fmt(format_args!("{:>5}{:<5}{:>5}{:>5}{:>8.3}{:>8.3}{:>8.3}\n",
                                            residue_number,
                                            base.code,
                                            atom.code,
                                            atom_number,
                                            x, y, z))?;

                j += 1;
            }

            i += 1;
        }
    }

    let (dx, dy, dz) = system.box_size
        .or(system.calc_box_size())
        .unwrap_or(Coord::new(0.0, 0.0, 0.0))
        .to_tuple();

    writer.write_fmt(format_args!("{:12.8} {:12.8} {:12.8}\n", dx, dy, dz))?;

    Ok(())
}
