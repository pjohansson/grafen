//!  Define and construct 3D volume objects.

use coord::{Coord, Direction, Periodic, Translate};
use describe::{unwrap_name, Describe};
use iterator::{AtomIterator, AtomIterItem};
use system::{Component, Residue};

use rand;
use rand::distributions::IndependentSample;
use std::f64::consts::PI;

impl_component![Cuboid, Cylinder];
impl_translate![Cuboid, Cylinder, Sphere];

/// Volumes can contain coordinates.
pub trait Contains: Describe {
    /// Whether a coordinate is contained within the volume's space.
    fn contains(&self, coord: Coord) -> bool;
}

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

#[allow(dead_code)]
impl Cuboid {
    /// Calculate the center position of the cuboid, relative to the origin.
    fn center(&self) -> Coord {
        Coord { x: self.size.x / 2.0, y: self.size.y / 2.0, z: self.size.y / 2.0 }
    }

    /// Calculate the box size.
    fn calc_box_size(&self) -> Coord {
        self.size
    }

    /// Fill the cuboid uniformally with coordinates.
    fn fill(self, num_atoms: u64) -> Cuboid {
        unimplemented!();
    }

    /// Construct a `Cylinder` from the cuboid by cutting its coordinates.
    /// It will be directed along the default cylinder alignment.
    fn to_cylinder(&self, radius: f64, height: f64) -> Cylinder {
        let alignment = Cylinder::DEFAULT_ALIGNMENT;

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
    #[serde(skip)]
    pub coords: Vec<Coord>,
}

impl Cylinder {
    #[allow(dead_code)]
    /// Default alignment for a cylinder is along the z axis.
    const DEFAULT_ALIGNMENT: Direction = Direction::Z;

    /// Calculate the box size.
    fn calc_box_size(&self) -> Coord {
        let diameter = 2.0 * self.radius;

        match self.alignment {
            Direction::X => Coord::new(self.height, diameter, diameter),
            Direction::Y => Coord::new(diameter, self.height, diameter),
            Direction::Z => Coord::new(diameter, diameter, self.height),
        }
    }

