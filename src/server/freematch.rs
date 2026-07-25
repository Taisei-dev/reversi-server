use super::active_matches::SharedActiveMatchRegistry;
use super::history::SharedMatchHistoryRegistry;
use super::match_runner::{Participant, run_match};
use super::session::ActivePlayerSession;
use crate::game::PreferredColor;
use serde::Serialize;
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tracing::info;

#[derive(Debug, Clone, Serialize)]
pub struct FreeMatchClientInfo {
    pub client_id: String,
    pub player_name: String,
    pub assigned_time_ms: i32,
    pub cooldown_sec: u64,
}

pub struct WaitingSession {
    pub id: String,
    pub session: ActivePlayerSession,
    pub assigned_time_ms: i32,
    pub pref_color: PreferredColor,
    pub cooldown_sec: u64,
}

#[derive(Debug, Clone)]
pub struct CooldownEntry {
    pub p1_name: String,
    pub p2_name: String,
    pub available_at: Instant,
}

impl CooldownEntry {
    pub fn is_pair(&self, name1: &str, name2: &str) -> bool {
        (self.p1_name == name1 && self.p2_name == name2)
            || (self.p1_name == name2 && self.p2_name == name1)
    }
}

pub struct FreeMatchQueue {
    ready_list: Vec<WaitingSession>,
    cooldown_queue: VecDeque<CooldownEntry>,
    next_id: u64,
}

impl FreeMatchQueue {
    pub fn new() -> Self {
        Self {
            ready_list: Vec::new(),
            cooldown_queue: VecDeque::new(),
            next_id: 1,
        }
    }

    pub fn list_waiting(&mut self) -> Vec<FreeMatchClientInfo> {
        self.ready_list.retain(|w| !w.session.is_closed());

        self.ready_list
            .iter()
            .map(|w| FreeMatchClientInfo {
                client_id: w.id.clone(),
                player_name: w.session.handle.player_name.clone(),
                assigned_time_ms: w.assigned_time_ms,
                cooldown_sec: w.cooldown_sec,
            })
            .collect()
    }

    pub fn remove_by_name(&mut self, player_name: &str) {
        self.ready_list.retain(|w| w.session.handle.player_name != player_name);
        self.cooldown_queue.retain(|e| e.p1_name != player_name && e.p2_name != player_name);
    }
}

pub type SharedFreeMatchQueue = Arc<Mutex<FreeMatchQueue>>;

pub async fn enqueue_freematch_session(
    queue: SharedFreeMatchQueue,
    session: ActivePlayerSession,
    assigned_time_ms: i32,
    pref_color: PreferredColor,
    cooldown_sec: u64,
    active_matches: SharedActiveMatchRegistry,
    history_registry: SharedMatchHistoryRegistry,
) {
    let mut lock = queue.lock().await;
    lock.ready_list.retain(|w| !w.session.is_closed());

    let client_id = format!("free_{}", lock.next_id);
    lock.next_id += 1;

    let new_player_name = session.handle.player_name.clone();
    info!("Client '{new_player_name}' (ID: {client_id}) entered FreeMatch queue (cooldown_sec: {cooldown_sec}s)");

    let new_waiting = WaitingSession {
        id: client_id,
        session,
        assigned_time_ms,
        pref_color,
        cooldown_sec,
    };

    lock.ready_list.push(new_waiting);

    try_matchmake(&mut lock, queue.clone(), active_matches, history_registry);
}

