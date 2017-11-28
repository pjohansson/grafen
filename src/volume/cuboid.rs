//! Cuboid objects.

use coord::{Coord, Direction, Periodic, Translate};
use describe::{unwrap_name, Describe};
use iterator::{AtomIterator, AtomIterItem};
use system::{Component, Residue};
use volume::*;

use rand;

#[derive(Clone, Debug, Deserialize, Serialize)]
/// A cuboid shaped volume box.
pub struct Cuboid {
    /// Component name.
    pub name: Option<String>,
    /// Component residue.
    pub residue: Option<Residue>,
    #[serde(skip)]
    /// Origin position of component.
    pub origin: Coord,
    #[serde(skip)]
    /// Size of component (nm).
    pub size: Coord,
    /// A density may be set for the component.
    pub density: Option<f64>,
    #[serde(skip)]
    /// Residue coordinates of component, relative to its `origin`.
    pub coords: Vec<Coord>,
}

impl_component![Cuboid];
impl_translate![Cuboid];

impl Cuboid {
    /// Calculate the center position of the cuboid, relative to the origin.
    fn center(&self) -> Coord {
        Coord { x: self.size.x / 2.0, y: self.size.y / 2.0, z: self.size.z / 2.0 }
    }

    /// Calculate the box size.
    fn calc_box_size(&self) -> Coord {
        self.size
    }

    /// Construct a `Cylinder` from the cuboid by cutting its coordinates.
    /// It will be directed along the default cylinder alignment.
    fn to_cylinder(&self, radius: f64, height: f64, alignment: Direction) -> Cylinder {
        // Check if we need to extend the cube to create the complete cylinder.
        let diameter = 2.0 * radius;
        let pbc_multiples = match alignment {
            Direction::X => {(
                (height / self.size.x).ceil() as usize,
                (diameter / self.size.y).ceil() as usize,
                (diameter / self.size.z).ceil() as usize
            )},
            Direction::Y => {(
                (diameter / self.size.x).ceil() as usize,
                (height / self.size.y).ceil() as usize,
                (diameter / self.size.z).ceil() as usize
            )},
            Direction::Z => {(
                (diameter / self.size.x).ceil() as usize,
                (diameter / self.size.y).ceil() as usize,
                (height / self.size.z).ceil() as usize
            )},
        };

        // Closure to calculate the coordinate in the center of the "bottom"
        // cuboid face from which the cylinder will be created.
        let get_bottom_center = |cuboid: &Cuboid| {
            match alignment {
                    Direction::X => Coord { x: 0.0, .. cuboid.center() },
                    Direction::Y => Coord { y: 0.0, .. cuboid.center() },
                    Direction::Z => Coord { z: 0.0, .. cuboid.center() },
            }
        };

        let coords = match pbc_multiples {
            (1, 1, 1) => {
                let bottom_center = get_bottom_center(&self);
                cut_to_cylinder(&self.coords, bottom_center, alignment, radius, height)
            },
            (nx, ny, nz) => {
                let extended = self.pbc_multiply(nx, ny, nz);
                let bottom_center = get_bottom_center(&self);
                cut_to_cylinder(&extended.coords, bottom_center, alignment, radius, height)
            },
        };

        Cylinder {
            name: None,
            residue: self.residue.clone(),
            origin: self.origin,
            radius,
            height,
            density: self.density,
            alignment,
            coords,
        }
    }

    /// Construct a `Sphere` from the cuboid by cutting its coordinates.
    fn to_sphere(&self, radius: f64) -> Sphere {
        // Check whether we need to extend the cuboid to create the full sphere
        let diameter = 2.0 * radius;
        let pbc_multiples = (
            (diameter / self.size.x).ceil() as usize,
            (diameter / self.size.y).ceil() as usize,
            (diameter / self.size.z).ceil() as usize,
        );

        let coords = match pbc_multiples {
            (1, 1, 1) => {
                cut_to_sphere(&self.coords, self.center(), radius)
            },
            (nx, ny, nz) => {
                let extended = self.pbc_multiply(nx, ny, nz);
                cut_to_sphere(&extended.coords, extended.center(), radius)
            }
        };

        Sphere {
            origin: self.origin,
            radius,
            coords,
        }
    }
}

