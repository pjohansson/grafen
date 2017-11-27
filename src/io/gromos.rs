//! Read Gromos87 (.gro) formatted files.

use coord::Coord;
use io::GrafenIoError;
use io::GrafenIoError::*;
use system::{Atom, Residue};
use volume::Cuboid;

use std::io::Read;
use std::path::Path;
use std::result;
use std::str::FromStr;

fn read_file(input: &Path) -> result::Result<Cuboid, GrafenIoError> {
    unimplemented!();
}

fn read_input<R: Read>(input: &mut R) -> result::Result<Cuboid, GrafenIoError> {
    let mut buf = String::new();
    input.read_to_string(&mut buf)?;

    let mut iter_lines = buf.lines();

    let title = iter_lines.next().ok_or(EOF("No title in file".into()))?;

    let num_atoms = iter_lines
        .next()
        .ok_or(EOF("No number of atoms in file".into()))?
        .parse::<usize>()
        .map_err(|_| {
            ParseError("Number of atoms could not be parsed as a number".into())
        })?;

    let mut atoms: Vec<Atom> = vec![];
    let mut head_atom_position: Option<Coord> = None;

    let mut res_name = String::new();
    let mut coords = vec![];

    let mut last_res_num = 0;

    for line in iter_lines.by_ref().take(num_atoms) {
        let atom_line = AtomLine::from_str(&line)?;

        // We need to construct the configuration residue: do this using the first residue
        // we read from the input. As long as we are in that residue, add the atoms to a list.
        // Their positions will be relative to the first atom of the residue, which has
        // its relative position to the residue coordinate.
        if atom_line.res_num == 1 {
            let atom = if atom_line.atom_num == 1 {
                res_name = atom_line.res_name.clone();

                // The first atom in the residue: Save its *system absolute* position.
                head_atom_position = Some(atom_line.position.clone());
                Atom { code: atom_line.atom_name, position: Coord::ORIGO }
            } else {
                // Relative to the head atom
                if head_atom_position == None {
                    return Err(ParseError("Atom numbering is incorrect".into()));
                }

                let relative_position = atom_line.position - head_atom_position.unwrap();
                Atom { code: atom_line.atom_name, position: relative_position }
            };

            atoms.push(atom);
        }

        if atom_line.res_num > last_res_num {
            if atom_line.res_name != res_name {
                return Err(ParseError(format!(
                    "Invalid input: configurations may (currently) only consist of a single \
                    residue type, but at least two were found (first: {}, second: {})",
                    res_name, atom_line.res_name)));
            }

            coords.push(atom_line.position);
            last_res_num = atom_line.res_num;
        }
    }

    let residue = Residue {
        code: res_name,
        atoms,
    };

    let box_size = Coord::from_str(iter_lines
        .next()
        .ok_or(EOF("No box size vectors in file".into()))?
    ).map_err(|_| ParseError("Box size vectors could not be parsed".into()))?;

    Ok(Cuboid {
        name: Some(title.into()),
        residue: Some(residue),
        origin: Coord::ORIGO,
        size: box_size,
        coords,
    })
}

struct AtomLine {
    res_num: u64,
    res_name: String,
    atom_num: u64,
    atom_name: String,
    position: Coord,
}

impl FromStr for AtomLine {
    type Err = GrafenIoError;

