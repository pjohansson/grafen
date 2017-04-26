#[macro_use]
extern crate grafen;

use grafen::system::{Atom, Coord, ResidueBase};
use grafen::substrate::{LatticeType, SubstrateConf};

#[test]
fn define_and_create_a_substrate() {
    // Define a simple residue
    let residue_atom_one = Atom { code: "C1", position: Coord::new(0.1, 0.2, 0.3) };
    let residue_atom_two = Atom { code: "C2", position: Coord::new(0.3, 0.2, 0.1) };

    let residue = ResidueBase {
        code: "RES",
        atoms: vec![
            residue_atom_one,
            residue_atom_two
        ],
    };

    // The triclinic lattice has an angle of 90 degrees which makes it
    // easy to evaluate the spacing to 1-by-0.5. Thus a size of 2.1-by-0.9
    // means that we should expect a 2-by-2 lattice point system to be
    // returned: The size is rounded to the closest multiple and PBC's
    // are taken into account.
    let conf = SubstrateConf {
        lattice: LatticeType::Triclinic { a: 1.0, b: 0.5, gamma: 90.0 },
        residue: residue,
        size: (2.1, 0.9),
        std_z: None,
    };

    let substrate = grafen::substrate::create_substrate(&conf).unwrap();

    assert_eq!(Coord::new(2.0, 1.0, 0.0), substrate.dimensions);

    // We should get the correct residue base positions
    let residues = substrate.residues;
    assert_eq!(4, residues.len());
    assert_eq!(Coord::new(0.0, 0.0, 0.0), residues[0].position);
    assert_eq!(Coord::new(1.0, 0.0, 0.0), residues[1].position);
    assert_eq!(Coord::new(0.0, 0.5, 0.0), residues[2].position);
    assert_eq!(Coord::new(1.0, 0.5, 0.0), residues[3].position);

    // ... and each residue should be correct
    for residue in residues {
        assert_eq!("RES", residue.code);
        assert_eq!(2, residue.atoms.len());
        assert_eq!(residue_atom_one, residue.atoms[0]);
        assert_eq!(residue_atom_two, residue.atoms[1]);
    }
}

// Ensure that the macro is exported
#[test]
fn define_a_residue_with_a_macro() {
    let res_orig = ResidueBase {
        code: "RES",
        atoms: vec![
            Atom { code: "A1", position: Coord::new(0.0, 0.0, 0.0) },
            Atom { code: "A2", position: Coord::new(1.0, 0.0, 0.0) },
        ],
    };
    let res_macro = resbase!["RES", ("A1", 0.0, 0.0, 0.0), ("A2", 1.0, 0.0, 0.0)];

    assert_eq!(res_orig, res_macro);
}
