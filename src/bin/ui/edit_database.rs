//! Edit a `DataBase`.

use database::{write_database, DataBase};
use error::Result;
use ui::utils;
use ui::utils::{CommandList, CommandParser};

use std::error::Error;

#[derive(Clone, Copy, Debug)]
enum Command {
    AddResidue,
    RemoveResidue,
    AddSubstrate,
    RemoveSubstrate,
    WriteToDisk,
    SetLocation,
    ShowDatabase,
    QuitAndSave,
    QuitWithoutSaving,
}

pub fn user_menu(database: &mut DataBase) -> Result<&'static str> {
    let command_list: CommandList<Command> = vec![
        ("ra", Command::AddResidue, "Add a residue definition"),
        ("rr", Command::RemoveResidue, "Remove a residue definition"),
        ("sa", Command::AddSubstrate, "Add a substrate definition"),
        ("sr", Command::RemoveSubstrate, "Remove a substrate definition"),
        ("w", Command::WriteToDisk, "Write database to disk"),
        ("c", Command::SetLocation, "Change output location of database"),
        ("l", Command::ShowDatabase, "List database content"),
        ("f", Command::QuitAndSave, "Finish editing database"),
        ("a", Command::QuitWithoutSaving, "Abort editing and discard changes"),
    ];
    let commands = CommandParser::from_list(command_list);

    let path_backup = database.path.clone();
    let residues_backup = database.residue_defs.clone();
    let substrates_backup = database.substrate_defs.clone();

    println!("Editing the current database.\n");
    database.describe();
    println!("");

    loop {
        commands.print_menu();
        let input = utils::get_input_string("Selection")?;

        if let Some((cmd, tail)) = commands.get_selection_and_tail(&input) {
            match cmd {
                Command::AddResidue => {
                    match define_residue::user_menu() {
                        Ok(residue) => {
                            println!("Added residue '{}' to database.", residue.code);
                            database.residue_defs.push(residue);
                        },
                        Err(err) => println!("Could not create residue: {}", err.description()),
                    }
                },
                Command::RemoveResidue => {
                    match utils::remove_item(&mut database.residue_defs, &tail) {
                        Ok(i) => println!("Removed residue with index {} from database.", i),
                        Err(err) => println!("Could not remove residue: {}", err.description()),
                    }
                },
                Command::AddSubstrate => {
                    println!("Unimplemented!");
                },
                Command::RemoveSubstrate => {
                    match utils::remove_item(&mut database.substrate_defs, &tail) {
                        Ok(i) => println!("Removed substrate with index {} from database.", i),
                        Err(err) => println!("Could not remove substrate: {}", err.description()),
                    }
                },
                Command::WriteToDisk => {
                    match write_database(&database) {
                        Ok(_) => println!("Wrote database to '{}'.",
                                          database.path.as_ref().unwrap().to_str().unwrap()),
                        Err(err) => println!("Could not write database: {}", err.description()),
                    }
                },
                Command::SetLocation => {
                    match database.set_path(&tail) {
                        Ok(_) => println!("Database path set to '{}'.",
                                          database.get_path_pretty()),
                        Err(err) => println!("Could not change database path: {}",
                                             err.description()),
                    }
                },
                Command::ShowDatabase => {
                    println!("");
                    database.describe();
                },
                Command::QuitAndSave => {
                    return Ok("Finished editing database.");
                },
                Command::QuitWithoutSaving => {
                    database.path = path_backup;
                    database.residue_defs = residues_backup;
                    database.substrate_defs = substrates_backup;

                    return Ok("Discarding changes to database.");
                },
            }
        } else {
            println!("Not a valid selection.");
        }

        println!("");
    }
}

mod define_residue {
    use error::{Result, GrafenCliError, UIErrorKind};
    use ui::utils;
    use ui::utils::{CommandList, CommandParser};

    use grafen::system::{Atom, Coord, ResidueBase};
    use std::error::Error;

    #[derive(Clone, Copy, Debug)]
    enum ResidueCommand {
        SetName,
        AddAtom,
        RemoveAtom,
        SwapAtoms,
        ShowResidue,
        QuitAndSave,
        QuitWithoutSaving,
    }

