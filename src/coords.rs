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

    pub fn add(&self, other: &Coord) -> Coord {
        Coord { x: self.x + other.x, y: self.y + other.y, z: self.z + other.z }
    }

    pub fn add_manual(&self, x: f64, y: f64, z: f64) -> Coord {
        Coord { x: self.x + x, y: self.y + y, z: self.z + z }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coord_translations() {
        let coord = Coord::new(0.0, 1.0, 2.0);
        assert_eq!(Coord{ x: 1.0, y: 0.0, z: 2.5 }, coord.add(&Coord { x: 1.0, y: -1.0, z: 0.5 }));
        assert_eq!(Coord{ x: 1.0, y: 0.0, z: 2.5 }, coord.add_manual(1.0, -1.0, 0.5));
    }
}
