#[macro_use]
extern crate clap;

use clap::{Arg, App};
use std::fs::File;
use std::io;
use std::io::{BufWriter, Write};
use std::path::PathBuf;

#[derive(Debug)]
struct Coord {
    x: f64,
    y: f64
}

impl Coord {
    fn at_index(&self, nx: u64, ny: u64, spacing: &Coord) -> Coord {
        Coord {
            x: self.x + spacing.x*(nx as f64),
            y: self.y + spacing.y*(ny as f64)
        }
    }
}

struct Residue {
    resname: String,
    atomname: String,
    resnum: u64,
    bondlength: f64
}

fn main() {
    let matches = App::new("create_graphene")
        .version("0.1")
        .author("Petter Johansson <pettjoha@kth.se>")
        .about("Create a graphene substrate and output it to a GROMOS formatted file")
        .arg(Arg::with_name("output")
            .help("output .gro file (the extension will be corrected)")
            .value_name("PATH")
            .required(true))
        .arg(Arg::with_name("nx")
            .help("number of layers along x")
            .value_name("INT")
            .required(true))
        .arg(Arg::with_name("ny")
            .help("number of layers along y")
            .value_name("INT")
            .required(true))
        .arg(Arg::with_name("title")
            .help("title of system (default: \"Graphene substrate\")")
            .short("t")
            .long("title")
            .value_name("STR")
            .takes_value(true)
            .required(false))
        .arg(Arg::with_name("z")
            .help("position of layer along z-axis (default: 0.5*bondlength)")
            .short("z")
            .value_name("FLOAT")
            .takes_value(true)
            .required(false))
        .get_matches();

    let output_file = value_t_or_exit!(matches, "output", String);
    let nx = value_t_or_exit!(matches, "nx", u64);
    let ny = value_t_or_exit!(matches, "ny", u64);

    let mut graphene = Residue {
        resname: "GRPH".to_string(),
        atomname: "C".to_string(),
        resnum: 0,
        bondlength: 0.142
    };

    let z = value_t!(matches, "z", f64).unwrap_or(0.5*graphene.bondlength);

    // The base used to create a hexagonal structure requires four points
    // which can be periodically replicated with the correct spacing
    // in terms of the bond length. This is the spacing:
    let spacing = Coord {
        x: 3.0_f64.sqrt()*graphene.bondlength,
        y: 3.0*graphene.bondlength
    };

    // And this the base vector
    let hexagonal_base = vec![
        Coord { x: 0.0,           y: 0.0 },
        Coord { x: 0.0,           y: graphene.bondlength },
        Coord { x: 0.5*spacing.x, y: 1.5*graphene.bondlength },
        Coord { x: 0.5*spacing.x, y: 2.5*graphene.bondlength }
    ];

    let path = PathBuf::from(output_file).with_extension("gro");
    let file = File::create(&path).unwrap_or_else(|_| {
        io::stderr().write_fmt(
            format_args!("error: could not open '{}' for writing\n", &path.to_str().unwrap())
        ).unwrap();
        std::process::exit(1);
    });
    let mut writer = BufWriter::new(file);

    let title = value_t!(matches, "title", String).unwrap_or("Graphene substrate".to_string());
    writer.write_fmt(format_args!("{}\n", title)).unwrap();

    let num_atoms = (hexagonal_base.len() as u64)*nx*ny;
    writer.write_fmt(format_args!("{}\n", num_atoms)).unwrap();

    for row in 0..ny {
        for col in 0..nx {
            for coord in hexagonal_base.iter().map(|c| c.at_index(col, row, &spacing)) {
                graphene.resnum += 1;
                write_atom_line(&mut writer, &coord, z, &graphene);
            }
        }
    }

    let (box_x, box_y, box_z) = (
        (nx as f64)*spacing.x,
        (ny as f64)*spacing.y,
        graphene.bondlength
    );
    writer.write_fmt(format_args!("{:12.8} {:12.8} {:12.8}\n",
        box_x, box_y, box_z)).unwrap();
}

fn write_atom_line(writer: &mut BufWriter<File>, coord: &Coord, z: f64, residue: &Residue) {
    // GROMOS files wrap atom and residue numbering after five digits
    // so we must output at most that
    let truncated_num = residue.resnum % 100_000;
    writer.write_fmt(format_args!("{:>5}{:<5}{:>5}{:>5}{:>8.3}{:>8.3}{:>8.3}\n",
        truncated_num, residue.resname, residue.atomname, truncated_num,
        coord.x, coord.y, z)).unwrap();
}
