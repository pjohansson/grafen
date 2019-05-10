//! Spherical objects.

use crate::{
    coord::{Coord, Translate},
    describe::{unwrap_name, Describe},
    iterator::{ResidueIter, ResidueIterOut},
    system::{Component, Residue},
    volume::*,
};

use rand::{thread_rng, distributions::{Distribution as _, Uniform}};
use serde_derive::{Deserialize, Serialize};
use std::f64::consts::PI;

#[derive(Clone, Debug, Deserialize, Serialize)]
/// A spherical volume.
pub struct Spheroid {
    pub name: Option<String>,
    pub residue: Option<Residue>,
    #[serde(skip)]
    pub origin: Coord,
    #[serde(skip)]
    pub radius: f64,
    /// A density may be set for the component.
    pub density: Option<f64>,
    #[serde(skip)]
    pub coords: Vec<Coord>,
}

impl_component![Spheroid];
impl_translate![Spheroid];

impl Spheroid {
    /// Calculate the box size.
    fn calc_box_size(&self) -> Coord {
        let diameter = 2.0 * self.radius;
        Coord::new(diameter, diameter, diameter)
    }
}

impl Contains for Spheroid {
    fn contains(&self, coord: Coord) -> bool {
        let dr = self.origin.distance(coord);
        dr <= self.radius
    }
}

impl Describe for Spheroid {
    fn describe(&self) -> String {
        format!(
            "{} (Spherical volume of radius {:.2} at {})",
            unwrap_name(&self.name),
            self.radius,
            self.origin
        )
    }

    fn describe_short(&self) -> String {
        format!("{} (Spherical volume)", unwrap_name(&self.name))
    }
}

impl Volume for Spheroid {
    fn fill(self, fill_type: FillType) -> Spheroid {
        match fill_type {
            FillType::Density(_) => {
                // Use the filling function from `Cuboid` to generate coordinates to cut from.
                // This is slightly inefficient, but for now it is easy to keep the generation
                // in a single function.
                let box_side = 2.1 * self.radius;
                let size = Coord::new(box_side, box_side, box_side);

                Cuboid {
                    name: self.name,
                    residue: self.residue,
                    origin: self.origin,
                    size,
                    ..Cuboid::default()
                }
                .fill(fill_type)
                .to_sphere(self.radius)
            }
            FillType::NumCoords(num_coords) => {
                // To fill with an exact number of coordinates, generate them explictly.
                // Note that this currently uses a generation that does not account for
                // coordinate clustering, which isn't great. The radius generation should
                // make fewer coordinates appear close to the center.
                let range_radius = Uniform::new(0.0, self.radius);
                let range_theta = Uniform::new(0.0, PI);
                let range_phi = Uniform::new(0.0, 2.0 * PI);

                let mut rng = thread_rng();

                let mut gen_coord = || {
                    let radius = range_radius.sample(&mut rng);
                    let theta = range_theta.sample(&mut rng);
                    let phi = range_phi.sample(&mut rng);

                    let x = radius * theta.sin() * phi.cos();
                    let y = radius * theta.sin() * phi.sin();
                    let z = radius * theta.cos();

                    Coord::new(x, y, z)
                };

                let coords = (0..num_coords).map(|_| gen_coord()).collect::<Vec<_>>();

                Spheroid {
                    coords,
                    ..self.clone()
                }
            }
        }
    }

    fn volume(&self) -> f64 {
        4.0 * PI * self.radius.powi(3) / 3.0
    }
}

#[cfg(test)]
mod tests {}
