use std::iter::Step;
use std::ops::Add;

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub enum Direction {
    Up = 8,
    Down = -8,
    Left = -1,
    Right = 1,
    UpLeft = 7,
    UpRight = 9,
    DownLeft = -9,
    DownRight = -7,

    UpUpLeft = 15,
    UpUpRight = 17,
    LeftLeftUp = 6,
    LeftLeftDown = -10,
    RightRightUp = 10,
    RightRightDown = -6,
    DownDownLeft = -17,
    DownDownRight = -15,
}

macro_rules! square_enum_impl {
    ($(#[$meta:meta])* $vis:vis enum $name:ident {
        $($(#[$vmeta:meta])* $vname:ident $(= $val:expr)?,)*
    }) => {
        $(#[$meta])*
        $vis enum $name {
            $($(#[$vmeta])* $vname $(= $val)?,)*
        }

        impl std::convert::From<usize> for $name {
            fn from(v: usize) -> Self {
                match v {
                    $(x if x == $name::$vname as usize => $name::$vname,)*
                    _ => panic!("invalid value {}", v),
                }
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $($name::$vname => write!(f, "{:?}", $name::$vname),)*
                }
            }
        }
    }
}

square_enum_impl! {
    #[derive(Copy, Clone, Eq, PartialEq, Debug, Ord, PartialOrd)]
    pub enum Square {
        A1 = 0,
        B1 = 1,
        C1 = 2,
        D1 = 3,
        E1 = 4,
        F1 = 5,
        G1 = 6,
        H1 = 7,
        A2 = 8,
        B2 = 9,
        C2 = 10,
        D2 = 11,
        E2 = 12,
        F2 = 13,
        G2 = 14,
        H2 = 15,
        A3 = 16,
        B3 = 17,
        C3 = 18,
        D3 = 19,
        E3 = 20,
        F3 = 21,
        G3 = 22,
        H3 = 23,
        A4 = 24,
        B4 = 25,
        C4 = 26,
        D4 = 27,
        E4 = 28,
        F4 = 29,
        G4 = 30,
        H4 = 31,
        A5 = 32,
        B5 = 33,
        C5 = 34,
        D5 = 35,
        E5 = 36,
        F5 = 37,
        G5 = 38,
        H5 = 39,
        A6 = 40,
        B6 = 41,
        C6 = 42,
        D6 = 43,
        E6 = 44,
        F6 = 45,
        G6 = 46,
        H6 = 47,
        A7 = 48,
        B7 = 49,
        C7 = 50,
        D7 = 51,
        E7 = 52,
        F7 = 53,
        G7 = 54,
        H7 = 55,
        A8 = 56,
        B8 = 57,
        C8 = 58,
        D8 = 59,
        E8 = 60,
        F8 = 61,
        G8 = 62,
        H8 = 63,
    }
}

impl Square {
    pub fn from_coordinates(row: u8, col: u8) -> Self {
        Square::from((((row - 1) * 8) + (col - 1)) as usize)
    }

    pub fn rank(&self) -> u8 {
        (*self as u8 / 8) + 1
    }

    pub fn file(&self) -> u8 {
        (*self as u8 % 8) + 1
    }
}

impl Add<Direction> for Square {
    type Output = Square;

    fn add(self, rhs: Direction) -> Self::Output {
        Square::from((self as i8 + rhs as i8) as usize)
    }
}

impl From<Square> for usize {
    fn from(s: Square) -> Self {
        s as usize
    }
}

impl Step for Square {
    fn steps_between(start: &Self, end: &Self) -> Option<usize> {
        Some(*end as usize - *start as usize)
    }

    fn forward_checked(start: Self, count: usize) -> Option<Self> {
        if start as usize + count > Square::H8.into() {
            return None;
        }
        Some(Square::from(start as usize + count))
    }

    fn backward_checked(start: Self, count: usize) -> Option<Self> {
        if start as usize - count < Square::A1.into() {
            return None;
        }

        Some(Square::from(start as usize - count))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_coordinates() {
        assert_eq!(Square::A1, Square::from_coordinates(1, 1));
        assert_eq!(Square::B1, Square::from_coordinates(1, 2));
        assert_eq!(Square::C1, Square::from_coordinates(1, 3));
        assert_eq!(Square::D1, Square::from_coordinates(1, 4));
        assert_eq!(Square::E1, Square::from_coordinates(1, 5));
        assert_eq!(Square::F1, Square::from_coordinates(1, 6));
        assert_eq!(Square::G1, Square::from_coordinates(1, 7));
        assert_eq!(Square::H1, Square::from_coordinates(1, 8));

        assert_eq!(Square::A4, Square::from_coordinates(4, 1));
        assert_eq!(Square::B4, Square::from_coordinates(4, 2));
        assert_eq!(Square::C4, Square::from_coordinates(4, 3));
        assert_eq!(Square::D4, Square::from_coordinates(4, 4));
        assert_eq!(Square::E4, Square::from_coordinates(4, 5));
        assert_eq!(Square::F4, Square::from_coordinates(4, 6));
        assert_eq!(Square::G4, Square::from_coordinates(4, 7));
        assert_eq!(Square::H4, Square::from_coordinates(4, 8));

        assert_eq!(Square::A8, Square::from_coordinates(8, 1));
        assert_eq!(Square::B8, Square::from_coordinates(8, 2));
        assert_eq!(Square::C8, Square::from_coordinates(8, 3));
        assert_eq!(Square::D8, Square::from_coordinates(8, 4));
        assert_eq!(Square::E8, Square::from_coordinates(8, 5));
        assert_eq!(Square::F8, Square::from_coordinates(8, 6));
        assert_eq!(Square::G8, Square::from_coordinates(8, 7));
        assert_eq!(Square::H8, Square::from_coordinates(8, 8));
    }

    #[test]
    fn test_row() {
        assert_eq!(1, Square::A1.rank());
        assert_eq!(1, Square::B1.rank());
        assert_eq!(1, Square::C1.rank());
        assert_eq!(1, Square::D1.rank());
        assert_eq!(1, Square::E1.rank());
        assert_eq!(1, Square::F1.rank());
        assert_eq!(1, Square::G1.rank());
        assert_eq!(1, Square::H1.rank());
    }

    #[test]
    fn test_col() {
        assert_eq!(1, Square::A1.file());
        assert_eq!(1, Square::A2.file());
        assert_eq!(1, Square::A3.file());
        assert_eq!(1, Square::A4.file());
        assert_eq!(1, Square::A5.file());
        assert_eq!(1, Square::A6.file());
        assert_eq!(1, Square::A7.file());
        assert_eq!(1, Square::A8.file());
    }
}
