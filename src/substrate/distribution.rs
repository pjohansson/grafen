//! Implement a Poisson Disc distribution algorithm.

use rand;
use rand::distributions::IndependentSample;
use std::cmp;

use substrate::points::Points;
use system::Coord;

pub struct PoissonDistribution {}

impl PoissonDistribution {
    pub fn new(rmin: f64, size_x: f64, size_y: f64) -> Points {
        let mut grid = PoissonGrid::new(rmin, size_x, size_y);

        let init_coord = gen_grid_coord(size_x, size_y);
        let mut active: Vec<Coord> = vec![init_coord];
        grid.set_coord(init_coord).expect("There was an error when creating the Poisson disc distribution");

        while !active.is_empty() {
            let index = select_coordinate(&active);

            if let Some(candidate) = find_candidate(&active[index], &grid) {
                if grid.set_coord(candidate).is_ok() {
                    active.push(candidate);
                }
            } else {
                active.remove(index);
            };
        }

        Points {
            box_size: Coord::new(size_x, size_y, 0.0),
            coords: grid.into_coords(),
        }
    }
}

struct PoissonGrid {
    spacing: f64,
    rmin: f64,
    size: (f64, f64),
    shape: (usize, usize),
    cells: Vec<Option<Coord>>,
}

impl PoissonGrid {
    fn new(rmin: f64, size_x: f64, size_y: f64) -> PoissonGrid {
        let a = rmin / 2.0f64.sqrt();
        let nx = (size_x / a).ceil() as usize;
        let ny = (size_y / a).ceil() as usize;

        PoissonGrid {
            spacing: a,
            rmin: rmin,
            size: (size_x, size_y),
            shape: (nx, ny),
            cells: vec![None; nx * ny],
        }
    }

    fn cell_at_position(&self, col: usize, row: usize) -> usize {
        let (nx, _) = self.shape;
        row * nx + col
    }

    fn cell_at_coord(&self, coord: &Coord) -> usize {
        let col = (coord.x / self.spacing).floor() as usize;
        let row = (coord.y / self.spacing).floor() as usize;
        self.cell_at_position(col, row)
    }

    fn collision(&self, coord: &Coord) -> bool {
        let index = self.cell_at_coord(&coord);
        self.get_neighbours(index)
            .iter()
            .filter_map(|opt| opt.map(|c| c.distance(*coord)))
            .any(|r| r < self.rmin)
    }

    fn get_neighbours(&self, index: usize) -> Vec<Option<Coord>> {
        let (nx, ny) = self.shape;
        let i = (index % nx) as isize;
        let j = (index / nx) as isize;

        let mut neighbours = Vec::new();

        // Since rmin = sqrt(2) * spacing we need to check cells
        // that are up to two positions away
        let (imin, imax) = (cmp::max(0, i - 2), cmp::min(nx as isize, i + 3));
        let (jmin, jmax) = (cmp::max(0, j - 2), cmp::min(ny as isize, j + 3));

        for col in imin..imax {
            for row in jmin..jmax {
                let neighbour_index = self.cell_at_position(col as usize, row as usize);
                neighbours.push(self.cells[neighbour_index]);
            }
        }

        neighbours
    }

    fn into_coords(self) -> Vec<Coord> {
        self.cells.iter().filter_map(|&c| c).collect()
    }

    fn set_coord(&mut self, coord: Coord) -> Result<(), &'static str> {
        let index = self.cell_at_coord(&coord);

        // Consistency check for the algorithm: this should never be reached
        // but we prefer to not panic if it happens
        if self.cells[index].is_some() {
            return Err("Cannot add a coordinate to a cell which is already occupied");
        }

        self.cells[index] = Some(coord);
        Ok(())
    }
}

fn find_candidate(coord: &Coord, grid: &PoissonGrid) -> Option<Coord> {
    const NUM_CANDIDATES: usize = 30;

    for _ in 0..NUM_CANDIDATES {
        let candidate = gen_coord_around(&coord, &grid);

        if !grid.collision(&candidate) {
            return Some(candidate);
        }
    }

    None
}

fn gen_coord_around(coord: &Coord, grid: &PoissonGrid) -> Coord {
    use std::f64::consts::PI;
    let mut rng = rand::thread_rng();
    let range_dr = rand::distributions::Range::new(grid.rmin, 2.0 * grid.rmin);
    let range_angle = rand::distributions::Range::new(0.0, 2.0 * PI);

    let (max_x, max_y) = grid.size;

    loop {
        let dr = range_dr.ind_sample(&mut rng);
        let angle = range_angle.ind_sample(&mut rng);
        let x = coord.x + dr * angle.cos();
        let y = coord.y + dr * angle.sin();

        if x >= 0.0 && x < max_x && y >= 0.0 && y < max_y {
            return Coord::new(x, y, 0.0);
        }
    }
}

fn gen_grid_coord(x: f64, y: f64) -> Coord {
    let mut rng = rand::thread_rng();
    let range_x = rand::distributions::Range::new(0.0, x);
    let range_y = rand::distributions::Range::new(0.0, y);

    Coord::new(range_x.ind_sample(&mut rng), range_y.ind_sample(&mut rng), 0.0)
}

fn select_coordinate(coords: &Vec<Coord>) -> usize {
    let mut rng = rand::thread_rng();
    let range = rand::distributions::Range::new(0, coords.len());

    range.ind_sample(&mut rng)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_poisson_distribution() {
        let rmin = 1.0;
        let (size_x, size_y) = (5.0, 10.0);
        let distribution = PoissonDistribution::new(rmin, size_x, size_y);

        // We can only assert that no coordinates are within the minimum
        // distance of each other, or outside the box.
        assert!(distribution.coords.len() > 0);

        for (i, &x1) in distribution.coords.iter().enumerate() {
            assert!(x1.x >= 0.0 && x1.x <= size_x);
            assert!(x1.y >= 0.0 && x1.y <= size_y);

            for &x2 in distribution.coords.iter().skip(i + 1) {
                assert!(Coord::distance(x1, x2) >= rmin);
            }
        }
    }
}
