use super::active_matches::{ActiveMatchInfo, SharedActiveMatchRegistry};
use super::ai_match::{AiType, start_ai_match};
use super::freematch::{FreeMatchClientInfo, SharedFreeMatchQueue, enqueue_freematch_session};
use super::history::{FinishedMatchInfo, RoomTournamentResult, SharedMatchHistoryRegistry};
use super::room::{RoomInfo, SharedRoomManager, join_room_session};
use super::session::{ActivePlayerSession, SessionHandle};
use super::wait_human::{SharedWaitHumanQueue, WaitingClientInfo, start_vs_human_match};
use crate::game::PreferredColor;
use crate::protocol::{ClientCommand, ServerCommand, parse_client_command};
use axum::{
    Json, Router,
    extract::{
        Path, Query, State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::Response,
    routing::{get, post},
};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::{ServeDir, ServeFile};
use tracing::info;

#[derive(Clone)]
pub struct AppState {
    pub freematch_queue: SharedFreeMatchQueue,
    pub room_mgr: SharedRoomManager,
    pub active_matches: SharedActiveMatchRegistry,
    pub wait_human_queue: SharedWaitHumanQueue,
    pub history_registry: SharedMatchHistoryRegistry,
}

#[derive(Deserialize)]
pub struct MatchQuery {
    pub time_ms: Option<i32>,
    pub level: Option<i32>,
    pub use_book: Option<bool>,
    pub color: Option<String>,
    pub cooldown_sec: Option<u64>,
}

#[derive(Deserialize)]
pub struct CreateRoomRequest {
    pub room_id: String,
    pub time_ms: Option<i32>,
    pub match_count: Option<u32>,
}

#[derive(Deserialize)]
pub struct UpdateRoomRequest {
    pub time_ms: Option<i32>,
    pub match_count: Option<u32>,
}

#[derive(Deserialize)]
pub struct AddAiRequest {
    pub ai_type: Option<String>,
    pub level: Option<i32>,
    pub use_book: Option<bool>,
}

#[derive(Deserialize)]
pub struct RemoveAiRequest {
    pub ai_name: String,
}

#[derive(Serialize)]
pub struct ApiResponse {
    pub success: bool,
}

#[derive(Serialize)]
pub struct LobbyResponse {
    pub open_rooms: Vec<RoomInfo>,
    pub waiting_clients: Vec<WaitingClientInfo>,
    pub freematch_clients: Vec<FreeMatchClientInfo>,
    pub active_matches: Vec<ActiveMatchInfo>,
}

#[derive(Serialize)]
pub struct HistoryResponse {
    pub recent_matches: Vec<FinishedMatchInfo>,
    pub tournament_results: Vec<RoomTournamentResult>,
}

pub fn create_web_router(state: AppState, web_dir: &str) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let index_html_path = format!("{web_dir}/index.html");
    let serve_dir = ServeDir::new(web_dir)
        .not_found_service(ServeFile::new(index_html_path));

    Router::new()
        .route("/api/lobby", get(get_lobby_handler))
        .route("/api/history", get(get_history_handler))
        .route("/api/rooms", post(create_room_handler))
        .route("/api/rooms/:room_id/delete", post(delete_room_handler))
        .route("/api/rooms/:room_id/update", post(update_room_handler))
        .route("/api/rooms/:room_id/add_ai", post(add_ai_handler))
        .route("/api/rooms/:room_id/remove_ai", post(remove_ai_handler))
        .route("/api/rooms/:room_id/start", post(start_room_handler))
        .route("/client/freematch", get(ws_freematch_handler))
        .route("/client/vs-human", get(ws_client_wait_human_handler))
        .route("/client/room/:room_id", get(ws_room_handler))
        .route("/client/ai/random", get(ws_ai_random_handler))
        .route("/client/ai/egaroucid", get(ws_ai_egaroucid_handler))
        .route("/ws/freematch", get(ws_freematch_handler))
        .route("/ws/room/:room_id", get(ws_room_handler))
        .route("/ws/client/wait_human", get(ws_client_wait_human_handler))
        .route("/ws/vs_human/join/:client_id", get(ws_vs_human_join_handler))
        .route("/ws/ai/random", get(ws_ai_random_handler))
        .route("/ws/ai/egaroucid", get(ws_ai_egaroucid_handler))
        .fallback_service(serve_dir)
        .layer(cors)
        .with_state(state)
}

async fn get_lobby_handler(
    State(state): State<AppState>,
) -> Json<LobbyResponse> {
    let open_rooms = state.room_mgr.lock().await.list_info();
    let waiting_clients = state.wait_human_queue.lock().await.list();
    let freematch_clients = state.freematch_queue.lock().await.list_waiting();
    let active_matches = state.active_matches.lock().await.list();

    Json(LobbyResponse {
        open_rooms,
        waiting_clients,
        freematch_clients,
        active_matches,
    })
}

#[derive(Deserialize)]
pub struct HistoryQuery {
    pub limit: Option<usize>,
}

