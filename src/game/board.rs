use super::color::Color;
use super::move_kind::Move;

pub type Pos = (usize, usize); // 1..=8, 1..=8

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Board {
    grid: [[Color; 10]; 10],
}

impl Board {
    pub fn new() -> Self {
        let mut grid = [[Color::Empty; 10]; 10];
        for i in 0..10 {
            grid[i][0] = Color::Sentinel;
            grid[i][9] = Color::Sentinel;
            grid[0][i] = Color::Sentinel;
            grid[9][i] = Color::Sentinel;
        }
        // 初期配置: D4=White, E5=White, D5=Black, E4=Black
        grid[4][4] = Color::White;
        grid[5][5] = Color::White;
        grid[4][5] = Color::Black;
        grid[5][4] = Color::Black;

        Self { grid }
    }

    const DIRS: [(isize, isize); 8] = [
        (-1, -1), (0, -1), (1, -1),
        (-1,  0),          (1,  0),
        (-1,  1), (0,  1), (1,  1),
    ];

    pub fn get(&self, (x, y): Pos) -> Color {
        self.grid[x][y]
    }

    pub fn set(&mut self, (x, y): Pos, color: Color) {
        self.grid[x][y] = color;
    }

    fn flippable_in_dir(&self, color: Color, (x, y): Pos, (dx, dy): (isize, isize)) -> Vec<Pos> {
        let opp = color.opposite();
        let mut res = Vec::new();
        let mut cur_x = x as isize + dx;
        let mut cur_y = y as isize + dy;

        while cur_x >= 1 && cur_x <= 8 && cur_y >= 1 && cur_y <= 8 {
            let p = (cur_x as usize, cur_y as usize);
            if self.get(p) == opp {
                res.push(p);
                cur_x += dx;
                cur_y += dy;
            } else if self.get(p) == color {
                return res;
            } else {
                break;
            }
        }
        Vec::new()
    }

    pub fn flippable_indices(&self, color: Color, p: Pos) -> Vec<Pos> {
        let mut flippables = Vec::new();
        for &dir in &Self::DIRS {
            let mut line = self.flippable_in_dir(color, p, dir);
            flippables.append(&mut line);
        }
        flippables
    }

    pub fn is_valid_move(&self, color: Color, p: Pos) -> bool {
        if self.get(p) != Color::Empty {
            return false;
        }
        !self.flippable_indices(color, p).is_empty()
    }

    pub fn valid_moves(&self, color: Color) -> Vec<Pos> {
        let mut moves = Vec::new();
        for x in 1..=8 {
            for y in 1..=8 {
                if self.is_valid_move(color, (x, y)) {
                    moves.push((x, y));
                }
            }
        }
        moves
    }

    pub fn apply_move(&mut self, m: Move, color: Color) -> bool {
        match m {
            Move::Pass | Move::GiveUp => true,
            Move::Coord { x, y } => {
                let p = (x as usize, y as usize);
                let flippables = self.flippable_indices(color, p);
                if flippables.is_empty() || self.get(p) != Color::Empty {
                    return false;
                }
                self.set(p, color);
                for flip_p in flippables {
                    self.set(flip_p, color);
                }
                true
            }
        }
    }

    pub fn count_stones(&self) -> (u32, u32) {
        let mut black = 0;
        let mut white = 0;
        for x in 1..=8 {
            for y in 1..=8 {
                match self.get((x, y)) {
                    Color::Black => black += 1,
                    Color::White => white += 1,
                    _ => {}
                }
            }
        }
        (black, white)
    }

    pub fn is_game_over(&self) -> bool {
        let black_moves = self.valid_moves(Color::Black);
        let white_moves = self.valid_moves(Color::White);
        black_moves.is_empty() && white_moves.is_empty()
    }

    /// Egaroucid 用 64要素 int 配列変換
    /// Egaroucid: 0=Black, 1=White, -1=Empty
    pub fn to_egaroucid_array(&self) -> [i32; 64] {
        let mut arr = [-1i32; 64];
        for y in 1..=8 {
            for x in 1..=8 {
                let idx = (y - 1) * 8 + (x - 1);
                arr[idx] = match self.get((x, y)) {
                    Color::Black => 0,
                    Color::White => 1,
                    _ => -1,
                };
            }
        }
        arr
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}
