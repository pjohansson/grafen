use coord::{Coord, Translate};
use describe::Describe;
use iterator::{ConfIter, ResidueIter, ResidueIterOut};
use system::Component;

use mdio;
use std::path::{Path, PathBuf};
use std::rc::Rc;

#[derive(Clone, Debug, Deserialize, Serialize)]
/// Wrap a configuration that is read from disk into an object we can handle.
pub struct ReadConf {
    #[serde(skip)]
    /// A configuration that may have been read by the system.
    pub conf: Option<mdio::Conf>,
    /// The path to the configuration on disk.
    pub path: PathBuf,
    /// A short description of the configuration.
    pub description: String,
}

impl ReadConf {
    /// Read a configuration from a GROMOS87 formatted file. Set its description to
    /// the title of the configuration file, and the path to that input.
    pub fn from_gromos87(path: &Path) -> Result<ReadConf, String> {
        let conf = mdio::Conf::from_gromos87(path).map_err(|err| err.to_string())?;

        let description = conf.title.clone();

        Ok(ReadConf {
            conf: Some(conf),
            path: PathBuf::from(path),
            description,
        })
    }
}

impl<'a> Component<'a> for ReadConf {
    fn assign_residues(&mut self, residues: &[ResidueIterOut]) {
        if let Some(conf) = self.conf.as_mut() {
            let mut atoms: Vec<mdio::Atom> = Vec::new();

            residues
                .iter()
                .for_each(|res| {
                    let res_name = res.get_residue();
                    res.get_atoms().iter().for_each(|atom_data| {
                        let (x, y, z) = atom_data.1.to_tuple();
                        let (residue, atom) = mdio::get_or_insert_atom_and_residue(
                            &res_name.borrow(), &atom_data.0.borrow(), &mut conf.residues
                        ).unwrap();

                        atoms.push(mdio::Atom {
                            name: Rc::clone(&atom),
                            residue: Rc::clone(&residue),
                            position: mdio::RVec { x, y, z },
                            velocity: None,
                        });
                    });
                });

            conf.atoms = atoms;
        }
    }

    /// Returns the box size of a read configuration, or (0, 0, 0) if it has not been read.
    ///
    /// # Notes
    /// The latter case should never happen, so a warning is printed to stderr.
    fn box_size(&self) -> Coord {
        match self.conf {
            Some(ref conf) => Coord::from(conf.size),
            None => {
                eprintln!("Warning: Tried to get box size of configuration that has not been read. This should not happen! Return default value.");
                Coord::ORIGO
            }
        }
    }

    fn get_origin(&self) -> Coord {
        self.conf.as_ref().map(|c| Coord::from(c.origin)).unwrap_or(Coord::ORIGO)
    }

    fn iter_residues(&self) -> ResidueIter<'_> {
        match self.conf {
            None => ResidueIter::None,
            Some(ref conf) => ResidueIter::Conf(ConfIter::new(&conf)),
        }
    }

    /// Returns the number of atoms in a read configuration, or 0 if it has not been read.
    fn num_atoms(&self) -> u64 {
        self.conf.as_ref().map(|c| c.atoms.len()).unwrap_or(0) as u64
    }

    /// Currently does nothing at all.
    fn with_pbc(self) -> Self {
        eprintln!("Warning: PBC adjustment has not been added to read configurations.");
        self
    }
}

impl Describe for ReadConf {
    fn describe(&self) -> String {
        let mut description = if self.description.is_empty() {
            "Unknown".to_string()
        } else {
            self.description.clone()
        };

        if self.conf.is_some() {
            description.push_str(&format!(" (Configuration of {} atoms at {} with size {})",
                self.num_atoms(), self.get_origin(), self.box_size()));
        }

        description
    }

    fn describe_short(&self) -> String {
        let description = if self.description.is_empty() {
            "Unknown".to_string()
        } else {
            self.description.clone()
        };

        description + " (Configuration)"
    }
}

impl Translate for ReadConf {
    fn translate(mut self, coord: Coord) -> Self {
        if let Some(ref mut conf) = self.conf {
            conf.origin += mdio::RVec { x: coord.x, y: coord.y, z: coord.z };
        }
        self
    }

