use coords::Coord;

#[derive(Clone, Copy)]
pub enum LatticeType {
    Hexagonal { length: f64 },
    Triclinic { a: f64, b: f64, gamma: f64 }
}
use self::LatticeType::*;

pub struct Crystal {
    a: f64,
    b: f64,
    gamma: f64,
    crystal_type: LatticeType,
}

struct Spacing (
    f64, // Space between columns (along x) in a lattice
    f64, // Space between rows (along y)
    f64  // Adjustment per row of x
);

impl Crystal {
    pub fn from_type(input: LatticeType) -> Crystal {
        let pi = ::std::f64::consts::PI;

        match input {
            Hexagonal { length } => Crystal {
                a: length,
                b: length,
                gamma: 2.0*pi/3.0,
                crystal_type: input
            },
            Triclinic { a, b, gamma } => Crystal {
                a: a,
                b: b,
                gamma: gamma,
                crystal_type: input
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

pub struct Lattice {
    pub box_size: Coord,
    pub coords: Vec<Coord>,
}

impl Lattice {
    fn new(crystal: &Crystal, nx: u64, ny: u64) -> Lattice {
        LatticeBuilder::new(&crystal, nx, ny)
    }

    pub fn from_size(crystal: &Crystal, size_x: f64, size_y: f64) -> Lattice {
        let Spacing(dx, dy, _) = crystal.spacing();
        let (nx, ny) = ((size_x/dx).round() as u64, (size_y/dy).round() as u64);

        Lattice::new(&crystal, nx, ny)
    }
}

// Use a builder to keep the details of Lattice construction opaque
// and the proper struct of a simple form
struct LatticeBuilder {
    spacing: Spacing,
    nx: u64,
    ny: u64,
    coords: Vec<Coord>
}

impl LatticeBuilder {
    fn new(crystal: &Crystal, nx: u64, ny: u64) -> Lattice {
        let mut builder = LatticeBuilder {
            spacing: crystal.spacing(),
            nx: nx,
            ny: ny,
            coords: vec![],
        };

        match crystal.crystal_type {
            Hexagonal { length } => builder.hexagonal(),
            _                                   => builder.generic()
        };

        builder.finalize()
    }

    /// The most simple lattice contructor:
    /// Replicate all points of the crystal lattice.
    fn generic(&mut self) {
        let Spacing(dx, dy, dx_per_row) = self.spacing;

        self.coords = (0..self.ny)
            .flat_map(|row| {
                (0..self.nx)
                    .map(move |col| Coord {
                        x: (col as f64)*dx + (row as f64)*dx_per_row,
                        y: (row as f64)*dy,
                        z: 0.0,
                    })
            })
            .collect();
    }

    /// Hexagonal lattices have a honeycomb appearance
    ///
    /// This constructor ensures that the topography is correct:
    /// Every third grid point is the middle point of a cell and removed.
    /// This cell is shifted by one step in every row.
    ///
    /// To ensure that the system is perfectly periodic the number of column
    /// and rows are set to the closest multiple of 3 and 2 respectively,
    /// rounding up.
    fn hexagonal(&mut self) {
        self.nx = ((self.nx as f64 / 3.0).ceil() * 3.0) as u64;
        self.ny = ((self.ny as f64 / 2.0).ceil() * 2.0) as u64;

        let Spacing(dx, dy, dx_per_row) = self.spacing;
        self.coords = (0..self.ny)
            .flat_map(|row| {
                (0..self.nx)
                    .filter(move |col| (col + row + 1) % 3 > 0)
                    .map(move |col| Coord {
                        x: (col as f64)*dx + (row as f64)*dx_per_row,
                        y: (row as f64)*dy,
                        z: 0.0,
                    })
            })
            .collect();
    }

    /// After the lattice is created we can finalize the dimensions
    fn finalize(self) -> Lattice {
        let Spacing(dx, dy, _) = self.spacing;
        let box_size = Coord { x: (self.nx as f64)*dx, y: (self.ny as f64)*dy, z: 0.0 };

        Lattice { box_size: box_size, coords: self.coords }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::LatticeType::*;
    use ::std::f64;

    #[test]
    fn hexagonal_crystal() {
        let system = Hexagonal { length: 1.0 };
        let crystal = Crystal::from_type(system);
        assert_eq!(1.0, crystal.a);
        assert_eq!(1.0, crystal.b);
        assert_eq!(2.0*f64::consts::PI/3.0, crystal.gamma);
    }

    #[test]
    fn triclinic_crystal() {
        let system = Triclinic { a: 1.0, b: 2.0, gamma: 3.0 };
        let crystal = Crystal::from_type(system);
        assert_eq!(1.0, crystal.a);
        assert_eq!(2.0, crystal.b);
        assert_eq!(3.0, crystal.gamma);
    }

    #[test]
    fn triclinic_lattice() {
        let dx = 1.0;
        let angle = f64::consts::PI/3.0; // 60 degrees
        let crystal = Crystal::from_type(Triclinic { a: dx, b: dx, gamma: angle });
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
    fn hexagonal_lattice_has_empty_points() {
        let crystal = Crystal::from_type(Hexagonal { length: 1.0 });
        let lattice = Lattice::new(&crystal, 6, 2);

        let Spacing(dx, dy, dx_per_row) = crystal.spacing();

        // The hexagonal lattice has every third point removed to create
        // a chicken wire fence structure.
        let mut iter = lattice.coords.iter();
        assert_eq!(Some(&Coord { x: 0.0,                 y: 0.0, z: 0.0 }), iter.next());
        assert_eq!(Some(&Coord { x: dx,                  y: 0.0, z: 0.0 }), iter.next());
        // REMOVED: assert_eq!(Some(&Coord { x: 2.0*dx, y: 0.0, z: 0.0 }), iter.next());
        assert_eq!(Some(&Coord { x: 3.0*dx,              y: 0.0, z: 0.0 }), iter.next());
        assert_eq!(Some(&Coord { x: 4.0*dx,              y: 0.0, z: 0.0 }), iter.next());
        // assert_eq!(Some(&Coord { x: 5.0*dx, y: 0.0, z: 0.0 }), iter.next());
        assert_eq!(Some(&Coord { x: dx_per_row,          y: dy,  z: 0.0 }), iter.next());
        // assert_eq!(Some(&Coord { x: dx_per_y + dx, y: dy, z: 0.0 }), iter.next());
        assert_eq!(Some(&Coord { x: dx_per_row + 2.0*dx, y: dy,  z: 0.0 }), iter.next());
        assert_eq!(Some(&Coord { x: dx_per_row + 3.0*dx, y: dy,  z: 0.0 }), iter.next());
        // assert_eq!(Some(&Coord { x: dx_per_row + 4.0*dx, y: dy, z: 0.0 }), iter.next());
        assert_eq!(Some(&Coord { x: dx_per_row + 5.0*dx, y: dy,  z: 0.0 }), iter.next());
        assert_eq!(None, iter.next());
    }

    #[test]
    fn hexagonal_lattice_has_corrected_periodicity() {
        // To perfectly replicate a hexagonal lattice along x and y
        // we need to account for the chicken wire fence structure
        // which removes every third point. We require that the final
        // nx is evenly divided by 3 and ny by 2.

        // The final shape of this system should be (6, 2).
        let crystal = Crystal::from_type(Hexagonal { length: 1.0 });
        let lattice = Lattice::new(&crystal, 4, 1);
        let expected = Lattice::new(&crystal, 6, 2);

        assert_eq!(expected.coords, lattice.coords);
        assert_eq!(expected.box_size, lattice.box_size);
    }

    #[test]
    fn lattice_from_size() {
        // This should result in a 2-by-2 triclinic lattice
        let crystal = Crystal::from_type(Triclinic { a: 1.0, b: 0.5, gamma: f64::consts::PI/2.0 });
        let lattice = Lattice::from_size(&crystal, 2.1, 0.9);
        let expected = Lattice::new(&crystal, 2, 2);

        assert_eq!(expected.coords, lattice.coords);
        assert_eq!(expected.box_size, lattice.box_size);
    }

    #[test]
    fn hexagonal_lattice_from_size() {
        // This should result in a 3-by-2 hexagonal lattice
        let crystal = Crystal::from_type(Hexagonal { length: 1.0 });
        let lattice = Lattice::from_size(&crystal,  2.1, 0.9);
        let expected = Lattice::new(&crystal, 3, 2);

        assert_eq!(expected.coords, lattice.coords);
        assert_eq!(expected.box_size, lattice.box_size);

    }

    #[test]
    fn crystal_spacing() {
        let crystal = Crystal::from_type(Triclinic { a: 1.0, b: 3.0, gamma: f64::consts::PI/3.0 });
        let Spacing(dx, dy, dx_per_row) = crystal.spacing();

        assert_eq!(1.0, dx);
        assert_eq!(3.0*f64::sqrt(3.0)/2.0, dy);
        assert!((1.5 - dx_per_row).abs() < 1e-6);
    }
}
