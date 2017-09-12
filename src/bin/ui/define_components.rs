//! Define a system components to create.
//!
//! This interface could use a lot of improvement.

use database::{AvailableComponents, CylinderClass, DataBase};
use error::{Result, UIResult};
use ui::utils;

use dialoguer::Input;

#[derive(Clone, Copy, Debug)]
/// User commands for defining the system.
enum DefineMenu {
    DefineSystem,
    RemoveSystem,
    ReorderList,
    QuitAndSave,
    QuitWithoutSaving,
}
use self::DefineMenu::*;

/// Edit the list of system definitions to construct from.
pub fn user_menu(database: &DataBase, mut system_defs: &mut Vec<AvailableComponents>)
        -> Result<()> {
    let (commands, item_texts) = create_menu_items![
        (DefineSystem, "Define a system to create"),
        (RemoveSystem, "Remove a system from the list"),
        (ReorderList, "Swap the order of two systems"),
        (QuitAndSave, "Finalize editing and return"),
        (QuitWithoutSaving, "Abort and discard changes to list")
    ];

    let backup = system_defs.clone();

    loop {
        utils::print_group("Defined components", &system_defs);
        let command = utils::select_command(item_texts, commands)?;

        match command {
            DefineSystem => {
                match create_definition(&database) {
                    Ok(def) => system_defs.push(def),
                    // TODO: This should give an error description once a proper error class
                    // for UI Errors has been added.
                    Err(_) => println!("Could not create definition"),
                }
            },
            RemoveSystem => {
                if let Err(err) = utils::remove_items(&mut system_defs) {
                    println!("error: Something went wrong when removing a system ({})", err);
                }
            },
            ReorderList => {
                if let Err(err) = utils::reorder_list(&mut system_defs) {
                    println!("error: Something went wrong when reordering the list ({})", err);
                }
            },
            QuitAndSave => {
                return Ok(());
            },
            QuitWithoutSaving => {
                system_defs.clear();
                system_defs.extend_from_slice(&backup);

                return Ok(());
            },
        };
    }
}


/// Prompt the user to fill in the missing information for a definition.
fn create_definition(database: &DataBase) -> UIResult<AvailableComponents> {
    use database::AvailableComponents::*;

    println!("Available components:");
    let selection = utils::select_item(&database.component_defs, 0)?;
    let component = database.component_defs[selection].clone();

    match component {
        Sheet(mut def) => {
            let position = utils::get_position_from_user(Some("0 0 0"))?;
            let size = select_size()?;

            def.position = Some(position);
            def.size = Some(size);

            Ok(Sheet(def))
        },
        Cylinder(mut def) => {
            let position = utils::get_position_from_user(Some("0 0 0"))?;
            let radius = Input::new("Radius (nm)").interact()?.parse::<f64>()?;
            let height = Input::new("Height (nm)").interact()?.parse::<f64>()?;

            // For volumes we need a number of residues to fill it with which
            // isn't necessarily saved in the database.
            if let CylinderClass::Volume(opt_num_residues) = def.class {
                let mut input = Input::new("Number of residues");

                // In case a default value exists in the database
                if let Some(default_num_residues) = opt_num_residues {
                    input.default(&format!("{}", default_num_residues));
                }

                let num_residues = input.interact()?.parse::<usize>()?;
                def.class = CylinderClass::Volume(Some(num_residues));
            }

            def.position = Some(position);
            def.radius = Some(radius);
            def.height = Some(height);

            Ok(Cylinder(def))
        }
    }
}

/// Get a 2D size.
fn select_size() -> UIResult<(f64, f64)> {
    let dx = Input::new("Length ΔX (nm)").interact()?.parse::<f64>()?;
    let dy = Input::new("Width ΔY (nm)").interact()?.parse::<f64>()?;

    Ok((dx, dy))
}
