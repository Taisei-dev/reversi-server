use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinishedMatchInfo {
    pub match_id: String,
    pub mode: String,
    pub black_name: String,
    pub white_name: String,
    pub winner_color: Option<String>,
    pub black_stones: u32,
    pub white_stones: u32,
    pub reason: String,
    pub moves: Vec<String>,
    pub finished_at_sec: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TournamentMatchDetail {
    pub p1_name: String,
    pub p2_name: String,
    pub p1_score: u32,
    pub p2_score: u32,
    pub result_text: String, // "win", "lose", "draw"
    pub moves: Vec<String>,
    pub round_index: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomTournamentResult {
    pub room_id: String,
    pub finished_at_sec: u64,
    pub stats: Vec<crate::protocol::PlayerStat>,
    pub match_matrix: Vec<TournamentMatchDetail>,
}

pub struct MatchHistoryRegistry {
    recent_matches: Vec<FinishedMatchInfo>,
    tournament_results: Vec<RoomTournamentResult>,
    db_path: Option<PathBuf>,
}

impl MatchHistoryRegistry {
    pub fn new() -> Self {
        Self::open_db("data/history.db")
    }

    pub fn open_db<P: AsRef<Path>>(path: P) -> Self {
        let db_path = path.as_ref();
        if let Some(parent) = db_path.parent() {
            if !parent.exists() {
                if let Err(e) = std::fs::create_dir_all(parent) {
                    warn!("Failed to create directory {}: {e}", parent.display());
                }
            }
        }

        let conn = match Connection::open(db_path) {
            Ok(c) => c,
            Err(e) => {
                warn!("Failed to open SQLite database at {}: {e}", db_path.display());
                return Self {
                    recent_matches: Vec::new(),
                    tournament_results: Vec::new(),
                    db_path: None,
                };
            }
        };

        let create_table_matches = "
            CREATE TABLE IF NOT EXISTS finished_matches (
                match_id TEXT PRIMARY KEY,
                mode TEXT NOT NULL,
                black_name TEXT NOT NULL,
                white_name TEXT NOT NULL,
                winner_color TEXT,
                black_stones INTEGER NOT NULL,
                white_stones INTEGER NOT NULL,
                reason TEXT NOT NULL,
                moves_json TEXT NOT NULL,
                finished_at_sec INTEGER NOT NULL
            );
        ";

        let create_table_tournaments = "
            CREATE TABLE IF NOT EXISTS room_tournaments (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                room_id TEXT NOT NULL,
                finished_at_sec INTEGER NOT NULL,
                stats_json TEXT NOT NULL,
                match_matrix_json TEXT NOT NULL
            );
        ";

        if let Err(e) = conn.execute(create_table_matches, []) {
            warn!("Failed to create finished_matches table: {e}");
        }
        if let Err(e) = conn.execute(create_table_tournaments, []) {
            warn!("Failed to create room_tournaments table: {e}");
        }

        // 過去の対局履歴を読み込む（最新順50件）
        let mut recent_matches = Vec::new();
        if let Ok(mut stmt) = conn.prepare(
            "SELECT match_id, mode, black_name, white_name, winner_color, black_stones, white_stones, reason, moves_json, finished_at_sec
             FROM finished_matches ORDER BY finished_at_sec DESC LIMIT 50"
        ) {
            let match_iter = stmt.query_map([], |row| {
                let moves_json: String = row.get(8)?;
                let moves: Vec<String> = serde_json::from_str(&moves_json).unwrap_or_default();
                Ok(FinishedMatchInfo {
                    match_id: row.get(0)?,
                    mode: row.get(1)?,
                    black_name: row.get(2)?,
                    white_name: row.get(3)?,
                    winner_color: row.get(4)?,
                    black_stones: row.get(5)?,
                    white_stones: row.get(6)?,
                    reason: row.get(7)?,
                    moves,
                    finished_at_sec: row.get(9)?,
                })
            });

            if let Ok(matches) = match_iter {
                for m in matches.flatten() {
                    recent_matches.push(m);
                }
            }
        }

        // 過去のトーナメント結果を読み込む（最新順50件）
        let mut tournament_results = Vec::new();
        if let Ok(mut stmt) = conn.prepare(
            "SELECT room_id, finished_at_sec, stats_json, match_matrix_json
             FROM room_tournaments ORDER BY finished_at_sec DESC LIMIT 50"
        ) {
            let tour_iter = stmt.query_map([], |row| {
                let stats_json: String = row.get(2)?;
                let match_matrix_json: String = row.get(3)?;
                let stats = serde_json::from_str(&stats_json).unwrap_or_default();
                let match_matrix = serde_json::from_str(&match_matrix_json).unwrap_or_default();
                Ok(RoomTournamentResult {
                    room_id: row.get(0)?,
                    finished_at_sec: row.get(1)?,
                    stats,
                    match_matrix,
                })
            });

            if let Ok(tours) = tour_iter {
                for t in tours.flatten() {
                    tournament_results.push(t);
                }
            }
        }

        info!(
            "SQLite DB opened at {}: loaded {} matches and {} tournament results",
            db_path.display(),
            recent_matches.len(),
            tournament_results.len()
        );

        Self {
            recent_matches,
            tournament_results,
            db_path: Some(db_path.to_path_buf()),
        }
    }

    pub fn record_match(&mut self, match_info: FinishedMatchInfo) {
        if !match_info.mode.starts_with("Room_") {
            if let Some(db_path) = &self.db_path {
                if let Ok(conn) = Connection::open(db_path) {
                    let moves_json = serde_json::to_string(&match_info.moves).unwrap_or_default();
                    let sql = "INSERT OR REPLACE INTO finished_matches
                        (match_id, mode, black_name, white_name, winner_color, black_stones, white_stones, reason, moves_json, finished_at_sec)
                        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)";
                    if let Err(e) = conn.execute(
                        sql,
                        params![
                            match_info.match_id,
                            match_info.mode,
                            match_info.black_name,
                            match_info.white_name,
                            match_info.winner_color,
                            match_info.black_stones,
                            match_info.white_stones,
                            match_info.reason,
                            moves_json,
                            match_info.finished_at_sec,
                        ],
                    ) {
                        warn!("Failed to insert match into SQLite DB: {e}");
                    }
                }
            }

            self.recent_matches.insert(0, match_info);
            if self.recent_matches.len() > 50 {
                self.recent_matches.pop();
            }
        }
    }

    pub fn record_tournament(&mut self, tournament: RoomTournamentResult) {
        if let Some(db_path) = &self.db_path {
            if let Ok(conn) = Connection::open(db_path) {
                let stats_json = serde_json::to_string(&tournament.stats).unwrap_or_default();
                let match_matrix_json = serde_json::to_string(&tournament.match_matrix).unwrap_or_default();
                let sql = "INSERT INTO room_tournaments
                    (room_id, finished_at_sec, stats_json, match_matrix_json)
                    VALUES (?1, ?2, ?3, ?4)";
                if let Err(e) = conn.execute(
                    sql,
                    params![
                        tournament.room_id,
                        tournament.finished_at_sec,
                        stats_json,
                        match_matrix_json,
                    ],
                ) {
                    warn!("Failed to insert tournament result into SQLite DB: {e}");
                }
            }
        }

        self.tournament_results.insert(0, tournament);
        if self.tournament_results.len() > 50 {
            self.tournament_results.pop();
        }
    }

    pub fn list_matches(&self) -> Vec<FinishedMatchInfo> {
        self.recent_matches.clone()
    }

    pub fn list_tournaments(&self) -> Vec<RoomTournamentResult> {
        self.tournament_results.clone()
    }

    pub fn list_matches_limited(&self, limit: usize) -> Vec<FinishedMatchInfo> {
        self.recent_matches.iter().take(limit).cloned().collect()
    }

    pub fn list_tournaments_limited(&self, limit: usize) -> Vec<RoomTournamentResult> {
        self.tournament_results.iter().take(limit).cloned().collect()
    }
}

pub type SharedMatchHistoryRegistry = Arc<Mutex<MatchHistoryRegistry>>;

