use coord::{Coord, Direction, Periodic, Translate};

#[derive(Clone, Debug)]
struct Cuboid {
    origin: Coord,
    size: Coord,
    coords: Vec<Coord>,
}

impl Cuboid {
    /// Calculate the center position of the cuboid, relative to the origin.
    fn center(&self) -> Coord {
        Coord { x: self.size.x / 2.0, y: self.size.y / 2.0, z: self.size.y / 2.0 }
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

impl Periodic for Cuboid {
    /// Clone cuboid coordinates into PBC multiples.
    fn pbc_multiply(&self, nx: usize, ny: usize, nz: usize) -> Cuboid {
        let coords = pbc_multiply_volume(&self.coords, self.size, nx, ny, nz);

        Cuboid {
            origin: self.origin,
            size: self.size.pbc_multiply(nx, ny, nz),
            coords,
        }
    }
}

impl Translate for Cuboid {
    fn translate(mut self, trans: Coord) -> Self {
        self.origin += trans;
        self
    }
}

struct Cylinder {
    origin: Coord,
    radius: f64,
    height: f64,
    alignment: Direction,
    coords: Vec<Coord>,
}

impl Cylinder {
    const DEFAULT_ALIGNMENT: Direction = Direction::Z;
}

impl Translate for Cylinder {
    fn translate(mut self, trans: Coord) -> Self {
        self.origin += trans;
        self
    }
}

struct Sphere {
    origin: Coord,
    radius: f64,
    coords: Vec<Coord>,
}

impl Translate for Sphere {
    fn translate(mut self, trans: Coord) -> Self {
        self.origin += trans;
        self
    }
}

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

/// Helper function to cut a set of coordinates into a sphere around a center point.
fn cut_to_sphere(coords: &[Coord], center: Coord, radius: f64) -> Vec<Coord> {
    coords.iter()
        .filter(|c| c.distance(center) <= radius)
        .map(|&c| c - center)
        .collect()
}

/// Helper function to periodically replicate a set of coordinates for a volume object.
fn pbc_multiply_volume(coords: &[Coord], size: Coord, nx: usize, ny: usize, nz: usize)
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
            origin: Coord::ORIGO,
            size: Coord::new(dx, dy, dz),
            coords,
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
            origin: Coord::ORIGO,
            size: Coord::new(2.0, 2.0, 2.0),
            coords: vec![Coord::new(0.5, 1.0, 1.5)],
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
            origin: Coord::ORIGO,
            size: Coord::new(2.0, 2.0, 2.0),
            coords: vec![Coord::new(0.5, 1.0, 1.5)],
        };

        let cloned_cuboid = cuboid.pbc_multiply(1, 1, 1);

        assert_eq!(cuboid.origin, cloned_cuboid.origin);
        assert_eq!(cuboid.size, cloned_cuboid.size);
        assert_eq!(cuboid.coords, cloned_cuboid.coords);
    }
}
