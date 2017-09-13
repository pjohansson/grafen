//! Collect definitions for `ResidueBase` and `SheetConf` objects
//! into a `DataBase` which can be read from or saved to disk.

//use error::{GrafenCliError, Result};

use cylinder::{Cylinder, CylinderConf};
use describe::{print_group, Describe};
use error::GrafenError;
use substrate::{create_substrate, LatticeType, SheetConf};
use system::{Coord, Component, ResidueBase, IntoComponent, Translate};

use serde_json;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::io;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::ffi::OsStr;
use std::fs::File;
use std::result;

#[derive(Copy, Clone, Debug)]
pub enum DataBaseError {
    BadPath,
}

#[derive(Deserialize, Serialize)]
/// A collection of residues and substrate configurations
/// which can be saved to and read from disk.
pub struct DataBase {
    #[serde(skip)]
    /// A path to the `DataBase` location on the hard drive.
    pub path: Option<PathBuf>,

    #[serde(rename = "residue_definitions", default = "Vec::new")]
    /// Definitions of `ResidueBase` objects.
    pub residue_defs: Vec<ResidueBase>,

    #[serde(rename = "component_definitions", default = "Vec::new")]
    /// Definitions of components.
    pub component_defs: Vec<AvailableComponents>,
}

impl DataBase {
    /// Construct an empty `DataBase`.
    pub fn new() -> DataBase {
        DataBase {
            path: None,
            residue_defs: vec![],
            component_defs: vec![],
        }
    }