async fn get_history_handler(
    Query(query): Query<HistoryQuery>,
    State(state): State<AppState>,
) -> Json<HistoryResponse> {
    let lock = state.history_registry.lock().await;
    let (recent_matches, tournament_results) = match query.limit {
        Some(limit) => (
            lock.list_matches_limited(limit),
            lock.list_tournaments_limited(limit),
        ),
        None => (
            lock.list_matches(),
            lock.list_tournaments(),
        ),
    };

    Json(HistoryResponse {
        recent_matches,
        tournament_results,
    })
}

use super::{DEFAULT_COOLDOWN_SEC, DEFAULT_TIME_MS};

async fn create_room_handler(
    State(state): State<AppState>,
    Json(req): Json<CreateRoomRequest>,
) -> Json<ApiResponse> {
    let time_ms = req.time_ms.unwrap_or(DEFAULT_TIME_MS);
    let match_count = req.match_count.unwrap_or(1);
    let mut mgr = state.room_mgr.lock().await;
    mgr.get_or_create(&req.room_id, time_ms, match_count);
    Json(ApiResponse { success: true })
}

async fn delete_room_handler(
    Path(room_id): Path<String>,
    State(state): State<AppState>,
) -> Json<ApiResponse> {
    let mut mgr = state.room_mgr.lock().await;
    let ok = mgr.remove_room(&room_id);
    Json(ApiResponse { success: ok })
}

async fn update_room_handler(
    Path(room_id): Path<String>,
    State(state): State<AppState>,
    Json(req): Json<UpdateRoomRequest>,
) -> Json<ApiResponse> {
    let mut mgr = state.room_mgr.lock().await;
    let ok = mgr.update_room(&room_id, req.time_ms, req.match_count);
    Json(ApiResponse { success: ok })
}

async fn add_ai_handler(
    Path(room_id): Path<String>,
    State(state): State<AppState>,
    Json(req): Json<AddAiRequest>,
) -> Json<ApiResponse> {
    let ai_type_str = req.ai_type.as_deref().unwrap_or("egaroucid");
    let level = req.level.unwrap_or(3).clamp(0, 20);
    let use_book = req.use_book.unwrap_or(true);
    let mut mgr = state.room_mgr.lock().await;
    let ok = mgr.add_ai_with_book(&room_id, ai_type_str, level, use_book);
    Json(ApiResponse { success: ok })
}

async fn remove_ai_handler(
    Path(room_id): Path<String>,
    State(state): State<AppState>,
    Json(req): Json<RemoveAiRequest>,
) -> Json<ApiResponse> {
    let mut mgr = state.room_mgr.lock().await;
    let ok = mgr.remove_ai(&room_id, &req.ai_name);
    Json(ApiResponse { success: ok })
}

async fn start_room_handler(
    Path(room_id): Path<String>,
    State(state): State<AppState>,
) -> Json<ApiResponse> {
    let mut mgr = state.room_mgr.lock().await;
    let ok = mgr.start_tournament(&room_id, state.active_matches.clone(), state.history_registry.clone());
    Json(ApiResponse { success: ok })
}

async fn ws_freematch_handler(
    Query(query): Query<MatchQuery>,
    State(state): State<AppState>,
    ws: WebSocketUpgrade,
) -> Response {
    let assigned_time = query.time_ms.unwrap_or(DEFAULT_TIME_MS);
    let cooldown_sec = query.cooldown_sec.unwrap_or(DEFAULT_COOLDOWN_SEC);
    let color = PreferredColor::from_str(query.color.as_deref().unwrap_or("random"));
    ws.on_upgrade(move |socket| handle_websocket(socket, WsMode::FreeMatch { cooldown_sec }, assigned_time, color, state))
}

async fn ws_room_handler(
    Path(room_id): Path<String>,
    Query(query): Query<MatchQuery>,
    State(state): State<AppState>,
    ws: WebSocketUpgrade,
) -> Response {
    let assigned_time = query.time_ms.unwrap_or(DEFAULT_TIME_MS);
    let color = PreferredColor::from_str(query.color.as_deref().unwrap_or("random"));
    ws.on_upgrade(move |socket| handle_websocket(socket, WsMode::Room { room_id }, assigned_time, color, state))
}

async fn ws_client_wait_human_handler(
    Query(query): Query<MatchQuery>,
    State(state): State<AppState>,
    ws: WebSocketUpgrade,
) -> Response {
    let assigned_time = query.time_ms.unwrap_or(DEFAULT_TIME_MS);
    ws.on_upgrade(move |socket| handle_websocket(socket, WsMode::ClientWaitHuman, assigned_time, PreferredColor::Random, state))
}

async fn ws_vs_human_join_handler(
    Path(client_id): Path<String>,
    Query(query): Query<MatchQuery>,
    State(state): State<AppState>,
    ws: WebSocketUpgrade,
) -> Response {
    let assigned_time = query.time_ms.unwrap_or(DEFAULT_TIME_MS);
    let color = PreferredColor::from_str(query.color.as_deref().unwrap_or("random"));
    ws.on_upgrade(move |socket| handle_websocket(socket, WsMode::VsHumanJoin { client_id }, assigned_time, color, state))
}

