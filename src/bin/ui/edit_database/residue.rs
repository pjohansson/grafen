//! Modify the list of `Residue` objects in a `DataBase`.

use error::{GrafenCliError, Result, UIErrorKind, UIResult};
use ui::utils::{MenuResult, remove_items, reorder_list, select_command, get_position_from_user,
                get_value_from_user, print_list_description};

use grafen::describe::describe_list;
use grafen::system::{Atom, Residue};

use std::error::Error;
use std::result;

pub fn user_menu(mut residue_list: &mut Vec<Residue>) -> MenuResult {
    let residues_backup = residue_list.clone();

    create_menu![
        @pre: { print_list_description("Residue definitions", &residue_list); };

        AddResidue, "Add a residue definition" => {
            new_residue()
                .map(|residue| {
                    residue_list.push(residue);
                    Some("Successfully created residue".to_string())
                })
                .map_err(|err| GrafenCliError::RunError(
                    format!("Could not create residue: {}", err.description())
                ))
        },
        RemoveResidue, "Remove residue definitions" => {
            remove_items(&mut residue_list)
                .map(|_| None)
                .map_err(|err| GrafenCliError::RunError(
                    format!("Could not remove a residue: {}", err.description())
                ))
        },
        ReorderList, "Reorder residue list" => {
            reorder_list(&mut residue_list)
                .map(|_| None)
                .map_err(|err| GrafenCliError::RunError(
                    format!("Could not reorder the list: {}", err.description())
                ))
        },
        QuitAndSave, "Finish editing residue list" => {
            return Ok("Finished editing residue list".to_string().into());
        },
        QuitWithoutSaving, "Abort and discard changes" => {
            *residue_list = residues_backup;
            return Ok("Discarding changes to residue list".to_string().into());
        }
    ];
}

struct ResidueBuilder {
    name: String,
    atoms: Vec<Atom>,
}

impl ResidueBuilder {
    fn new() -> ResidueBuilder {
        ResidueBuilder {
            name: String::new(),
            atoms: vec![],
        }
    }

    fn finalize(&self) -> result::Result<Residue, &str> {
        if self.name.is_empty() {
            Err("Cannot add residue: No name is set")
        } else if self.atoms.is_empty() {
            Err("Cannot add residue: No atoms are set")
        } else {
            Ok(Residue {
                code: self.name.clone(),
                atoms: self.atoms.clone(),
            })
        }
    }

    fn print_state(&self) {
        eprintln!("Name: {}", self.name);
        eprintln!("{}", describe_list("Atoms", &self.atoms));
    }
}

fn new_residue() -> Result<Residue> {
    println!("Creating a new residue.\n");
    let mut builder = ResidueBuilder::new();

    create_menu![
        @pre: { builder.print_state(); };

        SetName, "Set residue name" => {
            match get_value_from_user::<String>("Residue name") {
                Ok(new_name) => {
                    builder.name = new_name;
                    Ok(None)
                },
                Err(_) => Err(GrafenCliError::RunError("Could not read name".to_string()))
            }
        },
        AddAtom, "Add atom to residue" => {
            match create_atom() {
                Ok(atom) => {
                    builder.atoms.push(atom);
                    Ok(None)
                },
                // TODO: This should print an error description, too
                //Err(err) => println!("Could not add atom: {}", err.description()),
                Err(_) => Err(GrafenCliError::RunError("Could not add atom".to_string()))
            }
        },
        RemoveAtom, "Remove atom from residue" => {
            match remove_items(&mut builder.atoms) {
                Ok(_) => Ok(None),
                Err(err) => Err(GrafenCliError::RunError(
                    format!("Could not remove atom: {}", err)
                ))
            }
        },
        ReorderAtoms, "Reorder atom list" => {
            match reorder_list(&mut builder.atoms) {
                Ok(_) => Ok(None),
                Err(err) => Err(GrafenCliError::RunError(
                    format!("Something went wrong when reordering the list ({})", err)
                ))
            }
        },
        QuitAndAddResidue, "Finish and add residue to list" => {
            match builder.finalize() {
                Ok(residue) => return Ok(residue),
                Err(msg) => Err(GrafenCliError::RunError(format!("{}", msg))),
            }
        },
        QuitWithoutAddingResidue, "Abort and discard changes" => {
            return Err(GrafenCliError::from(UIErrorKind::Abort));
        }
    ];
}

fn create_atom() -> UIResult<Atom> {
    let name = get_value_from_user::<String>("Atom name")?;
    let position = get_position_from_user(None)?;

    Ok(Atom {
        code: name.to_uppercase().to_string(),
        position: position,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use grafen::coord::Coord;

    #[test]
    fn residue_builder_is_ok_if_all_are_set() {
        let mut builder = ResidueBuilder {
            name: "".to_string(),
            atoms: vec![],
        };

        assert!(builder.finalize().is_err());

        builder.name = "is_set".to_string();
        assert!(builder.finalize().is_err());

        builder.atoms.push(Atom { code: "A".to_string(), position: Coord::ORIGO });
        assert!(builder.finalize().is_ok());
    }
}
