use coord::{Coord, Direction, Translate};
use describe::Describe;
use iterator::{ConfIter, ResidueIter, ResidueIterOut};
use system::Component;
use volume::{Contains, keep_residues_within_volume};

use mdio;
use std::path::{Path, PathBuf};
use std::rc::Rc;


#[derive(Clone, Debug, Deserialize, Serialize)]
/// The read configuration can be morphed into different types of elements.
pub enum ConfType {
    /// The cuboid uses a specific size.
    Cuboid {
        #[serde(skip)]
        origin: Coord,
        size: Coord,
    },
    /// The cylinder requires some data about its construction.
    Cylinder {
        #[serde(skip)]
        origin: Coord,
        radius: f64,
        height: f64,
        normal: Direction,
    },
}

impl ConfType {
    fn calc_size(&self) -> Coord {
        match self {
            &ConfType::Cuboid { origin: _, size } => size,
            &ConfType::Cylinder { origin: _, radius, height, normal } => {
                match normal {
                    Direction::X => Coord::new(height, 2.0 * radius, 2.0 * radius),
                    Direction::Y => Coord::new(2.0 * radius, height, 2.0 * radius),
                    Direction::Z => Coord::new(2.0 * radius, 2.0 * radius, height),
                }
            }
        }
    }
}

