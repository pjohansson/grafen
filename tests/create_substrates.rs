extern crate grafen;

use grafen::substrates::{ResidueAtom, ResidueBase, LatticeType, SubstrateConf, System};
use grafen::lattice::Coord;

#[test]
fn define_and_create_a_substrate() {
    let residue = ResidueBase {
        code: "RES",
        atoms: vec![
            ResidueAtom { code: "C", position: Coord::new(0.1, 0.2, 0.3) },
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

    let substrate = grafen::substrates::create_substrate(&conf).unwrap();
    assert_eq!(Coord::new(2.0, 1.0, 0.0), substrate.dimensions);

    let atoms = substrate.atoms;
    assert_eq!(4, atoms.len());
    assert_eq!(Coord::new(0.1, 0.2, 0.3), atoms[0].position);
    assert_eq!(Coord::new(1.1, 0.2, 0.3), atoms[1].position);
    assert_eq!(Coord::new(0.1, 0.7, 0.3), atoms[2].position);
    assert_eq!(Coord::new(1.1, 0.7, 0.3), atoms[3].position);
}

fn setup_substrate() -> System {
    let residue = ResidueBase {
        code: "RES",
        atoms: vec![
            ResidueAtom { code: "C", position: Coord::new(0.0, 0.0, 0.0) },
        ],
    };

    let conf = SubstrateConf {
        lattice: LatticeType::Triclinic { a: 1.0, b: 0.5, gamma: 90.0 },
        residue: residue,
        size: (10.0, 10.0),
        std_z: None,
    };

    grafen::substrates::create_substrate(&conf).unwrap()
}

#[test]
fn translate_a_substrate() {
    let add = Coord::new(1.0, 0.5, -1.0);
    let substrate = setup_substrate();
    let translated = substrate.translate(&add);

    assert_eq!(translated.dimensions, substrate.dimensions);
    assert_eq!(translated.atoms.len(), substrate.atoms.len());

    for (new, orig) in translated.atoms.iter().zip(substrate.atoms.iter()) {
        assert_eq!(new.position, orig.position.add(&add));
    }
}

fn setup_residue() -> ResidueBase {
    ResidueBase {
        code: "RES",
        atoms: vec![
            ResidueAtom { code: "C", position: Coord::new(0.0, 0.0, 0.0) },
        ],
    }
}

#[test]
fn uniform_distribution_on_substrate() {
    let conf = SubstrateConf {
        lattice: LatticeType::Hexagonal { a: 1.0 },
        residue: setup_residue(),
        size: (10.0, 10.0),
        std_z: Some(1.0),
    };

    let substrate = grafen::substrates::create_substrate(&conf).unwrap();

    // Not all z-positions should be zero (most likely)
    assert_eq!(false, substrate.atoms.iter().all(|a| a.position.z == 0.0));

    // ... but all will be smaller or equal to the maximum deviation
    assert!(substrate.atoms.iter().all(|a| a.position.z.abs() <= 1.0));
}
