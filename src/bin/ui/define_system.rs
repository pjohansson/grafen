use database::{DataBase, SubstrateConfEntry};
use error::{GrafenCliError, Result, UIErrorKind};
use ui::SystemDefinition;
use ui::utils;

use grafen::system::Coord;

pub fn user_menu(database: &DataBase) -> Result<SystemDefinition> {
    let config = select_substrate(&database)?;
    let position = select_position()?;
    let size = select_size()?;
    let (x, y) = size;

    Ok(SystemDefinition {
        config: config.clone(),
        position: position,
        size: size,
        finalized: config.to_conf(x, y),
    })
}

fn select_substrate<'a>(database: &'a DataBase) -> Result<&'a SubstrateConfEntry> {
    println!("Available substrates:");
    for (i, sub) in database.substrate_defs.iter().enumerate() {
        println!("{}. {}", i, sub.name);
    }
    println!("");

    let selection = utils::get_input_string("Select substrate")?;
    selection
        .parse::<usize>()
        .map_err(|_| UIErrorKind::BadValue(format!("'{}' is not a valid index", &selection)))
        .and_then(|n| {
            database.substrate_defs
                .get(n)
                .ok_or(UIErrorKind::BadValue(format!("No substrate with index {} exists", n)))
        })
        .map_err(|err| GrafenCliError::from(err))
}

fn select_position() -> Result<Coord> {
    let selection = utils::get_input_string("Change position (default: (0.0, 0.0, 0.0))")?;
    if selection.is_empty() {
        return Ok(Coord::new(0.0, 0.0, 0.0));
    }

    let coords = utils::parse_string(&selection)?;
    let &x = coords.get(0).ok_or(UIErrorKind::BadValue("3 positions are required".to_string()))?;
    let &y = coords.get(1).ok_or(UIErrorKind::BadValue("3 positions are required".to_string()))?;
    let &z = coords.get(2).ok_or(UIErrorKind::BadValue("3 positions are required".to_string()))?;

    Ok(Coord::new(x, y, z))
}

fn select_size() -> Result<(f64, f64)> {
    let selection = utils::get_input_string("Set size")?;

    let size = utils::parse_string(&selection)?;
    let &dx = size.get(0).ok_or(UIErrorKind::BadValue("2 values are required".to_string()))?;
    let &dy = size.get(1).ok_or(UIErrorKind::BadValue("2 values are required".to_string()))?;

    Ok((dx, dy))
}
