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
        let (_res, p1, p2) = if human_is_white {
            run_match(
                ai_participant,
                Participant::Human(session),
                assigned_time_ms,
                ai_mode,
                active_matches,
                history_registry,
            )
            .await
        } else {
            run_match(
                Participant::Human(session),
                ai_participant,
                assigned_time_ms,
                ai_mode,
                active_matches,
                history_registry,
            )
            .await
        };

        let human_participant = if human_is_white { p2 } else { p1 };
        if let Participant::Human(h) = human_participant {
            let human_color = if human_is_white { crate::game::Color::White } else { crate::game::Color::Black };
            let ai_color = human_color.opposite();
            let human_stones = if human_is_white { _res.white_stones } else { _res.black_stones };
            let wins = if _res.winner_color == Some(human_color) { 1 } else { 0 };
            let loses = if _res.winner_color == Some(ai_color) { 1 } else { 0 };

            let stat = crate::protocol::PlayerStat {
                player_name: h.handle.player_name.clone(),
                score: human_stones as i32,
                wins,
                loses,
            };
            let _ = h.handle.send(crate::protocol::ServerCommand::Bye { stats: vec![stat] });
        }
    });
}
