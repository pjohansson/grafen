//! The main user interface from which the user will define systems to create.
//! They can also access and modify the `DataBase` of components to use in their
//! systems.

// trace_macros!(true);

#[macro_use] mod utils;
mod edit_database;

use super::Config;
use error::{GrafenCliError, Result, UIErrorKind};
use output;
use ui::utils::{MenuResult, get_value_from_user, get_position_from_user, print_description,
                remove_items, reorder_list, select_command, select_item};

use grafen::database::*;
use grafen::system::*;

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
        components: vec![],
    };

    create_menu![
        @pre: { print_description(&system); };

        AddComponent, "Construct a component" => {
            create_component(&mut system)
        },
        EditComponent, "Edit a component" => {
            edit_component(&mut system.components)
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

    match fill_component(component) {
        Ok(filled) => {
            system.components.push(filled);
            Ok(Some("Added component to system".to_string()))
        },
        Err(err) => Err(err),
    }
}

use ui::utils::select_item_index;
use grafen::coord::Translate;

/// Prompt the user to select a defined component and then edit it.
fn edit_component(components: &mut [ComponentEntry]) -> MenuResult {
    // The component should be a mutable reference to the object in the list,
    // since we want to edit it in-place.
    eprintln!("Select component to edit:");
    let index = select_item_index(components, 0)?;
    let mut component = components[index].clone();

    create_menu![
        @pre: {
            eprint!("State: ");
            print_description(&component);
            eprint!("\n");
        };

        Translate, "Translate the component" => {
            let coord = get_position_from_user(None)?;
            component.translate_in_place(coord);
            Ok(None)
        },
        PruneByVolume, "Remove residues which overlap another component" => {
            let volume: Result<Cuboid> = get_volume_from_user(components);
            Ok(None)
        },
        QuitAndSave, "Finish editing component" => {
            components[index] = component;
            return Ok(Some("Finished editing component".to_string()));
        },
        QuitWithoutSaving, "Abort editing and discard changes" => {
            return Ok(Some("Discarding changes to component".to_string()));
        }
    ];
}

use grafen::describe::Describe;
use grafen::volume::{Contains, Cuboid, Cylinder};

fn get_volume_from_user<T: Contains>(components: &[ComponentEntry]) -> Result<T> {
    // let volume_components = get_volume_objects(components);
    // let component = select_item(&volume_components, Some("Select component to cut with"))?;

    unimplemented!();
}

/// Prune the list of components to only return those that are volumes, without their
/// coordinates since we don't want to copy those.
fn get_volume_objects<T>(components: &[ComponentEntry])
        -> Vec<Box<T>> where T: Contains + Describe {
    components.iter()
              // Explicitly write the closure return type as pointers to `Contains`,
              // otherwise the compiler believes that the closure returns a pointer
              // to a not-generic object type that implements the trait.
              .filter_map(|comp| -> Option<Box<Contains + Describe>> {
                  match comp {
                      &ComponentEntry::VolumeCuboid(ref obj) => {
                          let volume = Cuboid {
                              name: obj.name.clone(),
                              residue: obj.residue.clone(),
                              origin: obj.origin,
                              size: obj.size,
                              coords: vec![],
                          };

                          Some(Box::new(volume))
                      },

                      &ComponentEntry::VolumeCylinder(ref obj) => {
                          let volume = Cylinder {
                              name: obj.name.clone(),
                              residue: obj.residue.clone(),
                              origin: obj.origin,
                              radius: obj.radius,
                              height: obj.height,
                              alignment: obj.alignment,
                              coords: vec![],
                          };

                          Some(Box::new(volume))
                      },

                      _ => None,
                  }
              })
              .collect::<Vec<Box<T>>>()
}

/// Ask the user for information about the selected component, then return the constructed object.
fn fill_component(component: ComponentEntry) -> Result<ComponentEntry> {
    match component {
        ComponentEntry::VolumeCuboid(_) => {
            /*
            let position = get_position_from_user(Some("0 0 0"))?;
            let length = get_value_from_user::<f64>("Length ΔX (nm)")?;
            let width = get_value_from_user::<f64>("Width ΔY (nm)")?;
            let height = get_value_from_user::<f64>("Height ΔZ (nm)")?;
            let num_residues = get_value_from_user::<f64>("Number of residues")?;

            conf.origin = position;
            conf.size = Coord::new(length, width, height);
            */

            Err(GrafenCliError::ConstructError("Cuboid volumes are not yet implemented".to_string()))
        },

        ComponentEntry::VolumeCylinder(mut conf) => {
            conf.origin = get_position_from_user(Some("0 0 0"))?;
            conf.radius = get_value_from_user::<f64>("Radius (nm)")?;
            conf.height = get_value_from_user::<f64>("Height (nm)")?;
            let num_residues = get_value_from_user::<u64>("Number of residues")?;

            Ok(ComponentEntry::VolumeCylinder(conf.fill(num_residues)))
        },

        ComponentEntry::SurfaceSheet(mut conf) => {
            conf.origin = get_position_from_user(Some("0 0 0"))?;
            conf.length = get_value_from_user::<f64>("Length ΔX (nm)")?;
            conf.width = get_value_from_user::<f64>("Width ΔY (nm)")?;

            Ok(ComponentEntry::SurfaceSheet(conf.construct().map_err(|_|
                UIErrorKind::from("Could not construct sheet")
            )?))
        },

        ComponentEntry::SurfaceCylinder(mut conf) => {
            conf.origin = get_position_from_user(Some("0 0 0"))?;
            conf.radius = get_value_from_user::<f64>("Radius (nm)")?;
            conf.height = get_value_from_user::<f64>("Height (nm)")?;

            Ok(ComponentEntry::SurfaceCylinder(conf.construct().map_err(|_|
                UIErrorKind::from("Could not construct cylinder")
            )?))
        },
    }
}
