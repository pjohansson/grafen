use database::{DataBase, SubstrateConfEntry};
use error::{GrafenCliError, Result, UIErrorKind};
use super::{get_input, SystemDefinition};

use grafen::system::Coord;
use std::error::Error;
use std::io;
use std::io::Write;

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

use std::result;

fn select_substrate<'a>(database: &'a DataBase) -> Result<&'a SubstrateConfEntry> {
    println!("Available substrates:");
    for (i, sub) in database.substrate_defs.iter().enumerate() {
        println!("{}. {}", i, sub.name);
    }
    println!("");

    let selection = get_input("Select substrate")?;
    selection
        .parse::<usize>()
        .map_err(|_| UIErrorKind::BadNumber(format!("'{}' is not a valid index", &selection)))
        .and_then(|n| {
            database.substrate_defs
                .get(n)
                .ok_or(UIErrorKind::BadNumber(format!("No substrate with index {} exists", n)))
        })
        .map_err(|err| GrafenCliError::from(err))
}

fn select_position() -> Result<Coord> {
    let selection = get_input("Change position (default: (0.0, 0.0, 0.0))")?;
    if selection.is_empty() {
        return Ok(Coord::new(0.0, 0.0, 0.0));
    }

    let coords: Vec<f64> = selection
        .split_whitespace()
        .take(3)
        .map(|s| {
            s.parse::<f64>()
             .map_err(|_| UIErrorKind::BadNumber("'{}' not a valid number".to_string()))
        })
        .collect::<result::Result<Vec<f64>, UIErrorKind>>()?;

    let &x = coords.get(0).ok_or(UIErrorKind::BadNumber("3 positions are required".to_string()))?;
    let &y = coords.get(1).ok_or(UIErrorKind::BadNumber("3 positions are required".to_string()))?;
    let &z = coords.get(2).ok_or(UIErrorKind::BadNumber("3 positions are required".to_string()))?;

    Ok(Coord::new(x, y, z))
}

fn select_size() -> Result<(f64, f64)> {
    let selection = get_input("Set size")?;

    let size: Vec<f64> = selection
        .split_whitespace()
        .take(2)
        .map(|s| {
            s.parse::<f64>()
             .map_err(|_| UIErrorKind::BadNumber("'{}' not a valid number".to_string()))
        })
        .collect::<result::Result<Vec<f64>, UIErrorKind>>()?;

    let &dx = size.get(0).ok_or(UIErrorKind::BadNumber("2 values are required".to_string()))?;
    let &dy = size.get(1).ok_or(UIErrorKind::BadNumber("2 values are required".to_string()))?;

    Ok((dx, dy))
}
