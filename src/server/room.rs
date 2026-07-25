use super::active_matches::SharedActiveMatchRegistry;
use super::history::{RoomTournamentResult, SharedMatchHistoryRegistry, TournamentMatchDetail};
use super::match_runner::{Participant, run_match};
use super::session::ActivePlayerSession;
use crate::game::Color;
use crate::protocol::{PlayerStat, ServerCommand};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;

#[derive(Debug, Clone, Serialize)]
pub struct RoomInfo {
    pub room_id: String,
    pub current_clients: usize,
    pub ai_count: usize,
    pub assigned_time_ms: i32,
    pub match_count: u32,
    pub client_names: Vec<String>,
    pub ai_names: Vec<String>,
    pub status: String,
}

pub struct Room {
    pub room_id: String,
    pub clients: Vec<ActivePlayerSession>,
    pub ais: Vec<(String, i32, bool)>, // (name, level, use_book)
    pub assigned_time_ms: i32,
    pub match_count: u32,
    pub is_running: bool,
}

impl Room {
    pub fn new(room_id: impl Into<String>, assigned_time_ms: i32, match_count: u32) -> Self {
        Self {
            room_id: room_id.into(),
            clients: Vec::new(),
            ais: Vec::new(),
            assigned_time_ms,
            match_count: match_count.max(1),
            is_running: false,
        }
    }

    pub fn add_ai(&mut self, ai_type_str: &str, level: i32, use_book: bool) {
        let ai_name = if ai_type_str == "random" {
            format!("AI_Random_{}", self.ais.len() + 1)
        } else {
            if use_book {
                format!("Egaroucid_L{level}")
            } else {
                format!("Egaroucid_L{level}_NoBook")
            }
        };
        self.ais.push((ai_name, level, use_book));
    }

    pub fn remove_ai(&mut self, ai_name: &str) -> bool {
        if self.is_running {
            return false;
        }
        if let Some(pos) = self.ais.iter().position(|(name, ..)| name == ai_name) {
            self.ais.remove(pos);
            return true;
        }
        false
    }

    pub fn to_info(&self) -> RoomInfo {
        RoomInfo {
            room_id: self.room_id.clone(),
            current_clients: self.clients.len(),
            ai_count: self.ais.len(),
            assigned_time_ms: self.assigned_time_ms,
            match_count: self.match_count,
            client_names: self.clients.iter().map(|s| s.handle.player_name.clone()).collect(),
            ai_names: self.ais.iter().map(|(name, ..)| name.clone()).collect(),
            status: if self.is_running { "running".to_string() } else { "waiting".to_string() },
        }
    }
}

pub struct RoomManager {
    rooms: HashMap<String, Room>,
}

impl RoomManager {
    pub fn new() -> Self {
        Self {
            rooms: HashMap::new(),
        }
    }

    pub fn get_or_create(&mut self, room_id: &str, assigned_time_ms: i32, match_count: u32) -> &mut Room {
        self.rooms
            .entry(room_id.to_string())
            .or_insert_with(|| Room::new(room_id, assigned_time_ms, match_count))
    }

    pub fn remove_room(&mut self, room_id: &str) -> bool {
        if let Some(room) = self.rooms.get(room_id) {
            if !room.is_running {
                self.rooms.remove(room_id);
                return true;
            }
        }
        false
    }

    pub fn remove_client_from_room(&mut self, room_id: &str, player_name: &str) -> bool {
        if let Some(room) = self.rooms.get_mut(room_id) {
            if !room.is_running {
                room.clients.retain(|c| c.handle.player_name != player_name);
                return true;
            }
        }
        false
    }

    pub fn update_room(&mut self, room_id: &str, time_ms: Option<i32>, match_count: Option<u32>) -> bool {
        if let Some(room) = self.rooms.get_mut(room_id) {
            if !room.is_running {
                if let Some(t) = time_ms {
                    room.assigned_time_ms = t;
                }
                if let Some(m) = match_count {
                    room.match_count = m.max(1);
                }
                return true;
            }
        }
        false
    }

    pub fn add_ai_with_book(&mut self, room_id: &str, ai_type_str: &str, level: i32, use_book: bool) -> bool {
        if let Some(room) = self.rooms.get_mut(room_id) {
            if !room.is_running {
                room.add_ai(ai_type_str, level, use_book);
                return true;
            }
        }
        false
    }

    pub fn remove_ai(&mut self, room_id: &str, ai_name: &str) -> bool {
        if let Some(room) = self.rooms.get_mut(room_id) {
            return room.remove_ai(ai_name);
        }
        false
    }

    pub fn start_tournament(
        &mut self,
        room_id: &str,
        active_matches: SharedActiveMatchRegistry,
        history_registry: SharedMatchHistoryRegistry,
    ) -> bool {
        if let Some(mut room) = self.rooms.remove(room_id) {
            if room.is_running || (room.clients.is_empty() && room.ais.is_empty()) {
                self.rooms.insert(room_id.to_string(), room);
                return false;
            }

            room.is_running = true;
            let room_time = room.assigned_time_ms;
            let match_cnt = room.match_count;
            let room_id_clone = room.room_id.clone();
            let clients = std::mem::take(&mut room.clients);
            let ais = std::mem::take(&mut room.ais);

            tokio::spawn(async move {
                run_round_robin_tournament(clients, ais, room_time, match_cnt, &room_id_clone, active_matches, history_registry).await;
            });
            return true;
        }
        false
    }

