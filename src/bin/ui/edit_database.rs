//! Edit a `DataBase`.

use error::Result;
use ui::utils::{remove_items, select_command};

use grafen::database::{write_database, DataBase};

use dialoguer::Input;
use std::error::Error;

#[derive(Clone, Copy, Debug)]
/// Editing commands.
enum DataBaseMenu {
    AddResidue,
    RemoveResidue,
    AddComponent,
    RemoveComponent,
    WriteToDisk,
    SetLocation,
    QuitAndSave,
    QuitWithoutSaving,
}
use self::DataBaseMenu::*;

pub fn user_menu(database: &mut DataBase) -> Result<String> {
    let (commands, item_texts) = create_menu_items!(
        (AddResidue, "Add a residue definition"),
        (RemoveResidue, "Remove a residue definition"),
        (AddComponent, "Add a component definition"),
        (RemoveComponent, "Remove a component definition"),
        (WriteToDisk, "Write database to disk"),
        (SetLocation, "Change output location of database"),
        (QuitAndSave, "Finish editing database"),
        (QuitWithoutSaving, "Abort editing and discard changes")
    );

    let path_backup = database.path.clone();
    let residues_backup = database.residue_defs.clone();
    let components_backup = database.component_defs.clone();

    loop {
        database.describe();

        let command = select_command(item_texts, commands)?;

        match command {
            AddResidue => {
                match define_residue::user_menu() {
                    Ok(residue) => {
                        database.residue_defs.push(residue);
                    },
                    Err(err) => eprintln!("Could not create residue: {}", err.description()),
                }
            },
            RemoveResidue => {
                if let Err(err) = remove_items(&mut database.residue_defs) {
                    eprintln!("error: Something went wrong when removing a residue ({})", err);
                }
            },
            AddComponent => {
                match define_component::user_menu(&database.residue_defs) {
                    Ok(component) => {
                        database.component_defs.push(component);
                    },
                    // TODO: Add description of error for UIErrorKind
                    Err(_) => eprintln!("Could not create component"),
                }
            },
            RemoveComponent => {
                if let Err(err) =  remove_items(&mut database.component_defs) {
                    eprintln!("error: Something went wrong when removing a component ({})", err);
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
                let path = Input::new("New path").interact()?;
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
    }
}

#[macro_use]
mod define_residue {
    //! Define a new `ResidueBase`.

    use error::{GrafenCliError, Result, UIErrorKind, UIResult};
    use ui::utils::{remove_items, reorder_list, select_command, get_position_from_user};

    use grafen::describe::describe_list;
    use grafen::system::{Atom, ResidueBase};

    use dialoguer::Input;
    use std::result;

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

        fn finalize(&self) -> result::Result<ResidueBase, &str> {
            if self.name.is_empty() {
                Err("Cannot add residue: No name is set")
            } else if self.atoms.is_empty() {
                Err("Cannot add residue: No atoms are set")
            } else {
                Ok(ResidueBase {
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
    enum ResidueMenu {
        SetName,
        AddAtom,
        RemoveAtom,
        ReorderAtoms,
        QuitAndSave,
        QuitWithoutSaving,
    }
    use self::ResidueMenu::*;

    pub fn user_menu() -> Result<ResidueBase> {
        let (commands, item_texts) = create_menu_items!(
            (SetName, "Set residue name"),
            (AddAtom, "Add atom to residue"),
            (RemoveAtom, "Remove atom from residue"),
            (ReorderAtoms, "Reorder atom list"),
            (QuitAndSave, "Finish and add residue to list"),
            (QuitWithoutSaving, "Abort and discard changes")
        );

        println!("Creating a new residue.\n");
        let mut builder = ResidueBuilder::new();

        loop {
            builder.print_state();

            let command = select_command(item_texts, commands)?;

            match command {
                SetName => {
                    match Input::new("Residue name").interact() {
                        Ok(new_name) => {
                            builder.name = new_name.trim().to_string();
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
                QuitAndSave => {
                    match builder.finalize() {
                        Ok(residue) => return Ok(residue),
                        Err(msg) => eprintln!("{}", msg),
                    }
                }
                QuitWithoutSaving => {
                    return Err(GrafenCliError::from(UIErrorKind::Abort));
                },
            }

            eprintln!("");
        }
    }

    fn create_atom() -> UIResult<Atom> {
        let name = Input::new("Atom name").interact()?.trim().to_string();
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
}

mod define_component {
    use error::{UIResult, UIErrorKind};
    use ui::utils::select_command;

    use dialoguer::{Checkboxes, Input, Select};
    use grafen::database::{AvailableComponents, Direction, CylinderCap, CylinderClass, CylinderConfEntry, SheetConfEntry};
    use grafen::substrate::LatticeType;
    use grafen::system::ResidueBase;
    use std::result;

    /***********************
     * Component selection *
     ***********************/

    /// Error enum to handle the case when we return to a previous menu to change a component,
    /// not because an error was encountered.
    enum ChangeOrError {
        ChangeComponent,
        Error(UIErrorKind),
    }

    impl From<UIErrorKind> for ChangeOrError {
        fn from(err: UIErrorKind) -> ChangeOrError {
            ChangeOrError::Error(err)
        }
    }

    #[derive(Clone, Copy, Debug)]
    /// Available component types.
    enum ComponentSelect {
        Sheet,
        Cylinder,
        Abort,
    }
    use self::ComponentSelect::*;


     /// This menu changes the main component type and then calls that type's construction menu.
    pub fn user_menu(residue_list: &[ResidueBase]) -> UIResult<AvailableComponents> {
        loop {
            let component_type = select_component_type()?;

            let result = match component_type {
                Sheet => create_sheet(&residue_list),
                Cylinder => create_cylinder(&residue_list),
                Abort => return Err(UIErrorKind::Abort),
            };

            match result {
                // All is good!
                Ok(component) => return Ok(component),

                // User asked to change a component.
                Err(ChangeOrError::ChangeComponent) => (),

                // User aborted the component creation.
                Err(ChangeOrError::Error(UIErrorKind::Abort)) => return Err(UIErrorKind::Abort),

                // Something went wrong when constructing a component. Reloop the menu.
                Err(ChangeOrError::Error(_)) => eprintln!("could not create component"),
            }
        }
    }

    fn select_component_type() -> UIResult<ComponentSelect> {
        let (choices, item_texts) = create_menu_items![
            (Sheet, "Sheet"),
            (Cylinder, "Cylinder"),
            (Abort, "(Abort)")
        ];

        eprintln!("Component type:");
        select_command(item_texts, choices).map_err(|err| UIErrorKind::from(err))
    }

    /**********************
     * Sheet construction *
     **********************/

    #[derive(Clone, Copy, Debug)]
    enum SheetMenu {
        ChangeComponent,
        SetName,
        SetLattice,
        SetResidue,
        SetVarianceZ,
        QuitAndSave,
        QuitWithoutSaving,
    }

    struct SheetBuilder {
        name: String,
        lattice: LatticeType,
        residue: ResidueBase,
        std_z: Option<f64>,
    }

    impl SheetBuilder {
        fn initialize(residue_list: &[ResidueBase]) -> UIResult<SheetBuilder> {
            let lattice = select_lattice()?;
            let residue = select_residue(&residue_list)?;

            Ok(SheetBuilder {
                name: String::new(),
                lattice,
                residue,
                std_z: None,
            })
        }

        fn finalize(&self) -> result::Result<AvailableComponents, &str> {
            if self.name.is_empty() {
                return Err("Cannot add component: No name is set")
            } else {
                Ok(AvailableComponents::Sheet(SheetConfEntry {
                    name: self.name.clone(),
                    lattice: self.lattice.clone(),
                    residue: self.residue.clone(),
                    std_z: self.std_z,
                    size: None,
                    position: None,
                }))
            }
        }

        fn print_state(&self) {
            eprintln!("");
            eprintln!("Name: {}", &self.name);
            eprintln!("Lattice: {:?}", &self.lattice);
            eprintln!("Residue: {}", &self.residue.code);
            eprintln!("Z-variance: {}", &self.std_z.unwrap_or(0.0));
            eprintln!("");
        }
    }

    fn create_sheet(residue_list: &[ResidueBase])
            -> result::Result<AvailableComponents, ChangeOrError> {
        use self::SheetMenu::*;

        let (commands, item_texts) = create_menu_items![
            (ChangeComponent, "Change component type"),
            (SetName, "Set name"),
            (SetResidue, "Set residue"),
            (SetLattice, "Set lattice"),
            (SetVarianceZ, "Set variance of residue positions along z"),
            (QuitAndSave, "Finalize component definition and return"),
            (QuitWithoutSaving, "Abort")
        ];

        let mut builder = SheetBuilder::initialize(&residue_list)?;

        loop {
            builder.print_state();

            let command = select_command(item_texts, commands)
                .map_err(|err| UIErrorKind::from(err))?;

            match command {
                ChangeComponent => return Err(ChangeOrError::ChangeComponent),
                SetName => match Input::new("Component name").interact() {
                    Ok(new_name) => {
                        builder.name = new_name.trim().to_string();
                    },
                    Err(_) => {
                        println!("error: Could not read name");
                    },
                },
                SetResidue => match select_residue(&residue_list) {
                    Ok(new_residue) => {
                        builder.residue = new_residue;
                    },
                    Err(_) => eprintln!("error: Could not select new residue"),
                },
                SetLattice => match select_lattice() {
                    Ok(new_lattice) => {
                        builder.lattice = new_lattice;
                    },
                    Err(_) => eprintln!("error: Could not select new lattice"),
                },
                SetVarianceZ => match get_variance() {
                    Ok(new_std_z) => {
                        builder.std_z = new_std_z;
                    },
                    Err(_) => eprintln!("error: Could not read new variance"),
                },
                QuitAndSave => match builder.finalize() {
                    Ok(component) => return Ok(component),
                    Err(msg) => eprintln!("{}", msg),
                },
                QuitWithoutSaving => return Err(ChangeOrError::Error(UIErrorKind::Abort)),
            }
        }
    }

    fn get_variance() -> UIResult<Option<f64>> {
        let std = Input::new("Standard deviation 'σ' of distribution (nm)")
            .default("0")
            .interact()?
            .trim()
            .parse::<f64>()?;

        if std == 0.0 {
            Ok(None)
        } else {
            Ok(Some(std))
        }
    }

    /*************************
     * Cylinder construction *
     *************************/

    #[derive(Clone, Copy, Debug)]
    enum CylinderMenu {
        ChangeComponent,
        ChangeCylinderType,
        SetName,
        SetResidue,
        SetCap,
        SetAlignment,
        QuitAndSave,
        QuitWithoutSaving,
    }

    #[derive(Clone, Copy, Debug)]
    /// Types of cylinders to construct.
    ///
    /// This is separate from `CylinderClass` in that this does not keep any extra data.
    enum CylinderSelection {
        Sheet,
        Volume,
    }

    struct CylinderBuilder {
        name: String,
        cylinder_type: CylinderClass,
        residue: ResidueBase,
        cap: Option<CylinderCap>,
        alignment: Direction,
    }

    impl CylinderBuilder {
        fn initialize(residue_list: &[ResidueBase]) -> UIResult<CylinderBuilder> {
            let cylinder_type = select_cylinder_type()?;
            let residue = select_residue(residue_list)?;

            Ok(CylinderBuilder {
                name: String::new(),
                cylinder_type,
                residue,
                cap: None,
                alignment: Direction::Z,
            })
        }

        fn finalize(&self) -> result::Result<AvailableComponents, &str> {
            if self.name.is_empty() {
                return Err("Cannot add component: No name is set")
            } else {
                Ok(AvailableComponents::Cylinder(CylinderConfEntry {
                    name: self.name.clone(),
                    residue: self.residue.clone(),
                    alignment: self.alignment,
                    cap: self.cap,
                    class: self.cylinder_type.clone(),
                    radius: None,
                    height: None,
                    position: None,
                }))
            }
        }

        fn print_state(&self) {
            eprintln!("Name: {}", &self.name);

            match self.cylinder_type {
                CylinderClass::Sheet(lattice) => {
                    eprintln!("Type: Cylinder Sheet");
                    eprintln!("Lattice: {:?}", lattice);
                    eprintln!("Residue: {}", self.residue.code);

                    // Unwrap the value from the Option<_>
                    eprintln!("Cap: {}", self.cap.map(|c| {
                            format!("{}", c)
                        }).unwrap_or("None".to_string()));
                },
                CylinderClass::Volume(_) => {
                    eprintln!("Type: Cylinder Volume");
                    eprintln!("Residue: {}", self.residue.code);
                },
            }

            eprintln!("Alignment: {}", self.alignment);
            eprintln!("");
        }
    }

    fn create_cylinder(residue_list: &[ResidueBase]) -> result::Result<AvailableComponents, ChangeOrError> {
        use self::CylinderMenu::*;

        let (commands, item_texts) = create_menu_items![
            (ChangeComponent, "Change component type"),
            (ChangeCylinderType, "Change cylinder type"),
            (SetName, "Set name"),
            (SetResidue, "Set residue"),
            (SetCap, "Cap either cylinder edge (Cylinder Sheet)"),
            (SetAlignment, "Set cylinder main axis alignment"),
            (QuitAndSave, "Finalize component definition and return"),
            (QuitWithoutSaving, "Abort")
        ];

        let mut builder = CylinderBuilder::initialize(&residue_list)?;

        loop {
            builder.print_state();

            let command = select_command(item_texts, commands)
                .map_err(|err| UIErrorKind::from(err))?;

            match command {
                ChangeComponent => return Err(ChangeOrError::ChangeComponent),
                ChangeCylinderType => match select_cylinder_type() {
                    Ok(new_type) => {
                        builder.cylinder_type = new_type;
                    },
                    Err(_) => eprintln!("error: Could not select new cylinder type"),
                },
                SetName => match Input::new("Component name").interact() {
                    Ok(new_name) => {
                        builder.name = new_name.trim().to_string();
                    },
                    Err(_) => {
                        eprintln!("error: Could not read name");
                    },
                },
                SetResidue => match select_residue(&residue_list) {
                    Ok(new_residue) => {
                        builder.residue = new_residue;
                    },
                    Err(_) => eprintln!("error: Could not select new residue"),
                },
                SetCap => match select_cap() {
                    Ok(new_cap) => {
                        builder.cap = new_cap;
                    },
                    Err(_) => eprintln!("error: Could not select new cap"),
                },
                SetAlignment => match select_direction() {
                    Ok(new_direction) => {
                        builder.alignment = new_direction;
                    },
                    Err(_) => eprintln!("error: Could not select new direction"),
                },
                QuitAndSave => match builder.finalize() {
                    Ok(component) => return Ok(component),
                    Err(msg) => eprintln!("{}", msg),
                },
                QuitWithoutSaving => return Err(ChangeOrError::Error(UIErrorKind::Abort)),
            }

            eprintln!("");
        }
    }

    fn select_cylinder_type() -> UIResult<CylinderClass> {
        use self::CylinderSelection::*;

        let (classes, item_texts) = create_menu_items![
            (Sheet, "Sheet"),
            (Volume, "Volume")
        ];

        let command = select_command(item_texts, classes)?;

        match command {
            Sheet => {
                let lattice = select_lattice()?;
                Ok(CylinderClass::Sheet(lattice))
            },
            Volume => Ok(CylinderClass::Volume(None)),
        }
    }

    fn select_cap() -> UIResult<Option<CylinderCap>> {
        let choices = &[
            "Bottom",
            "Top"
        ];

        eprintln!("Set caps on cylinder sides ([space] select, [enter] confirm):");
        let selections = Checkboxes::new().items(choices).interact()?;

        match (selections.contains(&0), selections.contains(&1)) {
            (true, true) => Ok(Some(CylinderCap::Both)),
            (true, false) => Ok(Some(CylinderCap::Bottom)),
            (false, true) => Ok(Some(CylinderCap::Top)),
            _ => Ok(None),
        }
    }

    fn select_direction() -> UIResult<Direction> {
        use grafen::database::Direction::*;

        let (choices, item_texts) = create_menu_items![
            (X, "X"),
            (Y, "Y"),
            (Z, "Z")
        ];

        select_command(item_texts, choices).map_err(|err| UIErrorKind::from(err))
    }

    /************************************
     * Selection of lattice and residue *
     ************************************/

    #[derive(Clone, Copy, Debug)]
    /// Available lattices to construct from. Each of these require
    /// a separate constructor since they have different qualities in
    /// their corresponding `LatticeType` unit.
    enum LatticeSelection {
        Triclinic,
        Hexagonal,
        PoissonDisc,
    }

    fn select_residue(residue_list: &[ResidueBase]) -> UIResult<ResidueBase> {
        // TODO: Rewrite this as soon as `Describe` is implemented for `ResidueBase`
        let item_texts: Vec<&str> = residue_list
            .iter()
            .map(|residue| residue.code.as_ref())
            .collect();
        let selection = Select::new().items(&item_texts).default(0).interact()?;

        Ok(residue_list[selection].clone())
    }

    fn select_lattice() -> UIResult<LatticeType> {
        use self::LatticeSelection::*;

        let (choices, item_texts) = create_menu_items![
            (Triclinic, "Triclinic lattice: two base vector lengths and an in-between angle"),
            (Hexagonal, "Hexagonal lattice: a honeycomb grid with a spacing"),
            (PoissonDisc, "Poisson disc: Randomly generated points with a density")
        ];

        let lattice = select_command(item_texts, choices)?;

        match lattice {
            Triclinic => {
                eprintln!("A triclinic lattice is constructed from two base ");
                eprintln!("vectors of length 'a' and 'b', separated by an angle 'γ'.");
                eprintln!("");

                let a = Input::new("Length 'a' (nm)").interact()?.trim().parse::<f64>()?;
                let b = Input::new("Length 'b' (nm)").interact()?.trim().parse::<f64>()?;
                let gamma = Input::new("Angle 'γ' (deg.)").interact()?.trim().parse::<f64>()?;

                Ok(LatticeType::Triclinic { a, b, gamma })
            },
            Hexagonal => {
                eprintln!("A hexagonal lattice is a honeycomb grid with an input side length 'a'.");
                eprintln!("");

                let a = Input::new("Spacing 'a' (nm)").interact()?.trim().parse::<f64>()?;

                Ok(LatticeType::Hexagonal { a })
            },
            PoissonDisc => {
                eprintln!("A Poisson disc is a generated set of points with an even distribution.");
                eprintln!("They are generated with an input density 'ρ' points per area.");
                eprintln!("");

                let density = Input::new("Density 'ρ' (1/nm^2)").interact()?.trim().parse::<f64>()?;

                Ok(LatticeType::PoissonDisc { density })
            },
        }
    }
}
