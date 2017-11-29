//! Implement elementary coordinate operations.

use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::ops::{Add, AddAssign, Sub, SubAssign, Neg, Mul, MulAssign};
use std::str::FromStr;

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
/// A three-dimensional carthesian coordinate.
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
    ///
    /// # Examples
    /// ```
    /// # use grafen::coord::Coord;
    /// let coord = Coord::new(0.0, 1.0, 2.0);
    /// assert_eq!(Coord { x: 0.0, y: 1.0, z: 2.0 }, coord);
    /// ```
    pub fn new(x: f64, y: f64, z: f64) -> Coord {
        Coord { x: x, y: y, z: z }
    }

    /// Unpack the coordinate into a tuple.
    ///
    /// # Examples
    /// ```
    /// # use grafen::coord::Coord;
    /// let (x, y, z) = Coord::new(0.0, 1.0, 2.0).to_tuple();
    /// assert_eq!((0.0, 1.0, 2.0), (x, y, z));
    /// ```
    pub fn to_tuple(&self) -> (f64, f64, f64) {
        (self.x, self.y, self.z)
    }

    /// Calculate the absolute distance between two coordinates.
    ///
    /// # Examples
    /// ```
    /// # use grafen::coord::Coord;
    /// let coord1 = Coord::new(0.0, 1.0, 4.0);
    /// let coord2 = Coord::new(4.0, 4.0, 4.0);
    /// assert!((coord1.distance(coord2) - 5.0).abs() < 1e-9);
    /// ```
    pub fn distance(self, other: Coord) -> f64 {
        let dx = self - other;

        (dx.x * dx.x + dx.y * dx.y + dx.z * dx.z).sqrt()
    }

    /// Calculate the cylindrical distance between two coordinates along an input `Direction`.
    /// Returns the 2-tuple (radius, height).
    ///
    /// # Examples
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

    /// Rotate the coordinate around an axis and return.
    ///
    /// The rotation is relative to (0, 0, 0).
    ///
    /// # Examples
    /// ```
    /// # use grafen::coord::{Coord, Direction};
    /// let coord = Coord::new(1.0, 0.0, 0.0);
    /// assert_eq!(coord.rotate(Direction::X), Coord::new(1.0, 0.0, 0.0));
    /// assert_eq!(coord.rotate(Direction::Y), Coord::new(0.0, 0.0, -1.0));
    /// assert_eq!(coord.rotate(Direction::Z), Coord::new(0.0, 1.0, 0.0));
    /// ```
    pub fn rotate(self, axis: Direction) -> Coord {
        match axis {
            Direction::X => Coord { x: self.x, y: -self.z, z: self.y },
            Direction::Y => Coord { x: self.z, y: self.y, z: -self.x },
            Direction::Z => Coord { x: -self.y, y: self.x, z: self.z },
        }
    }

    /// Return the coordinate with its position adjusted to lie within the input box.
    ///
    /// If an input box size side is 0.0 (or smaller) the coordinate is not changed
    /// since it doesn't make sense.
    ///
    /// # Examples
    /// ```
    /// # use grafen::coord::{Coord, Direction};
    /// let box_size = Coord::new(1.0, 1.0, 1.0);
    /// let coord = Coord::new(0.5, 2.5, -2.5);
    /// assert_eq!(Coord::new(0.5, 0.5, 0.5), coord.with_pbc(box_size));
    /// ```
    /// ```
    /// # use grafen::coord::{Coord, Direction};
    /// let box_size = Coord::new(1.0, -1.0, 1.0);
    /// let coord = Coord::new(0.5, 2.5, -2.5);
    /// assert_eq!(Coord::new(0.5, 2.5, 0.5), coord.with_pbc(box_size));
    /// ```
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

impl Periodic for Coord {
    /// Multiply a coordinate by integer amounts in all directions.
    ///
    /// # Examples
    /// ```
    /// # use grafen::coord::{Coord, Periodic};
    /// let coord = Coord::new(1.0, 2.0, 3.0);
    /// assert_eq!(Coord::new(1.0, 4.0, 9.0), coord.pbc_multiply(1, 2, 3));
    /// ```
    fn pbc_multiply(&self, nx: usize, ny: usize, nz: usize) -> Coord {
        Coord { x: self.x * nx as f64, y: self.y * ny as f64, z: self.z * nz as f64 }
    }
}

impl Default for Coord {
    fn default() -> Coord {
        Coord::ORIGO
    }
}

impl Display for Coord {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "({:.1}, {:.1}, {:.1})", self.x, self.y, self.z)
    }
}

impl FromStr for Coord {
    type Err = String;

