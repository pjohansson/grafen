//! Construct ResidueBase objects at runtime.

use config::{ConfigError, Result};
use grafen::system::{Atom, Coord, ResidueBase};

use std::io;
use std::io::Write;
use std::str::SplitWhitespace;

// Loop control.
enum Control {
    Finish,
    Continue,
    Cancel,
}

/// Ask the user to construct a `ResidueBase` object.
pub fn construct_residue() -> Result<ResidueBase> {
    let mut name = String::new();
    let mut atoms: Vec<Atom> = Vec::new();

    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut selection = String::new();

    print_commands();

    loop {
        print!("\nSelection: ");
        stdout.flush()?;

        selection.clear();
        stdin.read_line(&mut selection)?;
        selection = selection.to_lowercase();
        let mut args = selection.trim().split_whitespace();

        let control_message = match args.next() {
            Some("n") | Some("name")   => set_name(&mut name, args.next()),
            Some("a") | Some("add")    => add_atom(&mut atoms, &mut args),
            Some("r") | Some("remove") => remove_atom(&mut atoms, args.next()),
            Some("s") | Some("swap")   => swap_atoms(&mut atoms, &mut args),
            Some("p") | Some("print")  => print_residue(&name, &atoms),
            Some("h") | Some("help")   => print_commands(),
            Some("c") | Some("cancel") => Ok(Control::Cancel),
            Some("q") | Some("quit")   => {
                if name.is_empty() {
                    Err(ConfigError::from("No name is set: cannot save residue."))
                } else if atoms.is_empty() {
                    Err(ConfigError::from("No atoms are set: cannot save residue."))
                } else {
                    Ok(Control::Finish)
                }
            },
            _ => Err(ConfigError::from("Unknown command.")),
        };

        match control_message {
            Ok(Control::Finish) => break,
            Ok(Control::Cancel) => return Err(ConfigError::NoSubstrate),
            Ok(_) => (),
            Err(msg) => println!("{}", msg),
        }
    }

    Ok(ResidueBase {
        code: name,
        atoms: atoms,
    })
}

fn add_atom(atoms: &mut Vec<Atom>, args: &mut SplitWhitespace) -> Result<Control> {
    let name = check_name(args.next(), 4)?;

    let mut read_float = | | {
        args.next()
            .ok_or("Not all position arguments were supplied.")
            .and_then(|s| s.parse::<f64>().map_err(|_| "Bad position argument supplied."))
            .map_err(|s| ConfigError::from(s))
    };

    let x = read_float()?;
    let y = read_float()?;
    let z = read_float()?;

    let atom = Atom { code: name.to_string(), position: Coord::new(x, y, z) };
    atoms.push(atom);

    Ok(Control::Continue)
}

fn remove_atom(atoms: &mut Vec<Atom>, arg: Option<&str>) -> Result<Control> {
    let index = arg.ok_or("No index supplied.")
                   .and_then(|s| s.parse::<usize>().map_err(|_| "Bad index supplied."))
                   .map_err(|s| ConfigError::RunError(s.to_string()))?;

    if index == 0 || index > atoms.len() {
        return Err(ConfigError::from("Atom number does not exist in the residue."));
    }

    atoms.remove(index - 1);
    Ok(Control::Continue)
}

fn swap_atoms(atoms: &mut Vec<Atom>, args: &mut SplitWhitespace) -> Result<Control> {
    let mut read_uint = | | {
        args.next()
            .ok_or("Not all position arguments were supplied.")
            .and_then(|s| s.parse::<usize>().map_err(|_| "Bad position argument supplied."))
            .map_err(|s| ConfigError::from(s))
    };

    let i = read_uint()?;
    let j = read_uint()?;

    if i == 0 || i > atoms.len() || j == 0 || j > atoms.len() {
        return Err(ConfigError::from("An atom number does not exist in the residue."));
    }

    atoms.swap(i - 1, j - 1);
    Ok(Control::Continue)
}

fn set_name(name: &mut String, arg: Option<&str>) -> Result<Control> {
    let new_name = check_name(arg, 3)?;
    *name = new_name;

    Ok(Control::Continue)
}

fn check_name(arg: Option<&str>, max_len: usize) -> Result<String> {
    let name = arg.ok_or("No name was supplied.")
       .map(|name| name.to_uppercase())
       .map_err(|err| ConfigError::from(err))?;

    if name.len() > max_len {
        return Err(ConfigError::from(
                format!("Name cannot be longer than {} characters.", max_len).as_str()
            ));
    }

    Ok(name)
}

fn print_commands() -> Result<Control> {
    println!("
Commands:
---------
name [name]         Set residue name.
add [name] [x y z]  Add new atom with input name and relative position.
remove [index]      Remove atom with input index from residue.
swap [i] [j]        Swap atoms with indices i and j in list.

print               Print current residue.
help                Print this message.
cancel              Exit without saving residue.
quit                Finish and save residue.");

    Ok(Control::Continue)
}

fn print_residue(name: &str, atoms: &Vec<Atom>) -> Result<Control> {
    println!("\nResidue name: \"{}\"", name);
    print!("Atom list:");

    match atoms.len() {
        0 => println!(" (empty)"),
        _ => {
            for (i, atom) in atoms.iter().enumerate() {
                let coord = atom.position;
                print!("\n{:-4}. \"{}\" at ({:.3}, {:.3}, {:.3})", i + 1, atom.code, coord.x, coord.y, coord.z);
            }
            println!("");
        },
    }

    Ok(Control::Continue)
}
