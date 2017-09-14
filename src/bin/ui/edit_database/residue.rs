//! Modify the list of `Residue` objects in a `DataBase`.

use error::{GrafenCliError, Result, UIErrorKind, UIResult};
use ui::utils::{remove_items, reorder_list, select_command, get_position_from_user,
                get_value_from_user, print_list_description};

use grafen::describe::describe_list;
use grafen::system::{Atom, Residue};

use std::error::Error;
use std::result;

#[derive(Clone, Copy, Debug)]
enum ResidueMenu {
    AddResidue,
    RemoveResidue,
    ReorderList,
    QuitAndSave,
    QuitWithoutSaving,
}
use self::ResidueMenu::*;

pub fn user_menu(mut residue_list: &mut Vec<Residue>) -> Result<String> {
    let (commands, item_texts) = create_menu_items!(
        (AddResidue, "Add a residue definition"),
        (RemoveResidue, "Remove residue definitions"),
        (ReorderList, "Reorder residue list"),
        (QuitAndSave, "Finish editing residue list"),
        (QuitWithoutSaving, "Abort and discard changes")
    );

    let residues_backup = residue_list.clone();

    loop {
        print_list_description("Residue definitions", &residue_list);

        let command = select_command(item_texts, commands)?;

        match command {
            AddResidue => {
                match new_residue() {
                    Ok(residue) => {
                        residue_list.push(residue);
                        eprintln!("Successfully created residue");
                    },
                    Err(err) => eprintln!("Could not create residue: {}", err.description()),
                }
            },
            RemoveResidue => {
                if let Err(err) = remove_items(&mut residue_list) {
                    eprintln!("error: Something went wrong when removing a residue ({})", err);
                }
            },
            ReorderList => {
                if let Err(err) = reorder_list(&mut residue_list) {
                    eprintln!("error: Something went wrong when reordering the list ({})", err);
                }
            },
            QuitAndSave => {
                return Ok("Finished editing residue list".to_string());
            },
            QuitWithoutSaving => {
                *residue_list = residues_backup;
                return Ok("Discarding changes to residue list".to_string());
            }
        }

        eprintln!("");
    }
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

#[derive(Clone, Copy, Debug)]
enum NewResidueMenu {
    SetName,
    AddAtom,
    RemoveAtom,
    ReorderAtoms,
    QuitAndAddResidue,
    QuitWithoutAddingResidue,
}
use self::NewResidueMenu::*;

fn new_residue() -> Result<Residue> {
    let (commands, item_texts) = create_menu_items!(
        (SetName, "Set residue name"),
        (AddAtom, "Add atom to residue"),
        (RemoveAtom, "Remove atom from residue"),
        (ReorderAtoms, "Reorder atom list"),
        (QuitAndAddResidue, "Finish and add residue to list"),
        (QuitWithoutAddingResidue, "Abort and discard changes")
    );

    println!("Creating a new residue.\n");
    let mut builder = ResidueBuilder::new();

    loop {
        builder.print_state();

        let command = select_command(item_texts, commands)?;

        match command {
            SetName => {
                match get_value_from_user::<String>("Residue name") {
                    Ok(new_name) => {
                        builder.name = new_name;
                    },
                    Err(_) => eprintln!("error: Could not read name"),
                }
            },
            AddAtom => {
                match create_atom() {
                    Ok(atom) => {
                        builder.atoms.push(atom);
                    },
                    // TODO: This should print an error description, too
                    //Err(err) => println!("Could not add atom: {}", err.description()),
                    Err(_) => eprintln!("Could not add atom"),
                }
            },
            RemoveAtom => {
                eprintln!("Remove atoms:");
                if let Err(err) = remove_items(&mut builder.atoms) {
                    eprintln!("Could not remove atom: {}", err);
                }
            },
            ReorderAtoms => {
                eprintln!("Reorder atoms:");
                if let Err(err) = reorder_list(&mut builder.atoms) {
                    eprintln!("error: Something went wrong when reordering the list ({})", err);
                }
            },
            QuitAndAddResidue => {
                match builder.finalize() {
                    Ok(residue) => return Ok(residue),
                    Err(msg) => eprintln!("{}", msg),
                }
            }
            QuitWithoutAddingResidue => {
                return Err(GrafenCliError::from(UIErrorKind::Abort));
            },
        }

        eprintln!("");
    }
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