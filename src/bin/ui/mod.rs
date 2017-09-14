//! The main user interface from which the user will define systems to create.
//! They can also access and modify the `DataBase` of components to use in their
//! systems.

#[macro_use] mod utils;
mod edit_database;

use super::Config;
use error::{GrafenCliError, Result, UIErrorKind};
use output;
use ui::utils::{get_position_from_user, get_value_from_user, print_description, select_command,
                remove_items, reorder_list, select_item};

use grafen::database::ComponentEntry;
use grafen::system::System;

use std::error::Error;

#[derive(Clone, Copy, Debug)]
/// User commands for defining the system.
enum MainMenu {
    AddComponent,
    RemoveComponent,
    ReorderList,
    SaveSystem,
    EditDatabase,
    Quit,
}
use self::MainMenu::*;

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

    let (commands, item_texts) = create_menu_items![
        (AddComponent, "Construct a component"),
        (RemoveComponent, "Remove a component from the list"),
        (ReorderList, "Reorder list of components"),
        (SaveSystem, "Save the constructed components to disk as a system"),
        (EditDatabase, "Edit the database of residue and object definitions"),
        (Quit, "Quit the program")
    ];

    loop {
        print_description(&system);

        let command = select_command(item_texts, commands)?;

        let result = match command {
            AddComponent => {
                create_component(&mut system)
            },
            RemoveComponent => {
                remove_items(&mut system.components)
                    .map(|_| "Successfully removed component.".to_string())
            },
            ReorderList => {
                reorder_list(&mut system.components)
                    .map(|_| "Successfully reordered list.".to_string())
            },
            EditDatabase => {
                edit_database::user_menu(&mut system.database)
            },
            SaveSystem => {
                output::write_gromos(&system)
                    .map(|_| "Saved system to disk".to_string())
            },
            Quit => {
                return Ok(());
            },
        };

        match result {
            Ok(msg) => { eprintln!("{}", msg); },
            Err(err) => { eprintln!("error: {}", err.description()); },
        }

        eprintln!("");
    }
}

/// Prompt the user to select a defined component from the `DataBase`, then create it.
fn create_component(system: &mut System) -> Result<String> {
    let component = select_item(&system.database.component_defs, Some("Available components"))?
        .clone();

    match fill_component(component) {
        Ok(filled) => {
            system.components.push(filled);
            Ok("Added component to system".to_string())
        },
        Err(err) => Err(err),
    }
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