impl Contains for ConfType {
    fn contains(&self, coord: Coord) -> bool {
        match self {
            &ConfType::Cuboid { origin, size } => {
                coord.x >= origin.x && coord.x <= (origin.x + size.x)
                    && coord.y >= origin.y && coord.y <= (origin.y + size.y)
                    && coord.z >= origin.z && coord.z <= (origin.z + size.z)
            },
            &ConfType::Cylinder { origin, radius, height, normal } => {
                // Check distance from the "bottom center" of the cylinder
                let center = origin + match normal {
                    Direction::X => Coord::new(0.0, radius, radius),
                    Direction::Y => Coord::new(radius, 0.0, radius),
                    Direction::Z => Coord::new(radius, radius, 0.0),
                };

                let (dr, dh) = center.distance_cylindrical(coord, normal);

                dr <= radius && dh >= 0.0 && dh <= height
            },
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
/// Wrap a configuration that is read from disk into an object we can handle.
pub struct ReadConf {
    #[serde(skip)]
    /// A configuration that may have been read by the system.
    pub conf: Option<mdio::Conf>,
    #[serde(skip)]
    /// A backup configuration before any modifications were done.
    ///
    /// In the case of several modifications to the system being done, this should
    /// be used as the base element since it was the original. TODO: Implement!
    pub backup_conf: Option<mdio::Conf>,
    /// The path to the configuration on disk.
    pub path: PathBuf,
    /// A short description of the configuration.
    pub description: String,
    /// The set construction of the configuration.
    pub volume_type: ConfType,
}

impl ReadConf {
    /// Read a configuration from a GROMOS87 formatted file. Set its description to
    /// the title of the configuration file, and the path to that input.
    pub fn from_gromos87(path: &Path) -> Result<ReadConf, String> {
        let conf = mdio::Conf::from_gromos87(path).map_err(|err| err.to_string())?;

        let description = conf.title.clone();
        let origin = Coord::ORIGO;
        let size = Coord::from(conf.size);

        Ok(ReadConf {
            conf: Some(conf),
            backup_conf: None,
            path: PathBuf::from(path),
            description,
            volume_type: ConfType::Cuboid { origin, size },
        })
    }

    /// Calculate the size of the component using the volume type.
    ///
    /// Does not use the values of the read `mdio::Conf` object, but those set in the
    /// `ConfType` variant.
    fn calc_size(&self) -> Coord {
        self.volume_type.calc_size()
    }

    /// Get the component origin as it will be displayed to the user, not
    /// the internal value which is used to translate all coordinates.
    fn get_displayed_origin(&self) -> Coord {
        match self.volume_type {
            ConfType::Cuboid { origin: _, size: _ } => self.get_origin(),
            ConfType::Cylinder { origin: _, radius, height: _, normal } => {
                self.get_origin() + match normal {
                    Direction::X => Coord::new(0.0, radius, radius),
                    Direction::Y => Coord::new(radius, 0.0, radius),
                    Direction::Z => Coord::new(radius, radius, 0.0),
                }
            },
        }
    }

    pub fn construct(&mut self) {
        let volume_type = self.volume_type.clone();
        self.reconstruct(volume_type);
    }

    pub fn reconstruct(&mut self, new_conf_type: ConfType) {
        // Ensure that the volume we want to create has our origin.
        let new_conf_type = match new_conf_type {
            ConfType::Cuboid { origin: _, size } => {
                ConfType::Cuboid { origin: self.get_origin(), size }
            },
            ConfType::Cylinder { origin: _, radius, height, normal } => {
                ConfType::Cylinder { origin: self.get_origin(), radius, height, normal }
            },
        };

        if let Some(conf) = self.conf.as_ref() {
            let mut current_size = self.calc_size();

            if current_size == Coord::ORIGO {
                current_size = Coord::from(conf.size);
            }

            let new_size = new_conf_type.calc_size();

            let (nx, ny, nz) = (
                ((new_size.x / current_size.x).ceil() as usize).max(1),
                ((new_size.y / current_size.y).ceil() as usize).max(1),
                ((new_size.z / current_size.z).ceil() as usize).max(1),
            );

            self.conf = Some(conf.pbc_multiply(nx, ny, nz));
        }

        let contained_residues = keep_residues_within_volume(self, &new_conf_type);
        self.assign_residues(&contained_residues);

        self.volume_type = new_conf_type;
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

    fn box_size(&self) -> Coord {
        self.conf
            .as_ref()
            .map(|c| Coord::from(c.origin))
            .unwrap_or(Coord::ORIGO)
            + self.calc_size()
    }

    fn get_origin(&self) -> Coord {
        match self.volume_type {
            ConfType::Cuboid { origin, size: _ } => origin,
            ConfType::Cylinder { origin, radius: _, height: _, normal: _ } => origin,
        }
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
            match self.volume_type {
                ConfType::Cuboid { origin: _, size } => {
                    description.push_str(&format!(
                        " (Configuration of {} atoms at {} with size {})",
                        self.num_atoms(), self.get_displayed_origin(), size
                    ));
                },
                ConfType::Cylinder { origin: _, radius, height, normal: _ } => {
                    description.push_str(&format!(
                        " (Configuration cylinder of {} atoms at {} with radius {:.1} and height {:.1})",
                        self.num_atoms(), self.get_displayed_origin(), radius, height
                    ));
                },
            }
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
            backup_conf: None,
            path: PathBuf::from(""),
            description: String::new(),
            volume_type: ConfType::Cuboid { origin: Coord::ORIGO, size: Coord::ORIGO },
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
            backup_conf: None,
            path: PathBuf::from(""),
            description: String::new(),
            volume_type: ConfType::Cuboid { origin: Coord::ORIGO, size: Coord::ORIGO },
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
            backup_conf: None,
            path: PathBuf::from(""),
            description: String::new(),
            volume_type: ConfType::Cuboid { origin: Coord::ORIGO, size: Coord::ORIGO },
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
            backup_conf: None,
            path: PathBuf::from(""),
            description: String::new(),
            volume_type: ConfType::Cuboid { origin: Coord::ORIGO, size: Coord::ORIGO },
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

    #[test]
    fn read_configuration_cuboids_describe_their_positions_unchanged() {
        let origin = Coord::new(10.0, 20.0, 30.0);
        let (x0, y0, z0) = origin.to_tuple();

        let conf = mdio::Conf {
            title: "A title".to_string(),
            origin: RVec { x: x0, y: y0, z: z0 },
            size: RVec { x: 0.0, y: 0.0, z: 0.0 },
            residues: Vec::new(),
            atoms: Vec::new(),
        };

        let comp = ReadConf {
            conf: Some(conf),
            backup_conf: None,
            path: PathBuf::from(""),
            description: "".to_string(),
            volume_type: ConfType::Cuboid { origin, size: Coord::ORIGO },
        };

        let description = comp.describe();
        let needle = format!("at {}", Coord::new(x0, y0, z0));
        assert!(description.contains(&needle));
    }

    #[test]
    fn read_configuration_cylinders_describe_their_position_as_the_bottom_center() {
        let origin = Coord::new(10.0, 20.0, 30.0);
        let (x0, y0, z0) = origin.to_tuple();

        let conf = mdio::Conf {
            title: "A title".to_string(),
            origin: RVec { x: x0, y: y0, z: z0 },
            size: RVec { x: 0.0, y: 0.0, z: 0.0 },
            residues: Vec::new(),
            atoms: Vec::new(),
        };

        let radius = 6.3;
        let normal = Direction::Y;

        let comp = ReadConf {
            conf: Some(conf),
            backup_conf: None,
            path: PathBuf::from(""),
            description: "".to_string(),
            volume_type: ConfType::Cylinder {
                origin,
                radius,
                height: 0.1, // unused here
                normal,
            },
        };

        // It's aligned with its normal to the y plane
        let x = x0 + radius;
        let y = y0;
        let z = z0 + radius;

        let description = comp.describe();
        let needle = format!("at {}", Coord::new(x, y, z));
        assert!(description.contains(&needle));
    }

    #[test]
    fn get_displayed_origin_of_read_confs() {
        let origin = Coord::new(3.0, 5.0, 7.0);
        let (x0, y0, z0) = origin.to_tuple();

        let conf = mdio::Conf {
            title: "A title".to_string(),
            origin: RVec { x: x0, y: y0, z: z0 },
            size: RVec { x: 0.0, y: 0.0, z: 0.0 },
            residues: Vec::new(),
            atoms: Vec::new(),
        };

        let cuboid = ReadConf {
            conf: Some(conf.clone()),
            backup_conf: None,
            path: PathBuf::from(""),
            description: "".to_string(),
            volume_type: ConfType::Cuboid { origin, size: Coord::ORIGO },
        };

        assert_eq!(cuboid.get_displayed_origin(), origin);

        let radius = 6.3;

        let cylinder_x = ReadConf {
            conf: Some(conf.clone()),
            backup_conf: None,
            path: PathBuf::from(""),
            description: "".to_string(),
            volume_type: ConfType::Cylinder {
                origin,
                radius,
                height: 0.1, // unused here
                normal: Direction::X,
            },
        };

        let cyl_origin = origin + Coord::new(0.0, radius, radius);
        assert_eq!(cylinder_x.get_displayed_origin(), cyl_origin);

        let cylinder_y = ReadConf {
            conf: Some(conf.clone()),
            backup_conf: None,
            path: PathBuf::from(""),
            description: "".to_string(),
            volume_type: ConfType::Cylinder {
                origin,
                radius,
                height: 0.1, // unused here
                normal: Direction::Y,
            },
        };

        let cyl_origin = origin + Coord::new(radius, 0.0, radius);
        assert_eq!(cylinder_y.get_displayed_origin(), cyl_origin);

        let cylinder_z = ReadConf {
            conf: Some(conf.clone()),
            backup_conf: None,
            path: PathBuf::from(""),
            description: "".to_string(),
            volume_type: ConfType::Cylinder {
                origin,
                radius,
                height: 0.1, // unused here
                normal: Direction::Z,
            },
        };

        let cyl_origin = origin + Coord::new(radius, radius, 0.0);
        assert_eq!(cylinder_z.get_displayed_origin(), cyl_origin);
    }

    #[test]
    fn use_read_conf_to_construct_larger_cuboid_multiplies_and_cuts_the_base() {
        let residues = vec![
            Rc::new(RefCell::new(mdio::Residue {
                name: Rc::new(RefCell::new("RES1".to_string())),
                atoms: vec![Rc::new(RefCell::new("AT1".to_string()))],
            })),
        ];

        let origin = Coord::new(20.0, 20.0, 30.0);
        let (x0, y0, z0) = origin.to_tuple();

        let size = Coord::new(2.0, 2.0, 2.0);
        let (dx, dy, dz) = size.to_tuple();

        // Two atoms
        let coord1 = Coord::new(0.0, 0.0, 0.0);
        let (x1, y1, z1) = coord1.to_tuple();
        let coord2 = Coord::new(1.5, 1.0, 1.0); // in second half of box along x
        let (x2, y2, z2) = coord2.to_tuple();

        let conf = mdio::Conf {
            title: "A title".to_string(),
            origin: RVec { x: x0, y: y0, z: z0 },
            size: RVec { x: dx, y: dy, z: dz },
            residues: residues.clone(),
            atoms: vec![
                mdio::Atom {
                    name: Rc::clone(&residues[0].borrow().atoms[0]),
                    residue: Rc::clone(&residues[0]),
                    position: RVec { x: x1, y: y1, z: z1 },
                    velocity: None,
                },
                mdio::Atom {
                    name: Rc::clone(&residues[0].borrow().atoms[0]),
                    residue: Rc::clone(&residues[0]),
                    position: RVec { x: x2, y: y2, z: z2 },
                    velocity: None,
                },
            ]
        };

        let mut cuboid = ReadConf {
            conf: Some(conf),
            backup_conf: None,
            path: PathBuf::from(""),
            description: String::new(),
            volume_type: ConfType::Cuboid { origin, size: Coord::ORIGO },
        };

        // Construct this cuboid which is 1.5x larger along x (thus only the first atom
        // is multiplied) and 2x larger along y and z

        let new_size = Coord::new(dx * 1.5, dy * 2.0, dz * 2.0);
        let new_volume = ConfType::Cuboid { origin: Coord::ORIGO, size: new_size };

        cuboid.reconstruct(new_volume);

        eprintln!("{:?}", cuboid.conf.as_ref().unwrap().atoms);
        assert_eq!(cuboid.calc_size(), new_size);
        assert_eq!(cuboid.num_atoms(), 12); // 1.5 * 2 * 2 * 2
    }

    #[test]
    fn read_configurations_box_size_is_set_by_the_volume_type() {
        let origin = Coord::new(10.0, 20.0, 30.0);
        let (x0, y0, z0) = origin.to_tuple();
        let size = Coord::new(3.0, 5.0, 7.0);

        // This size is set to the mdio::Conf, but will not be used to get the box size
        let bad_size_in_conf = size + Coord::new(1.0, 2.0, 3.0);
        let (dx_bad, dy_bad, dz_bad) = bad_size_in_conf.to_tuple();

        let conf = mdio::Conf {
            title: "A title".to_string(),
            origin: RVec { x: x0, y: y0, z: z0 },
            size: RVec { x: dx_bad, y: dy_bad, z: dz_bad }, // Will not be used
            residues: Vec::new(),
            atoms: Vec::new(),
        };

        let cuboid = ReadConf {
            conf: Some(conf.clone()),
            backup_conf: None,
            path: PathBuf::from(""),
            description: "".to_string(),
            volume_type: ConfType::Cuboid { origin: Coord::ORIGO, size },
        };

        assert_eq!(cuboid.box_size(), origin + size);

        let radius = 5.0;
        let height = 3.0;
        let normal = Direction::Y;

        let cylinder = ReadConf {
            conf: Some(conf.clone()),
            backup_conf: None,
            path: PathBuf::from(""),
            description: "".to_string(),
            volume_type: ConfType::Cylinder { origin: Coord::ORIGO, radius, height, normal },
        };

        // The cylinder normal is aligned along y
        let cylinder_size = Coord::new(2.0 * radius, height, 2.0 * radius);
        assert_eq!(cylinder.box_size(), origin + cylinder_size);
    }

    #[test]
    fn size_of_confs_uses_values_in_volume_type() {
        let size = Coord::new(3.0, 5.0, 7.0);

        // This size is set to the mdio::Conf, but will not be used to get the size
        let bad_size_in_conf = size + Coord::new(1.0, 2.0, 3.0);
        let (dx_bad, dy_bad, dz_bad) = bad_size_in_conf.to_tuple();

        let conf = mdio::Conf {
            title: "A title".to_string(),
            origin: RVec { x: 1.0, y: 2.0, z: 3.0 },
            size: RVec { x: dx_bad, y: dy_bad, z: dz_bad }, // Will not be used
            residues: Vec::new(),
            atoms: Vec::new(),
        };

        let cuboid = ReadConf {
            conf: Some(conf.clone()),
            backup_conf: None,
            path: PathBuf::from(""),
            description: "".to_string(),
            volume_type: ConfType::Cuboid { origin: Coord::ORIGO, size },
        };

        assert_eq!(cuboid.calc_size(), size);

        let radius = 5.0;
        let height = 3.0;
        let normal = Direction::X;

        let cylinder_x = ReadConf {
            conf: Some(conf.clone()),
            backup_conf: None,
            path: PathBuf::from(""),
            description: "".to_string(),
            volume_type: ConfType::Cylinder { origin: Coord::ORIGO, radius, height, normal },
        };

        let cylinder_size = Coord::new(height, 2.0 * radius, 2.0 * radius);
        assert_eq!(cylinder_x.calc_size(), cylinder_size);

        let normal = Direction::Y;
        let cylinder_y = ReadConf {
            conf: Some(conf.clone()),
            backup_conf: None,
            path: PathBuf::from(""),
            description: "".to_string(),
            volume_type: ConfType::Cylinder { origin: Coord::ORIGO, radius, height, normal },
        };

        let cylinder_size = Coord::new(2.0 * radius, height, 2.0 * radius);
        assert_eq!(cylinder_y.calc_size(), cylinder_size);

        let normal = Direction::Z;
        let cylinder_z = ReadConf {
            conf: Some(conf.clone()),
            backup_conf: None,
            path: PathBuf::from(""),
            description: "".to_string(),
            volume_type: ConfType::Cylinder { origin: Coord::ORIGO, radius, height, normal },
        };

        let cylinder_size = Coord::new(2.0 * radius, 2.0 * radius, height);
        assert_eq!(cylinder_z.calc_size(), cylinder_size);
    }

    #[test]
    fn origin_of_confs_is_set_in_volume_type() {
        let origin = Coord::new(10.0, 20.0, 30.0);
        // let (x0, y0, z0) = origin.to_tuple();

        // This size is set to the mdio::Conf, but will not be used to get the origin
        let bad_origin = origin + Coord::new(1.0, 2.0, 3.0);
        let (x0_bad, y0_bad, z0_bad) = bad_origin.to_tuple();

        let conf = mdio::Conf {
            title: "A title".to_string(),
            origin: RVec { x: x0_bad, y: y0_bad, z: z0_bad }, // Will not be used
            size: RVec { x: 0.0, y: 0.0, z: 0.0 },
            residues: Vec::new(),
            atoms: Vec::new(),
        };

        let cuboid = ReadConf {
            conf: Some(conf.clone()),
            backup_conf: None,
            path: PathBuf::from(""),
            description: "".to_string(),
            volume_type: ConfType::Cuboid { origin, size: Coord::ORIGO },
        };

        assert_eq!(cuboid.get_origin(), origin);

        let radius = 5.0;
        let height = 3.0;
        let normal = Direction::Y;

        let cylinder = ReadConf {
            conf: Some(conf.clone()),
            backup_conf: None,
            path: PathBuf::from(""),
            description: "".to_string(),
            volume_type: ConfType::Cylinder { origin, radius, height, normal },
        };

        // The cylinder normal is aligned along y
        let cylinder_size = Coord::new(2.0 * radius, height, 2.0 * radius);
        assert_eq!(cylinder.get_origin(), origin);
    }
}
