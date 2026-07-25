use super::active_matches::SharedActiveMatchRegistry;
use super::ai_match::{AiType, start_ai_match};
use super::freematch::{SharedFreeMatchQueue, enqueue_freematch_session};
use super::history::SharedMatchHistoryRegistry;
use super::room::{SharedRoomManager, join_room_session};
use super::session::{ActivePlayerSession, SessionHandle};
use super::wait_human::SharedWaitHumanQueue;
use crate::game::PreferredColor;
use crate::protocol::{ClientCommand, ServerCommand, parse_client_command};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tracing::{info, warn};

pub async fn start_tcp_listener(
    port: u16,
    mode: TcpMode,
    freematch_queue: SharedFreeMatchQueue,
    room_mgr: SharedRoomManager,
    wait_human_queue: SharedWaitHumanQueue,
    active_matches: SharedActiveMatchRegistry,
    history_registry: SharedMatchHistoryRegistry,
) {
    let addr = format!("0.0.0.0:{port}");
    let listener = match TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(e) => {
            warn!("Failed to bind TCP listener on {addr}: {e}");
            return;
        }
    };

    info!("TCP Listener running on {addr}");

    loop {
        let (stream, peer_addr) = match listener.accept().await {
            Ok(res) => res,
            Err(_) => break,
        };

        let freematch = freematch_queue.clone();
        let rm_mgr = room_mgr.clone();
        let wt_human = wait_human_queue.clone();
        let mode = mode.clone();
        let matches = active_matches.clone();
        let history = history_registry.clone();

        tokio::spawn(async move {
            info!("Accepted TCP connection from {peer_addr} on port {port}");
            handle_tcp_client(stream, mode, freematch, rm_mgr, wt_human, matches, history).await;
        });
    }
}

#[derive(Clone)]
pub enum TcpMode {
    FreeMatch,
    Room { room_id: String },
    ClientWaitHuman,
    Ai { ai_type: AiType },
}

async fn handle_tcp_client(
    stream: TcpStream,
    mode: TcpMode,
    freematch_queue: SharedFreeMatchQueue,
    room_mgr: SharedRoomManager,
    wait_human_queue: SharedWaitHumanQueue,
    active_matches: SharedActiveMatchRegistry,
    history_registry: SharedMatchHistoryRegistry,
) {
    let (reader, mut writer) = stream.into_split();
    let mut buf_reader = BufReader::new(reader);

    let mut first_line = String::new();
    if buf_reader.read_line(&mut first_line).await.is_err() {
        return;
    }

    let client_cmd = match parse_client_command(&first_line) {
        Ok(cmd) => cmd,
        Err(e) => {
            warn!("Invalid TCP handshake: {first_line:?} ({e})");
            return;
        }
    };

    let player_name = match client_cmd {
        ClientCommand::Open { player_name } => player_name,
        _ => return,
    };

    info!("Player '{player_name}' connected via TCP");

    let (tx_server, mut rx_server) = tokio::sync::mpsc::unbounded_channel::<ServerCommand>();
    let (tx_client, rx_client) = tokio::sync::mpsc::unbounded_channel::<ClientCommand>();

    tokio::spawn(async move {
        while let Some(cmd) = rx_server.recv().await {
            let mut out = cmd.to_string();
            out.push('\n');
            if writer.write_all(out.as_bytes()).await.is_err() {
                break;
            }
        }
    });

    tokio::spawn(async move {
        let mut line = String::new();
        loop {
            line.clear();
            match buf_reader.read_line(&mut line).await {
                Ok(0) => break,
                Ok(_) => {
                    if let Ok(cmd) = parse_client_command(&line) {
                        if tx_client.send(cmd).is_err() {
                            break;
                        }
                    }
                }
                Err(_) => break,
            }
        }
    });

    let session_handle = SessionHandle::new(player_name, tx_server);
    let active_session = ActivePlayerSession {
        handle: session_handle,
        rx: rx_client,
    };

    match mode {
        TcpMode::FreeMatch => {
            enqueue_freematch_session(freematch_queue, active_session, 1000, PreferredColor::Random, active_matches, history_registry).await;
        }
        TcpMode::Room { room_id } => {
            join_room_session(room_mgr, room_id, active_session, 1000).await;
        }
        TcpMode::ClientWaitHuman => {
            let mut queue = wait_human_queue.lock().await;
            let id = queue.register(active_session, 1000);
            info!("TCP Client registered to wait for human match: ID {id}");
        }
        TcpMode::Ai { ai_type } => {
            start_ai_match(active_session, ai_type, 1000, PreferredColor::Random, active_matches, history_registry).await;
        }
    }
}
