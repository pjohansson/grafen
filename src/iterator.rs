//! Iterate over atoms in components.

use coord::Coord;
use system::Residue;

use mdio;

use std::cell::RefCell;
use std::rc::Rc;
use std::slice::Iter;

/// Iteration object which owns the iterator. Used to ensure that it always exists.
///
/// May be unnecessary, should test sometime.
pub struct ConfIter<'a> {
    /// The iterator over residues inside the borrowed configuration.
    iter: mdio::ResidueIter<'a>,
}

impl<'a> ConfIter<'a> {
    /// Create an iteration object from an input configuration which owns the iterator.
    pub fn new(conf: &'a mdio::Conf) -> ConfIter<'a> {
        ConfIter {
            iter: conf.iter_residues(),
        }
    }
}

pub enum ResidueIter<'a> {
    Conf(ConfIter<'a>),
    Component(&'a Residue, Iter<'a, Coord>),
    None,
}

#[derive(Debug, Clone)]
pub enum ResidueIterOut {
    FromConf(Vec<Rc<RefCell<mdio::Atom>>>),
    FromComp(Rc<RefCell<String>>, Vec<(Rc<RefCell<String>>, Coord)>),
}

impl ResidueIterOut {
    /// Return a reference counted pointer to the residue name.
    pub fn get_residue(&self) -> Rc<RefCell<String>> {
        match self {
            &ResidueIterOut::FromConf(ref atoms) => {
                let atom = Rc::clone(&atoms[0]);
                let residue = Rc::clone(&atom.borrow().residue);
                let name = Rc::clone(&residue.borrow().name);

                Rc::clone(&name)
            },
            &ResidueIterOut::FromComp(ref res, _) => Rc::clone(&res),
        }
    }

    /// Return a list of the atoms in the residue. The atom names are pointers.
    pub fn get_atoms(&self) -> Vec<(Rc<RefCell<String>>, Coord)> {
        match self {
            &ResidueIterOut::FromConf(ref atoms) => {
                atoms
                    .iter()
                    .map(|atom| (Rc::clone(&atom.borrow().name), Coord::from(atom.borrow().position)))
                    .collect()
            },
            &ResidueIterOut::FromComp(_, ref atoms) => atoms.clone(),
        }
    }
}

impl<'a> Iterator for ResidueIter<'a> {
    type Item = ResidueIterOut;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            &mut ResidueIter::None => None,
            &mut ResidueIter::Conf(ref mut conf_iter) => {
                conf_iter.iter
                    .next()
                    .map(|res| res.unwrap())
                    .map(|res| {
                        ResidueIterOut::FromConf(res.iter().map(|&atom| Rc::new(RefCell::new(atom.clone()))).collect())
                    })
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
