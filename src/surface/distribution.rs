//! Implement a Poisson Disc distribution algorithm.

use crate::{
    coord::Coord,
    surface::points::Points
};

use rand;
use std::cmp;

/// Container for constructing different randomly sampled distributions.
pub struct Distribution;

impl Distribution {
    /// Return a set of points that have been generated using a Poisson disk sampling
    /// algorithm. They will be separated from each other at minimum by an input distance.
    pub fn poisson(rmin: f64, size_x: f64, size_y: f64) -> Points {
        self::density::PoissonDistribution::new(rmin, size_x, size_y)
    }

    /// Return a set of an input number of points that have been generated using
    /// a blue noise sampling algorithm.
    pub fn blue_noise(num_points: u64, size_x: f64, size_y: f64) -> Points {
        self::number::BlueNoiseDistribution::new(num_points, size_x, size_y)
    }
}

mod number {
    use rand::distributions::IndependentSample;
    use super::*;

    pub struct BlueNoiseDistribution;

    impl BlueNoiseDistribution {
        // This algorithm is quite expensive since every candidate has to be checked
        // against every coordinate that has been constructed, and the number of candidates
        // which we create increases equally. Thus it scales very poorly with the number
        // of points that are created.
        //
        // Could be sped up by implementing a grid for the final coordinates and compare
        // every candidate to those inside a neighbouring area only. Also by making it
        // parallel, but more power cannot substitute bad algorithms.
        //
        // **Note that before any performance improvements are made, a benchmark test should
        // be created!**
        pub fn new(num_points: u64, size_x: f64, size_y: f64) -> Points {
            let mut coords = vec![gen_coord(size_x, size_y)];
            const NUM_CANDIDATES_MULTIPLIER: u64 = 1;

            for i in 1..num_points {
                let mut current_best = gen_coord(size_x, size_y);
                let mut max_dist = calc_min_dist(current_best, &coords, size_x, size_y);

                for _ in 0..(NUM_CANDIDATES_MULTIPLIER * i) {
                    let candidate = gen_coord(size_x, size_y);
                    let dist = calc_min_dist(candidate, &coords, size_x, size_y);

                    if dist > max_dist {
                        max_dist = dist;
                        current_best = candidate;
                    }
                };

                coords.push(current_best);
            }

            Points {
                box_size: Coord::new(size_x, size_y, 0.0),
                coords,
            }
        }
    }

    pub fn calc_min_dist(coord: Coord, samples: &[Coord], size_x: f64, size_y: f64) -> f64 {
        use std::f64::MAX;

        samples.iter()
               .fold(MAX, |dist, &other| {
                    dist.min(calc_toroidal_distance(coord, other, size_x, size_y))
               })
    }

    pub fn calc_toroidal_distance(coord: Coord, other: Coord, size_x: f64, size_y: f64) -> f64 {
        let mut dx = (coord.x - other.x).abs();
        let mut dy = (coord.y - other.y).abs();

        if dx > size_x / 2.0 {
           dx = size_x - dx;
        }

        if dy > size_y / 2.0 {
            dy = size_y - dy;
        }

        dx.powi(2) + dy.powi(2)
    }

    fn gen_coord(dx: f64, dy: f64) -> Coord {
        let mut rng = rand::thread_rng();
        let range_x = rand::distributions::Range::new(0.0, dx);
        let range_y = rand::distributions::Range::new(0.0, dy);

        Coord::new(range_x.ind_sample(&mut rng), range_y.ind_sample(&mut rng), 0.0)
    }
}

mod density {
    use rand::distributions::IndependentSample;
    use super::*;

    pub struct PoissonDistribution;

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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_poisson_distribution() {
        let rmin = 1.0;
        let (size_x, size_y) = (5.0, 10.0);
        let distribution = density::PoissonDistribution::new(rmin, size_x, size_y);

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

    #[test]
    fn create_blue_noise_distribution() {
        let num_points = 152;
        let (size_x, size_y) = (5.0, 10.0);
        let distribution = number::BlueNoiseDistribution::new(num_points, size_x, size_y);

        // We can only (easily) assert that we have the input number of points
        // and that none are outside of the box.
        assert_eq!(distribution.coords.len() , num_points as usize);

        for coord in distribution.coords {
            assert!(coord.x >= 0.0 && coord.x <= size_x);
            assert!(coord.y >= 0.0 && coord.y <= size_y);
        }
    }

    #[test]
    fn calculate_toroidal_distance_is_squared_and_wraps_coordinates_to_closest() {
        let coord = Coord::new(0.0, 0.0, 10.0); // skip the z value
        let other = Coord::new(1.5, 2.5, -10.0);

        let (size_x, size_y) = (2.0_f64, 3.0_f64);

        // Both coordinates are closer by wrapping around! Also, ignore the z coordinate!
        let dist = (size_x - 1.5).powi(2) + (size_y - 2.5).powi(2);

        assert_eq!(number::calc_toroidal_distance(coord, other, size_x, size_y), dist);
    }

    #[test]
    fn calculate_min_distance_squared_between_a_coordinate_and_a_set() {
        let coord = Coord::new(2.0, 2.0, 2.0);

        let candidates = vec![
            Coord::new(0.0, 0.0, 0.0), // distance squared: 2^2 + 2^2 =  8
            Coord::new(3.0, 1.0, 3.0), //                   1^2 + 1^2 =  2, minimum!
            Coord::new(5.0, 5.0, 5.0)  //                   3^2 + 3^2 = 18
        ];

        let dist = 2.0 * 1.0_f64.powi(2);

        let (size_x, size_y) = (10.0, 10.0);

        assert_eq!(number::calc_min_dist(coord, &candidates, size_x, size_y), dist);
    }
}
