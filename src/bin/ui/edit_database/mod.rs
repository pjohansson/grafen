//! Edit a `DataBase`.

mod component;
mod residue;

use error::GrafenCliError;
use ui::utils::{MenuResult, print_description, select_command};

use grafen::database::{write_database, DataBase};
use std::error::Error;

pub fn user_menu(database: &mut DataBase) -> MenuResult {
    let path_backup = database.path.clone();
    let residues_backup = database.residue_defs.clone();
    let components_backup = database.component_defs.clone();

    create_menu![
        @pre: { print_description(database); };

        EditResidues, "Edit list of residues" => {
            residue::user_menu(&mut database.residue_defs).map(|msg| msg.into())
        },
        EditComponents, "Edit list of components" => {
            component::user_menu(&mut database.component_defs, &database.residue_defs)
                .map(|msg| msg.into())
        },
        WriteToDisk, "Write changes of database to disk" => {
            write_database(&database)
                .map(|_| String::from("Successfully wrote changes of database to disk.").into())
                .map_err(|err| GrafenCliError::RunError(
                    format!("Could not write database: {}", err.description())
                ))
        },
        QuitAndSave, "Finish editing database" => {
            return Ok("Finished editing database".to_string().into());
        },
        QuitWithoutSaving, "Abort editing and discard changes" => {
            database.path = path_backup;
            database.residue_defs = residues_backup;
            database.component_defs = components_backup;

            return Ok("Discarding changes to database".to_string().into());
        }
    ];
}