    /// Parse an input string slice for a coordinate.
    ///
    /// # Errors
    /// Raises an error if three floating point values could not be parsed.
    ///
    /// # Examples
    /// ```
    /// # use grafen::coord::Coord;
    /// # use std::str::FromStr;
    /// assert_eq!(Ok(Coord::new(0.1, 1.0, -2.0)), Coord::from_str("0.1 1.0 -2.0"));
    /// assert_eq!(Ok(Coord::new(0.1, 1.0, -2.0)), Coord::from_str("0.1\t1.0\t-2.0\n"));
    /// ```
    /// ```
    /// # use grafen::coord::Coord;
    /// # use std::str::FromStr;
    /// assert!(Coord::from_str("0.1 1.1").is_err());
    /// assert!(Coord::from_str("a0.1 1.1 2.1").is_err());
    /// ```
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

impl Mul<f64> for Coord {
    type Output = Coord;

    fn mul(self, value: f64) -> Coord {
        Coord::new(self.x * value, self.y * value, self.z * value)
    }
}

impl MulAssign<f64> for Coord {
    fn mul_assign(&mut self, value: f64) {
        *self = Coord {
            x: self.x * value,
            y: self.y * value,
            z: self.z * value,
        }
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
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
/// Component direction axis. Eg. for `Cylinder`s this is the cylinder axis.
/// For a `Sheet` the normal.
pub enum Direction { X, Y, Z }

impl Display for Direction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            Direction::X => write!(f, "X"),
            Direction::Y => write!(f, "Y"),
            Direction::Z => write!(f, "Z"),
        }
    }
}

/// Rotate a set of coordinates around an axis.
pub fn rotate_coords(coords: &[Coord], axis: Direction) -> Vec<Coord> {
    coords.iter()
        .map(|&coord| coord.rotate(axis))
        .collect()
}

/// Rotate a set of coordinates from one alignment to another.
///
/// Note that this is meant mostly for planar objects, ie. sheets! The rotation along `Y`
/// is actually negative, to make a plane aligned along `Z` with its origin at origo
/// spread out in the positive z plane after rotation (ie. with its normal poining in
/// the direction of negative y). This is obviously pretty stupid design. I need to revisit
/// it sometime to make that a special case.
///
/// This code highlights how stupid the current rotation implementation is.
pub fn rotate_planar_coords_to_alignment(coords: &[Coord], from: Direction, to: Direction) -> Vec<Coord> {
    use self::Direction::*;

    match (from, to) {
        (X, Y) => {
            rotate_coords(&rotate_coords(&rotate_coords(coords, Z), Z), Z)
        },
        (X, Z) => {
            rotate_coords(coords, Y)
        },
        (Y, X) => {
            rotate_coords(coords, Z)
        },
        (Y, Z) => {
            rotate_coords(&rotate_coords(&rotate_coords(coords, X), X), X)
        },
        (Z, X) => {
            rotate_coords(&rotate_coords(&rotate_coords(coords, Y), Y), Y)
        },
        (Z, Y) => {
            rotate_coords(coords, X)
        },
        _ => coords.into(),
    }
}

/// Translate an object by a `Coord`.
pub trait Translate {
    fn translate(self, coord: Coord) -> Self;
    fn translate_in_place(&mut self, coord: Coord);
}

#[macro_export]
/// Macro to implement `Translate` for an object with an `origin` variable.
macro_rules! impl_translate {
    ( $($class:path),+ ) => {
        $(
            impl Translate for $class {
                /// Translate the object by an input `Coord`.
                fn translate(mut self, coord: Coord) -> Self {
                    self.origin += coord;
                    self
                }

                /// Translate the object by an input `Coord` in-place.
                fn translate_in_place(&mut self, coord: Coord) {
                    self.origin += coord;
                }
            }
        )*
    }
}

