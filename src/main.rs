mod ai;
mod game;
mod protocol;
mod server;

use server::active_matches::ActiveMatchRegistry;
use server::freematch::FreeMatchQueue;
use server::history::MatchHistoryRegistry;
use server::room::RoomManager;
use server::wait_human::WaitHumanQueue;
use server::ws_bridge::{AppState, create_web_router};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    info!("Starting Reversi Unified WebSocket Server...");

    let freematch_queue = Arc::new(Mutex::new(FreeMatchQueue::new()));
    let room_mgr = Arc::new(Mutex::new(RoomManager::new()));
    let active_matches = Arc::new(Mutex::new(ActiveMatchRegistry::new()));
    let wait_human_queue = Arc::new(Mutex::new(WaitHumanQueue::new()));
    let history_registry = Arc::new(Mutex::new(MatchHistoryRegistry::new()));

    // Web UI & Unified WebSocket Bridge Server (Port 8080)
    let state = AppState {
        freematch_queue: freematch_queue.clone(),
        room_mgr: room_mgr.clone(),
        active_matches: active_matches.clone(),
        wait_human_queue: wait_human_queue.clone(),
        history_registry: history_registry.clone(),
    };

    let app = create_web_router(state, "dist");

    let addr: SocketAddr = "0.0.0.0:8080".parse()?;
    info!("Web UI and WebSocket Server listening on http://{addr}");
    info!("Client WebSocket endpoints available under ws://{addr}/client/...");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