impl Contains for Cuboid {
    fn contains(&self, coord: Coord) -> bool {
        let (x, y, z) = coord.to_tuple();
        let (x0, y0, z0) = self.origin.to_tuple();
        let (x1, y1, z1) = (self.origin + self.size).to_tuple();

        x >= x0 && x <= x1 && y >= y0 && y <= y1 && z >= z0 && z <= z1
    }
}

impl Default for Cuboid {
    fn default() -> Cuboid {
        Cuboid {
            name: None,
            residue: None,
            origin: Coord::ORIGO,
            size: Coord::ORIGO,
            density: None,
            coords: vec![],
        }
    }
}

impl Describe for Cuboid {
    fn describe(&self) -> String {
        format!("{} (Box of size {} at {})", unwrap_name(&self.name), self.size, self.origin)
    }

    fn describe_short(&self) -> String {
        format!("{} (Box)", unwrap_name(&self.name))
    }
}

impl Periodic for Cuboid {
    /// Clone cuboid coordinates into PBC multiples.
    fn pbc_multiply(&self, nx: usize, ny: usize, nz: usize) -> Cuboid {
        let coords = pbc_multiply_volume(&self.coords, self.size, nx, ny, nz);

        Cuboid {
            origin: self.origin,
            size: self.size.pbc_multiply(nx, ny, nz),
            coords,
            // TODO: Add explicit parameters here
            .. self.clone()
        }
    }
}

impl Volume for Cuboid {
    fn fill(self, fill_type: FillType) -> Cuboid {
        let num_coords = fill_type.to_num_coords(&self);

        // To fill the cuboid in a uniform manner, construct a lattice grid which can contain
        // the desired number of atoms. Then, select the desired number of cells from this
        // list and add their corresponding coordinate.
        let cell_volume = self.volume() / (num_coords as f64);
        let target_cell_length = cell_volume.powf(1.0 / 3.0);

        // Use `ceil` since we want the upper limit of available cells
        let nx = (self.size.x / target_cell_length).ceil() as u64;
        let ny = (self.size.y / target_cell_length).ceil() as u64;
        let nz = (self.size.z / target_cell_length).ceil() as u64;
        let num_cells = nx * ny * nz;

        let mut rng = rand::thread_rng();
        let selected_indices = rand::sample(&mut rng, 0..num_cells, num_coords as usize);

        let dx = self.size.x / (nx as f64);
        let dy = self.size.y / (ny as f64);
        let dz = self.size.z / (nz as f64);

        let coords = selected_indices
            .into_iter()
            .map(|i| {
                let ix = i % nx;
                let iy = (i / nx) % ny;
                let iz = i / (nx * ny);

                Coord::new(
                    dx * (ix as f64 + 0.5),
                    dy * (iy as f64 + 0.5),
                    dz * (iz as f64 + 0.5)
                )
            })
            .collect::<Vec<_>>();

        let density = Some((num_coords as f64) / self.volume());

        Cuboid {
            density,
            coords,
            .. self
        }
    }