    /// Print the `DataBase` content to stdout.
    /// TODO: This function name collides with the `Describe` trait. Consider implementing that.
    pub fn describe(&self) {
        println!("Database path: {}", self.get_path_pretty());
        println!("");

        print_group("Component definitions", &self.component_defs);

        println!("[ Residue definitions ]");
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
    pub fn set_path<T>(&mut self, new_path: &T) -> Result<(), DataBaseError>
            where T: ?Sized + AsRef<OsStr> {
        let mut path = PathBuf::from(new_path);

        if path.file_stem().is_some() {
            path.set_extension("json");
            self.path = Some(path);
            Ok(())
        } else {
            Err(DataBaseError::BadPath)
        }
    }

    /// Parse a reader for a JSON formatted `DataBase`.
    ///
    /// This and the `to_writer` method are defined to enable a unit
    /// test which ensures that the behaviour for reading and writing
    /// a `DataBase` is consistent.
    fn from_reader<R: Read>(reader: R) -> Result<DataBase, io::Error> {
        serde_json::from_reader(reader).map_err(|e| io::Error::from(e))
    }

    /// Write a `DataBase` as a JSON formatted object to an input writer.
    fn to_writer<W: Write>(&self, writer: &mut W) -> result::Result<(), io::Error> {
        serde_json::to_writer_pretty(writer, self).map_err(|e| io::Error::from(e))
    }
}

/// Read a `DataBase` from a JSON formatted file.
/// The owned path is set to the input path.
pub fn read_database(from_path: &str) -> Result<DataBase, io::Error> {
    let path = Path::new(from_path);
    let buffer = File::open(&path)?;

    let mut database = DataBase::from_reader(buffer)?;
    database.path = Some(PathBuf::from(&from_path));

    Ok(database)
}

/// Write a `DataBase` as a JSON formatted file.
/// The function writes to that owned by the object.
pub fn write_database(database: &DataBase) -> Result<(), io::Error> {
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
/// List of components that can be constructed.
///
/// To add a new type of component that can be constructed:
/// 1. Define an entry which can be serialized in the `DataBase`.
/// 2. Add the entry to this enum.
/// 3. Implement the required methods using a pattern match.
pub enum AvailableComponents {
    Sheet(SheetConfEntry),
    Cylinder(CylinderConfEntry),
}

impl AvailableComponents {
    /// Return a verbose description of the component that is to be created.
    pub fn describe_long(&self) -> String {
        match self {
            &AvailableComponents::Sheet(ref conf) => {
                let (dx, dy) = conf.size.unwrap_or((0.0, 0.0));
                let (x0, y0, z0) = conf.position.unwrap_or(Coord::new(0.0, 0.0, 0.0)).to_tuple();
                format!("Sheet of {} and size ({:.2}, {:.2}) at position ({:.2}, {:.2}, {:.2})",
                        conf.residue.code, dx, dy, x0, y0, z0)
            },
            &AvailableComponents::Cylinder(ref conf) => {
                let radius = conf.radius.unwrap_or(0.0);
                let height = conf.height.unwrap_or(0.0);
                let (x0, y0, z0) = conf.position.unwrap_or(Coord::new(0.0, 0.0, 0.0)).to_tuple();
                format!("Cylinder of {} with radius {:.2} and height {:.2} at position ({:.2}, {:.2}, {:.2})",
                        conf.residue.code, radius, height, x0, y0, z0)
            },
        }
    }

    /// Return a name for the component.
    pub fn name(&self) -> &str {
        match *self {
            AvailableComponents::Sheet(ref conf) => &conf.name,
            AvailableComponents::Cylinder(ref conf) => &conf.name,
        }
    }

    /// Construct the component from the definition.
    pub fn into_component(self) -> Result<Component, GrafenError> {
        use ::std::f64::consts::PI;

        match self {
            AvailableComponents::Sheet(conf) => {
                let position = conf.position.unwrap_or(Coord::new(0.0, 0.0, 0.0));
                let sheet = conf.to_conf()?;
                let component = create_substrate(&sheet)?;

                Ok(component.translate(&position).into_component())
            },
            AvailableComponents::Cylinder(conf) => {
                let radius = conf.radius
                    .ok_or(GrafenError::RunError("A cylinder radius was not set".to_string()))?;
                let height = conf.height
                    .ok_or(GrafenError::RunError("A cylinder height was not set".to_string()))?;
                let position = conf.position.unwrap_or(Coord::new(0.0, 0.0, 0.0));

                let cylinder = match conf.class {
                    CylinderClass::Sheet(ref lattice) => {
                            let mut sheet_conf = SheetConf {
                            lattice: lattice.clone(),
                            residue: conf.residue.clone(),
                            size: (2.0 * PI * radius, height),
                            std_z: None,
                        };
                        let sheet = create_substrate(&sheet_conf)?;

                        // Create the cylinder rotated to align along the z axis during construction.
                        // This is to easily set the bottom and top caps.
                        let mut cylinder = Cylinder::from_sheet(&sheet).into_component().rotate_x();

                        if let Some(cap) = conf.cap {
                            // Cut circles of the same material to work as bottom and top caps.
                            // Make them of slightly smaller radius to not overlap coordinates.
                            // TODO: This should be improved. IDEA: No offset, rotate circle
                            // and score nearest neighbor distance or something. Expensive, though.
                            const OFFSET: f64 = 0.0;
                            sheet_conf.size = (2.0 * radius, 2.0 * radius);
                            let circle_sheet = create_substrate(&sheet_conf)?;

                            let bottom = circle_sheet.into_circle(radius - OFFSET).into_component();
                            let top = bottom.clone().translate(&Coord::new(0.0, 0.0, height));

                            match cap {
                                CylinderCap::Bottom => {
                                    cylinder.extend(bottom);
                                },
                                CylinderCap::Top => {
                                    cylinder.extend(top);
                                },
                                CylinderCap::Both => {
                                    cylinder.extend(bottom);
                                    cylinder.extend(top);
                                },
                            }
                        }
                        cylinder
                    },
                    CylinderClass::Volume(opt_num_residues) => {
                        let num_residues = opt_num_residues.ok_or(
                            GrafenError::RunError(
                                "No number of residues to fill the cylinder with is set".to_string()
                        ))?;

                        CylinderConf {
                            origin: Coord::ORIGO,
                            radius,
                            height,
                            residue_base: conf.residue.clone(),
                        }
                        .fill_z(num_residues)
                        .into_component()
                    },
                };

                // Rotate it to the final position.
                // TODO: A proper general rotate function would be good here.
                let cylinder = match conf.alignment {
                    Direction::X => cylinder.rotate_y().rotate_y().rotate_y(),
                    Direction::Y => cylinder.rotate_x().rotate_x().rotate_x(),
                    Direction::Z => cylinder,
                };

                Ok(cylinder.translate(&position))
            },
        }
    }
}

impl Describe for AvailableComponents {
    /// Describe the components type with its name.
    fn describe(&self) -> String {
        match *self {
            AvailableComponents::Sheet(_) => format!("(Sheet) {}", self.name()),
            AvailableComponents::Cylinder(_) => format!("(Cylinder) {}", self.name()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
/// A definition (catalog entry) for a `SheetConf`.
///
/// See `SheetConf` for more information. The final configuration
/// requires a size, which is not kept in the definition.
pub struct SheetConfEntry {
    /// Definition name.
    pub name: String,
    /// Lattice constructor.
    pub lattice: LatticeType,
    /// Base residue.
    pub residue: ResidueBase,
    /// Optional distribution of positions along z.
    pub std_z: Option<f64>,
    #[serde(skip)]
    /// Size of sheet.
    pub size: Option<(f64, f64)>,
    #[serde(skip)]
    /// Position of sheet.
    pub position: Option<Coord>,
}

impl SheetConfEntry {
    /// Supply a size to construct a `SheetConf` definition.
    pub fn to_conf(&self) -> Result<SheetConf, GrafenError> {
        if let Some(size) = self.size {
            Ok(SheetConf {
                lattice: self.lattice.clone(),
                residue: self.residue.clone(),
                size,
                std_z: self.std_z,
            })
        } else {
            Err(GrafenError::RunError("No size was set for the sheet".to_string()))
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
/// Component direction axis. Eg. for `Cylinder`s this is the cylinder axis.
/// For a `Sheet` it could be the normal.
pub enum Direction { X, Y, Z }

impl Direction {
    fn default_cylinder() -> Direction {
        Direction::Z
    }
}

impl Display for Direction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            Direction::X => write!(f, "X"),
            Direction::Y => write!(f, "Y"),
            Direction::Z => write!(f, "Z"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
/// Cylinders can be capped in either or both ends.
pub enum CylinderCap {
    Top,
    Bottom,
    Both,
}

impl Display for CylinderCap {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            CylinderCap::Top => write!(f, "Top"),
            CylinderCap::Bottom => write!(f, "Bottom"),
            CylinderCap::Both => write!(f, "Both"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
/// Cylinders can either be a volume (residues inside) or a sheet (residues) on the outside.
pub enum CylinderClass {
    /// The cylinder is a folded sheet of an input lattice type.
    ///
    /// The number of residues is a result of which lattice is created.
    Sheet(LatticeType),
    /// The cylinder is filled with some residues.
    ///
    /// The number of residues are given directly to fill it with. It is an option since
    /// it should not actually be read from or saved to disk in the database. I believe
    /// this requires a custom de/serialize function? Works for now at least, but is not
    /// pretty.
    ///
    /// TODO: Ensure that the number of residues is not de/serialized.
    Volume(Option<usize>),
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
/// A catalog entry for a `CylinderConf`.
///
/// The final configuration requires a radius and height, not stored in this. But should it be?
pub struct CylinderConfEntry {
    /// Definition name.
    pub name: String,
    /// Base residue.
    pub residue: ResidueBase,
    #[serde(default = "Direction::default_cylinder")]
    /// Cylinder axis alignment.
    pub alignment: Direction,
    /// Cylinder cap.
    pub cap: Option<CylinderCap>,
    /// Cylinder is a layer or a filled volume.
    pub class: CylinderClass,
    #[serde(skip)]
    /// Cylinder radius.
    pub radius: Option<f64>,
    #[serde(skip)]
    /// Cylinder height.
    pub height: Option<f64>,
    #[serde(skip)]
    /// Position of cylinder.
    pub position: Option<Coord>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use system::*;

    #[test]
    fn substrate_conf_entry_into_conf() {
        let base = ResidueBase {
            code: "RES".to_string(),
            atoms: vec![
                Atom { code: "A".to_string(), position: Coord::new(0.0, 1.0, 2.0) },
            ]
        };
        let (size_x, size_y) = (2.0, 3.0);

        let conf = SheetConfEntry {
                    name: "".to_string(),
                    lattice: LatticeType::Hexagonal { a: 1.0 },
                    residue: base.clone(),
                    std_z: Some(0.5),
                    size: Some((size_x, size_y)),
                    position: None,
        }.to_conf().unwrap();

        assert_eq!(LatticeType::Hexagonal { a: 1.0 }, conf.lattice);
        assert_eq!(base, conf.residue);
        assert_eq!((size_x, size_y), conf.size);
        assert_eq!(Some(0.5), conf.std_z);
    }

    #[test]
    fn substrate_conf_entry_without_size_is_err() {
        let base = ResidueBase {
            code: "RES".to_string(),
            atoms: vec![
                Atom { code: "A".to_string(), position: Coord::new(0.0, 1.0, 2.0) },
            ]
        };

        let conf = SheetConfEntry {
                    name: "".to_string(),
                    lattice: LatticeType::Hexagonal { a: 1.0 },
                    residue: base,
                    std_z: None,
                    size: None,
                    position: None,
        }.to_conf();

        assert!(conf.is_err());
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

        let mut conf = SheetConfEntry {
            name: "Test".to_string(),
            lattice: LatticeType::PoissonDisc { density: 1.0 },
            residue: base,
            std_z: None,
            size: None,
            position: None,
        };

        let serialized = serde_json::to_string(&conf).unwrap();
        let deserialized: SheetConfEntry = serde_json::from_str(&serialized).unwrap();

        assert_eq!(conf, deserialized);

        // std is saved
        conf.std_z = Some(1.0);
        let serialized = serde_json::to_string(&conf).unwrap();
        let deserialized: SheetConfEntry = serde_json::from_str(&serialized).unwrap();
        assert_eq!(conf, deserialized);

        // size and position is not
        conf.size = Some((10.0, 5.0));
        conf.position = Some(Coord::new(0.0, 0.0, 0.0));
        let serialized = serde_json::to_string(&conf).unwrap();
        let deserialized: SheetConfEntry = serde_json::from_str(&serialized).unwrap();
        assert_eq!(None, deserialized.size);
        assert_eq!(None, deserialized.position);
    }

    #[test]
    fn database_by_default_sets_empty_vectors_if_not_available() {
        let database: DataBase = serde_json::from_str("{}").unwrap();
        assert!(database.residue_defs.is_empty());
        assert!(database.component_defs.is_empty());
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

        let conf = SheetConfEntry {
            name: "Test".to_string(),
            lattice: LatticeType::PoissonDisc { density: 1.0 },
            residue: base.clone(),
            std_z: None,
            size: None,
            position: None,
        };

        let database = DataBase {
            path: Some(PathBuf::from("This/will/be/removed")),
            residue_defs: vec![base.clone()],
            component_defs: vec![AvailableComponents::Sheet(conf.clone())],
        };

        let mut serialized: Vec<u8> = Vec::new();
        database.to_writer(&mut serialized).unwrap();
        let deserialized = DataBase::from_reader(serialized.as_slice()).unwrap();

        assert_eq!(None, deserialized.path);
        assert_eq!(database.residue_defs, deserialized.residue_defs);
        assert_eq!(database.component_defs, deserialized.component_defs);
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

    #[cfg(unix)]
    #[test]
    fn get_database_path() {
        let mut database = DataBase::new();
        assert_eq!("None", &database.get_path_pretty());

        database.set_path("/a/file.json").unwrap();
        assert_eq!("'/a/file.json'", &database.get_path_pretty());
    }
}
