use super::active_matches::SharedActiveMatchRegistry;
use super::history::{FinishedMatchInfo, SharedMatchHistoryRegistry};
use super::session::ActivePlayerSession;
use crate::ai::{EgaroucidAi, decide_random_move};
use crate::game::{Board, Color, Move};
use crate::protocol::{ClientCommand, ServerCommand, Wl};
use std::time::{Duration, Instant};
use tokio::time::timeout;
use tracing::{info, warn};

pub enum Participant {
    Human(ActivePlayerSession),
    RandomAi { name: String },
    EgaroucidAi { name: String, level: i32, use_book: bool },
}

impl Participant {
    pub fn name(&self) -> &str {
        match self {
            Self::Human(sess) => &sess.handle.player_name,
            Self::RandomAi { name } => name,
            Self::EgaroucidAi { name, .. } => name,
        }
    }

    // AI または Web プレイヤーであるか (時間の減算・タイムアウトを適用しない)
    pub fn is_unlimited_time(&self) -> bool {
        match self {
            Self::RandomAi { .. } | Self::EgaroucidAi { .. } => true,
            Self::Human(sess) => sess.handle.player_name.starts_with("Player_Web"),
        }
    }
}

pub struct MatchResult {
    pub black_name: String,
    pub white_name: String,
    pub winner_color: Option<Color>,
    pub black_stones: u32,
    pub white_stones: u32,
    pub reason: String,
    pub moves: Vec<String>,
}