    fn volume(&self) -> f64 {
        self.size.x * self.size.y * self.size.z
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_cuboid(dx: f64, dy: f64, dz: f64, spacing: f64) -> Cuboid {
        let mut coords = Vec::new();

        let mut x = 0.0;
        while x < dx - spacing {
            let mut y = 0.0;

            while y < dy - spacing {
                let mut z = 0.0;

                while z < dz - spacing {
                    coords.push(Coord::new(x, y, z));
                    z += spacing;
                }
                y += spacing;
            }
            x += spacing;
        }

        Cuboid {
            size: Coord::new(dx, dy, dz),
            coords,
            .. Cuboid::default()
        }
    }

    #[test]
    fn translate_a_cuboid() {
        let translate = Coord::new(1.0, 2.0, 3.0);
        let cuboid = setup_cuboid(0.0, 0.0, 0.0, 0.0).translate(translate);
        assert_eq!(translate, cuboid.origin);
    }

    #[test]
    fn calculate_cuboid_center() {
        let cuboid = setup_cuboid(1.0, 1.0, 1.0, 0.1);
        let center = Coord::new(0.5, 0.5, 0.5);

        assert_eq!(center, cuboid.center());
    }

    #[test]
    fn translated_cuboid_center_is_correct() {
        let translate = Coord::new(1.0, 1.0, 1.0);
        let cuboid = setup_cuboid(1.0, 1.0, 1.0, 0.1).translate(translate);
        let center = Coord::new(0.5, 0.5, 0.5);

        assert_eq!(center, cuboid.center());
    }

    #[test]
    fn calc_box_size_of_cuboid() {
        let cuboid = setup_cuboid(1.0, 2.0, 3.0, 0.1);

        assert_eq!(Coord::new(1.0, 2.0, 3.0), cuboid.calc_box_size());
    }

    #[test]
    fn cuboid_into_cylinder() {
        let translate = Coord::new(1.0, 2.0, 3.0);
        let cuboid = setup_cuboid(10.0, 10.0, 10.0, 1.0).translate(translate);

        let radius = 2.5;
        let height = 8.0;
        let alignment = Direction::Z;
        let cylinder = cuboid.to_cylinder(radius, height, alignment);

        assert!(cylinder.coords.len() > 0);
        assert_eq!(cuboid.origin, cylinder.origin);

        for coord in cylinder.coords {
            let (dr, dh) = Coord::ORIGO.distance_cylindrical(coord, alignment);
            assert!(dr <= radius);
            assert!(dh >= 0.0 && dh <= height);
        }
    }

    #[test]
    fn cuboid_expands_to_create_full_cylinder_if_too_small() {
        let cuboid = setup_cuboid(10.0, 10.0, 10.0, 1.0);

        let too_large_radius = 10.0;
        let too_large_height = 15.0;
        let alignment = Direction::Z;
        let large_cylinder = cuboid.to_cylinder(too_large_radius, too_large_height,
            alignment);

        assert!(large_cylinder.coords.len() > cuboid.coords.len());
    }

    #[test]
    fn cuboid_into_sphere() {
        let translate = Coord::new(1.0, 2.0, 3.0);
        let cuboid = setup_cuboid(10.0, 10.0, 10.0, 1.0).translate(translate);

        let radius = 5.0;
        let sphere = cuboid.to_sphere(radius);

        assert!(sphere.coords.len() > 0);
        assert_eq!(cuboid.origin, sphere.origin);

        for coord in sphere.coords {
            assert!(coord.distance(Coord::ORIGO) <= radius);
        }
    }

    #[test]
    fn cuboid_expands_to_create_full_sphere_if_too_small() {
        let cuboid = setup_cuboid(10.0, 10.0, 10.0, 1.0);

        let too_large_radius = 10.0;
        let large_sphere = cuboid.to_sphere(too_large_radius);

        assert!(large_sphere.coords.len() > cuboid.coords.len());
    }

    #[test]
    fn create_periodic_multiple_of_cuboid() {
        let cuboid = Cuboid {
            size: Coord::new(2.0, 2.0, 2.0),
            coords: vec![Coord::new(0.5, 1.0, 1.5)],
            .. Cuboid::default()
        };

        let cuboid_octupled = cuboid.pbc_multiply(2, 2, 2);
        assert_eq!(8 * cuboid.coords.len(), cuboid_octupled.coords.len());

        let expected_coords = vec![
            Coord::new(0.5, 1.0, 1.5), // base coordinate (1, 1, 1)
            Coord::new(0.5, 1.0, 3.5), // (1, 1, 2)
            Coord::new(0.5, 3.0, 1.5), // (1, 2, 1)
            Coord::new(0.5, 3.0, 3.5), // (1, 2, 2)
            Coord::new(2.5, 1.0, 1.5), // (2, 1, 1)
            Coord::new(2.5, 1.0, 3.5), // (2, 1, 2)
            Coord::new(2.5, 3.0, 1.5), // (2, 2, 1)
            Coord::new(2.5, 3.0, 3.5), // (2, 2, 2)
        ];

        for coord in expected_coords {
            assert!(cuboid_octupled.coords.contains(&coord));
        }
    }

    #[test]
    fn create_no_added_periodic_multiples_of_cuboid_just_clones() {
        let cuboid = Cuboid {
            size: Coord::new(2.0, 2.0, 2.0),
            coords: vec![Coord::new(0.5, 1.0, 1.5)],
            .. Cuboid::default()
        };

        let cloned_cuboid = cuboid.pbc_multiply(1, 1, 1);

        assert_eq!(cuboid.origin, cloned_cuboid.origin);
        assert_eq!(cuboid.size, cloned_cuboid.size);
        assert_eq!(cuboid.coords, cloned_cuboid.coords);
    }

    #[test]
    fn cuboid_contains_coordinates_in_absolute_space() {
        let cuboid = Cuboid {
            origin: Coord::new(1.0, 1.0, 1.0),
            size: Coord::new(1.0, 1.0, 1.0),
            .. Cuboid::default()
        };

        let err = 1e-9;

        // Inside
        assert!(cuboid.contains(Coord::new(1.0 + err, 1.0 + err, 1.0 + err)));
        assert!(cuboid.contains(Coord::new(2.0 - err, 2.0 - err, 2.0 - err)));

        // Outside
        assert!(!cuboid.contains(Coord::new(1.0 - err, 1.0 + err, 1.0 + err)));
        assert!(!cuboid.contains(Coord::new(1.0 + err, 1.0 - err, 1.0 + err)));
        assert!(!cuboid.contains(Coord::new(1.0 + err, 1.0 + err, 1.0 - err)));

        // Outside
        assert!(!cuboid.contains(Coord::new(2.0 + err, 2.0 - err, 2.0 - err)));
        assert!(!cuboid.contains(Coord::new(2.0 - err, 2.0 + err, 2.0 - err)));
        assert!(!cuboid.contains(Coord::new(2.0 - err, 2.0 - err, 2.0 + err)));
    }

    #[test]
    fn density_is_set_after_fill() {
        let num_atoms = 1000;
        let size = Coord::new(1.0, 2.0, 3.0);
        let cuboid = Cuboid {
            size,
            .. Cuboid::default()
        }.fill(FillType::NumCoords(num_atoms));

        assert_eq!(cuboid.coords.len(), num_atoms as usize);

        let volume = size.x * size.y * size.z;
        let expected_density = num_atoms as f64 / volume;
        let density = cuboid.density.unwrap();
        let ratio = density / expected_density;

        assert!(ratio >= 0.9 && ratio <= 1.1);
    }

    #[test]
    fn cuboid_volume_is_correct() {
        let cuboid = Cuboid {
            size: Coord::new(1.0, 3.0, 7.0),
            .. Cuboid::default()
        };

        assert_eq!(cuboid.volume(), 1.0 * 3.0 * 7.0);
    }

    #[test]
    fn cuboid_center_calculation_is_correct() {
        let size = Coord::new(1.0, 7.0, 13.0);

        let cuboid = Cuboid {
            size,
            .. Cuboid::default()
        };

        assert_eq!(cuboid.center(), Coord::new(size.x / 2.0, size.y / 2.0, size.z / 2.0));
    }
}
