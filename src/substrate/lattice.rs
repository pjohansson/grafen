//! Implements the `Lattice` struct which as the name suggests contains
//! information about and grid coordinates of lattices. It comes
//! with easy-to-use constructors for different lattice types.

use substrate::points::Points;
use system::Coord;

/// A lattice with coordinates of its grid and a total size.
///
/// The lattice is constructed using its builder methods
/// for the various types of lattices.
pub struct Lattice {
    /// Size of the lattice box.
    pub box_size: Coord,
    /// List of coordinates belonging to the lattice.
    pub coords: Vec<Coord>,
}

impl Lattice {
    /// Constructor for a hexagonal lattice with spacing a.
    pub fn hexagonal(a: f64) -> LatticeBuilder {
        let crystal = Crystal::hexagonal(a);
        LatticeBuilder::new(crystal)
    }

    /// Constructor for a triclinic lattice with vectors of length (a, b)
    /// separated by an angle gamma in radians.
    pub fn triclinic(a: f64, b: f64, gamma: f64) -> LatticeBuilder {
        let crystal = Crystal::triclinic(a, b, gamma);
        LatticeBuilder::new(crystal)
    }
}

/// Constructor for a Lattice.
pub struct LatticeBuilder {
    crystal: Crystal,
    nx: u64,
    ny: u64,
}

// Use a builder to keep the details of Lattice construction opaque
// and the proper struct in a simple form.
impl LatticeBuilder {
    /// Set the desired size of the Lattice. The final size is adjusted
    /// along both directions to the closest multiple of the calculated
    /// crystal spacing. As such the system is prepared to be periodically
    /// replicated.
    pub fn with_size(self, size_x: f64, size_y: f64) -> LatticeBuilder {
        let Spacing(dx, dy, _) = self.crystal.spacing();
        let nx = (size_x / dx).round() as u64;
        let ny = (size_y / dy).round() as u64;

        self.with_bins(nx, ny)
    }

    /// Finalize and return the Lattice. Note that if a desired size has
    /// not been set the lattice will be empty.
    pub fn finalize(mut self) -> Points {
        let coords = match self.crystal.lattice_type {
            Hexagonal => self.hexagonal(),
            _ => self.generic(),
        };

        let Spacing(dx, dy, _) = self.crystal.spacing();
        let box_size = Coord::new((self.nx as f64) * dx, (self.ny as f64) * dy, 0.0);

        Points {
            box_size: box_size,
            coords: coords,
        }
    }

    fn new(crystal: Crystal) -> LatticeBuilder {
        LatticeBuilder {
            crystal: crystal,
            nx: 0,
            ny: 0,
        }
    }

    fn with_bins(mut self, nx: u64, ny: u64) -> LatticeBuilder {
        self.nx = nx;
        self.ny = ny;
        self
    }

    /// The most simple lattice contructor:
    /// Replicate all points of the crystal lattice.
    fn generic(&mut self) -> Vec<Coord> {
        let Spacing(dx, dy, dx_per_row) = self.crystal.spacing();

        (0..self.ny)
            .flat_map(|row| {
                (0..self.nx)
                    .map(move |col| {
                        Coord::new((col as f64) * dx + (row as f64) * dx_per_row,
                                   (row as f64) * dy,
                                   0.0)
                        })
            })
            .collect()
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
    fn hexagonal(&mut self) -> Vec<Coord> {
        self.nx = ((self.nx as f64 / 3.0).ceil() * 3.0) as u64;
        self.ny = ((self.ny as f64 / 2.0).ceil() * 2.0) as u64;
        let Spacing(dx, dy, dx_per_row) = self.crystal.spacing();

        (0..self.ny)
            .flat_map(|row| {
                (0..self.nx)
                    .filter(move |col| (col + row + 1) % 3 > 0)
                    .map(move |col| {
                            Coord::new((col as f64) * dx + (row as f64) * dx_per_row,
                                       (row as f64) * dy,
                                       0.0)
                        })
            })
            .collect()
    }
}

enum LatticeType {
    Hexagonal,
    Triclinic,
}
use self::LatticeType::*;

/// A crystal base for a 2D lattice. It consists of two vectors
/// who are used to step onto neighbouring lattice sites.
struct Crystal {
    /// Vector length a.
    a: f64,
    /// Vector length b.
    b: f64,
    /// Angle (in radians) between vectors a and b.
    gamma: f64,
    /// Type of lattice.
    lattice_type: LatticeType,
}

/// Constructors of crystal bases from which lattices are replicated.
impl Crystal {
    /// Hexagon lattices are created with a common vector length and an angle of 120 degrees.
    fn hexagonal(a: f64) -> Crystal {
        Crystal {
            a: a,
            b: a,
            gamma: 2.0 * ::std::f64::consts::PI / 3.0, // 120 degrees
            lattice_type: Hexagonal,
        }
    }

    /// Triclinic lattics have two vectors of length (a, b) separated by an angle gamma.
    fn triclinic(a: f64, b: f64, gamma: f64) -> Crystal {
        Crystal {
            a: a,
            b: b,
            gamma: gamma,
            lattice_type: Triclinic,
        }
    }

    fn spacing(&self) -> Spacing {
        let dx = self.a;
        let dy = self.b * self.gamma.sin();
        let dx_per_row = self.b * self.gamma.cos();

        Spacing(dx, dy, dx_per_row)
    }
}

struct Spacing(f64, // Space between columns (along x) in a lattice
               f64, // Space between rows (along y)
               f64); // Adjustment per row of x

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64;

    #[test]
    fn hexagonal_crystal() {
        let crystal = Crystal::hexagonal(1.0);
        assert_eq!(1.0, crystal.a);
        assert_eq!(1.0, crystal.b);
        assert_eq!(2.0 * f64::consts::PI / 3.0, crystal.gamma);
    }

