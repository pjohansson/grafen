//! Write systems to disk.

use error::Result;

use grafen::system::System;

use std::fs::File;
use std::io::{BufWriter, Write};

/// Output a system to disk as a GROMOS formatted file.
/// The filename extension is adjusted to .gro.
///
/// # Errors
/// Returns an error if the file could not be written to.
pub fn write_gromos(system: &System) -> Result<()> {
    let path = system.output_path.with_extension("gro");
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    writer.write_fmt(format_args!("{}\n", system.title))?;
    writer.write_fmt(format_args!("{}\n", system.num_atoms()))?;

    for current in system.iter_atoms() {
        // GROMOS loops the atom and residue indices at five digits, and so do we.
        let atom_index = (current.atom_index + 1) % 100_000;
        let residue_index = (current.residue_index + 1) % 100_000;

        let (x, y, z) = current.position.to_tuple();

        writer.write_fmt(format_args!("{:>5}{:<5}{:>5}{:>5}{:>8.3}{:>8.3}{:>8.3}\n",
                                    residue_index,
                                    current.residue.code,
                                    current.atom.code,
                                    atom_index,
                                    x, y, z))?;
    }

    let (dx, dy, dz) = system.box_size().to_tuple();
    writer.write_fmt(format_args!("{:12.8} {:12.8} {:12.8}\n", dx, dy, dz))?;

    Ok(())
}
