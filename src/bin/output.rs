//! Write systems to disk.

use error::Result;

use grafen::system::{Component, System};

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

    let mut res_num_total = 1;
    let mut atom_num_total = 1;

    for component in &system.components {
        let (x0, y0, z0) = component.get_origin().to_tuple();

        for residue in component.iter_residues() {
            let res_name = residue.get_residue();

            for (atom_name, position) in residue.get_atoms() {
                // GROMOS loops the atom and residue indices at five digits, and so do we.
                let res_num = res_num_total % 100_000;
                let atom_num = atom_num_total % 100_000;

                let (x, y, z) = (x0 + position.x, y0 + position.y, z0 + position.z);

                write!(&mut writer, "{:>5}{:<5}{:>5}{:>5}{:>8.3}{:>8.3}{:>8.3}\n",
                    res_num, res_name.borrow(), atom_name.borrow(), atom_num,
                    x, y, z)?;

                atom_num_total += 1;
            }

            res_num_total += 1;
        }
    }

    let (dx, dy, dz) = system.box_size().to_tuple();
    writer.write_fmt(format_args!("{:12.8} {:12.8} {:12.8}\n", dx, dy, dz))?;

    Ok(())
}
