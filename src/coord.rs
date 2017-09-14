//! Implement elementary coordinate operations.

use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::ops::{Add, AddAssign, Sub, SubAssign, Neg};
use std::str::FromStr;

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
/// A three-dimensional coordinate.
///
/// # Examples
/// ```
/// # use grafen::coord::Coord;
/// let coord1 = Coord::new(1.0, 0.0, 1.0);
/// let coord2 = Coord::new(0.5, 0.5, 0.5);
///
/// assert_eq!(Coord::new(1.5, 0.5, 1.5), coord1 + coord2);
/// assert_eq!(Coord::new(0.5, -0.5, 0.5), coord1 - coord2);
/// ```
pub struct Coord {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Coord {
    /// A coordinate at origo.
    pub const ORIGO: Self = Coord { x: 0.0, y: 0.0, z: 0.0 };

    /// Construct a new coordinate.
    pub fn new(x: f64, y: f64, z: f64) -> Coord {
        Coord { x: x, y: y, z: z }
    }

    /// Unpack the coordinate into a tuple.
    pub fn to_tuple(&self) -> (f64, f64, f64) {
        (self.x, self.y, self.z)
    }

    /// Calculate the absolute distance between two coordinates.
    pub fn distance(self, other: Coord) -> f64 {
        let dx = self - other;

        (dx.x * dx.x + dx.y * dx.y + dx.z * dx.z).sqrt()
    }

    /// Calculate the cylindrical distance between two coordinates along an input `Direction`.
    /// Returns the 2-tuple (radius, height).
    ///
    /// # Examples:
    /// ```
    /// # use grafen::coord::{Coord, Direction};
    /// let coord1 = Coord::new(0.0, 0.0, 0.0);
    /// let coord2 = Coord::new(3.0, 4.0, 1.0);
    /// let (dr, dh) = coord1.distance_cylindrical(coord2, Direction::Z);
    ///
    /// assert_eq!((5.0, 1.0), (dr, dh));
    pub fn distance_cylindrical(self, other: Coord, dir: Direction) -> (f64, f64) {
        use self::Direction::*;

        // Get the in-plane (radius) generalized coordinate differences a, b
        // and generalized height (directed) difference h
        let (a, b, dh) = match dir {
            X => (self.y - other.y, self.z - other.z, other.x - self.x),
            Y => (self.x - other.x, self.z - other.z, other.y - self.y),
            Z => (self.x - other.x, self.y - other.y, other.z - self.z),
        };

        let dr = (a * a + b * b).sqrt();

        (dr, dh)
    }

    /// Return the coordinate with its position adjusted to lie within the input box.
    ///
    /// If an input box size side is 0.0 (or smaller) the coordinate is not changed.
    pub fn with_pbc(self, box_size: Coord) -> Coord {
        let do_pbc = |mut c: f64, size: f64| {
            if size <= 0.0 {
                c
            } else {
                while c < 0.0 {
                    c += size;
                }

                c % size
            }
        };

        let (x, y, z) = self.to_tuple();
        Coord::new(do_pbc(x, box_size.x), do_pbc(y, box_size.y), do_pbc(z, box_size.z))
    }
}

impl Display for Coord {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "({:.1}, {:.1}, {:.1})", self.x, self.y, self.z)
    }
}

impl Add for Coord {
    type Output = Coord;

    fn add(self, other: Coord) -> Self::Output {
        Coord::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }

}

impl AddAssign for Coord {
    fn add_assign(&mut self, other: Coord) {
        *self = Coord {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        };
    }
}

impl Sub for Coord {
    type Output = Coord;

    fn sub(self, other: Coord) -> Self::Output {
        Coord::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }

}

impl SubAssign for Coord {
    fn sub_assign(&mut self, other: Coord) {
        *self = Coord {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        };
    }
}

impl Neg for Coord {
    type Output = Coord;

    fn neg(self) -> Self::Output {
        Coord { x: -self.x, y: -self.y, z: -self.z }
    }
}

impl PartialEq for Coord {
    fn eq(&self, other: &Coord) -> bool {
        let atol = 1e-9;
        (self.x - other.x).abs() < atol
            && (self.y - other.y).abs() < atol
            && (self.z - other.z).abs() < atol
    }
}

impl FromStr for Coord {
    type Err = String;

    fn from_str(input: &str) -> Result<Coord, Self::Err> {
        let parse_opt_value = |value: Option<&str>| {
            value.ok_or("Not enough values to parse".to_string())
                 .and_then(|v| v.parse::<f64>().map_err(|err| err.description().to_string()))
        };

        let mut split = input.split_whitespace();
        let x = parse_opt_value(split.next())?;
        let y = parse_opt_value(split.next())?;
        let z = parse_opt_value(split.next())?;

        return Ok(Coord { x, y, z})
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
/// Component direction axis. Eg. for `Cylinder`s this is the cylinder axis.
/// For a `Sheet` the normal.
pub enum Direction { X, Y, Z }

impl Direction {
    /// Default alignment of a cylinder.
    pub fn default_cylinder() -> Direction {
        Direction::Z
    }
}

impl Display for Direction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            Direction::X => write!(f, "X"),
            Direction::Y => write!(f, "Y"),
            Direction::Z => write!(f, "Z"),
        }
    }
}

