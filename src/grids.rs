use coords::Coord;

pub struct SystemBox<T> {
    pub dimensions: Coord,
    pub coords: Vec<T>
}

pub type Grid = SystemBox<Coord>;

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
    use std::f64;
    use super::*;

    #[test]
    fn gen_a_small_hexagonal_grid() {
        // A hexagonal grid must be replicated from four base points
        // which means that a 2-by-2 input replications should result
        // in 16 grid points. Assert that the final four are
        // correctly placed.
        let spacing = get_hexagonal_spacing(1.0);
        let grid = gen_hexagonal_grid(2, 2, 1.0, spacing);

        // We generate the base grid points used in the creation
        // of the grid. The angle from the first point to the second
        // is 30 degrees which gives us the difference along x and y
        // since cos(30) = sqrt(3)/2 and sin(30) = 0.5
        let dx = f64::sqrt(3.0)/2.0;
        let dy = 0.5;
        let base = vec![
            Coord::new(0.0, 0.0,          0.0),
            Coord::new(dx,  dy,           0.0),
            Coord::new(dx,  dy + 1.0,     0.0),
            Coord::new(0.0, 2.0*dy + 1.0, 0.0)
        ];
        assert_eq!(base, &grid[0..4]);

        let spacing = Coord::new(2.0*dx, 2.0+2.0*dy, 0.0);
        let expect_last: Vec<Coord> = base.iter().map(|c| c.at_index(1, 1, 0, &spacing)).collect();
        assert_eq!(expect_last, &grid[12..]);
    }
}
