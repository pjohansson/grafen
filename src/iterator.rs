//! Iterate over atoms in components.

use coord::Coord;
use system::{Atom, Residue};

use mdio;

use std::cell::RefCell;
use std::rc::Rc;
use std::slice::Iter;

/// Iteration object which owns the iterator. Used to ensure that it always exists.
///
/// May be unnecessary, should test sometime.
pub struct ConfIter<'a> {
    conf: &'a mdio::Conf,
    /// The iterator over residues inside the borrowed configuration.
    iter: mdio::ResidueIter<'a>,
}

impl<'a> ConfIter<'a> {
    /// Create an iteration object from an input configuration which owns the iterator.
    pub fn new(conf: &'a mdio::Conf) -> ConfIter<'a> {
        ConfIter {
            conf: conf,
            iter: conf.iter_residues(),
        }
    }
}

pub enum ResidueIter<'a> {
    Conf(ConfIter<'a>),
    Component(&'a Residue, Iter<'a, Coord>),
    None,
}

pub enum ResidueIterOut<'a> {
    FromConf(Vec<&'a mdio::Atom>),
    FromComp(Rc<RefCell<String>>, Vec<(Rc<RefCell<String>>, Coord)>),
}

impl<'a> ResidueIterOut<'a> {
    /// Return a reference counted pointer to the residue name.
    pub fn get_residue(&self) -> Rc<RefCell<String>> {
        match self {
            &ResidueIterOut::FromConf(ref atoms) => Rc::clone(&atoms[0].residue.borrow().name),
            &ResidueIterOut::FromComp(ref res, _) => Rc::clone(&res),
        }
    }

    /// Return a list of the atoms in the residue. The atom names are pointers.
    pub fn get_atoms(&self) -> Vec<(Rc<RefCell<String>>, Coord)> {
        match self {
            &ResidueIterOut::FromConf(ref atoms) => {
                atoms
                    .iter()
                    .map(|atom| (Rc::clone(&atom.name), Coord::from(atom.position)))
                    .collect()
            },
            &ResidueIterOut::FromComp(_, ref atoms) => atoms.clone(),
        }
    }
}

impl<'a> Iterator for ResidueIter<'a> {
    type Item = ResidueIterOut<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            &mut ResidueIter::None => None,
            &mut ResidueIter::Conf(ref mut conf_iter) => {
                conf_iter.iter
                    .next()
                    .map(|res| res.unwrap())
                    .map(|res| ResidueIterOut::FromConf(res))
            },
            &mut ResidueIter::Component(ref res, ref mut iter) => {
                iter.next()
                    .map(|&coord| ResidueIterOut::FromComp(
                        Rc::new(RefCell::new(res.code.clone())),
                        res.atoms
                            .iter()
                            .map(|atom| (
                                Rc::new(RefCell::new(atom.code.clone())),
                                atom.position + coord
                            ))
                            .collect::<Vec<_>>()
                    ))
            },
        }
    }
}

/// The return type for `Iterator` functions.
///
/// Has to be boxed to return a fixed size. `impl Iterator` could be used
/// but it is in nightly, not stable.
pub type AtomIterItem<'a> = Box<Iterator<Item = CurrentAtom<'a>> + 'a>;
//pub type AtomIterItem<'a> = impl Iterator<Item = CurrentAtom<'a>>;

#[derive(Debug, PartialEq)]
/// Information about an atom, ready for output.
pub struct CurrentAtom<'a> {
    /// Relative atom index in the component. 0-indexed.
    pub atom_index: u64,
    /// Relative residue index in the component. 0-indexed.
    pub residue_index: u64,
    /// Current atom type.
    pub atom: &'a Atom,
    /// Current residue type.
    pub residue: &'a Residue,
    /// Atom position, relative to the component origin.
    pub position: Coord,
}

/// An `Iterator` over all the `Atom`s in a component.
///
/// Will yield objects of type `CurrentAtom`.
pub struct AtomIterator<'a> {
    /// Current index of atom in the `Residue`.
    atom_index: usize,
    /// Current index in coordinate list.
    coord_index: usize,
    /// Current absolute atom index in the component.
    atom_count: u64,
    /// A reference to the `Residue` that is yielded.
    residue: Option<&'a Residue>,
    /// Component origin position which is added to the relative atom positions.
    origin: Coord,
    /// List of coordinates to yield.
    coords: &'a [Coord],
}

impl<'a> AtomIterator<'a> {
    /// Construct an `Iterator` by supplying the `Residue`, list of `Coord`s
    /// and origin of a component.
    ///
    /// The `Residue` is given as an (optional) reference to avoid a lot of copies.
    pub fn new(residue: Option<&'a Residue>, coords: &'a [Coord], origin: Coord)
            -> AtomIterator<'a> {
        AtomIterator {
            atom_index: 0,
            coord_index: 0,
            atom_count: 0,
            residue: residue,
            origin,
            coords: &coords,
        }
    }
}

impl<'a> Iterator for AtomIterator<'a> {
    type Item = CurrentAtom<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.coords.get(self.coord_index) {
            Some(&coord) => {
                if let Some(residue) = self.residue {
                    if let Some(atom) = residue.atoms.get(self.atom_index) {
                        let position = atom.position + coord + self.origin;

                        let current = CurrentAtom {
                            atom_index: self.atom_count,
                            residue_index: self.coord_index as u64,
                            atom: &atom,
                            residue: &residue,
                            position,
                        };

                        self.atom_index += 1;
                        self.atom_count += 1;

                        Some(current)

                    } else {
                        self.atom_index = 0;
                        self.coord_index += 1;

                        self.next()
                    }
                } else {
                    None
                }
            },

            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atom_iterator_yields_correct_values() {
        let atom1 = Atom { code: "A".to_string(), position: Coord::new(0.0, 0.1, 0.2) };
        let atom2 = Atom { code: "B".to_string(), position: Coord::new(0.5, 0.6, 0.7) };
        let residue = Residue {
            code: "RES".to_string(),
            atoms: vec![atom1.clone(), atom2.clone()]
        };

        let coord1 = Coord::new(10.0, 11.0, 12.0);
        let coord2 = Coord::new(20.0, 21.0, 22.0);
        let coords = [coord1.clone(), coord2.clone()];

        let origin = Coord::new(5.0, 6.0, 7.0);

        let mut iter = AtomIterator::new(Some(&residue), &coords, origin);

        // Verify the third atom
        assert!(iter.next().is_some());
        assert!(iter.next().is_some());

        let current = iter.next().unwrap();
        assert_eq!(2, current.atom_index);
        assert_eq!(1, current.residue_index);
        assert_eq!(&atom1, current.atom);
        assert_eq!(&residue, current.residue);
        assert_eq!(atom1.position + coord2 + origin, current.position);

        assert!(iter.next().is_some());
        assert!(iter.next().is_none());
    }

    #[test]
    fn atom_iterator_with_no_residue_is_empty() {
        let coords = [Coord::new(0.0, 0.0, 0.0)];
        let mut iter = AtomIterator::new(None, &coords, Coord::ORIGO);
        assert!(iter.next().is_none());
    }
}
