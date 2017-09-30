//! Edit a `DataBase`.

mod component;
mod residue;

use error::Result;
use ui::utils::{get_value_from_user, print_description, select_command};

use grafen::database::{write_database, DataBase};

use std::error::Error;

#[derive(Clone, Copy, Debug)]
/// Editing commands.
enum DataBaseMenu {
    EditResidues,
    EditComponents,
    WriteToDisk,
    SetLocation,
    QuitAndSave,
    QuitWithoutSaving,
}
use self::DataBaseMenu::*;

pub fn user_menu(database: &mut DataBase) -> Result<String> {
    let (commands, item_texts) = create_menu_items!(
        (EditResidues, "Edit list of residues"),
        (EditComponents, "Edit list of components"),
        (WriteToDisk, "Write database to disk"),
        (SetLocation, "Change output location of database"),
        (QuitAndSave, "Finish editing database"),
        (QuitWithoutSaving, "Abort editing and discard changes")
    );

    let path_backup = database.path.clone();
    let residues_backup = database.residue_defs.clone();
    let components_backup = database.component_defs.clone();

    loop {
        print_description(database);

        let command = select_command(item_texts, commands)?;

        match command {
            EditResidues => {
                match residue::user_menu(&mut database.residue_defs) {
                    Ok(msg) => eprintln!("{}", msg),
                    Err(err) => eprintln!("error: {}", err),
                }
            },
            EditComponents => {
                match component::user_menu(&mut database.component_defs, &database.residue_defs) {
                    Ok(msg) => eprintln!("{}", msg),
                    Err(err) => eprintln!("error: {}", err),
                }
            },
            WriteToDisk => {
                match write_database(&database) {
                    Ok(_) => eprintln!("Wrote database to '{}'.",
                                      database.path.as_ref().unwrap().to_str().unwrap()),
                    Err(err) => eprintln!("Could not write database: {}", err.description()),
                }
            },
            SetLocation => {
                let path = get_value_from_user::<String>("New path")?;
                if let Err(_) = database.set_path(&path) {
                    // TODO: Describe error
                    eprintln!("Could not change database path");
                }
            },
            QuitAndSave => {
                return Ok("Finished editing database".to_string());
            },
            QuitWithoutSaving => {
                database.path = path_backup;
                database.residue_defs = residues_backup;
                database.component_defs = components_backup;

                return Ok("Discarding changes to database".to_string());
            },
        }

        eprintln!("");
    }
}
