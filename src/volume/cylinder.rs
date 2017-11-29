//! Cylinder objects.

use coord::{Coord, Direction, Translate};
use describe::{unwrap_name, Describe};
use iterator::{AtomIterator, AtomIterItem};
use system::{Component, Residue};
use volume::*;

use rand;
use rand::distributions::IndependentSample;
use std::f64::consts::PI;

#[derive(Clone, Debug, Deserialize, Serialize)]
/// A cylindrical volume.
pub struct Cylinder {
    pub name: Option<String>,
    pub residue: Option<Residue>,
    pub alignment: Direction,
    #[serde(skip)]
    pub origin: Coord,
    #[serde(skip)]
    pub radius: f64,
    #[serde(skip)]
    pub height: f64,
    /// A density may be set for the component.
    pub density: Option<f64>,
    #[serde(skip)]
    pub coords: Vec<Coord>,
}

impl_component![Cylinder];
impl_translate![Cylinder];

impl Cylinder {
    /// Calculate the box size.
    fn calc_box_size(&self) -> Coord {
        let diameter = 2.0 * self.radius;

        match self.alignment {
            Direction::X => Coord::new(self.height, diameter, diameter),
            Direction::Y => Coord::new(diameter, self.height, diameter),
            Direction::Z => Coord::new(diameter, diameter, self.height),
        }
    }
}

impl Contains for Cylinder {
    fn contains(&self, coord: Coord) -> bool {
        let (dr, dh) = self.origin.distance_cylindrical(coord, self.alignment);

        dr <= self.radius && dh >= 0.0 && dh <= self.height
    }
}

impl Describe for Cylinder {
    fn describe(&self) -> String {
        format!("{} (Cylinder volume of radius {:.2} and height {:.2} at {})",
            unwrap_name(&self.name), self.radius, self.height, self.origin)
    }

    fn describe_short(&self) -> String {
        format!("{} (Cylinder volume)", unwrap_name(&self.name))
    }
}

impl Volume for Cylinder {
    fn fill(self, fill_type: FillType) -> Cylinder {
        match fill_type {
            FillType::Density(_) => {
                // Use the filling function from `Cuboid` to generate coordinates to cut from.
                // This is slightly inefficient, but for now it is easy to keep the generation
                // in a single function.
                let box_height = 1.05 * self.height;
                let box_side = 2.1 * self.radius;

                let size = match self.alignment {
                    Direction::X => Coord::new(box_height, box_side, box_side),
                    Direction::Y => Coord::new(box_side, box_height, box_side),
                    Direction::Z => Coord::new(box_side, box_side, box_height),
                };

                Cuboid {
                    name: self.name,
                    residue: self.residue,
                    origin: self.origin,
                    size,
                    .. Cuboid::default()
                }.fill(fill_type).to_cylinder(self.radius, self.height, self.alignment)
            },
            FillType::NumCoords(num_coords) => {
                // To fill with an exact number of coordinates, generate them explictly.
                // Note that this currently uses a generation that does not account for
                // coordinate clustering, which isn't great. The radius generation should
                // make fewer coordinates appear close to the center.
                let range_radius = rand::distributions::Range::new(0.0, self.radius);
                let range_height = rand::distributions::Range::new(0.0, self.height);
                let range_angle = rand::distributions::Range::new(0.0, 2.0 * PI);

                let mut rng = rand::thread_rng();

                let mut gen_coord = | | {
                    let radius = range_radius.ind_sample(&mut rng);
                    let angle = range_angle.ind_sample(&mut rng);

                    // Generalized coordinates for radial and height positions
                    let r0 = radius * angle.cos();
                    let r1 = radius * angle.sin();
                    let h = range_height.ind_sample(&mut rng);

                    match self.alignment {
                        Direction::X => Coord::new(h, r0, r1),
                        Direction::Y => Coord::new(r0, h, r1),
                        Direction::Z => Coord::new(r0, r1, h),
                    }
                };

                let coords = (0..num_coords).map(|_| gen_coord()).collect::<Vec<_>>();

                Cylinder {
                    coords,
                    .. self.clone()
                }
            }
        }
    }

