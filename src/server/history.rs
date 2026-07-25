use serde::Serialize;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize)]
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

#[derive(Debug, Clone, Serialize)]
pub struct TournamentMatchDetail {
    pub p1_name: String,
    pub p2_name: String,
    pub p1_score: u32,
    pub p2_score: u32,
    pub result_text: String, // "win", "lose", "draw"
    pub moves: Vec<String>,
    pub round_index: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct RoomTournamentResult {
    pub room_id: String,
    pub finished_at_sec: u64,
    pub stats: Vec<crate::protocol::PlayerStat>,
    pub match_matrix: Vec<TournamentMatchDetail>,
}

pub struct MatchHistoryRegistry {
    recent_matches: Vec<FinishedMatchInfo>,
    tournament_results: Vec<RoomTournamentResult>,
}

impl MatchHistoryRegistry {
    pub fn new() -> Self {
        Self {
            recent_matches: Vec::new(),
            tournament_results: Vec::new(),
        }
    }

    pub fn record_match(&mut self, match_info: FinishedMatchInfo) {
        if !match_info.mode.starts_with("Room_") {
            self.recent_matches.insert(0, match_info);
            if self.recent_matches.len() > 50 {
                self.recent_matches.pop();
            }
        }
    }

    pub fn record_tournament(&mut self, tournament: RoomTournamentResult) {
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