/// Trait denoting the ability to `Translate` an object with a `Coord`.
pub trait Translate {
    fn translate(self, Coord) -> Self;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn coord_origo_is_correct() {
        assert_eq!(Coord::new(0.0, 0.0, 0.0), Coord::ORIGO);
    }

    #[test]
    fn coord_eq_tolerance_small_deviation_passes() {
        // Allow for some deviation when testing for equality, since floating point
        // numbers are stupid.
        let coord = Coord::new(0.0, 0.0, 0.0);
        assert_eq!(coord, Coord::new(1e-10, 2e-10, 3e-10));
    }

    #[test]
    #[should_panic]
    fn coord_eq_tolerance_larger_deviation_does_not() {
        let coord = Coord::new(0.0, 0.0, 0.0);
        assert_eq!(coord, Coord::new(1e-9, 2e-9, 3e-9));
    }

    #[test]
    fn coord_distance_calc() {
        let coord1 = Coord::new(1.0, 1.0, 1.0);
        let coord2 = Coord::new(3.0, 3.0, 2.0);

        assert_eq!(3.0, Coord::distance(coord1, coord2));
        assert_eq!(3.0, coord1.distance(coord2));
    }

    #[test]
    fn coord_to_tuple() {
        let coord = Coord::new(1.0, 2.0, 3.0);
        assert_eq!((1.0, 2.0, 3.0), coord.to_tuple());
    }

    #[test]
    fn coord_with_set_pbc() {
        let box_size = Coord::new(2.0, 4.0, 6.0);
        assert_eq!(Coord::new(1.0, 1.0, 1.0), Coord::new(1.0, 1.0, 1.0).with_pbc(box_size));
        assert_eq!(Coord::new(1.0, 1.0, 1.0), Coord::new(3.0, 1.0, 1.0).with_pbc(box_size));
        assert_eq!(Coord::new(1.0, 0.5, 1.0), Coord::new(1.0, 4.5, 1.0).with_pbc(box_size));
        assert_eq!(Coord::new(1.0, 1.0, 1.0), Coord::new(1.0, 1.0, 13.0).with_pbc(box_size));
        assert_eq!(Coord::new(1.0, 1.0, 1.0), Coord::new(-1.0, 1.0, 1.0).with_pbc(box_size));
        assert_eq!(Coord::new(1.0, 3.0, 1.0), Coord::new(1.0, -1.0, 1.0).with_pbc(box_size));
    }

    #[test]
    fn coords_adjusted_by_pbc_with_size_0_does_not_change() {
        let box_size = Coord::new(0.0, 0.0, 0.0);
        let coord = Coord::new(1.0, 2.0, 3.0);
        assert_eq!(coord, coord.with_pbc(box_size));
    }

    #[test]
    fn coord_display_format() {
        let coord = Coord::ORIGO;
        assert_eq!("(0.0, 0.0, 0.0)", &format!("{}", coord));
    }

    #[test]
    fn coord_parsed_from_string() {
        assert_eq!(Ok(Coord::new(1.0, -1.0, 2.0)), Coord::from_str("1.0 -1.0 2.0"));
        assert_eq!(Ok(Coord::new(1.0, -1.0, 2.0)), Coord::from_str("1 -1.0 2"));
        assert_eq!(Ok(Coord::new(1.0, -1.0, 2.0)), Coord::from_str("\t1.0 -1.0 2.0"));
        assert!(Coord::from_str("").is_err());
        assert!(Coord::from_str("2.0 1.0").is_err());
    }

    #[test]
    fn coord_operators() {
        let coord1 = Coord::new(0.0, 1.0, 2.0);
        let coord2 = Coord::new(3.0, 4.0, 5.0);

        assert_eq!(Coord::new(3.0, 5.0, 7.0), coord1 + coord2);
        assert_eq!(Coord::new(3.0, 3.0, 3.0), coord2 - coord1);
        assert_eq!(Coord::new(0.0, -1.0, -2.0), -coord1);
    }

    #[test]
    fn coord_assign_operators() {
        let mut coord1 = Coord::ORIGO;
        let coord2 = Coord::new(1.0, 2.0, 3.0);

        coord1 += coord2;
        assert_eq!(coord2, coord1);

        coord1 -= coord2;
        assert_eq!(Coord::ORIGO, coord1);
    }

    #[test]
    fn coord_distance_along_plane() {
        let mut coord1 = Coord::new(0.0, 0.0, 0.0);
        let mut coord2 = Coord::new(1.0, 1.0, 5.0);

        assert_eq!((26.0f64.sqrt(), 1.0), coord1.distance_cylindrical(coord2, Direction::X));
        assert_eq!((26.0f64.sqrt(), 1.0), coord1.distance_cylindrical(coord2, Direction::Y));
        assert_eq!((2.0f64.sqrt(), 5.0), coord1.distance_cylindrical(coord2, Direction::Z));
    }
}
