//! Collect definitions for `ResidueBase` and `SubstrateConf` objects
//! into a `DataBase` which can be read from or saved to disk.

use error::{GrafenCliError, Result};

use grafen::substrate::{LatticeType, SubstrateConf};
use grafen::system::ResidueBase;
use serde_json;
use std::io;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::ffi::OsStr;
use std::fs::File;
use std::result;

#[derive(Deserialize, Serialize)]
/// A collection of residues and substrate configurations
/// which can be saved to and read from disk.
pub struct DataBase {
    #[serde(skip_serializing, skip_deserializing)]
    /// A path to the `DataBase` location on the hard drive.
    pub path: Option<PathBuf>,

    #[serde(rename = "residue_definitions")]
    /// Definitions of `ResidueBase` objects.
    pub residue_defs: Vec<ResidueBase>,

    #[serde(rename = "substrate_definitions")]
    /// Definitions of `SubstrateConf` objects without their size.
    pub substrate_defs: Vec<SubstrateConfEntry>,
}

impl DataBase {
    /// Construct an empty `DataBase`.
    pub fn new() -> DataBase {
        DataBase {
            path: None,
            residue_defs: Vec::new(),
            substrate_defs: Vec::new(),
        }
    }

    /// Print the `DataBase` content to stdout.
    pub fn describe(&self) {
        println!("Database path: {}", self.get_path_pretty());

        println!("Substrate definitions:");
        for (i, def) in self.substrate_defs.iter().enumerate() {
            println!("{:4}. {}", i, def.name);
        }

        println!("Residue definitions:");
        for (i, def) in self.residue_defs.iter().enumerate() {
            println!("{:4}. {}", i, def.code);
        }
    }

    /// Get the database path enclosed in single quotes if it exists,
    /// otherwise the unenclosed string "None".
    pub fn get_path_pretty(&self) -> String {
        self.path.as_ref()
            .map(|path| path.to_string_lossy().to_owned())
            .map(|path| format!("'{}'", path))
            .unwrap_or("None".to_string())
    }

    /// Set a new path for the `DataBase`. The input path is asserted to
    /// be a non-empty file and the extension is set to 'json'.
    pub fn set_path<'a, T>(&mut self, new_path: &'a T) -> Result<()>
            where T: ?Sized + AsRef<OsStr> {
        let mut path = PathBuf::from(new_path);

        if path.file_stem().is_some() {
            path.set_extension("json");
            self.path = Some(path);
            Ok(())
        } else {
            Err(GrafenCliError::IoError(
                io::Error::new(io::ErrorKind::NotFound, "Input path is not a filename")
            ))
        }
    }

    /// Parse a reader for a JSON formatted `DataBase`.
    ///
    /// This and the `to_writer` method are defined to enable a unit
    /// test which ensures that the behaviour for reading and writing
    /// a `DataBase` is consistent.
    fn from_reader<R: Read>(reader: R) -> result::Result<DataBase, io::Error> {
        serde_json::from_reader(reader).map_err(|e| io::Error::from(e))
    }

    /// Write a `DataBase` as a JSON formatted object to an input writer.
    fn to_writer<W: Write>(&self, writer: &mut W) -> result::Result<(), io::Error> {
        serde_json::to_writer_pretty(writer, self).map_err(|e| io::Error::from(e))
    }
}

/// Read a `DataBase` from a JSON formatted file.
/// The owned path is set to the input path.
pub fn read_database<'a>(from_path: &'a str) -> result::Result<DataBase, io::Error> {
    let path = Path::new(from_path);
    let buffer = File::open(&path)?;

    let mut database = DataBase::from_reader(buffer)?;
    database.path = Some(PathBuf::from(&from_path));

    Ok(database)
}

/// Write a `DataBase` as a JSON formatted file.
/// The function writes to that owned by the object.
pub fn write_database(database: &DataBase) -> result::Result<(), io::Error> {
    if let Some(ref path) = database.path {
        let mut buffer = File::create(&path)?;
        database.to_writer(&mut buffer)?;

        return Ok(());
    }

    Err(io::Error::new(
        io::ErrorKind::Other,
        "No path was set when trying to write the database to disk")
    )
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
/// A definition (catalog entry) for a `SubstrateConf`.
///
/// See `SubstrateConf` for more information. The final configuration
/// requires a size, which is not kept in the definition.
pub struct SubstrateConfEntry {
    /// Definition name.
    pub name: String,
    /// Lattice constructor.
    pub lattice: LatticeType,
    /// Base residue.
    pub residue: ResidueBase,
    /// Optional distribution of positions along z.
    pub std_z: Option<f64>,
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
            lattice: LatticeType::PoissonDisc { density: 1.0 },
            residue: base,
            std_z: None,
        };

        let serialized = serde_json::to_string(&conf).unwrap();
        let deserialized: SubstrateConfEntry = serde_json::from_str(&serialized).unwrap();

        assert_eq!(conf, deserialized);

        conf.std_z = Some(1.0);
        let serialized = serde_json::to_string(&conf).unwrap();
        let deserialized: SubstrateConfEntry = serde_json::from_str(&serialized).unwrap();
        assert_eq!(conf, deserialized);
    }

    #[test]
    fn read_and_write_database() {
        let base = ResidueBase {
            code: "RES".to_string(),
            atoms: vec![
                Atom { code: "A1".to_string(), position: Coord::new(0.0, 1.0, 2.0) },
                Atom { code: "A2".to_string(), position: Coord::new(3.0, 4.0, 5.0) },
            ]
        };

        let conf = SubstrateConfEntry {
            name: "Test".to_string(),
            lattice: LatticeType::PoissonDisc { density: 1.0 },
            residue: base.clone(),
            std_z: None,
        };

        let database = DataBase {
            path: Some(PathBuf::from("This/will/be/removed")),
            residue_defs: vec![base.clone()],
            substrate_defs: vec![conf.clone()],
        };

        let mut serialized: Vec<u8> = Vec::new();
        database.to_writer(&mut serialized).unwrap();
        let deserialized = DataBase::from_reader(serialized.as_slice()).unwrap();

        assert_eq!(None, deserialized.path);
        assert_eq!(database.residue_defs, deserialized.residue_defs);
        assert_eq!(database.substrate_defs, deserialized.substrate_defs);
    }

    #[test]
    fn set_database_path() {
        let mut database = DataBase::new();
        assert!(database.set_path("test").is_ok());
        assert_eq!("test.json", database.path.unwrap().to_str().unwrap());
    }

    #[test]
    fn set_database_to_empty_path_is_error() {
        let mut database = DataBase::new();
        database.path = Some(PathBuf::from("unchanged.json"));
        assert!(database.set_path("").is_err());
        assert_eq!("unchanged.json", database.path.unwrap().to_str().unwrap());
    }

    #[test]
    fn get_database_path() {
        let mut database = DataBase::new();
        assert_eq!("None", &database.get_path_pretty());

        database.set_path("/a/file.json").unwrap();
        assert_eq!("'/a/file.json'", &database.get_path_pretty());
    }
}