    /// Fill the cylinder with (roughly) uniformly distributed coordinates
    /// and return the object.
    pub fn fill(self, num_coords: u64) -> Cylinder {
        let mut rng = rand::thread_rng();

        let range_radius = rand::distributions::Range::new(0.0, self.radius);
        let range_height = rand::distributions::Range::new(0.0, self.height);
        let range_angle = rand::distributions::Range::new(0.0, 2.0 * PI);

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

        let coords: Vec<_> = (0..num_coords).map(|_| gen_coord()).collect();

        Cylinder {
            coords,
            .. self.clone()
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

#[allow(dead_code)]
/// A spherical volume.
pub struct Sphere {
    origin: Coord,
    radius: f64,
    coords: Vec<Coord>,
}

#[allow(dead_code)]
/// Helper function to cut a set of coordinates into a cylinder around a center point.
fn cut_to_cylinder(coords: &[Coord], bottom_center: Coord, alignment: Direction,
        radius: f64, height: f64) -> Vec<Coord> {
    coords.iter()
        .filter(|&c| {
            let (dr, dh) = bottom_center.distance_cylindrical(*c, alignment);
            dr <= radius && dh >= 0.0 && dh <= height
        })
        .map(|&c| c - bottom_center)
        .collect()
}

#[allow(dead_code)]
/// Helper function to cut a set of coordinates into a sphere around a center point.
fn cut_to_sphere(coords: &[Coord], center: Coord, radius: f64) -> Vec<Coord> {
    coords.iter()
        .filter(|c| c.distance(center) <= radius)
        .map(|&c| c - center)
        .collect()
}

/// Helper function to periodically replicate a set of coordinates for a volume object.
pub fn pbc_multiply_volume(coords: &[Coord], size: Coord, nx: usize, ny: usize, nz: usize)
        -> Vec<Coord> {
    let capacity = (nx * ny * nz) as usize * coords.len();

    // Only go through the loop if we have periodic multiples, otherwise just clone it.
    // Even if the compiler is good at rolling out the loops there are extra operations.
    match capacity {
        1 => {
            let mut new_coords = Vec::new();
            new_coords.extend_from_slice(coords);
            new_coords
        },
        _ => {
            let mut new_coords = Vec::with_capacity(capacity);

            for i in 0..nx {
                for j in 0..ny {
                    for k in 0..nz {
                        let pbc_add = size.pbc_multiply(i, j, k);

                        for &coord in coords {
                            new_coords.push(coord + pbc_add);
                        }
                    }
                }
            }

            new_coords
        },
    }
}

/// Return residue coordinates which are not contained by a volume.
///
/// Checks all atoms within the input residue to see if any are contained by the volume.
/// If any are, the residue coordinate is kept in the returned list.
pub fn prune_residues_from_volume<T: ?Sized>(coords: &[Coord], residue: &Residue, volume: &T)
        -> Vec<Coord> where T: Contains {
    coords.iter()
          .filter(|&c0| {
              residue.atoms
                .iter()
                .map(|ref atom| atom.position + *c0)
                .all(|c1| !volume.contains(c1))
          })
          .cloned()
          .collect::<Vec<_>>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use system::Atom;

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
        let cylinder = cuboid.to_cylinder(radius, height);

        assert!(cylinder.coords.len() > 0);
        assert_eq!(cuboid.origin, cylinder.origin);

        for coord in cylinder.coords {
            let (dr, dh) = Coord::ORIGO.distance_cylindrical(coord, Cylinder::DEFAULT_ALIGNMENT);
            assert!(dr <= radius);
            assert!(dh >= 0.0 && dh <= height);
        }
    }

    #[test]
    fn cuboid_expands_to_create_full_cylinder_if_too_small() {
        let cuboid = setup_cuboid(10.0, 10.0, 10.0, 1.0);

        let too_large_radius = 10.0;
        let too_large_height = 15.0;
        let large_cylinder = cuboid.to_cylinder(too_large_radius, too_large_height);

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
            alignment: Direction::Z,
            coords: vec![],
        };

        // Default alignment: Z
        let cylinder = conf.clone().fill(num_coords);
        assert_eq!(num_coords as usize, cylinder.coords.len());

        for coord in cylinder.coords {
            let (r, h) = Coord::ORIGO.distance_cylindrical(coord, Direction::Z);
            assert!(r <= cylinder.radius);
            assert!(h >= 0.0 && h <= cylinder.height);
        }

        // Along the other axes
        conf.alignment = Direction::X;
        for coord in conf.clone().fill(num_coords).coords {
            let (r, h) = Coord::ORIGO.distance_cylindrical(coord, Direction::X);
            assert!(r <= cylinder.radius);
            assert!(h >= 0.0 && h <= cylinder.height);
        }

        // Along the other axes
        conf.alignment = Direction::Y;
        for coord in conf.clone().fill(num_coords).coords {
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
    fn cylinder_contains_coordinates_in_absolute_space_depending_on_direction() {
        let mut cylinder = Cylinder {
            name: None,
            residue: None,
            origin: Coord::new(1.0, 1.0, 1.0),
            radius: 1.0,
            height: 2.0,
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
    fn coordinates_within_cuboid_are_pruned() {
        let residue = resbase!["RES", ("A", 0.0, 0.0, 0.0)];
        let cuboid = Cuboid {
            size: Coord::new(1.0, 1.0, 1.0),
            .. Cuboid::default()
        };

        let coords_within = vec![
            Coord::new(0.1, 0.1, 0.1),
            Coord::new(0.5, 0.5, 0.5),
            Coord::new(0.9, 0.9, 0.9)
        ];

        let coords_without = vec![
            Coord::new(-0.1, 0.1, 0.1),
            Coord::new(1.1, 0.9, 0.9)
        ];

        let coords = coords_within
            .iter()
            .chain(coords_without.iter())
            .cloned()
            .collect::<Vec<Coord>>();

        let pruned = prune_residues_from_volume(&coords, &residue, &cuboid);

        assert_eq!(coords_without, pruned);
    }

    #[test]
    fn coordinates_within_cuboid_prune_with_respect_to_residue_atoms() {
        let residue = resbase![
            "RES",
            ("A", 0.0, 0.0, 0.0),
            ("B", 1.0, 0.0, 0.0) // Shifted by 1
        ];
        let cuboid = Cuboid {
            size: Coord::new(1.0, 1.0, 1.0),
            .. Cuboid::default()
        };

        let coords_within = vec![
            Coord::new(-0.9, 0.1, 0.1), // Atom B within
            Coord::new(-0.5, 0.5, 0.5),
            Coord::new(-0.1, 0.9, 0.9),
            Coord::new(0.1, 0.9, 0.9), // Atom A within
            Coord::new(0.5, 0.9, 0.9),
            Coord::new(0.9, 0.9, 0.9)
        ];

        let coords_without = vec![
            Coord::new(-1.1, 0.1, 0.1),
            Coord::new(1.1, 0.9, 0.9)
        ];

        let coords = coords_within
            .iter()
            .chain(coords_without.iter())
            .cloned()
            .collect::<Vec<Coord>>();

        let pruned = prune_residues_from_volume(&coords, &residue, &cuboid);

        assert_eq!(coords_without, pruned);
    }

    #[test]
    fn density_is_set_after_fill() {
        let num_atoms = 1000;
        let size = Coord::new(1.0, 2.0, 3.0);
        let cuboid = Cuboid {
            size,
            .. Cuboid::default()
        }.fill(num_atoms);

        assert_eq!(cuboid.coords.len(), num_atoms as usize);

        let volume = size.x * size.y * size.z;
        let expected_density = num_atoms as f64 / volume;
        let density = cuboid.density.unwrap();
        let ratio = density / expected_density;

        assert!(ratio >= 0.9 && ratio <= 1.1);
    }
}