    #[test]
    fn triclinic_crystal() {
        let crystal = Crystal::triclinic(1.0, 2.0, 3.0);
        assert_eq!(1.0, crystal.a);
        assert_eq!(2.0, crystal.b);
        assert_eq!(3.0, crystal.gamma);
    }

    #[test]
    fn triclinic_lattice() {
        let dx = 1.0;
        let angle = 60f64.to_radians();

        let lattice = Lattice::triclinic(dx, dx, angle)
            .with_bins(3, 2)
            .finalize();

        // Calculate shifts for x and y when shifting along y
        let dy = dx * angle.sin();
        let dx_per_y = dx * angle.cos();

        // Check the dimensions
        assert_eq!(Coord::new(3.0 * dx, 2.0 * dy, 0.0), lattice.box_size);

        // ... and the coordinates
        let mut iter = lattice.coords.iter();
        assert_eq!(Some(&Coord::new(0.0, 0.0, 0.0)), iter.next());
        assert_eq!(Some(&Coord::new(dx, 0.0, 0.0)), iter.next());
        assert_eq!(Some(&Coord::new(2.0 * dx, 0.0, 0.0)), iter.next());
        assert_eq!(Some(&Coord::new(dx_per_y, dy, 0.0)), iter.next());
        assert_eq!(Some(&Coord::new(dx_per_y + dx, dy, 0.0)), iter.next());
        assert_eq!(Some(&Coord::new(dx_per_y + 2.0 * dx, dy, 0.0)), iter.next());
        assert_eq!(None, iter.next());
    }

    #[test]
    fn hexagonal_lattice_has_empty_points() {
        let lattice = Lattice::hexagonal(1.0).with_bins(6, 2).finalize();

        let crystal = Crystal::hexagonal(1.0);
        let Spacing(dx, dy, dx_per_row) = crystal.spacing();

        // The hexagonal lattice has every third point removed to create
        // a chicken wire fence structure.
        let mut iter = lattice.coords.iter();
        assert_eq!(Some(&Coord::new(0.0, 0.0, 0.0)), iter.next());
        assert_eq!(Some(&Coord::new(dx, 0.0, 0.0)), iter.next());
        // REMOVED: assert_eq!(Some(&Coord::new(2.0*dx, 0.0, 0.0)), iter.next());
        assert_eq!(Some(&Coord::new(3.0 * dx, 0.0, 0.0)), iter.next());
        assert_eq!(Some(&Coord::new(4.0 * dx, 0.0, 0.0)), iter.next());
        // REMOVED: assert_eq!(Some(&Coord::new(5.0*dx, 0.0, 0.0)), iter.next());
        assert_eq!(Some(&Coord::new(dx_per_row, dy, 0.0)), iter.next());
        // REMOVED: assert_eq!(Some(&Coord::new(dx_per_y + dx, dy, 0.0)), iter.next());
        assert_eq!(Some(&Coord::new(dx_per_row + 2.0 * dx, dy, 0.0)), iter.next());
        assert_eq!(Some(&Coord::new(dx_per_row + 3.0 * dx, dy, 0.0)), iter.next());
        // REMOVED: assert_eq!(Some(&Coord::new(dx_per_row + 4.0*dx, dy, 0.0)), iter.next());
        assert_eq!(Some(&Coord::new(dx_per_row + 5.0 * dx, dy, 0.0)), iter.next());
        assert_eq!(None, iter.next());
    }

    #[test]
    fn hexagonal_lattice_has_corrected_periodicity() {
        // To perfectly replicate a hexagonal lattice along x and y
        // we need to account for the chicken wire fence structure
        // which removes every third point. We require that the final
        // nx is evenly divided by 3 and ny by 2.

        // The final shape of this system should be (6, 2).
        let lattice = Lattice::hexagonal(1.0).with_bins(4, 1).finalize();
        let expected = Lattice::hexagonal(1.0).with_bins(6, 2).finalize();

        assert_eq!(expected.coords, lattice.coords);
        assert_eq!(expected.box_size, lattice.box_size);
    }

    #[test]
    fn lattice_with_size() {
        // This should result in a 2-by-2 triclinic lattice
        let lattice = Lattice::triclinic(1.0, 0.5, 90f64.to_radians())
            .with_size(2.1, 0.9)
            .finalize();
        let expected = Lattice::triclinic(1.0, 0.5, 90f64.to_radians())
            .with_bins(2, 2)
            .finalize();

        assert_eq!(expected.coords, lattice.coords);
        assert_eq!(expected.box_size, lattice.box_size);
    }

    #[test]
    fn hexagonal_lattice_with_size() {
        // This should result in a 3-by-2 hexagonal lattice
        let lattice = Lattice::hexagonal(1.0).with_size(2.1, 0.9).finalize();
        let expected = Lattice::hexagonal(1.0).with_bins(3, 2).finalize();

        assert_eq!(expected.coords, lattice.coords);
        assert_eq!(expected.box_size, lattice.box_size);
    }

    #[test]
    fn lattice_constructed_without_size_is_empty() {
        let lattice = Lattice::hexagonal(1.0).finalize();

        assert_eq!(Coord::new(0.0, 0.0, 0.0), lattice.box_size);
        assert!(lattice.coords.is_empty());
    }

    #[test]
    fn crystal_spacing() {
        let crystal = Crystal::triclinic(1.0, 3.0, f64::consts::PI / 3.0);
        let Spacing(dx, dy, dx_per_row) = crystal.spacing();

        assert_eq!(1.0, dx);
        assert_eq!(3.0 * 3.0f64.sqrt() / 2.0, dy);
        assert!((1.5 - dx_per_row).abs() < 1e-6);
    }
}