pub async fn run_match(
    mut player_black: Participant,
    mut player_white: Participant,
    assigned_time_ms: i32,
    mode_name: &str,
    active_matches: SharedActiveMatchRegistry,
    history_registry: SharedMatchHistoryRegistry,
) -> (MatchResult, Participant, Participant) {
    let black_name = player_black.name().to_string();
    let white_name = player_white.name().to_string();

    info!("Starting match: {black_name} (Black) vs {white_name} (White) (Time limit: {assigned_time_ms}ms)");

    // クライアントが対戦終了後に新しい対戦へ参加するまでに 500ms クールダウンを確保し、遅延パケットが到達し切るのを待つ
    tokio::time::sleep(Duration::from_millis(500)).await;

    // 遅れて届いた前対局の余剰メッセージ (MOVE PASS等) をドレイン/クリアして認識のズレを防止
    if let Participant::Human(sess) = &mut player_black {
        while sess.rx.try_recv().is_ok() {}
    }
    if let Participant::Human(sess) = &mut player_white {
        while sess.rx.try_recv().is_ok() {}
    }

    let match_id = {
        let mut lock = active_matches.lock().await;
        lock.register(mode_name, &black_name, &white_name)
    };

    // 初期持ち時間 (ms)
    let mut black_remaining_ms = assigned_time_ms.max(100);
    let mut white_remaining_ms = assigned_time_ms.max(100);

    // START 送信と同時に「START送出後の思考タイマー」を起動！
    let mut black_timer_start = Instant::now();
    let mut white_timer_start = Instant::now();

    if let Participant::Human(h) = &player_black {
        let time_to_send = if player_black.is_unlimited_time() { 86400000 } else { black_remaining_ms };
        let _ = h.handle.send(ServerCommand::Start {
            color: Color::Black,
            opponent_name: white_name.clone(),
            assigned_time_ms: time_to_send,
        });
        black_timer_start = Instant::now();
    }
    if let Participant::Human(h) = &player_white {
        let time_to_send = if player_white.is_unlimited_time() { 86400000 } else { white_remaining_ms };
        let _ = h.handle.send(ServerCommand::Start {
            color: Color::White,
            opponent_name: black_name.clone(),
            assigned_time_ms: time_to_send,
        });
        white_timer_start = Instant::now();
    }

    let mut board = Board::new();
    let mut current_turn = Color::Black;
    let mut consecutive_passes = 0;
    let mut recorded_moves: Vec<String> = Vec::new();

    loop {
        if board.is_game_over() || consecutive_passes >= 2 {
            info!("Game ended naturally or by passes");
            break;
        }

        let valid_moves = board.valid_moves(current_turn);

        let current_remaining_ms = if current_turn == Color::Black {
            black_remaining_ms
        } else {
            white_remaining_ms
        };

        let active_timer_start = if current_turn == Color::Black {
            black_timer_start
        } else {
            white_timer_start
        };

        let active_participant = if current_turn == Color::Black {
            &mut player_black
        } else {
            &mut player_white
        };

        let is_unlimited = active_participant.is_unlimited_time();
        // AIや人間Webプレイヤーは24時間無制限、プログラムClientは残り持ち時間でタイムアウト
        let timeout_duration = if is_unlimited {
            Duration::from_secs(86400)
        } else {
            Duration::from_millis(current_remaining_ms.max(1) as u64)
        };

        // クライアントからの MOVE 受信または AI 思考
        let chosen_move = match active_participant {
            Participant::Human(sess) => {
                let recv_res = timeout(timeout_duration, sess.rx.recv()).await;
                match recv_res {
                    Ok(Some(ClientCommand::Move(m))) => m,
                    Ok(Some(ClientCommand::Open { .. })) => {
                        warn!("Unexpected OPEN during match");
                        Move::GiveUp
                    }
                    Ok(None) => {
                        warn!("Player disconnected during match: {}", sess.handle.player_name);
                        Move::GiveUp
                    }
                    Err(_) => {
                        warn!("Player timed out: {}", sess.handle.player_name);
                        Move::GiveUp
                    }
                }
            }
            Participant::RandomAi { .. } => {
                tokio::time::sleep(Duration::from_millis(150)).await;
                decide_random_move(&board, current_turn)
            }
            Participant::EgaroucidAi { level, use_book, .. } => {
                let ai = EgaroucidAi::new(*level, *use_book, 2000);
                ai.decide_move(&board, current_turn)
            }
        };

        // プログラム Client の場合のみ実測経過時間を引き算
        let new_remaining_ms = if is_unlimited {
            current_remaining_ms
        } else {
            let elapsed_ms = active_timer_start.elapsed().as_millis() as i32;
            let actual_cost_ms = elapsed_ms.max(1);
            (current_remaining_ms - actual_cost_ms).max(0)
        };

        if current_turn == Color::Black {
            black_remaining_ms = new_remaining_ms;
        } else {
            white_remaining_ms = new_remaining_ms;
        }

        recorded_moves.push(chosen_move.to_string());

        // GIVEUP または タイムアウト判定 (AI や人間Webプレイヤーはタイムアウト除外)
        let is_timed_out = !is_unlimited && new_remaining_ms <= 0;
        if chosen_move == Move::GiveUp || is_timed_out {
            let (b_stones, w_stones) = board.count_stones();
            let winner = current_turn.opposite();
            let loser_name = if current_turn == Color::Black { &black_name } else { &white_name };
            let reason = if is_timed_out {
                format!("{loser_name}_time_out")
            } else {
                format!("{loser_name}_gave_up_or_disconnected")
            };

            notify_end(&player_black, &player_white, Some(winner), b_stones, w_stones, &reason);

            {
                let mut lock = active_matches.lock().await;
                lock.unregister(&match_id);
            }

            let result = MatchResult {
                black_name: black_name.clone(),
                white_name: white_name.clone(),
                winner_color: Some(winner),
                black_stones: b_stones,
                white_stones: w_stones,
                reason: reason.clone(),
                moves: recorded_moves.clone(),
            };

            record_history(&history_registry, &match_id, mode_name, &result).await;
            return (result, player_black, player_white);
        }

        // パス判定
        if valid_moves.is_empty() {
            consecutive_passes += 1;
            let next_start = notify_move_and_ack(&player_black, &player_white, current_turn, Move::Pass, new_remaining_ms);
            if current_turn == Color::Black {
                white_timer_start = next_start;
            } else {
                black_timer_start = next_start;
            }
            current_turn = current_turn.opposite();
            continue;
        }

        // バリデーション
        if !board.apply_move(chosen_move, current_turn) {
            warn!("Invalid move by {current_turn:?}: {chosen_move}");
            let (b_stones, w_stones) = board.count_stones();
            let winner = current_turn.opposite();
            let reason = format!("Invalid_move_{chosen_move}");

            notify_end(&player_black, &player_white, Some(winner), b_stones, w_stones, &reason);

            {
                let mut lock = active_matches.lock().await;
                lock.unregister(&match_id);
            }

            let result = MatchResult {
                black_name: black_name.clone(),
                white_name: white_name.clone(),
                winner_color: Some(winner),
                black_stones: b_stones,
                white_stones: w_stones,
                reason: reason.clone(),
                moves: recorded_moves.clone(),
            };

            record_history(&history_registry, &match_id, mode_name, &result).await;
            return (result, player_black, player_white);
        }

        {
            let mut lock = active_matches.lock().await;
            lock.update_move(&match_id);
        }

        consecutive_passes = 0;

        let next_start = notify_move_and_ack(&player_black, &player_white, current_turn, chosen_move, new_remaining_ms);
        if current_turn == Color::Black {
            white_timer_start = next_start;
        } else {
            black_timer_start = next_start;
        }

        current_turn = current_turn.opposite();
    }

    let (b_stones, w_stones) = board.count_stones();
    let winner = if b_stones > w_stones {
        Some(Color::Black)
    } else if w_stones > b_stones {
        Some(Color::White)
    } else {
        None
    };

    notify_end(&player_black, &player_white, winner, b_stones, w_stones, "OK");

    {
        let mut lock = active_matches.lock().await;
        lock.unregister(&match_id);
    }

    let result = MatchResult {
        black_name: black_name.clone(),
        white_name: white_name.clone(),
        winner_color: winner,
        black_stones: b_stones,
        white_stones: w_stones,
        reason: "OK".to_string(),
        moves: recorded_moves,
    };

    record_history(&history_registry, &match_id, mode_name, &result).await;
    (result, player_black, player_white)
}

