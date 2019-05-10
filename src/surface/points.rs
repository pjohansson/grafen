//! The base collection of `Points` onto which Residue's
//! can be broadcast. Beyond the creation of points (ie.
//! using a Lattice or Poisson Disc generator) all transformations
//! of the points belong in this module.

use crate::coord::Coord;
use rand;

/// A collection of points to broadcast residues onto.
pub struct Points {
    /// Box dimensions.
    pub box_size: Coord,
    /// Points.
    pub coords: Vec<Coord>,
}

impl Points {
    /// Get a copy of the `Points` in which the positions along z
    /// have been shifted by a uniform random distribution.
    /// The positions are shifted on a range of (-std_z, +std_z)
    /// where std_z is the input deviation.
    pub fn uniform_distribution(&self, std_z: f64) -> Points {
        use rand::distributions::IndependentSample;

        let range = rand::distributions::Range::new(-std_z, std_z);
        let mut rng = rand::thread_rng();

        let coords: Vec<Coord> = self
            .coords
            .iter()
            .map(|&c| {
                let add_z = range.ind_sample(&mut rng);
                //c.add(Coord::new(0.0, 0.0, add_z))
                c + Coord::new(0.0, 0.0, add_z)
            })
            .collect();

        Points {
            box_size: self.box_size,
            coords: coords,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uniform_distribution_of_positions() {
        let z0 = 1.0;
        let points = Points {
            box_size: Coord::new(1.0, 1.0, 1.0),
            coords: vec![Coord::new(0.0, 0.0, z0); 100],
        };

        let dz = 0.1;
        let distributed_points = points.uniform_distribution(dz);

        // Assert that the positions are centered around z0 with non-zero variance
        assert!(distributed_points
            .coords
            .iter()
            .all(|&c| c.z.abs() - z0 <= dz));

        let len = distributed_points.coords.len();
        let var_z: f64 = distributed_points
            .coords
            .iter()
            .map(|c| (c.z - z0) * (c.z - z0))
            .sum::<f64>()
            / (len as f64);
        assert!(var_z > 0.0);
    }
}
