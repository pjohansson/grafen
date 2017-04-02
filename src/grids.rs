use coords::Coord;

pub struct SystemBox<T> {
    pub dimensions: Coord,
    pub coords: Vec<T>
}

pub type Grid = SystemBox<Coord>;

enum CrystalSystem {
    Hexagonal { length: f64 },
    Triclinic { a: f64, b: f64, gamma: f64 }
}

struct Crystal {
    a: f64,
    b: f64,
    gamma: f64,
}

struct Spacing ( f64, f64, f64);

impl Crystal {
    fn from_system(input: CrystalSystem) -> Crystal {
        let pi = ::std::f64::consts::PI;

        match input {
            CrystalSystem::Hexagonal { length } => Crystal {
                a: length,
                b: length,
                gamma: 2.0*pi/3.0
            },
            CrystalSystem::Triclinic { a, b, gamma } => Crystal {
                a: a,
                b: b,
                gamma: gamma
            }
        }
    }

    fn spacing(&self) -> Spacing {
        let dx = self.a;
        let dy = self.b * self.gamma.sin();
        let dx_per_row = self.b * self.gamma.cos();

        Spacing(dx, dy, dx_per_row)
    }
}

struct Lattice {
    box_size: Coord,
    coords: Vec<Coord>,
}

impl Lattice {
    fn new(crystal: &Crystal, nx: u64, ny: u64) -> Lattice {
        let Spacing(dx, dy, dx_per_row) = crystal.spacing();

        let box_size = Coord { x: (nx as f64)*dx, y: (ny as f64)*dy, z: 0.0 };
        let coords = (0..ny)
            .flat_map(|row| {
                (0..nx).map(move |col| Coord {
                    x: (col as f64)*dx + (row as f64)*dx_per_row,
                    y: (row as f64)*dy,
                    z: 0.0,
                })
            })
            .collect();

        Lattice { box_size: box_size, coords: coords }
    }

    fn from_size(crystal: &Crystal, size_x: f64, size_y: f64) -> Lattice {
        let Spacing(dx, dy, _) = crystal.spacing();
        let (nx, ny) = ((size_x/dx).round() as u64, (size_y/dy).round() as u64);

        Lattice::new(&crystal, nx, ny)
    }
}

/// Return a hexagonal grid of input size and base length.
pub fn hexagonal_grid(size_x: f64, size_y: f64, base_length: f64, z0: f64)
        -> Grid {
    // Calculate the box dimension and number of base vector replications
    let spacing = get_hexagonal_spacing(base_length);
    let (nx, ny) = get_num_replications(size_x, size_y, spacing);

    Grid {
        dimensions: get_system_dimensions(nx, ny, spacing, z0),
        coords: gen_hexagonal_grid(nx, ny, base_length, spacing)
    }
}

fn get_hexagonal_spacing(base_length: f64) -> Coord {
    let dx = base_length*f64::sqrt(3.0)/2.0;
    let dy = base_length*0.5;

    // The spacing to the next set of four points is two times the
    // move to the first point along x and adding another bond spacing
    // to the last point along y.
    Coord::new(2.0*dx, 2.0*dy + 2.0*base_length, 0.0)
}


fn get_num_replications(size_x: f64, size_y: f64, spacing: Coord) -> (u64, u64) {
    (f64::round(size_x/spacing.x) as u64, f64::round(size_y/spacing.y) as u64)
}

fn get_system_dimensions(nx: u64, ny: u64, spacing: Coord, z0: f64) -> Coord {
    Coord::new((nx as f64)*spacing.x, (ny as f64)*spacing.y, z0)
}

