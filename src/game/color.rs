use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Color {
    Empty,
    Black,
    White,
    Sentinel,
}

impl Color {
    #[inline]
    pub fn opposite(self) -> Self {
        match self {
            Self::Black => Self::White,
            Self::White => Self::Black,
            other => other,
        }
    }

    pub fn to_char(self) -> char {
        match self {
            Self::Black => 'X',
            Self::White => 'O',
            Self::Empty => '.',
            Self::Sentinel => '#',
        }
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "EMPTY"),
            Self::Black => write!(f, "BLACK"),
            Self::White => write!(f, "WHITE"),
            Self::Sentinel => write!(f, "SENTINEL"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreferredColor {
    Black,
    White,
    Random,
}

impl PreferredColor {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "black" | "b" => Self::Black,
            "white" | "w" => Self::White,
            _ => Self::Random,
        }
    }
}