    fn volume(&self) -> f64 {
        PI * self.radius.powi(2) * self.height
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fill_cylinder_with_coords() {
        let radius = 2.0;
        let height = 5.0;
        let num_coords = 100;

        let mut conf = Cylinder {
            name: None,
            residue: None,
            origin: Coord::default(),
            radius,
            height,
            density: None,
            alignment: Direction::Z,
            coords: vec![],
        };

        // Default alignment: Z
        let cylinder = conf.clone().fill(FillType::NumCoords(num_coords));
        assert_eq!(num_coords as usize, cylinder.coords.len());

        for coord in cylinder.coords {
            let (r, h) = Coord::ORIGO.distance_cylindrical(coord, Direction::Z);
            assert!(r <= cylinder.radius);
            assert!(h >= 0.0 && h <= cylinder.height);
        }

        // Along the other axes
        conf.alignment = Direction::X;
        for coord in conf.clone().fill(FillType::NumCoords(num_coords)).coords {
            let (r, h) = Coord::ORIGO.distance_cylindrical(coord, Direction::X);
            assert!(r <= cylinder.radius);
            assert!(h >= 0.0 && h <= cylinder.height);
        }

        // Along the other axes
        conf.alignment = Direction::Y;
        for coord in conf.clone().fill(FillType::NumCoords(num_coords)).coords {
            let (r, h) = Coord::ORIGO.distance_cylindrical(coord, Direction::Y);
            assert!(r <= cylinder.radius);
            assert!(h >= 0.0 && h <= cylinder.height);
        }
    }

    #[test]
    fn calc_box_size_of_cylinder() {
        let radius = 2.0;
        let height = 5.0;

        let mut cylinder = Cylinder {
            name: None,
            residue: None,
            origin: Coord::default(),
            radius,
            height,
            density: None,
            alignment: Direction::X,
            coords: vec![],
        };

        let diameter = 2.0 * radius;

        assert_eq!(Coord::new(height, diameter, diameter), cylinder.calc_box_size());

        cylinder.alignment = Direction::Y;
        assert_eq!(Coord::new(diameter, height, diameter), cylinder.calc_box_size());

        cylinder.alignment = Direction::Z;
        assert_eq!(Coord::new(diameter, diameter, height), cylinder.calc_box_size());
    }

    #[test]
    fn cylinder_contains_coordinates_in_absolute_space_depending_on_direction() {
        let mut cylinder = Cylinder {
            name: None,
            residue: None,
            origin: Coord::new(1.0, 1.0, 1.0),
            radius: 1.0,
            height: 2.0,
            density: None,
            alignment: Direction::X,
            coords: vec![],
        };

        let err = 1e-9;

        // Inside
        assert!(cylinder.contains(Coord::new(1.0 + err, 1.0, 1.0)));
        assert!(cylinder.contains(Coord::new(3.0 - err, 1.0, 1.0)));
        assert!(cylinder.contains(Coord::new(1.0 + err, 2.0 - err, 1.0)));
        assert!(cylinder.contains(Coord::new(1.0 + err, 1.0, 2.0 - err)));

        // Outside
        assert!(!cylinder.contains(Coord::new(1.0 - err, 1.0, 1.0)));
        assert!(!cylinder.contains(Coord::new(3.0 + err, 1.0, 1.0)));
        assert!(!cylinder.contains(Coord::new(1.0 + err, 2.0 + err, 1.0)));
        assert!(!cylinder.contains(Coord::new(1.0 + err, 2.0, 2.0 + err)));

        cylinder.alignment = Direction::Y;

        // Inside
        assert!(cylinder.contains(Coord::new(1.0, 1.0 + err, 1.0)));
        assert!(cylinder.contains(Coord::new(1.0, 3.0 - err, 1.0)));
        assert!(cylinder.contains(Coord::new(2.0 - err, 1.0 + err, 1.0)));
        assert!(cylinder.contains(Coord::new(1.0, 1.0 + err, 2.0 - err)));

        // Outside
        assert!(!cylinder.contains(Coord::new(1.0, 1.0 - err, 1.0)));
        assert!(!cylinder.contains(Coord::new(1.0, 3.0 + err, 1.0)));
        assert!(!cylinder.contains(Coord::new(2.0 + err, 1.0 + err, 1.0)));
        assert!(!cylinder.contains(Coord::new(1.0, 1.0 + err, 2.0 + err)));

        cylinder.alignment = Direction::Z;

        // Inside
        assert!(cylinder.contains(Coord::new(1.0, 1.0, 1.0 + err)));
        assert!(cylinder.contains(Coord::new(1.0, 1.0, 3.0 - err)));
        assert!(cylinder.contains(Coord::new(2.0 - err, 1.0, 1.0 + err)));
        assert!(cylinder.contains(Coord::new(1.0, 2.0 - err, 3.0 - err)));

        // Outside
        assert!(!cylinder.contains(Coord::new(1.0, 1.0, 1.0 - err)));
        assert!(!cylinder.contains(Coord::new(1.0, 1.0, 3.0 + err)));
        assert!(!cylinder.contains(Coord::new(2.0 + err, 1.0, 1.0 + err)));
        assert!(!cylinder.contains(Coord::new(1.0, 2.0 + err, 3.0 - err)));
    }

    #[test]
    fn cylinder_volume_is_correct() {
        let radius = 2.0;
        let height = 5.0;

        let cylinder = Cylinder {
            name: None,
            residue: None,
            origin: Coord::ORIGO,
            radius,
            height,
            density: None,
            alignment: Direction::X,
            coords: vec![],
        };

        let base = PI * radius * radius;
        assert_eq!(cylinder.volume(), base * height);
    }

    #[test]
    fn cylinder_from_density_makes_an_expected_number_of_coordinates() {
        let density = 100.0;

        let radius = 2.5;
        let height = 1.0;

        let cylinder = Cylinder {
            name: None,
            residue: None,
            origin: Coord::ORIGO,
            radius,
            height,
            density: None,
            alignment: Direction::Y,
            coords: vec![],
        }.fill(FillType::Density(density));

        let expected_coords = (cylinder.volume() * density).round() as usize;
        let ratio = cylinder.coords.len() as f64 / expected_coords as f64;

        assert!(ratio >= 0.95 && ratio <= 1.05);
    }
}
