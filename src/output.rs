//! Write systems to disk.

use substrates::System;

use std::error::Error;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;

/// Output the system to disk as a GROMOS formatted file.
/// The filename extension is adjusted to .gro.
///
/// # Errors
/// Returns an error if the file could not be written to.
pub fn write_gromos(system: &System, output_file: &str, title: &str) -> Result<(), Box<Error>> {
    let path = PathBuf::from(output_file).with_extension("gro");
    let file = File::create(&path)?;
    let mut writer = BufWriter::new(file);

    writer.write_fmt(format_args!("{}\n", title))?;
    writer.write_fmt(format_args!("{}\n", system.atoms.len()))?;

    for atom in &system.atoms {
        // GROMOS files wrap atom and residue numbering after five digits
        // so we must output at most that. We also switch to indexing the
        // numbers from 1 instead of from 0.
        let residue_number = (atom.residue_number + 1) % 100_000;
        let atom_number = (atom.atom_number + 1) % 100_000;

        writer.write_fmt(format_args!("{:>5}{:<5}{:>5}{:>5}{:>8.3}{:>8.3}{:>8.3}\n",
                                    residue_number,
                                    atom.residue_name,
                                    atom.atom_name,
                                    atom_number,
                                    atom.position.x,
                                    atom.position.y,
                                    atom.position.z))?;
    }

    writer.write_fmt(format_args!("{:12.8} {:12.8} {:12.8}\n",
                                system.dimensions.x,
                                system.dimensions.y,
                                system.dimensions.z))?;

    Ok(())
}
