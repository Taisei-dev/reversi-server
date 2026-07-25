use crate::game::{Board, Color, Move};
use rand::Rng;

pub fn decide_random_move(board: &Board, color: Color) -> Move {
    let valid_moves = board.valid_moves(color);
    if valid_moves.is_empty() {
        Move::Pass
    } else {
        let mut rng = rand::thread_rng();
        let idx = rng.gen_range(0..valid_moves.len());
        let (x, y) = valid_moves[idx];
        Move::Coord {
            x: x as u8,
            y: y as u8,
        }
    }
}
