#[macro_use]
extern crate grafen;

use grafen::coord::Coord;
use grafen::system::{Atom, ResidueBase};
use grafen::substrate::{LatticeType, SheetConf};

#[test]
fn define_and_create_a_substrate() {
    // Define a simple residue
    let residue_atom_one = Atom { code: "C1".to_string(), position: Coord::new(0.1, 0.2, 0.3) };
    let residue_atom_two = Atom { code: "C2".to_string(), position: Coord::new(0.3, 0.2, 0.1) };

    let residue = ResidueBase {
        code: "RES".to_string(),
        atoms: vec![
            residue_atom_one.clone(),
            residue_atom_two.clone()
        ],
    };

    // The triclinic lattice has an angle of 90 degrees which makes it
    // easy to evaluate the spacing to 1-by-0.5. Thus a size of 2.1-by-0.9
    // means that we should expect a 2-by-2 lattice point system to be
    // returned: The size is rounded to the closest multiple and PBC's
    // are taken into account.
    let conf = SheetConf {
        lattice: LatticeType::Triclinic { a: 1.0, b: 0.5, gamma: 90.0 },
        residue: residue.clone(),
        size: (2.1, 0.9),
        std_z: None,
    };

    let substrate = grafen::substrate::create_substrate(&conf).unwrap();

    assert_eq!(Coord::new(2.0, 1.0, 0.0), substrate.size);

    // The residue should be correct
    assert_eq!(residue, substrate.residue_base);

    // We should get the correct residue base positions
    let residues = substrate.residue_coords;
    assert_eq!(4, residues.len());
    assert_eq!(Coord::new(0.0, 0.0, 0.0), residues[0]);
    assert_eq!(Coord::new(1.0, 0.0, 0.0), residues[1]);
    assert_eq!(Coord::new(0.0, 0.5, 0.0), residues[2]);
    assert_eq!(Coord::new(1.0, 0.5, 0.0), residues[3]);
}

// Ensure that the macro is exported
#[test]
fn define_a_residue_with_a_macro() {
    let res_orig = ResidueBase {
        code: "RES".to_string(),
        atoms: vec![
            Atom { code: "A1".to_string(), position: Coord::new(0.0, 0.0, 0.0) },
            Atom { code: "A2".to_string(), position: Coord::new(1.0, 0.0, 0.0) },
        ],
    };
    let res_macro = resbase!["RES", ("A1", 0.0, 0.0, 0.0), ("A2", 1.0, 0.0, 0.0)];

    assert_eq!(res_orig, res_macro);
}
