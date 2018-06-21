//! The main user interface from which the user will define systems to create.
//! They can also access and modify the `DataBase` of components to use in their
//! systems.

#[macro_use] mod utils;
mod edit_component;
mod edit_database;

use super::Config;
use error::{GrafenCliError, Result, UIErrorKind, UIResult};
use output;
use ui::utils::{MenuResult, YesOrNo,
    get_value_from_user, get_value_or_default_from_user, get_coord_from_user,
    get_position_from_user, remove_items, reorder_list, select_command,
    select_direction, select_item};

use grafen::coord::{Coord, Translate};
use grafen::database::*;
use grafen::read_conf::{ConfType, ReadConf};
use grafen::surface::LatticeType;
use grafen::system::*;
use grafen::volume::{FillType, Volume};

use std::path::{Path, PathBuf};

/// Loop over a menu in which the user can define the system which will be created, etc.
///
/// The idea of this interface is relatively simple:
///
/// 1. The user reads or constructs a `DataBase` of residue (`Residue`)
///    and component (`ComponentEntry`) definitions.
/// 2. Using these construct the actual components which make up the system.
/// 3. Modifies or transforms these components by copying, translating, rotating etc.
/// 4. Finally saves the full system to disk.
pub fn user_menu(config: Config) -> Result<()> {
    let mut system = System {
        title: config.title,
        output_path: config.output_path,
        database: config.database,
        components: config.components,
    };

    create_menu![
        @pre: { system.print_state() };

        AddComponent, "Construct a component" => {
            create_component(&mut system)
        },
        EditComponent, "Edit or clone a component" => {
            edit_component::user_menu(&mut system.components)
        },
        RemoveItems, "Remove a component from the list" => {
            remove_items(&mut system.components).map(|_| None)
        },
        ReorderList, "Reorder list of components" => {
            reorder_list(&mut system.components).map(|_| None)
        },
        EditDatabase, "Edit the database of residue and object definitions" => {
            edit_database::user_menu(&mut system.database)
        },
        SaveSystem, "Save the constructed components to disk as a system" => {
            output::write_gromos(&system).map(|_| "Saved system to disk".to_string().into())
        },
        Quit, "Quit the program" => {
            return Ok(());
        }
    ];
}

/// Prompt the user to select a defined component from the `DataBase`, then create it.
fn create_component(system: &mut System) -> MenuResult {
    let component = select_item(&system.database.component_defs, Some("Available components"))?
        .clone();

    match fill_component(component, system.database.path.as_ref()) {
        Ok(filled) => {
            system.components.push(filled);
            Ok(Some("Added component to system".to_string()))
        },
        Err(err) => Err(err),
    }
}