    pub fn user_menu() -> Result<ResidueBase> {
        let command_list: CommandList<ResidueCommand> = vec![
            ("n", ResidueCommand::SetName, "Set residue name"),
            ("at", ResidueCommand::AddAtom, "Add atom to residue"),
            ("r", ResidueCommand::RemoveAtom, "Remove atom from residue"),
            ("s", ResidueCommand::SwapAtoms, "Swap two atoms in list"),
            ("l", ResidueCommand::ShowResidue, "List current residue data"),
            ("f", ResidueCommand::QuitAndSave, "Finish and add residue to list"),
            ("a", ResidueCommand::QuitWithoutSaving, "Abort and discard changes")
        ];
        let commands = CommandParser::from_list(command_list);

        println!("Creating a new residue.\n");

        let mut name = String::new();
        let mut atoms: Vec<Atom> = Vec::new();

        loop {
            commands.print_menu();
            let input = utils::get_input_string("Selection")?;
            println!("");

            if let Some((cmd, tail)) = commands.get_selection_and_tail(&input) {
                match cmd {
                    ResidueCommand::SetName => {
                        name = tail.to_uppercase().to_string();
                        println!("Set residue name to '{}'", &name);
                    },
                    ResidueCommand::AddAtom => {
                        match parse_string_for_atom(&tail) {
                            Ok(atom) => {
                                println!("Added atom '{}' to residue.", &atom.code);
                                atoms.push(atom);
                            },
                            Err(err) => println!("Could not add atom: {}", err.description()),
                        }

                    },
                    ResidueCommand::RemoveAtom => {
                        match utils::remove_item(&mut atoms, &tail) {
                            Ok(i) => println!("Removed atom with index {} from residue.", i),
                            Err(err) => println!("Could not remove atom: {}", err.description()),
                        }
                    },
                    ResidueCommand::SwapAtoms => {
                        match utils::swap_items(&mut atoms, &tail) {
                            Ok((i, j)) => println!("Swapped atoms at index {} with atom at {}.",
                                                   i, j),
                            Err(err) => println!("Could not swap atoms: {}", err.description()),
                        }
                    },
                    ResidueCommand::ShowResidue => {
                        describe_residue(&name, &atoms);
                    },
                    ResidueCommand::QuitAndSave => {
                        if name.is_empty() {
                            println!("Cannot add residue: No name is set");
                        } else if atoms.is_empty() {
                            println!("Cannot add residue: No atoms are set");
                        } else {
                            return Ok(ResidueBase {
                                code: name,
                                atoms: atoms,
                            });
                        }
                    }
                    ResidueCommand::QuitWithoutSaving => {
                        return Err(GrafenCliError::from(UIErrorKind::Abort));
                    },
                }
            } else {
                println!("Not a valid selection.");
            }

            println!("");
        }
    }

    fn parse_string_for_atom<'a>(input: &'a str) -> Result<Atom> {
        let mut iter = input.splitn(2, ' ');
        let name = iter.next().and_then(|s| {
                if s.is_empty() {
                    None
                } else {
                    Some(s)
                }
            })
            .ok_or(UIErrorKind::BadValue("No name was given".to_string()))?;

        let tail = iter.next().unwrap_or("");

        let coords = utils::parse_string(&tail)?;
        let &x = coords.get(0)
                       .ok_or(UIErrorKind::BadValue("3 positions are required".to_string()))?;
        let &y = coords.get(1)
                       .ok_or(UIErrorKind::BadValue("3 positions are required".to_string()))?;
        let &z = coords.get(2)
                       .ok_or(UIErrorKind::BadValue("3 positions are required".to_string()))?;

        Ok(Atom {
            code: name.to_uppercase().to_string(),
            position: Coord::new(x, y, z),
        })
    }

    fn describe_residue<'a>(name: &'a str, atoms: &[Atom]) {
        println!("Residue name: '{}'", name);
        println!("Atoms:");
        for (i, atom) in atoms.iter().enumerate() {
            let (x, y, z) = atom.position.to_tuple();
            println!("{:4}. {} at ({:.1}, {:.1}, {:.1})", i, atom.code, x, y, z);
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn parse_atom_string() {
            let atom = Atom { code: "A1".to_string(), position: Coord::new(1.0, 2.0, 0.0) };
            assert_eq!(atom, parse_string_for_atom("A1 1 2 0").unwrap());
        }

        #[test]
        fn parse_atoms_without_name_or_some_values_is_error() {
            assert!(parse_string_for_atom("\t\n").is_err());
            assert!(parse_string_for_atom("\tname\n 1.0").is_err());
            assert!(parse_string_for_atom("\tname 1.0\t2.0").is_err());
            assert!(parse_string_for_atom("\tname 1.0 2.0 3").is_ok());
            assert!(parse_string_for_atom("\tname 1.0 2.0 a").is_err());
        }

        #[test]
        fn parse_atoms_sets_name_to_uppercase() {
            assert_eq!("AT1", parse_string_for_atom("at1 1 2 0").unwrap().code);
        }
    }
}

mod define_substrate {

}
