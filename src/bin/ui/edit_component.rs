//! Edit constructed `ComponentEntry` objects.

use error::{GrafenCliError, Result};
use ui::utils::{MenuResult, get_value_from_user, get_position_from_user, print_description,
                select_command, select_item, select_item_index};

use grafen::coord::Direction;
use grafen::database::*;
use grafen::system::*;
use grafen::coord::{Coord, Translate};
use grafen::volume::{Contains, Cuboid, Cylinder, prune_residues_from_volume};

/// Prompt the user to select a defined component and then edit it.
pub fn user_menu(components: &mut [ComponentEntry]) -> MenuResult {
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
            let volume = get_volume_from_user(components)?;
            let num_before = component.num_atoms();

            let pruned_coords = prune_residues_from_volume(component.get_coords(),
                component.get_origin(),
                component.get_residue().as_ref().unwrap(),
                volume.as_ref());

            component.get_coords_mut().clone_from(&pruned_coords);
            let num_after = component.num_atoms();

            Ok(Some(format!("Removed {} atoms from the component", num_before - num_after)))
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

/// Ask the user to select a volume object that has been constructed.
fn get_volume_from_user(components: &[ComponentEntry]) -> Result<Box<Contains>> {
    let volume_components = get_volume_objects(components);
    let component = select_item(&volume_components, Some("Select component to cut with"))?
        .clone();

    let margin: f64 = get_value_from_user("Margin around volume to also exclude (nm)")?;

    match component {
        ComponentEntry::VolumeCuboid(mut obj) => {
            let coord_margins = Coord::new(margin, margin, margin);
            obj.origin -= coord_margins;
            obj.size += coord_margins * 2.0;

            Ok(Box::new(obj))
        },
        ComponentEntry::VolumeCylinder(mut obj) => {
            obj.radius += margin;
            obj.height += 2.0 * margin;

            match obj.alignment {
                Direction::X => obj.translate_in_place(Coord::new(-margin, 0.0, 0.0)),
                Direction::Y => obj.translate_in_place(Coord::new(0.0, -margin, 0.0)),
                Direction::Z => obj.translate_in_place(Coord::new(0.0, 0.0, -margin)),
            }

            Ok(Box::new(obj))
        },
        _ => Err(GrafenCliError::RunError(String::from(
            "Tried to get a volume type that has not been implemented: this should be impossible")
        )),
    }
}

/// Prune the list of components to only return those that are volumes, without their
/// coordinates since we don't want to copy those.
// fn get_volume_objects(components: &[ComponentEntry]) -> Vec<VolumeComponent> {
fn get_volume_objects(components: &[ComponentEntry]) -> Vec<ComponentEntry> {
    components.iter()
              .filter_map(|comp| {
                  match comp {
                      &ComponentEntry::VolumeCuboid(ref obj) => {
                          let volume = Cuboid {
                              name: obj.name.clone(),
                              residue: obj.residue.clone(),
                              origin: obj.origin,
                              size: obj.size,
                              density: obj.density,
                              coords: vec![],
                          };

                          Some(ComponentEntry::from(volume))
                      },

                      &ComponentEntry::VolumeCylinder(ref obj) => {
                          let volume = Cylinder {
                              name: obj.name.clone(),
                              residue: obj.residue.clone(),
                              origin: obj.origin,
                              radius: obj.radius,
                              height: obj.height,
                              density: obj.density,
                              alignment: obj.alignment,
                              coords: vec![],
                          };

                          Some(ComponentEntry::from(volume))
                      },

                      &ComponentEntry::SurfaceCylinder(ref obj) => {
                          let volume = Cylinder {
                              name: obj.name.clone(),
                              residue: obj.residue.clone(),
                              origin: obj.origin,
                              radius: obj.radius,
                              height: obj.height,
                              density: None,
                              alignment: obj.alignment,
                              coords: vec![],
                          };

                          Some(ComponentEntry::from(volume))
                      },

                      _ => None,
                  }
              })
              .collect::<Vec<_>>()
}
