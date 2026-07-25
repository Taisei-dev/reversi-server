use super::ffi::{
    egaroucid_create, egaroucid_destroy, egaroucid_global_init, egaroucid_search_array,
    EgaroucidSearchOptions, EgaroucidSearchResult,
};
use crate::game::{Board, Color, Move};
use std::ffi::CString;
use std::path::PathBuf;
use std::sync::Once;
use tracing::{info, warn};

static INIT: Once = Once::new();

pub struct EgaroucidAi {
    pub level: i32,
    pub use_book: bool,
    pub timeout_ms: i32,
}

impl EgaroucidAi {
    pub fn new(level: i32, use_book: bool, timeout_ms: i32) -> Self {
        INIT.call_once(|| unsafe {
            // Egaroucid の評価ファイル・定石ファイル (eval.egev2, book.egbk3) が含まれるパスを指定
            let mut resource_dir = PathBuf::from("Egaroucid/bin/resources");
            if !resource_dir.exists() {
                resource_dir = PathBuf::from("resources");
            }
            let abs_path = std::fs::canonicalize(&resource_dir)
                .unwrap_or(resource_dir);

            info!("Initializing Egaroucid engine with resource_dir: {:?}", abs_path);

            let c_path = CString::new(abs_path.to_string_lossy().as_bytes()).unwrap();
            let status = egaroucid_global_init(c_path.as_ptr());
            if status == 0 {
                info!("Egaroucid engine initialized successfully (eval.egev2 & book.egbk3 loaded)");
            } else {
                warn!("Egaroucid global_init failed with status: {status}");
            }
        });
        Self { level, use_book, timeout_ms }
    }

    pub fn decide_move(&self, board: &Board, color: Color) -> Move {
        let valid_moves = board.valid_moves(color);
        if valid_moves.is_empty() {
            return Move::Pass;
        }

        let arr = board.to_egaroucid_array();
        let player_int = match color {
            Color::Black => 0,
            Color::White => 1,
            _ => 0,
        };

        let mut options = EgaroucidSearchOptions::default();
        options.level = self.level.clamp(0, 20);
        options.time_limit_ms = self.timeout_ms;
        options.use_book = if self.use_book { 1 } else { 0 };
        options.show_log = 0;

        let mut result = EgaroucidSearchResult::default();

        let raw_move = unsafe {
            let engine = egaroucid_create();
            if engine.is_null() {
                -1
            } else {
                let status = egaroucid_search_array(
                    engine,
                    arr.as_ptr(),
                    player_int,
                    &options,
                    &mut result,
                );
                egaroucid_destroy(engine);
                if status == 0 {
                    result.r#move
                } else {
                    warn!("egaroucid_search_array failed with status: {status}");
                    -1
                }
            }
        };

        if raw_move >= 0 && raw_move < 64 {
            let x = (raw_move % 8 + 1) as usize;
            let y = (raw_move / 8 + 1) as usize;

            if valid_moves.contains(&(x, y)) {
                info!(
                    "Egaroucid (level {}) selected move ({x}, {y}) [eval: {}, depth: {}, nodes: {}]",
                    self.level, result.value, result.depth, result.nodes
                );
                return Move::Coord {
                    x: x as u8,
                    y: y as u8,
                };
            }
        }

        // Fallback: valid_moves の先頭の有効手を正しく使用
        info!(
            "Egaroucid returned raw_move {raw_move}, fallback to first valid move {:?}",
            valid_moves[0]
        );
        let (vx, vy) = valid_moves[0];
        Move::Coord {
            x: vx as u8,
            y: vy as u8,
        }
    }
}