/// Ask the user for information about the selected component, then return the constructed object.
fn fill_component(component: ComponentEntry, database_path: Option<&PathBuf>)
        -> Result<ComponentEntry> {
    match component {
        ComponentEntry::VolumeCuboid(mut conf) => {
            let position = get_position_from_user(Some("0 0 0"))?;
            let length = get_value_from_user::<f64>("Length ΔX (nm)")?;
            let width = get_value_from_user::<f64>("Width ΔY (nm)")?;
            let height = get_value_from_user::<f64>("Height ΔZ (nm)")?;

            let fill_type = select_num_coords_or_density_with_default(conf.density)?;

            conf.origin = position;
            conf.size = Coord::new(length, width, height);

            Ok(ComponentEntry::from(conf.fill(fill_type)))
        },

        ComponentEntry::VolumeCylinder(mut conf) => {
            conf.origin = get_position_from_user(Some("0 0 0"))?;
            conf.radius = get_value_from_user::<f64>("Radius (nm)")?;
            conf.height = get_value_from_user::<f64>("Height (nm)")?;

            let fill_type = select_num_coords_or_density_with_default(conf.density)?;

            Ok(ComponentEntry::from(conf.fill(fill_type)))
        },

        ComponentEntry::VolumeSpheroid(mut conf) => {
            conf.origin = get_position_from_user(Some("0 0 0"))?;
            conf.radius = get_value_from_user::<f64>("Radius (nm)")?;

            let fill_type = select_num_coords_or_density_with_default(conf.density)?;

            Ok(ComponentEntry::from(conf.fill(fill_type)))
        },

        ComponentEntry::SurfaceSheet(mut conf) => {
            conf.origin = get_position_from_user(Some("0 0 0"))?;
            conf.length = get_value_from_user::<f64>("Length ΔX (nm)")?;
            conf.width = get_value_from_user::<f64>("Width ΔY (nm)")?;

            match conf.lattice {
                LatticeType::BlueNoise { ref mut number } => {
                    *number = get_value_from_user::<u64>("Number of residues")?;
                },
                _ => (),
            }

            Ok(
                ComponentEntry::from(
                    conf.construct().map_err(|_| {
                        UIErrorKind::from("Could not construct sheet")
                    })?
                ).with_pbc()
            )
        },

        ComponentEntry::SurfaceCuboid(mut conf) => {
            conf.origin = get_position_from_user(Some("0 0 0"))?;

            let length = get_value_from_user::<f64>("Length ΔX (nm)")?;
            let width = get_value_from_user::<f64>("Width ΔY (nm)")?;
            let height = get_value_from_user::<f64>("Height ΔZ (nm)")?;
            conf.size = Coord::new(length, width, height);

            Ok(ComponentEntry::from(conf.construct().map_err(|_| {
                    UIErrorKind::from("Could not construct cuboid surface")
                })?
            ))
        },

        ComponentEntry::SurfaceCylinder(mut conf) => {
            conf.origin = get_position_from_user(Some("0 0 0"))?;
            conf.radius = get_value_from_user::<f64>("Radius (nm)")?;
            conf.height = get_value_from_user::<f64>("Height (nm)")?;

            Ok(ComponentEntry::from(conf.construct().map_err(|_|
                UIErrorKind::from("Could not construct cylinder")
            )?))
        },

        ComponentEntry::ConfigurationFile(conf) => {
            let default_volume = conf.volume_type.clone();

            let origin = get_position_from_user(Some("0 0 0"))?;

            let to_volume = match default_volume {
                ConfType::Cuboid { origin: _, size: default_size } => {
                    let (x, y, z) = default_size.to_tuple();
                    let size = get_coord_from_user(
                        "Size (x y z nm)", Some(&format!("{} {} {}", x, y, z)))?;

                    ConfType::Cuboid { origin, size }
                },
                ConfType::Cylinder { origin: _, radius, height, normal } => {
                    let radius = get_value_or_default_from_user::<f64>(
                        "Radius (nm)", &format!("{}", radius))?;
                    let height = get_value_or_default_from_user::<f64>(
                        "Height (nm)", &format!("{}", height))?;
                    let normal = select_direction(Some("Select normal"), Some(normal))?;

                    ConfType::Cylinder { origin, radius, height, normal }
                },
                ConfType::Spheroid { origin: _, radius } => {
                    let radius = get_value_or_default_from_user::<f64>(
                        "Radius (nm)", &format!("{}", radius))?;

                    ConfType::Spheroid { origin, radius }
                },
            };

            // If the path is relative, it is relative to the database location.
            // Construct the full path.
            let path = if conf.path.is_absolute() {
                conf.path
            } else {
                database_path
                    .and_then(|db_path| db_path.parent())
                    .map(|db_dir| PathBuf::from(db_dir))
                    // If the database has no path, it has to be relative to
                    // the current directory. Join with an empty path.
                    .unwrap_or(PathBuf::new())
                    .join(conf.path)
            };

            let mut new_conf = read_configuration(&path)?;

            new_conf.description = conf.description;
            new_conf.reconstruct(to_volume);

            // Make sure that the origin is adjusted to that desired by the user.
            let displayed_origin = new_conf.get_displayed_origin();
            new_conf.translate_in_place(origin - displayed_origin);

            Ok(ComponentEntry::from(new_conf))
        },
    }
}

pub fn read_configuration(path: &Path) -> Result<ReadConf> {
    match path.to_str() {
        Some(p) => eprint!("Reading configuration at '{}' ... ", p),
        None => eprint!("Reading configuration with a non-utf8 path ... "),
    }

    let conf = ReadConf::from_gromos87(&path)
        .map_err(|err| GrafenCliError::ReadConfError(format!("Failed! {}.", err)))?;

    eprintln!("Done! Read {} atoms.", conf.num_atoms());

    Ok(conf)
}

fn select_num_coords_or_density_with_default(default_density: Option<f64>) -> UIResult<FillType> {
    match default_density {
        Some(density) => {
            let (commands, item_texts) = create_menu_items![
                (YesOrNo::Yes, "Yes"),
                (YesOrNo::No, "No")
            ];

            eprintln!("Use default density for component ({})?", density);
            let command = select_command(item_texts, commands)?;

            match command {
                YesOrNo::Yes => Ok(FillType::Density(density)),
                YesOrNo::No => select_num_coords_or_density(),
            }
        },
        None => {
            select_num_coords_or_density()
        },
    }
}

fn select_num_coords_or_density() -> UIResult<FillType> {
    create_menu![
        @pre: { };

        Density, "Use density" => {
            let density = get_value_from_user::<f64>("Density (1/nm^3)")?;

            if density > 0.0 {
                return Ok(FillType::Density(density));
            } else {
                Err(GrafenCliError::ConstructError("Invalid density: it must be positive".into()))
            }
        },
        NumCoords, "Use a specific number of residues" => {
            let num_coords = get_value_from_user::<u64>("Number of residues")?;

            return Ok(FillType::NumCoords(num_coords));
        }
    ];
}
