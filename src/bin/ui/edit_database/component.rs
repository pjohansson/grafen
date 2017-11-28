//! Modify the list of `ComponentEntry` objects in a `DataBase`.

// This module is a bit of a mess.

use error::{GrafenCliError, UIResult, UIErrorKind};
use ui::utils::{MenuResult, get_value_from_user, print_description, print_list_description_short,
                remove_items, reorder_list, select_command, select_item};

use grafen::coord::{Coord, Direction};
use grafen::database::ComponentEntry;
use grafen::database::ComponentEntry::*;
use grafen::describe::Describe;
use grafen::surface;
use grafen::surface::{CylinderCap, LatticeType};
use grafen::system::Residue;
use grafen::volume;

use dialoguer::Checkboxes;
use std::error::Error;
use std::fmt::Write;
use std::result;

pub fn user_menu(mut component_list: &mut Vec<ComponentEntry>, residue_list: &[Residue])
        -> MenuResult {
    let components_backup = component_list.clone();

    create_menu![
        @pre: { print_list_description_short("Component definitions", &component_list); };

        AddComponent, "Create a component definition" => {
            new_component(&residue_list)
                .map(|component| {
                    component_list.push(component);
                    Some("Successfully created component definition".to_string())
                })
                .map_err(|_| GrafenCliError::RunError(
                    "Could not create component definition".to_string()
                ))
        },
        RemoveComponent, "Remove a component definition" => {
            remove_items(&mut component_list)
                .map(|_| None)
                .map_err(|err| GrafenCliError::RunError(
                    format!("Could not remove a component: {}", err.description())
                ))
        },
        ReorderList, "Reorder component definition list" => {
            reorder_list(&mut component_list)
                .map(|_| None)
                .map_err(|err| GrafenCliError::RunError(
                    format!("Could not reorder the list: {}", err.description())
                ))
        },
        QuitAndSave, "Finish editing component definition list" => {
            return Ok(Some("Finished editing component definition list".to_string()));
        },
        QuitWithoutSaving, "Abort and discard changes" => {
            *component_list = components_backup;
            return Ok(Some("Discarding changes to component definition list".to_string()));
        }
    ];
}

#[derive(Clone, Copy, Debug)]
/// Available component types.
enum ComponentSelect {
    Sheet,
    Cylinder,
    Cuboid,
    Abort,
}
use self::ComponentSelect::*;

