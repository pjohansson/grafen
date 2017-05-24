use grafen::substrate::{LatticeType, SubstrateConf};
use grafen::system::ResidueBase;
use serde_json;

use std::io;
use std::path::{Path, PathBuf};
use std::fs::File;

#[derive(Deserialize, Serialize)]
/// A collection of residues and substrate configurations
/// which can be saved to and read from disk.
pub struct DataBase {
    #[serde(skip_serializing, skip_deserializing)]
    /// A path to the `DataBase` location on the hard drive.
    pub filename: Option<PathBuf>,
    #[serde(rename="residue_definitions")]
    /// Definitions of `ResidueBase` objects.
    pub residues: Vec<ResidueBase>,
    #[serde(rename="substrate_definitions")]
    /// Definitions of `SubstrateConf` objects without their size.
    pub substrate_confs: Vec<SubstrateConfEntry>,
}

impl DataBase {
    /// By default a `DataBase` is empty.
    pub fn new() -> DataBase {
        DataBase {
            filename: None,
            residues: vec![],
            substrate_confs: vec![],
        }
    }
}

impl DataBase {
    pub fn from_file<'a>(input_path: &'a str) -> Result<DataBase, io::Error> {
        let path = Path::new(input_path);
        let buffer = File::open(&path)?;

        let mut database: DataBase = serde_json::from_reader(buffer)?;
        database.filename = Some(PathBuf::from(&path));

        Ok(database)
    }

    pub fn write(&self) -> Result<(), io::Error> {
        if let Some(ref path) = self.filename {
            let buffer = File::create(&path)?;
            serde_json::to_writer_pretty(buffer, self)?;
            Ok(())
        } else {
            unreachable!();
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
/// A definition (catalog entry) for a `SubstrateConf`.
///
/// See `SubstrateConf` for more information. The final configuration
/// requires a size, which is not kept in the definition.
pub struct SubstrateConfEntry {
    /// Definition name.
    pub name: String,
    /// Lattice constructor.
    lattice: LatticeType,
    /// Base residue.
    residue: ResidueBase,
    /// Optional distribution of positions along z.
    std_z: Option<f64>,
}

impl SubstrateConfEntry {
    /// Supply a size to construct a `SubstrateConf` definition.
    pub fn to_conf(&self, size_x: f64, size_y: f64) -> SubstrateConf {
        SubstrateConf {
            lattice: self.lattice.clone(),
            residue: self.residue.clone(),
            size: (size_x, size_y),
            std_z: self.std_z,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use grafen::system::*;

    #[test]
    fn substrate_conf_entry_into_conf() {
        let base = ResidueBase {
            code: "RES".to_string(),
            atoms: vec![
                Atom { code: "A".to_string(), position: Coord::new(0.0, 1.0, 2.0) },
            ]
        };
        let (size_x, size_y) = (2.0, 3.0);

        let conf = SubstrateConfEntry {
                    name: "".to_string(),
                    lattice: LatticeType::Hexagonal { a: 1.0 },
                    residue: base.clone(),
                    std_z: Some(0.5),
        }.to_conf(size_x, size_y);

        assert_eq!(LatticeType::Hexagonal { a: 1.0 }, conf.lattice);
        assert_eq!(base, conf.residue);
        assert_eq!((size_x, size_y), conf.size);
        assert_eq!(Some(0.5), conf.std_z);
    }

    #[test]
    fn serialize_and_deserialize_residue_entry() {
        let base = ResidueBase {
            code: "RES".to_string(),
            atoms: vec![
                Atom { code: "A1".to_string(), position: Coord::new(0.0, 1.0, 2.0) },
                Atom { code: "A2".to_string(), position: Coord::new(3.0, 4.0, 5.0) },
            ]
        };

        let serialized = serde_json::to_string(&base).unwrap();
        let deserialized: ResidueBase = serde_json::from_str(&serialized).unwrap();

        assert_eq!(base, deserialized);
    }

    #[test]
    fn serialize_and_deserialize_substrateconf_entry() {
        let base = ResidueBase {
            code: "RES".to_string(),
            atoms: vec![
                Atom { code: "A1".to_string(), position: Coord::new(0.0, 1.0, 2.0) },
                Atom { code: "A2".to_string(), position: Coord::new(3.0, 4.0, 5.0) },
            ]
        };

        let mut conf = SubstrateConfEntry {
            name: "Test".to_string(),
            lattice: LatticeType::PoissonDisc{ density: 1.0 },
            residue: base,
            std_z: None,
        };

        let serialized = serde_json::to_string(&conf).unwrap();
        let deserialized: SubstrateConfEntry = serde_json::from_str(&serialized).unwrap();

        assert_eq!(conf.name, deserialized.name);
        assert_eq!(conf.residue, deserialized.residue);
        assert_eq!(conf.std_z, deserialized.std_z);

        conf.std_z = Some(1.0);
        let serialized = serde_json::to_string(&conf).unwrap();
        let deserialized: SubstrateConfEntry = serde_json::from_str(&serialized).unwrap();
        assert_eq!(conf.std_z, deserialized.std_z);
    }
}