    fn translate_in_place(&mut self, coord: Coord) {
        if let Some(ref mut conf) = self.conf {
            conf.origin += mdio::RVec { x: coord.x, y: coord.y, z: coord.z };
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use mdio::RVec;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn iterate_over_residues_in_a_read_configuration_works_and_ignores_origin() {
        let residues = vec![
            Rc::new(RefCell::new(mdio::Residue {
                name: Rc::new(RefCell::new("RES1".to_string())),
                atoms: vec![Rc::new(RefCell::new("AT1".to_string()))],
            })),
            Rc::new(RefCell::new(mdio::Residue {
                name: Rc::new(RefCell::new("RES2".to_string())),
                atoms: vec![Rc::new(RefCell::new("AT2".to_string()))],
            })),
        ];

        let conf = mdio::Conf {
            title: "A title".to_string(),
            origin: RVec { x: 10.0, y: 20.0, z: 30.0 }, // Will be ignored
            size: RVec { x: 0.0, y: 0.0, z: 0.0 },
            residues: residues.clone(),
            atoms: vec![
                // Residue 2
                mdio::Atom {
                    name: Rc::clone(&residues[1].borrow().atoms[0]),
                    residue: Rc::clone(&residues[1]),
                    position: RVec { x: 0.0, y: 1.0, z: 2.0 },
                    velocity: Some(RVec { x: 0.0, y: 0.1, z: 0.2 }),
                },
                // Residue 1
                mdio::Atom {
                    name: Rc::clone(&residues[0].borrow().atoms[0]),
                    residue: Rc::clone(&residues[0]),
                    position: RVec { x: 3.0, y: 4.0, z: 5.0 },
                    velocity: Some(RVec { x: 0.3, y: 0.4, z: 0.5 }),
                },
            ]
        };

        let read_conf = ReadConf {
            conf: Some(conf),
            path: PathBuf::from(""),
            description: String::new(),
        };

        let mut iter = read_conf.iter_residues();

        let res1 = iter.next().unwrap();
        let res1_name = res1.get_residue();
        let atoms1 = res1.get_atoms();

        assert_eq!(*res1_name.borrow(), "RES2");
        assert_eq!(atoms1.len(), 1);
        assert_eq!(*atoms1[0].0.borrow(), "AT2");
        assert_eq!(atoms1[0].1, Coord::new(0.0, 1.0, 2.0));

        let res2 = iter.next().unwrap();
        let res2_name = res2.get_residue();
        let atoms2 = res2.get_atoms();

        assert_eq!(*res2_name.borrow(), "RES1");
        assert_eq!(atoms2.len(), 1);
        assert_eq!(*atoms2[0].0.borrow(), "AT1");
        assert_eq!(atoms2[0].1, Coord::new(3.0, 4.0, 5.0));

        assert!(iter.next().is_none());
    }

    #[test]
    fn iterating_over_residues_in_read_configuration_returns_pointers_to_the_original() {
        let residues = vec![
            Rc::new(RefCell::new(mdio::Residue {
                name: Rc::new(RefCell::new("RES1".to_string())),
                atoms: vec![Rc::new(RefCell::new("AT1".to_string()))],
            })),
            Rc::new(RefCell::new(mdio::Residue {
                name: Rc::new(RefCell::new("RES2".to_string())),
                atoms: vec![Rc::new(RefCell::new("AT2".to_string()))],
            })),
        ];

        let conf = mdio::Conf {
            title: "A title".to_string(),
            origin: RVec { x: 10.0, y: 20.0, z: 30.0 }, // Will be ignored
            size: RVec { x: 0.0, y: 0.0, z: 0.0 },
            residues: residues.clone(),
            atoms: vec![
                // Residue 1
                mdio::Atom {
                    name: Rc::clone(&residues[0].borrow().atoms[0]),
                    residue: Rc::clone(&residues[0]),
                    position: RVec { x: 0.0, y: 1.0, z: 2.0 },
                    velocity: Some(RVec { x: 0.0, y: 0.1, z: 0.2 }),
                },
                // Residue 2
                mdio::Atom {
                    name: Rc::clone(&residues[1].borrow().atoms[0]),
                    residue: Rc::clone(&residues[1]),
                    position: RVec { x: 3.0, y: 4.0, z: 5.0 },
                    velocity: Some(RVec { x: 0.3, y: 0.4, z: 0.5 }),
                },
            ]
        };

        let read_conf = ReadConf {
            conf: Some(conf),
            path: PathBuf::from(""),
            description: String::new(),
        };

        let mut iter = read_conf.iter_residues();

        let res1 = iter.next().unwrap();
        let atoms = res1.get_atoms();
        assert!(Rc::ptr_eq(&res1.get_residue(), &residues[0].borrow().name));
        assert!(Rc::ptr_eq(&atoms[0].0, &residues[0].borrow().atoms[0]));

        let res2 = iter.next().unwrap();
        let atoms = res2.get_atoms();
        assert!(Rc::ptr_eq(&res2.get_residue(), &residues[1].borrow().name));
        assert!(Rc::ptr_eq(&atoms[0].0, &residues[1].borrow().atoms[0]));
    }

    #[test]
    fn iterate_over_residues_in_an_unread_configuration_returns_none() {
        let unread_conf = ReadConf {
            conf: None,
            path: PathBuf::from(""),
            description: String::new(),
        };
        assert!(unread_conf.iter_residues().next().is_none());
    }

    #[test]
    fn assign_modified_residues_to_read_configuration() {
        let residues = vec![
            Rc::new(RefCell::new(mdio::Residue {
                name: Rc::new(RefCell::new("RES1".to_string())),
                atoms: vec![Rc::new(RefCell::new("AT1".to_string()))],
            })),
            Rc::new(RefCell::new(mdio::Residue {
                name: Rc::new(RefCell::new("RES2".to_string())),
                atoms: vec![Rc::new(RefCell::new("AT2".to_string()))],
            })),
        ];

        let conf = mdio::Conf {
            title: "A title".to_string(),
            origin: RVec { x: 10.0, y: 20.0, z: 30.0 }, // Will be ignored
            size: RVec { x: 0.0, y: 0.0, z: 0.0 },
            residues: residues.clone(),
            atoms: vec![
                // Residue 1
                mdio::Atom {
                    name: Rc::clone(&residues[0].borrow().atoms[0]),
                    residue: Rc::clone(&residues[0]),
                    position: RVec { x: 0.0, y: 1.0, z: 2.0 },
                    velocity: Some(RVec { x: 0.0, y: 0.1, z: 0.2 }),
                },
                // Residue 2
                mdio::Atom {
                    name: Rc::clone(&residues[1].borrow().atoms[0]),
                    residue: Rc::clone(&residues[1]),
                    position: RVec { x: 3.0, y: 4.0, z: 5.0 },
                    velocity: Some(RVec { x: 0.3, y: 0.4, z: 0.5 }),
                },
            ]
        };

        let mut read_conf = ReadConf {
            conf: Some(conf),
            path: PathBuf::from(""),
            description: String::new(),
        };

        let original: Vec<ResidueIterOut> = read_conf.iter_residues().collect::<Vec<_>>();
        assert_eq!(original.len(), 2);

        // Modify the residue list by reversing it
        let modified = original
                .iter()
                .cloned()
                .rev()
                .collect::<Vec<_>>();

        read_conf.assign_residues(&modified);

        // Assert that the residues have been modified
        let result = read_conf.iter_residues().collect::<Vec<_>>();
        assert_eq!(result.len(), original.len());

        for (orig, res) in original.iter().rev().zip(result.iter()) {
            assert!(Rc::ptr_eq(&orig.get_residue(), &res.get_residue()));
            assert!(Rc::ptr_eq(&orig.get_atoms()[0].0, &res.get_atoms()[0].0));
            assert_eq!(orig.get_atoms()[0].1, res.get_atoms()[0].1);
        }
    }
}