async fn ws_ai_random_handler(
    Query(query): Query<MatchQuery>,
    State(state): State<AppState>,
    ws: WebSocketUpgrade,
) -> Response {
    let assigned_time = query.time_ms.unwrap_or(DEFAULT_TIME_MS);
    let color = PreferredColor::from_str(query.color.as_deref().unwrap_or("random"));
    ws.on_upgrade(move |socket| handle_websocket(socket, WsMode::Ai { ai_type: AiType::Random }, assigned_time, color, state))
}

async fn ws_ai_egaroucid_handler(
    Query(query): Query<MatchQuery>,
    State(state): State<AppState>,
    ws: WebSocketUpgrade,
) -> Response {
    let assigned_time = query.time_ms.unwrap_or(DEFAULT_TIME_MS);
    let level = query.level.unwrap_or(3).clamp(0, 20);
    let use_book = query.use_book.unwrap_or(true);
    let color = PreferredColor::from_str(query.color.as_deref().unwrap_or("random"));
    ws.on_upgrade(move |socket| handle_websocket(socket, WsMode::Ai { ai_type: AiType::Egaroucid { level, use_book } }, assigned_time, color, state))
}

enum WsMode {
    FreeMatch { cooldown_sec: u64 },
    Room { room_id: String },
    ClientWaitHuman,
    VsHumanJoin { client_id: String },
    Ai { ai_type: AiType },
}

async fn handle_websocket(
    socket: WebSocket,
    mode: WsMode,
    assigned_time_ms: i32,
    preferred_color: PreferredColor,
    state: AppState,
) {
    let (mut ws_sender, mut ws_receiver) = socket.split();
    let (client_tx, client_rx) = tokio::sync::mpsc::unbounded_channel::<ClientCommand>();
    let (server_tx, mut server_rx) = tokio::sync::mpsc::unbounded_channel::<ServerCommand>();

    let write_task = tokio::spawn(async move {
        while let Some(cmd) = server_rx.recv().await {
            let wire = cmd.to_string();
            if ws_sender.send(Message::Text(wire)).await.is_err() {
                break;
            }
        }
    });

    let read_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(text))) = ws_receiver.next().await {
            let line = text.trim();
            if line.is_empty() {
                continue;
            }
            if let Ok(cmd) = parse_client_command(line) {
                if client_tx.send(cmd).is_err() {
                    break;
                }
            }
        }
    });

    let mut client_rx = client_rx;
    let mut player_name = "Anon.".to_string();

    while let Some(cmd) = client_rx.recv().await {
        if let ClientCommand::Open { player_name: name } = cmd {
            player_name = name;
            break;
        }
    }

    let handle = SessionHandle::new(player_name.clone(), server_tx);
    let active_session = ActivePlayerSession {
        handle: handle.clone(),
        rx: client_rx,
    };

    info!("Player '{player_name}' connected via WebSocket (assigned_time: {assigned_time_ms}ms)");

    match &mode {
        WsMode::FreeMatch { cooldown_sec } => {
            enqueue_freematch_session(
                state.freematch_queue.clone(),
                active_session,
                assigned_time_ms,
                preferred_color,
                *cooldown_sec,
                state.active_matches.clone(),
                state.history_registry.clone(),
            )
            .await;
        }
        WsMode::Room { room_id } => {
            join_room_session(state.room_mgr.clone(), room_id.clone(), active_session, assigned_time_ms).await;
        }
        WsMode::ClientWaitHuman => {
            let mut queue = state.wait_human_queue.lock().await;
            queue.register(active_session, assigned_time_ms);
        }
        WsMode::VsHumanJoin { client_id } => {
            start_vs_human_match(
                state.wait_human_queue.clone(),
                client_id,
                active_session,
                preferred_color,
                state.active_matches.clone(),
                state.history_registry.clone(),
            )
            .await;
        }
        WsMode::Ai { ai_type } => {
            start_ai_match(
                active_session,
                ai_type.clone(),
                assigned_time_ms,
                preferred_color,
                state.active_matches.clone(),
                state.history_registry.clone(),
            )
            .await;
        }
    }

    let _ = write_task.await;
    let _ = read_task.await;

    // WebSocket 接続が切断 (close / ネットワーク切断) されたため、各キューから即座に削除・クリーンアップ
    info!("WebSocket connection closed for player '{player_name}', cleaning up queues");
    match mode {
        WsMode::ClientWaitHuman => {
            let mut queue = state.wait_human_queue.lock().await;
            queue.remove_by_name(&player_name);
        }
        WsMode::FreeMatch { .. } => {
            let mut queue = state.freematch_queue.lock().await;
            queue.remove_by_name(&player_name);
        }
        WsMode::Room { room_id } => {
            let mut mgr = state.room_mgr.lock().await;
            mgr.remove_client_from_room(&room_id, &player_name);
        }
        _ => {}
    }
}