async fn record_history(
    history_registry: &SharedMatchHistoryRegistry,
    match_id: &str,
    mode_name: &str,
    res: &MatchResult,
) {
    let finished_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let info = FinishedMatchInfo {
        match_id: match_id.to_string(),
        mode: mode_name.to_string(),
        black_name: res.black_name.clone(),
        white_name: res.white_name.clone(),
        winner_color: res.winner_color.map(|c| c.to_string()),
        black_stones: res.black_stones,
        white_stones: res.white_stones,
        reason: res.reason.clone(),
        moves: res.moves.clone(),
        finished_at_sec: finished_at,
    };

    let mut lock = history_registry.lock().await;
    lock.record_match(info);
}

fn notify_move_and_ack(
    p_black: &Participant,
    p_white: &Participant,
    turn: Color,
    m: Move,
    remaining_time_ms: i32,
) -> Instant {
    let (active, opponent) = if turn == Color::Black {
        (p_black, p_white)
    } else {
        (p_white, p_black)
    };

    if let Participant::Human(h) = active {
        let time_to_send = if active.is_unlimited_time() { 86400000 } else { remaining_time_ms };
        let _ = h.handle.send(ServerCommand::Ack {
            assigned_time_ms: time_to_send,
        });
    }
    if let Participant::Human(h) = opponent {
        let _ = h.handle.send(ServerCommand::Move(m));
    }

    Instant::now()
}

fn notify_end(
    p_black: &Participant,
    p_white: &Participant,
    winner: Option<Color>,
    b_stones: u32,
    w_stones: u32,
    reason: &str,
) {
    if let Participant::Human(h) = p_black {
        let res = match winner {
            Some(Color::Black) => Wl::Win,
            Some(Color::White) => Wl::Lose,
            _ => Wl::Tie,
        };
        let _ = h.handle.send(ServerCommand::End {
            result: res,
            your_stone_count: b_stones,
            opponent_stone_count: w_stones,
            reason: reason.to_string(),
        });
    }

    if let Participant::Human(h) = p_white {
        let res = match winner {
            Some(Color::White) => Wl::Win,
            Some(Color::Black) => Wl::Lose,
            _ => Wl::Tie,
        };
        let _ = h.handle.send(ServerCommand::End {
            result: res,
            your_stone_count: w_stones,
            opponent_stone_count: b_stones,
            reason: reason.to_string(),
        });
    }
}
