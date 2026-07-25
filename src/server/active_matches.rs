use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize)]
pub struct ActiveMatchInfo {
    pub match_id: String,
    pub mode: String,
    pub black_name: String,
    pub white_name: String,
    pub move_count: u32,
    pub started_at_sec: u64,
}

pub struct ActiveMatchRegistry {
    matches: HashMap<String, ActiveMatchInfo>,
    next_id: u64,
}

impl ActiveMatchRegistry {
    pub fn new() -> Self {
        Self {
            matches: HashMap::new(),
            next_id: 1,
        }
    }

    pub fn register(&mut self, mode: &str, black_name: &str, white_name: &str) -> String {
        let match_id = format!("match_{}", self.next_id);
        self.next_id += 1;

        let start_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let info = ActiveMatchInfo {
            match_id: match_id.clone(),
            mode: mode.to_string(),
            black_name: black_name.to_string(),
            white_name: white_name.to_string(),
            move_count: 0,
            started_at_sec: start_time,
        };

        self.matches.insert(match_id.clone(), info);
        match_id
    }

    pub fn update_move(&mut self, match_id: &str) {
        if let Some(info) = self.matches.get_mut(match_id) {
            info.move_count += 1;
        }
    }

    pub fn unregister(&mut self, match_id: &str) {
        self.matches.remove(match_id);
    }

    pub fn list(&self) -> Vec<ActiveMatchInfo> {
        self.matches.values().cloned().collect()
    }
}

pub type SharedActiveMatchRegistry = Arc<Mutex<ActiveMatchRegistry>>;
