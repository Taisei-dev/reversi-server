use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Move {
    Coord { x: u8, y: u8 }, // x: 1..=8 (A..H), y: 1..=8 (1..8)
    Pass,
    GiveUp,
}

impl Move {
    pub fn from_str(s: &str) -> Option<Self> {
        let s = s.trim().to_uppercase();
        if s == "PASS" {
            return Some(Self::Pass);
        }
        if s == "GIVEUP" {
            return Some(Self::GiveUp);
        }
        if s.len() == 2 {
            let bytes = s.as_bytes();
            let col = bytes[0];
            let row = bytes[1];
            if (b'A'..=b'H').contains(&col) && (b'1'..=b'8').contains(&row) {
                let x = col - b'A' + 1;
                let y = row - b'1' + 1;
                return Some(Self::Coord { x, y });
            }
        }
        None
    }
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pass => write!(f, "PASS"),
            Self::GiveUp => write!(f, "GIVEUP"),
            Self::Coord { x, y } => {
                let col_char = (b'A' + x - 1) as char;
                let row_char = (b'1' + y - 1) as char;
                write!(f, "{col_char}{row_char}")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_move_parse_and_display() {
        let m1 = Move::from_str("C4").unwrap();
        assert_eq!(m1, Move::Coord { x: 3, y: 4 });
        assert_eq!(m1.to_string(), "C4");

        let m_pass = Move::from_str("PASS").unwrap();
        assert_eq!(m_pass, Move::Pass);
        assert_eq!(m_pass.to_string(), "PASS");
    }
}
