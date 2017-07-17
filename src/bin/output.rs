//! Write systems to disk.

use error::Result;
//use grafen::system::Component;
use super::Config;
use ui::ConstructedComponent;

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

/// Output a system to disk as a GROMOS formatted file.
/// The filename extension is adjusted to .gro.
///
/// # Errors
/// Returns an error if the file could not be written to.
pub fn write_gromos(components: &[ConstructedComponent], config: &Config) -> Result<()> {
    unimplemented!();
    /*
    let path = config.output_path.with_extension("gro");
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    let num_atoms: usize = components.iter().map(|comp| comp.component.num_atoms()).sum();
    writer.write_fmt(format_args!("{}\n", config.title))?;
    writer.write_fmt(format_args!("{}\n", num_atoms))?;

    // Absolute atom numbering.
    let mut j = 0;

    for (i, &residue) in system.residue_coords.iter().enumerate() {
        // GROMOS files wrap atom and residue numbering after five digits
        // so we must output at most that. We also switch to indexing the
        // numbers from 1 instead of from 0.
        let residue_number = (i + 1) % 100_000;

        for atom in system.residue_base.atoms.iter() {
            // Ibid.
            let atom_number = (j + 1) % 100_000;
            j += 1;

            let position = residue + atom.position;
            let (x, y, z) = position.to_tuple();

            writer.write_fmt(format_args!("{:>5}{:<5}{:>5}{:>5}{:>8.3}{:>8.3}{:>8.3}\n",
                                        residue_number,
                                        system.residue_base.code,
                                        atom.code,
                                        atom_number,
                                        x, y, z))?;
        }
    }

    let (dx, dy, dz) = system.box_size.to_tuple();
    writer.write_fmt(format_args!("{:12.8} {:12.8} {:12.8}\n", dx, dy, dz))?;

    Ok(())
    */
}
