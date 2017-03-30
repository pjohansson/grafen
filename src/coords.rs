#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Coord {
    pub x: f64,
    pub y: f64,
    pub z: f64
}

impl Coord {
    pub fn new(x: f64, y: f64, z: f64) -> Coord {
        Coord {x: x, y: y, z: z}
    }

    pub fn add(&self, other: Coord) -> Coord {
        Coord { x: self.x + other.x, y: self.y + other.y, z: self.z + other.z }
    }

    pub fn add_manual(&self, x: f64, y: f64, z: f64) -> Coord {
        Coord { x: self.x + x, y: self.y + y, z: self.z + z }
    }

    pub fn at_index(&self, nx: u64, ny: u64, nz: u64, spacing: &Coord) -> Coord {
        Coord {
            x: self.x + (nx as f64)*spacing.x,
            y: self.y + (ny as f64)*spacing.y,
            z: self.z + (nz as f64)*spacing.z
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coord_at_index_with_spacing_multipliers_are_correct() {
        let coord = Coord::new(0.0, 1.0, 2.0);
        let spacing = Coord::new(2.0, 3.0, 4.0);
        assert_eq!(Coord::new(2.0, 7.0, 14.0), coord.at_index(1, 2, 3, &spacing));
    }
}