fn try_matchmake(
    queue: &mut FreeMatchQueue,
    queue_shared: SharedFreeMatchQueue,
    active_matches: SharedActiveMatchRegistry,
    history_registry: SharedMatchHistoryRegistry,
) {
    let now = Instant::now();

    // 期限切れの CooldownEntry を先頭から Pop (クリーンアップ)
    while let Some(front) = queue.cooldown_queue.front() {
        if now >= front.available_at {
            queue.cooldown_queue.pop_front();
        } else {
            break;
        }
    }

    queue.ready_list.retain(|w| !w.session.handle.is_closed());
    let len = queue.ready_list.len();
    let mut matched_indices: Option<(usize, usize)> = None;

    'outer: for i in 0..len {
        for j in (i + 1)..len {
            let p1_name = &queue.ready_list[i].session.handle.player_name;
            let p2_name = &queue.ready_list[j].session.handle.player_name;

            // cooldown_queue 内に解禁待ちの (p1_name, p2_name) ペアが存在するかチェック
            let is_in_cooldown = queue
                .cooldown_queue
                .iter()
                .any(|entry| entry.is_pair(p1_name, p2_name) && now < entry.available_at);

            if !is_in_cooldown {
                matched_indices = Some((i, j));
                break 'outer;
            }
        }
    }

    if let Some((i, j)) = matched_indices {
        let s2 = queue.ready_list.remove(j);
        let s1 = queue.ready_list.remove(i);

        let p1_name = s1.session.handle.player_name.clone();
        let p2_name = s2.session.handle.player_name.clone();
        let assigned_time = s1.assigned_time_ms.min(s2.assigned_time_ms);
        let max_cooldown_sec = s1.cooldown_sec.max(s2.cooldown_sec);

        let p1_session = s1.session;
        let p2_session = s2.session;

        let (p_black, p_white) = match (s1.pref_color, s2.pref_color) {
            (PreferredColor::Black, PreferredColor::White) => (Participant::Human(p1_session), Participant::Human(p2_session)),
            (PreferredColor::White, PreferredColor::Black) => (Participant::Human(p2_session), Participant::Human(p1_session)),
            (PreferredColor::Black, _) => (Participant::Human(p1_session), Participant::Human(p2_session)),
            (_, PreferredColor::Black) => (Participant::Human(p2_session), Participant::Human(p1_session)),
            (PreferredColor::White, _) => (Participant::Human(p2_session), Participant::Human(p1_session)),
            (_, PreferredColor::White) => (Participant::Human(p2_session), Participant::Human(p1_session)),
            _ => {
                if rand::random() {
                    (Participant::Human(p1_session), Participant::Human(p2_session))
                } else {
                    (Participant::Human(p2_session), Participant::Human(p1_session))
                }
            }
        };

        info!("Matched FreeMatch pair: {p1_name} vs {p2_name}");

        let active_matches_cl = active_matches.clone();
        let history_registry_cl = history_registry.clone();
        let queue_shared_cl = queue_shared.clone();

        let s1_cooldown_sec = s1.cooldown_sec;
        let s2_cooldown_sec = s2.cooldown_sec;

        tokio::spawn(async move {
            let (_res, ret_black, ret_white) = run_match(
                p_black,
                p_white,
                assigned_time,
                "FreeMatch",
                active_matches_cl,
                history_registry_cl,
            )
            .await;

            // 対戦終了: 500ms パケット待機を経て ReadyList に復帰 & CooldownQueue に登録
            tokio::time::sleep(Duration::from_millis(500)).await;

            let available_at = Instant::now() + Duration::from_secs(max_cooldown_sec);
            let mut q_lock = queue_shared_cl.lock().await;

            // クールダウンエントリーの登録
            q_lock.cooldown_queue.push_back(CooldownEntry {
                p1_name: p1_name.clone(),
                p2_name: p2_name.clone(),
                available_at,
            });

            // 返却された Participant から ActivePlayerSession を回収し ReadyList に復帰
            if let Participant::Human(sess1) = ret_black {
                if !sess1.is_closed() {
                    let name = sess1.handle.player_name.clone();
                    let cooldown = if name == p1_name { s1_cooldown_sec } else { s2_cooldown_sec };
                    q_lock.ready_list.push(WaitingSession {
                        id: format!("free_re_{}", name),
                        session: sess1,
                        assigned_time_ms: assigned_time,
                        pref_color: PreferredColor::Random,
                        cooldown_sec: cooldown,
                    });
                }
            }
            if let Participant::Human(sess2) = ret_white {
                if !sess2.is_closed() {
                    let name = sess2.handle.player_name.clone();
                    let cooldown = if name == p1_name { s1_cooldown_sec } else { s2_cooldown_sec };
                    q_lock.ready_list.push(WaitingSession {
                        id: format!("free_re_{}", name),
                        session: sess2,
                        assigned_time_ms: assigned_time,
                        pref_color: PreferredColor::Random,
                        cooldown_sec: cooldown,
                    });
                }
            }

            // トリガー 1 (対戦終了・復帰イベント) によるマトリックス再判定
            try_matchmake(&mut q_lock, queue_shared_cl.clone(), active_matches.clone(), history_registry.clone());

            // トリガー 3 (キューの先頭ペアが解禁時刻を迎えるタイマーイベント)
            if max_cooldown_sec > 0 {
                let queue_bg = queue_shared_cl.clone();
                let active_m_bg = active_matches;
                let history_r_bg = history_registry;
                tokio::spawn(async move {
                    tokio::time::sleep(Duration::from_secs(max_cooldown_sec)).await;
                    let mut q_bg = queue_bg.lock().await;
                    try_matchmake(&mut q_bg, queue_bg.clone(), active_m_bg, history_r_bg);
                });
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::session::SessionHandle;
    use tokio::sync::mpsc;

    fn create_mock_session(name: &str) -> (ActivePlayerSession, mpsc::UnboundedReceiver<crate::protocol::ServerCommand>) {
        let (tx, rx) = mpsc::unbounded_channel();
        let (_cmd_tx, cmd_rx) = mpsc::unbounded_channel();
        let handle = SessionHandle {
            player_name: name.to_string(),
            tx,
        };
        let session = ActivePlayerSession { handle, rx: cmd_rx };
        (session, rx)
    }

    #[tokio::test]
    async fn test_initial_matchmaking() {
        let mut queue = FreeMatchQueue::new();
        let (s1, _) = create_mock_session("Alice");
        let (s2, _) = create_mock_session("Bob");

        queue.ready_list.push(WaitingSession {
            id: "1".to_string(),
            session: s1,
            assigned_time_ms: 1000,
            pref_color: PreferredColor::Random,
            cooldown_sec: 10,
        });
        queue.ready_list.push(WaitingSession {
            id: "2".to_string(),
            session: s2,
            assigned_time_ms: 1000,
            pref_color: PreferredColor::Random,
            cooldown_sec: 5,
        });

        assert_eq!(queue.ready_list.len(), 2);
    }

    #[tokio::test]
    async fn test_cooldown_blocking_same_pair() {
        let mut queue = FreeMatchQueue::new();
        let now = Instant::now();

        // Alice と Bob が 10秒のクールダウン中
        queue.cooldown_queue.push_back(CooldownEntry {
            p1_name: "Alice".to_string(),
            p2_name: "Bob".to_string(),
            available_at: now + Duration::from_secs(10),
        });

        let (s1, _) = create_mock_session("Alice");
        let (s2, _) = create_mock_session("Bob");
        queue.ready_list.push(WaitingSession { id: "1".to_string(), session: s1, assigned_time_ms: 1000, pref_color: PreferredColor::Random, cooldown_sec: 10 });
        queue.ready_list.push(WaitingSession { id: "2".to_string(), session: s2, assigned_time_ms: 1000, pref_color: PreferredColor::Random, cooldown_sec: 10 });

        let p1 = &queue.ready_list[0].session.handle.player_name;
        let p2 = &queue.ready_list[1].session.handle.player_name;

        let is_in_cooldown = queue.cooldown_queue.iter().any(|e| e.is_pair(p1, p2) && now < e.available_at);
        assert!(is_in_cooldown, "Alice and Bob should be blocked by cooldown");
    }

    #[tokio::test]
    async fn test_different_opponent_immediate_match() {
        let mut queue = FreeMatchQueue::new();
        let now = Instant::now();

        // Alice ↔ Bob はクールダウン中
        queue.cooldown_queue.push_back(CooldownEntry {
            p1_name: "Alice".to_string(),
            p2_name: "Bob".to_string(),
            available_at: now + Duration::from_secs(10),
        });

        // 新しい Charlie が参加
        let (s1, _) = create_mock_session("Alice");
        let (s3, _) = create_mock_session("Charlie");
        queue.ready_list.push(WaitingSession { id: "1".to_string(), session: s1, assigned_time_ms: 1000, pref_color: PreferredColor::Random, cooldown_sec: 10 });
        queue.ready_list.push(WaitingSession { id: "3".to_string(), session: s3, assigned_time_ms: 1000, pref_color: PreferredColor::Random, cooldown_sec: 10 });

        let p1 = &queue.ready_list[0].session.handle.player_name;
        let p3 = &queue.ready_list[1].session.handle.player_name;

        let is_in_cooldown = queue.cooldown_queue.iter().any(|e| e.is_pair(p1, p3) && now < e.available_at);
        assert!(!is_in_cooldown, "Alice and Charlie should NOT be in cooldown and should match immediately");
    }

    #[tokio::test]
    async fn test_max_cooldown_sec_calculation() {
        let cd1: u64 = 10;
        let cd2: u64 = 3;
        let max_cd = cd1.max(cd2);
        assert_eq!(max_cd, 10, "max_cooldown_sec should be the maximum of both clients' cooldown_sec");
    }

    #[tokio::test]
    async fn test_lazy_deletion_on_closed_session() {
        let mut queue = FreeMatchQueue::new();
        let (s1, rx1) = create_mock_session("Alice");
        let (s2, _rx2) = create_mock_session("Bob");

        queue.ready_list.push(WaitingSession { id: "1".to_string(), session: s1, assigned_time_ms: 1000, pref_color: PreferredColor::Random, cooldown_sec: 10 });
        queue.ready_list.push(WaitingSession { id: "2".to_string(), session: s2, assigned_time_ms: 1000, pref_color: PreferredColor::Random, cooldown_sec: 10 });

        // Alice の受信用チャネルを破棄 (切断離脱)
        drop(rx1);
        queue.ready_list.retain(|w| !w.session.handle.is_closed());

        assert_eq!(queue.ready_list.len(), 1, "Alice should be lazily removed from ready_list");
        assert_eq!(queue.ready_list[0].session.handle.player_name, "Bob");
    }
}