/// Trait denoting periodic boundary condition operations on objects.
pub trait Periodic {
    /// Extend an object by some integer amounts.
    fn pbc_multiply(&self, nx: usize, ny: usize, nz: usize) -> Self;
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
        assert_eq!(Coord::new(2.0, 4.0, 6.0), Coord::new(1.0, 2.0, 3.0) * 2.0);
    }

    #[test]
    fn coord_assign_operators() {
        let mut coord1 = Coord::ORIGO;
        let coord2 = Coord::new(1.0, 2.0, 3.0);

        coord1 += coord2;
        assert_eq!(coord2, coord1);

        coord1 -= coord2;
        assert_eq!(Coord::ORIGO, coord1);

        coord1 += coord2;
        coord1 *= 2.0;
        assert_eq!(coord2 + coord2, coord1);
    }

    #[test]
    fn coord_distance_along_plane() {
        let coord1 = Coord::new(0.0, 0.0, 0.0);
        let coord2 = Coord::new(1.0, 1.0, 5.0);

        assert_eq!((26.0f64.sqrt(), 1.0), coord1.distance_cylindrical(coord2, Direction::X));
        assert_eq!((26.0f64.sqrt(), 1.0), coord1.distance_cylindrical(coord2, Direction::Y));
        assert_eq!((2.0f64.sqrt(), 5.0), coord1.distance_cylindrical(coord2, Direction::Z));
    }

    #[test]
    fn periodic_multiple_of_coords() {
        let coord = Coord::new(1.0, 2.0, 3.0);
        assert_eq!(Coord::new(2.0, 6.0, 12.0), coord.pbc_multiply(2, 3, 4));
    }

    #[test]
    fn default_coordinate() {
        assert_eq!(Coord::ORIGO, Coord::default());
    }

    #[test]
    fn rotate_a_coordinate() {
        let coord = Coord::new(1.0, 2.0, 3.0);

        assert_eq!(Coord::new(1.0, -3.0, 2.0), coord.clone().rotate(Direction::X));
        assert_eq!(Coord::new(3.0, 2.0, -1.0), coord.clone().rotate(Direction::Y));
        assert_eq!(Coord::new(-2.0, 1.0, 3.0), coord.clone().rotate(Direction::Z));
    }

    #[test]
    fn impl_translate_object() {
        struct TranslateTest { origin: Coord }
        impl_translate![TranslateTest];

        let coord = Coord::new(1.0, 2.0, 3.0);
        let translated = TranslateTest {
            origin: Coord::ORIGO,
        }.translate(coord);

        assert_eq!(translated.origin, coord);
    }

    #[test]
    fn impl_translate_object_in_place() {
        struct TranslateTest { origin: Coord }
        impl_translate![TranslateTest];

        let mut object = TranslateTest {
            origin: Coord::ORIGO,
        };

        let coord = Coord::new(1.0, 2.0, 3.0);
        object.translate_in_place(coord);

        assert_eq!(object.origin, coord);
    }

    #[test]
    fn rotating_a_sheet_to_alignment_works() {
        use super::Direction::*;

        let sheet_z = vec![
            Coord::new(0.0, 0.0, 0.0),
            Coord::new(1.0, 0.0, 0.0),
            Coord::new(0.0, 1.0, 0.0),
            Coord::new(1.0, 1.0, 0.0),
        ];

        // Z to Y and back
        let sheet_y = rotate_planar_coords_to_alignment(&sheet_z, Z, Y);
        let expected = vec![
            Coord::new(0.0, 0.0, 0.0),
            Coord::new(1.0, 0.0, 0.0),
            Coord::new(0.0, 0.0, 1.0),
            Coord::new(1.0, 0.0, 1.0)
        ];

        assert_eq!(sheet_y, expected);
        assert_eq!(sheet_z, rotate_planar_coords_to_alignment(&sheet_y, Y, Z));

        // Z to X and back
        let sheet_x = rotate_planar_coords_to_alignment(&sheet_z, Z, X);
        let expected = vec![
            Coord::new(0.0, 0.0, 0.0),
            Coord::new(0.0, 0.0, 1.0),
            Coord::new(0.0, 1.0, 0.0),
            Coord::new(0.0, 1.0, 1.0)
        ];

        assert_eq!(sheet_x, expected);
        assert_eq!(sheet_z, rotate_planar_coords_to_alignment(&sheet_x, X, Z));

        // X to Y and back (create from scratch since rotations around
        // different axes do not commute)
        let sheet_x = vec![
            Coord::new(0.0, 0.0, 0.0),
            Coord::new(0.0, 1.0, 0.0),
            Coord::new(0.0, 0.0, 1.0),
            Coord::new(0.0, 1.0, 1.0)
        ];

        let sheet_y = rotate_planar_coords_to_alignment(&sheet_x, X, Y);
        let expected = vec![
            Coord::new(0.0, 0.0, 0.0),
            Coord::new(1.0, 0.0, 0.0),
            Coord::new(0.0, 0.0, 1.0),
            Coord::new(1.0, 0.0, 1.0)
        ];

        assert_eq!(sheet_y, expected);
        assert_eq!(sheet_x, rotate_planar_coords_to_alignment(&sheet_y, Y, X));

        // No rotation changes expected
        assert_eq!(sheet_x, rotate_planar_coords_to_alignment(&sheet_x, X, X));
        assert_eq!(sheet_y, rotate_planar_coords_to_alignment(&sheet_y, Y, Y));
        assert_eq!(sheet_z, rotate_planar_coords_to_alignment(&sheet_z, Z, Z));
    }
}
