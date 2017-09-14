//! Write systems to disk.

use super::Config;
use error::Result;
use ui::System;

use grafen::coord::Coord;

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
    let mut j = 0;

    for (i, (coord, ref base)) in system.iter_residues().enumerate() {
        let residue_number = (i + 1) % 100_000;

        for atom in &base.atoms {
            let atom_number = (j + 1) % 100_000;
            let position = coord + atom.position;
            let (x, y, z) = position.to_tuple();

            writer.write_fmt(format_args!("{:>5}{:<5}{:>5}{:>5}{:>8.3}{:>8.3}{:>8.3}\n",
                                        residue_number,
                                        base.code,
                                        atom.code,
                                        atom_number,
                                        x, y, z))?;
            j += 1;
        }
    }

    let (dx, dy, dz) = system.box_size
        .or(system.calc_box_size())
        .unwrap_or(Coord::new(0.0, 0.0, 0.0))
        .to_tuple();

    writer.write_fmt(format_args!("{:12.8} {:12.8} {:12.8}\n", dx, dy, dz))?;

    Ok(())
}