/// Generate a hexagonal grid of nx*ny base vectors. The distance
/// between each grid point is the input base length.
///
/// Each base vector consists of four grid points since this
/// is the amount required to create a periodically replicating
/// hexagonal grid.
fn gen_hexagonal_grid(nx: u64, ny: u64, base_length: f64, spacing: Coord) -> Vec<Coord> {
    // Starting at (0.0, 0.0) we construct these four base points
    // in the order of going counter-clockwise with an angle of 30 degrees,
    // then one full bond spacing above that, and finally counter-clockwise
    // with 150 degrees.
    let dx = base_length*f64::sqrt(3.0)/2.0;
    let dy = base_length*0.5;

    let base_coords = vec![
        Coord::new(0.0, 0.0,    0.0),
        Coord::new(dx,  dy,     0.0),
        Coord::new(dx,  dy + base_length, 0.0),
        Coord::new(0.0, 2.0*dy + base_length, 0.0)
    ];

    let mut grid_points = Vec::new();
    for i in 0..nx {
        for j in 0..ny {
            for coord in &base_coords {
                grid_points.push(coord.at_index(i, j, 0, &spacing));
            }
        }
    }

    grid_points
}

#[cfg(test)]
mod tests {
    use super::*;
    use ::std::f64;

    #[test]
    fn hexagonal_crystal() {
        let system = CrystalSystem::Hexagonal { length: 1.0 };
        let crystal = Crystal::from_system(system);
        assert_eq!(1.0, crystal.a);
        assert_eq!(1.0, crystal.b);
        assert_eq!(2.0*f64::consts::PI/3.0, crystal.gamma);
    }

    #[test]
    fn triclinic_crystal() {
        let system = CrystalSystem::Triclinic { a: 1.0, b: 2.0, gamma: 3.0 };
        let crystal = Crystal::from_system(system);
        assert_eq!(1.0, crystal.a);
        assert_eq!(2.0, crystal.b);
        assert_eq!(3.0, crystal.gamma);
    }

    #[test]
    fn triclinic_lattice() {
        let dx = 1.0;
        let angle = f64::consts::PI/3.0; // 60 degrees
        let crystal = Crystal { a: dx, b: dx, gamma: angle };
        let lattice = Lattice::new(&crystal, 3, 2);

        // Calculate shifts for x and y when shifting along y
        let dy = dx*f64::sin(angle);
        let dx_per_y = dx*f64::cos(angle);

        // Check the dimensions
        assert_eq!(Coord { x: 3.0*dx, y: 2.0*dy, z: 0.0 }, lattice.box_size);

        // ... and the coordinates
        let mut iter = lattice.coords.iter();
        assert_eq!(Some(&Coord { x: 0.0,               y: 0.0, z: 0.0 }), iter.next());
        assert_eq!(Some(&Coord { x: dx,                y: 0.0, z: 0.0 }), iter.next());
        assert_eq!(Some(&Coord { x: 2.0*dx,            y: 0.0, z: 0.0 }), iter.next());
        assert_eq!(Some(&Coord { x: dx_per_y,          y: dy,  z: 0.0 }), iter.next());
        assert_eq!(Some(&Coord { x: dx_per_y + dx,     y: dy,  z: 0.0 }), iter.next());
        assert_eq!(Some(&Coord { x: dx_per_y + 2.0*dx, y: dy,  z: 0.0 }), iter.next());
        assert_eq!(None, iter.next());
    }

    #[test]
    fn lattice_from_size() {
        // This should result in a 2-by-2 lattice
        let crystal = Crystal { a: 1.0, b: 0.5, gamma: f64::consts::PI/2.0 };
        let lattice = Lattice::from_size(&crystal, 2.1, 0.9);
        let expected = Lattice::new(&crystal, 2, 2);

        assert_eq!(expected.coords, lattice.coords);
        assert_eq!(expected.box_size, lattice.box_size);
    }

    #[test]
    fn crystal_spacing() {
        let crystal = Crystal { a: 1.0, b: 3.0, gamma: f64::consts::PI/3.0 };
        let Spacing(dx, dy, dx_per_row) = crystal.spacing();

        assert_eq!(1.0, dx);
        assert_eq!(3.0*f64::sqrt(3.0)/2.0, dy);
        assert!((1.5 - dx_per_row).abs() < 1e-6);
    }
}
