use super::active_matches::SharedActiveMatchRegistry;
use super::history::SharedMatchHistoryRegistry;
use super::match_runner::{Participant, run_match};
use super::session::ActivePlayerSession;
use crate::game::PreferredColor;
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;

#[derive(Debug, Clone, Serialize)]
pub struct WaitingClientInfo {
    pub client_id: String,
    pub player_name: String,
    pub assigned_time_ms: i32,
}

pub struct WaitingClient {
    pub client_id: String,
    pub session: ActivePlayerSession,
    pub assigned_time_ms: i32,
}

pub struct WaitHumanQueue {
    clients: Vec<WaitingClient>,
    next_id: u64,
}

impl WaitHumanQueue {
    pub fn new() -> Self {
        Self {
            clients: Vec::new(),
            next_id: 1,
        }
    }

    pub fn register(&mut self, session: ActivePlayerSession, assigned_time_ms: i32) -> String {
        let client_id = format!("client_{}", self.next_id);
        self.next_id += 1;
        self.clients.push(WaitingClient {
            client_id: client_id.clone(),
            session,
            assigned_time_ms,
        });
        client_id
    }

    pub fn take(&mut self, client_id: &str) -> Option<(ActivePlayerSession, i32)> {
        // 切断済みをクリーンアップ
        self.clients.retain(|c| !c.session.is_closed());

        if let Some(pos) = self.clients.iter().position(|c| c.client_id == client_id) {
            let client = self.clients.remove(pos);
            Some((client.session, client.assigned_time_ms))
        } else {
            None
        }
    }

    pub fn remove_by_name(&mut self, player_name: &str) {
        self.clients.retain(|c| c.session.handle.player_name != player_name);
    }

    pub fn list(&mut self) -> Vec<WaitingClientInfo> {
        // 切断されたゴーストセッションを自動除去！
        self.clients.retain(|c| !c.session.is_closed());

        self.clients
            .iter()
            .map(|c| WaitingClientInfo {
                client_id: c.client_id.clone(),
                player_name: c.session.handle.player_name.clone(),
                assigned_time_ms: c.assigned_time_ms,
            })
            .collect()
    }
}

pub type SharedWaitHumanQueue = Arc<Mutex<WaitHumanQueue>>;

pub async fn start_vs_human_match(
    queue: SharedWaitHumanQueue,
    client_id: &str,
    web_session: ActivePlayerSession,
    pref_color: PreferredColor,
    active_matches: SharedActiveMatchRegistry,
    history_registry: SharedMatchHistoryRegistry,
) -> bool {
    let mut lock = queue.lock().await;
    let (client_sess, time_ms) = match lock.take(client_id) {
        Some(res) => res,
        None => return false,
    };
    drop(lock);

    let web_name = web_session.handle.player_name.clone();
    let client_name = client_sess.handle.player_name.clone();

    info!("Starting VsHuman match: Web Player '{web_name}' vs Client '{client_name}'");

    let web_is_white = match pref_color {
        PreferredColor::White => true,
        PreferredColor::Black => false,
        PreferredColor::Random => rand::random(),
    };

    let (p_black, p_white) = if web_is_white {
        (Participant::Human(client_sess), Participant::Human(web_session))
    } else {
        (Participant::Human(web_session), Participant::Human(client_sess))
    };

    let queue_for_requeue = queue.clone();

    tokio::spawn(async move {
        let (_res, p1_out, p2_out) = run_match(
            p_black,
            p_white,
            time_ms,
            "vs-Human",
            active_matches,
            history_registry,
        )
        .await;

        // vs-human 対局終了後、Client が切断されていなければ自動で WaitHumanQueue へ再登録！
        let client_sess_opt = match (p1_out, p2_out) {
            (Participant::Human(h1), _) if h1.handle.player_name == client_name => Some(h1),
            (_, Participant::Human(h2)) if h2.handle.player_name == client_name => Some(h2),
            _ => None,
        };

        if let Some(client_sess) = client_sess_opt {
            if !client_sess.handle.is_closed() {
                let mut q_lock = queue_for_requeue.lock().await;
                let new_id = q_lock.register(client_sess, time_ms);
                info!("Client automatically re-registered to wait_human queue: ID {new_id}");
            } else {
                info!("Client '{client_name}' was disconnected, skipping re-registration.");
            }
        }
    });

    true
}