    pub fn list_info(&self) -> Vec<RoomInfo> {
        self.rooms.values().map(|r| r.to_info()).collect()
    }
}

pub type SharedRoomManager = Arc<Mutex<RoomManager>>;

pub async fn join_room_session(
    mgr: SharedRoomManager,
    room_id: String,
    session: ActivePlayerSession,
    assigned_time_ms: i32,
) {
    let mut lock = mgr.lock().await;
    let room = lock.get_or_create(&room_id, assigned_time_ms, 1);
    if !room.is_running {
        info!("Client '{}' joined room '{}'", session.handle.player_name, room_id);
        room.clients.push(session);
    }
}

enum TempSlot {
    Participant(Participant),
    Empty,
}

async fn run_round_robin_tournament(
    client_sessions: Vec<ActivePlayerSession>,
    ais: Vec<(String, i32, bool)>,
    assigned_time_ms: i32,
    match_count: u32,
    room_id: &str,
    active_matches: SharedActiveMatchRegistry,
    history_registry: SharedMatchHistoryRegistry,
) {
    let mut slots: Vec<TempSlot> = Vec::new();

    for sess in client_sessions {
        slots.push(TempSlot::Participant(Participant::Human(sess)));
    }
    for (ai_name, level, use_book) in ais {
        if ai_name.starts_with("AI_Random") {
            slots.push(TempSlot::Participant(Participant::RandomAi { name: ai_name }));
        } else {
            slots.push(TempSlot::Participant(Participant::EgaroucidAi {
                name: ai_name,
                level,
                use_book,
            }));
        }
    }

    let n = slots.len();
    info!("Starting Round-Robin tournament for {n} participants in room '{room_id}' (Match count: {match_count})");

    let mut stats_map: HashMap<String, PlayerStat> = HashMap::new();
    for slot in &slots {
        if let TempSlot::Participant(p) = slot {
            let name = p.name().to_string();
            stats_map.insert(
                name.clone(),
                PlayerStat {
                    player_name: name,
                    score: 0,
                    wins: 0,
                    loses: 0,
                },
            );
        }
    }

    let mode_str = format!("Room_{room_id}");
    let mut match_matrix_details: Vec<TournamentMatchDetail> = Vec::new();

    for round_idx in 1..=match_count {
        for i in 0..n {
            for j in (i + 1)..n {
                let p1 = match std::mem::replace(&mut slots[i], TempSlot::Empty) {
                    TempSlot::Participant(p) => p,
                    TempSlot::Empty => continue,
                };
                let p2 = match std::mem::replace(&mut slots[j], TempSlot::Empty) {
                    TempSlot::Participant(p) => p,
                    TempSlot::Empty => {
                        slots[i] = TempSlot::Participant(p1);
                        continue;
                    }
                };

                let name1 = p1.name().to_string();
                let name2 = p2.name().to_string();

                let (res, p1_out, p2_out) = run_match(
                    p1,
                    p2,
                    assigned_time_ms,
                    &mode_str,
                    active_matches.clone(),
                    history_registry.clone(),
                )
                .await;

                slots[i] = TempSlot::Participant(p1_out);
                slots[j] = TempSlot::Participant(p2_out);

                let mut p1_score = res.black_stones;
                let mut p2_score = res.white_stones;
                let mut res_text = "draw".to_string();

                if name1 != res.black_name {
                    std::mem::swap(&mut p1_score, &mut p2_score);
                }

                if let Some(winner) = res.winner_color {
                    if winner == Color::Black {
                        if let Some(st) = stats_map.get_mut(&res.black_name) {
                            st.score += 1;
                            st.wins += 1;
                        }
                        if let Some(st) = stats_map.get_mut(&res.white_name) {
                            st.score -= 1;
                            st.loses += 1;
                        }
                        res_text = if name1 == res.black_name { "win".to_string() } else { "lose".to_string() };
                    } else {
                        if let Some(st) = stats_map.get_mut(&res.white_name) {
                            st.score += 1;
                            st.wins += 1;
                        }
                        if let Some(st) = stats_map.get_mut(&res.black_name) {
                            st.score -= 1;
                            st.loses += 1;
                        }
                        res_text = if name1 == res.white_name { "win".to_string() } else { "lose".to_string() };
                    }
                }

                match_matrix_details.push(TournamentMatchDetail {
                    p1_name: name1,
                    p2_name: name2,
                    p1_score,
                    p2_score,
                    result_text: res_text,
                    moves: res.moves,
                    round_index: round_idx,
                });
            }
        }
    }

    let mut stats_list: Vec<PlayerStat> = stats_map.into_values().collect();
    stats_list.sort_by(|a, b| b.score.cmp(&a.score));

    info!("Tournament ended in room '{room_id}'! Results: {:?}", stats_list);

    for slot in &slots {
        if let TempSlot::Participant(Participant::Human(h)) = slot {
            let _ = h.handle.send(ServerCommand::Bye {
                stats: stats_list.clone(),
            });
        }
    }

    let finished_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let tournament_res = RoomTournamentResult {
        room_id: room_id.to_string(),
        finished_at_sec: finished_at,
        stats: stats_list,
        match_matrix: match_matrix_details,
    };

    let mut lock = history_registry.lock().await;
    lock.record_tournament(tournament_res);
}