/// This menu changes the main component type and then calls that type's construction menu.
fn new_component(residue_list: &[Residue]) -> UIResult<ComponentEntry> {
    loop {
        let component_type = select_component_type()?;

        let result = match component_type {
            Sheet => create_sheet(&residue_list),
            Cylinder => create_cylinder(&residue_list),
            Cuboid => create_cuboid(&residue_list),
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
        (Cuboid, "Cuboid box"),
        (Abort, "(Abort)")
    ];

    eprintln!("Component type:");
    select_command(item_texts, choices).map_err(|err| UIErrorKind::from(err))
}

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

#[derive(Clone, Copy, Debug, PartialEq)]
/// Types of components to construct.
enum ComponentType {
    Surface,
    Volume,
}
use self::ComponentType::*;

/**********************
 * Sheet construction *
 **********************/

/// Use a builder to get all value before constructing the component.
struct SheetBuilder {
    name: String,
    lattice: LatticeType,
    residue: Residue,
    std_z: Option<f64>,
}

impl SheetBuilder {
    fn initialize(residue_list: &[Residue]) -> UIResult<SheetBuilder> {
        let lattice = select_lattice()?;
        let residue = select_residue(&residue_list)?;

        Ok(SheetBuilder {
            name: String::new(),
            lattice,
            residue,
            std_z: None,
        })
    }

    fn finalize(&self) -> result::Result<ComponentEntry, &str> {
        if self.name.is_empty() {
            return Err("Cannot add component: No name is set")
        } else {
            Ok(SurfaceSheet(surface::Sheet {
                name: Some(self.name.clone()),
                residue: Some(self.residue.clone()),
                lattice: self.lattice.clone(),
                std_z: self.std_z,
                origin: Coord::default(),
                length: 0.0,
                width: 0.0,
                coords: vec![],
            }))
        }
    }
}

impl Describe for SheetBuilder {
    fn describe(&self) -> String {
        let mut description = String::new();
        const ERR: &'static str = "could not construct a string";

        writeln!(description, "Name: {}", &self.name).expect(ERR);
        writeln!(description, "Lattice: {:?}", &self.lattice).expect(ERR);
        writeln!(description, "Residue: {}", &self.residue.code).expect(ERR);
        writeln!(description, "Z-variance: {}", &self.std_z.unwrap_or(0.0)).expect(ERR);

        description
    }

    fn describe_short(&self) -> String { self.describe() }
}

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

fn create_sheet(residue_list: &[Residue]) -> result::Result<ComponentEntry, ChangeOrError> {
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
        eprintln!("{}", builder.describe());

        let command = select_command(item_texts, commands).map_err(|err| UIErrorKind::from(err))?;

        match command {
            ChangeComponent => return Err(ChangeOrError::ChangeComponent),
            SetName => match get_value_from_user::<String>("Component name") {
                Ok(new_name) => {
                    builder.name = new_name;
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
    let std = get_value_from_user::<f64>("Standard deviation 'σ' of distribution (nm)")?;

    if std == 0.0 {
        Ok(None)
    } else {
        Ok(Some(std))
    }
}

/*************************
 * Cylinder construction *
 *************************/

struct CylinderBuilder {
    name: String,
    cylinder_type: ComponentType,
    lattice: Option<LatticeType>,
    residue: Residue,
    density: Option<f64>,
    cap: Option<CylinderCap>,
    alignment: Direction,
}

impl CylinderBuilder {
    fn initialize(residue_list: &[Residue]) -> UIResult<CylinderBuilder> {
        let cylinder_type = select_cylinder_type()?;
        let residue = select_residue(&residue_list)?;

        let lattice = if cylinder_type == Surface {
            Some(select_lattice()?)
        } else {
            None
        };

        Ok(CylinderBuilder {
            name: String::new(),
            cylinder_type,
            lattice,
            residue,
            density: None,
            cap: None,
            alignment: Direction::Z,
        })
    }

    fn finalize(&self) -> result::Result<ComponentEntry, &str> {
        if self.name.is_empty() {
            return Err("Cannot add component: No name is set")
        } else {
            match self.cylinder_type {
                Surface => {
                    Ok(SurfaceCylinder(surface::Cylinder {
                        name: Some(self.name.clone()),
                        residue: Some(self.residue.clone()),
                        lattice: self.lattice.unwrap(),
                        alignment: self.alignment,
                        cap: self.cap,
                        origin: Coord::default(),
                        radius: 0.0,
                        height: 0.0,
                        coords: vec![],
                    }))
                },

                Volume => {
                    Ok(VolumeCylinder(volume::Cylinder {
                        name: Some(self.name.clone()),
                        residue: Some(self.residue.clone()),
                        alignment: self.alignment,
                        origin: Coord::default(),
                        radius: 0.0,
                        height: 0.0,
                        density: self.density,
                        coords: vec![],
                    }))
                },
            }
        }
    }
}

impl Describe for CylinderBuilder {
    fn describe(&self) -> String {
        let mut description = String::new();
        const ERR: &'static str = "could not construct a string";

        writeln!(description, "Name: {}", &self.name).expect(ERR);

        match self.cylinder_type {
            Surface => {
                writeln!(description, "Type: Cylinder Surface").expect(ERR);
                writeln!(description, "Lattice: {:?}", self.lattice.unwrap()).expect(ERR);
                writeln!(description, "Residue: {}", self.residue.code).expect(ERR);

                let cap_string = self.cap
                    .map(|cap| format!("{}", cap))
                    .unwrap_or("None".to_string());

                writeln!(description, "Cap: {}", cap_string).expect(ERR);
            },
            Volume => {
                writeln!(description, "Type: Cylinder Volume").expect(ERR);
                writeln!(description, "Residue: {}", self.residue.code).expect(ERR);

                let density_string = self.density
                    .map(|dens| format!("{}", dens))
                    .unwrap_or("None".into());
                writeln!(description, "Density: {}", density_string).expect(ERR);
            },
        }

        writeln!(description, "Alignment: {}", self.alignment).expect(ERR);

        description
    }

    fn describe_short(&self) -> String { self.describe() }
}

#[derive(Clone, Copy, Debug)]
enum CylinderSurfaceMenu {
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
enum CylinderVolumeMenu {
    ChangeComponent,
    ChangeCylinderType,
    SetName,
    SetResidue,
    SetDensity,
    SetAlignment,
    QuitAndSave,
    QuitWithoutSaving,
}

fn create_cylinder(residue_list: &[Residue]) -> result::Result<ComponentEntry, ChangeOrError> {
    let mut builder = CylinderBuilder::initialize(&residue_list)?;

    loop {
        print_description(&builder);

        // Always match against the type to select the correct menu
        match builder.cylinder_type {
            Surface => {
                use self::CylinderSurfaceMenu::*;

                // These are statically compiled so we can keep the construction in this loop
                let (surface_commands, surface_texts) = create_menu_items![
                    (ChangeComponent, "Change component type"),
                    (ChangeCylinderType, "Change cylinder type"),
                    (SetName, "Set name"),
                    (SetResidue, "Set residue"),
                    (SetCap, "Cap either cylinder edge"),
                    (SetAlignment, "Set cylinder main axis alignment"),
                    (QuitAndSave, "Finalize component definition and return"),
                    (QuitWithoutSaving, "Abort")
                ];

                let command = select_command(surface_texts, surface_commands)
                    .map_err(|err| UIErrorKind::from(err))?;

                match command {
                    ChangeComponent => return Err(ChangeOrError::ChangeComponent),
                    ChangeCylinderType => match select_cylinder_type() {
                        Ok(new_type) => {
                            builder.cylinder_type = new_type;
                        },
                        Err(_) => eprintln!("error: Could not select new cylinder type"),
                    },
                    SetName => match get_value_from_user::<String>("Component name") {
                        Ok(new_name) => {
                            builder.name = new_name;
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
            },

            Volume => {
                use self::CylinderVolumeMenu::*;

                // These are statically compiled so we can keep the construction in this loop
                let (volume_commands, volume_texts) = create_menu_items![
                    (ChangeComponent, "Change component type"),
                    (ChangeCylinderType, "Change cylinder type"),
                    (SetName, "Set name"),
                    (SetResidue, "Set residue"),
                    (SetDensity, "Set default density"),
                    (SetAlignment, "Set cylinder main axis alignment"),
                    (QuitAndSave, "Finalize component definition and return"),
                    (QuitWithoutSaving, "Abort")
                ];

                let command = select_command(volume_texts, volume_commands)
                    .map_err(|err| UIErrorKind::from(err))?;

                match command {
                    ChangeComponent => return Err(ChangeOrError::ChangeComponent),
                    ChangeCylinderType => match select_cylinder_type() {
                        Ok(new_type) => {
                            builder.cylinder_type = new_type;
                        },
                        Err(_) => eprintln!("error: Could not select new cylinder type"),
                    },
                    SetName => match get_value_from_user::<String>("Component name") {
                        Ok(new_name) => {
                            builder.name = new_name;
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
                    SetDensity => match get_density() {
                        Ok(density) => {
                            builder.density = density;
                        },
                        Err(_) => eprintln!("error: Could not set density"),
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
            },
        }

        eprintln!("");
    }
}

fn select_cylinder_type() -> UIResult<ComponentType> {
    let (classes, item_texts) = create_menu_items![
        (Surface, "Surface"),
        (Volume, "Volume")
    ];

    select_command(item_texts, classes)
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
    use grafen::coord::Direction::*;

    let (choices, item_texts) = create_menu_items![
        (X, "X"),
        (Y, "Y"),
        (Z, "Z")
    ];

    select_command(item_texts, choices).map_err(|err| UIErrorKind::from(err))
}

/***********************
 * Cuboid construction *
 ***********************/

struct CuboidBuilder {
    name: String,
    residue: Residue,
    density: Option<f64>,
}

impl CuboidBuilder {
    fn initialize(residue_list: &[Residue]) -> UIResult<CuboidBuilder> {
        let residue = select_residue(&residue_list)?;

        Ok(CuboidBuilder {
            name: String::new(),
            residue,
            density: None,
        })
    }

    fn finalize(&self) -> result::Result<ComponentEntry, &str> {
        if self.name.is_empty() {
            return Err("Cannot add component: No name is set")
        } else {
            Ok(VolumeCuboid(volume::Cuboid {
                name: Some(self.name.clone()),
                residue: Some(self.residue.clone()),
                density: self.density.clone(),
                .. volume::Cuboid::default()
            }))
        }
    }
}

impl Describe for CuboidBuilder {
    fn describe(&self) -> String {
        let mut description = String::new();
        const ERR: &'static str = "could not construct a string";

        writeln!(description, "Name: {}", &self.name).expect(ERR);
        writeln!(description, "Residue: {}", self.residue.code).expect(ERR);

        let density_string = self.density.map(|dens| format!("{}", dens)).unwrap_or("None".into());
        writeln!(description, "Density: {}", density_string).expect(ERR);

        description
    }

    fn describe_short(&self) -> String { self.describe() }
}

#[derive(Clone, Copy, Debug)]
enum CuboidMenu {
    ChangeComponent,
    SetName,
    SetResidue,
    SetDensity,
    QuitAndSave,
    QuitWithoutSaving,
}

fn create_cuboid(residue_list: &[Residue]) -> result::Result<ComponentEntry, ChangeOrError> {
    let mut builder = CuboidBuilder::initialize(&residue_list)?;

    use self::CuboidMenu::*;

    loop {
        print_description(&builder);

        let (commands, item_texts) = create_menu_items![
            (ChangeComponent, "Change component type"),
            (SetName, "Set name"),
            (SetResidue, "Set residue"),
            (SetDensity, "Set default density"),
            (QuitAndSave, "Finalize component definition and return"),
            (QuitWithoutSaving, "Abort")
        ];

        let command = select_command(item_texts, commands)
            .map_err(|err| UIErrorKind::from(err))?;

        match command {
            ChangeComponent => return Err(ChangeOrError::ChangeComponent),
            SetName => match get_value_from_user::<String>("Component name") {
                Ok(new_name) => {
                    builder.name = new_name;
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
            SetDensity => match get_density() {
                Ok(density) => {
                    builder.density = density;
                },
                Err(_) => eprintln!("error: Could not set density"),
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

fn get_density() -> UIResult<Option<f64>> {
    let density = get_value_from_user::<f64>("Density (1/nm^3, negative: unset)")?;

    if density > 0.0 {
        Ok(Some(density))
    } else {
        Ok(None)
    }
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

fn select_residue(residue_list: &[Residue]) -> UIResult<Residue> {
    select_item(&residue_list, None).map(|res| res.clone())
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

            let a = get_value_from_user::<f64>("Length 'a' (nm)")?;
            let b = get_value_from_user::<f64>("Length 'b' (nm)")?;
            let gamma = get_value_from_user::<f64>("Angle 'γ' (deg.)")?;

            Ok(LatticeType::Triclinic { a, b, gamma })
        },
        Hexagonal => {
            eprintln!("A hexagonal lattice is a honeycomb grid with an input side length 'a'.");
            eprintln!("");

            let a = get_value_from_user::<f64>("Spacing 'a' (nm)")?;

            Ok(LatticeType::Hexagonal { a })
        },
        PoissonDisc => {
            eprintln!("A Poisson disc is a generated set of points with an even distribution.");
            eprintln!("They are generated with an input density 'ρ' points per area.");
            eprintln!("");

            let density = get_value_from_user::<f64>("Density 'ρ' (1/nm^2)")?;

            Ok(LatticeType::PoissonDisc { density })
        },
    }
}