    fn from_str(s: &str) -> result::Result<Self, Self::Err> {
        let mut chars = s.chars();

        let res_num = chars.by_ref().take(5).collect::<String>().trim().parse::<u64>()?;
        let res_name = chars.by_ref().take(5).collect::<String>().trim().to_string();

        let atom_name = chars.by_ref().take(5).collect::<String>().trim().to_string();
        let atom_num = chars.by_ref().take(5).collect::<String>().trim().parse::<u64>()?;

        let x = chars.by_ref().take(8).collect::<String>().trim().parse::<f64>()?;
        let y = chars.by_ref().take(8).collect::<String>().trim().parse::<f64>()?;
        let z = chars.by_ref().take(8).collect::<String>().trim().parse::<f64>()?;

        Ok(AtomLine {
            res_num,
            res_name,
            atom_num,
            atom_name,
            position: Coord::new(x, y, z),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::BufReader;

    #[test]
    fn read_one_atom_file() {
        let string: Vec<u8> = "\
Title
1
    1RES      A    1   1.000   2.000   3.000
    1.50000   2.50000   3.50000\n\
            ".into();

        let mut reader = BufReader::new(string.as_slice());
        let result = read_input(&mut reader).unwrap();

        assert_eq!(result.name.unwrap().as_str(), "Title");
        assert_eq!(result.residue.unwrap(), resbase!["RES", ("A", 0.0, 0.0, 0.0)]);
        assert_eq!(result.origin, Coord::ORIGO);
        assert_eq!(result.size, Coord::new(1.5, 2.5, 3.5));
        assert_eq!(result.coords, vec![Coord::new(1.0, 2.0, 3.0)]);
    }

    #[test]
    fn read_two_residues_file() {
        let string: Vec<u8> = "\
Title
2
    1RES      A    1   1.000   2.000   3.000
    2RES      A    1   2.000   3.000   4.000
    1.50000   2.50000   3.50000\n\
            ".into();

        let mut reader = BufReader::new(string.as_slice());
        let result = read_input(&mut reader).unwrap();

        assert_eq!(result.residue.unwrap(), resbase!["RES", ("A", 0.0, 0.0, 0.0)]);
        assert_eq!(result.size, Coord::new(1.5, 2.5, 3.5));
        assert_eq!(result.coords, vec![Coord::new(1.0, 2.0, 3.0), Coord::new(2.0, 3.0, 4.0)]);
    }

    #[test]
    fn read_one_residue_with_two_atoms_file() {
        let string: Vec<u8> = "\
Title
2
    1RES      A    1   1.000   2.000   3.000
    1RES      B    2   2.000   3.000   4.000
    1.50000   2.50000   3.50000\n\
            ".into();

        let mut reader = BufReader::new(string.as_slice());
        let result = read_input(&mut reader).unwrap();

        let expected_residue = resbase!["RES", ("A", 0.0, 0.0, 0.0), ("B", 1.0, 1.0, 1.0)];
        assert_eq!(result.residue.unwrap(), expected_residue);
        assert_eq!(result.size, Coord::new(1.5, 2.5, 3.5));
        assert_eq!(result.coords, vec![Coord::new(1.0, 2.0, 3.0)]);
    }

    #[test]
    fn read_atom_line() {
        let string = "    1RES     A     1   1.000   2.000   3.000";
        let atom_line = AtomLine::from_str(&string).unwrap();

        assert_eq!(&atom_line.res_name, "RES");
        assert_eq!(atom_line.res_num, 1);
        assert_eq!(&atom_line.atom_name, "A");
        assert_eq!(atom_line.atom_num, 1);
        assert_eq!(atom_line.position, Coord::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn read_atom_line_with_extreme_values() {
        let string = "12345RESIDATOMA678901000.0012000.0023000.003";
        let atom_line = AtomLine::from_str(&string).unwrap();

        assert_eq!(&atom_line.res_name, "RESID");
        assert_eq!(atom_line.res_num, 12345);
        assert_eq!(&atom_line.atom_name, "ATOMA");
        assert_eq!(atom_line.atom_num, 67890);
        assert_eq!(atom_line.position, Coord::new(1000.001, 2000.002, 3000.003));
    }

    #[test]
    /// TODO: Configurations should be read as a heterogenous system, somehow. Needs design.
    fn read_conf_with_multiple_residues_fails() {
        let string: Vec<u8> = "\
Title
2
    1RESA     A    1   1.000   2.000   3.000
    2RESB     B    2   2.000   3.000   4.000
    1.50000   2.50000   3.50000\n\
            ".into();

        let mut reader = BufReader::new(string.as_slice());
        assert!(read_input(&mut reader).is_err());
    }

    #[test]
    /// TODO: If its just a number mismatch it should be fixed at runtime. Needs design.
    fn read_conf_with_incorrect_numbering_fails() {
        let bad_atom_numbering: Vec<u8> = "\
Title
2
    1RES      A    2   1.000   2.000   3.000
    2RES      B    3   2.000   3.000   4.000
    1.50000   2.50000   3.50000\n\
            ".into();

        let mut reader = BufReader::new(bad_atom_numbering.as_slice());
        assert!(read_input(&mut reader).is_err());

        let bad_residue_numbering: Vec<u8> = "\
Title
2
    2RES      A    1   1.000   2.000   3.000
    3RES      B    2   2.000   3.000   4.000
    1.50000   2.50000   3.50000\n\
            ".into();

        let mut reader = BufReader::new(bad_residue_numbering.as_slice());
        assert!(read_input(&mut reader).is_err());
    }
}
