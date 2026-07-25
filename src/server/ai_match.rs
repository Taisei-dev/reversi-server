use super::active_matches::SharedActiveMatchRegistry;
use super::history::SharedMatchHistoryRegistry;
use super::match_runner::{Participant, run_match};
use super::session::ActivePlayerSession;
use crate::game::PreferredColor;
use tracing::info;

#[derive(Debug, Clone)]
pub enum AiType {
    Random,
    Egaroucid { level: i32, use_book: bool },
}

pub async fn start_ai_match(
    session: ActivePlayerSession,
    ai_type: AiType,
    assigned_time_ms: i32,
    pref_color: PreferredColor,
    active_matches: SharedActiveMatchRegistry,
    history_registry: SharedMatchHistoryRegistry,
) {
    let (ai_participant, ai_mode) = match ai_type {
        AiType::Random => (
            Participant::RandomAi {
                name: "AI_Random".to_string(),
            },
            "vs-AI (Random)",
        ),
        AiType::Egaroucid { level, use_book } => (
            Participant::EgaroucidAi {
                name: if use_book { format!("Egaroucid_L{level}") } else { format!("Egaroucid_L{level}_NoBook") },
                level,
                use_book,
            },
            "vs-AI (Egaroucid)",
        ),
    };

    let human_is_white = match pref_color {
        PreferredColor::White => true,
        PreferredColor::Black => false,
        PreferredColor::Random => rand::random(),
    };

    let human_name = session.handle.player_name.clone();
    info!(
        "Starting AI Match: Human '{}' ({}) vs AI '{}' ({})",
        human_name,
        if human_is_white { "White/後手" } else { "Black/先手" },
        ai_participant.name(),
        if human_is_white { "Black/先手" } else { "White/後手" }
    );

    tokio::spawn(async move {
        if human_is_white {
            run_match(
                ai_participant,
                Participant::Human(session),
                assigned_time_ms,
                ai_mode,
                active_matches,
                history_registry,
            )
            .await;
        } else {
            run_match(
                Participant::Human(session),
                ai_participant,
                assigned_time_ms,
                ai_mode,
                active_matches,
                history_registry,
            )
            .await;
        }
    });
}
