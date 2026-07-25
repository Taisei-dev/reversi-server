use crate::game::{Color, Move};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Wl {
    Win,
    Lose,
    Tie,
}

impl fmt::Display for Wl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Win => write!(f, "WIN"),
            Self::Lose => write!(f, "LOSE"),
            Self::Tie => write!(f, "TIE"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerStat {
    pub player_name: String,
    pub score: i32,
    pub wins: u32,
    pub loses: u32,
}

impl fmt::Display for PlayerStat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {} {}",
            self.player_name, self.score, self.wins, self.loses
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClientCommand {
    Open { player_name: String },
    Move(Move),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServerCommand {
    Start {
        color: Color,
        opponent_name: String,
        assigned_time_ms: i32,
    },
    Move(Move),
    Ack {
        assigned_time_ms: i32,
    },
    End {
        result: Wl,
        your_stone_count: u32,
        opponent_stone_count: u32,
        reason: String,
    },
    Bye {
        stats: Vec<PlayerStat>,
    },
}

impl fmt::Display for ServerCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Start {
                color,
                opponent_name,
                assigned_time_ms,
            } => {
                write!(f, "START {color} {opponent_name} {assigned_time_ms}")
            }
            Self::Move(m) => write!(f, "MOVE {m}"),
            Self::Ack { assigned_time_ms } => write!(f, "ACK {assigned_time_ms}"),
            Self::End {
                result,
                your_stone_count,
                opponent_stone_count,
                reason,
            } => {
                let reason_str = if reason.is_empty() {
                    String::from("OK")
                } else {
                    reason.replace(' ', "_")
                };
                write!(
                    f,
                    "END {result} {your_stone_count} {opponent_stone_count} {reason_str}"
                )
            }
            Self::Bye { stats } => {
                let formatted_stats: Vec<String> = stats
                    .iter()
                    .map(|s| format!("{} {} {} {}", s.player_name, s.score, s.wins, s.loses))
                    .collect();
                write!(f, "BYE {}", formatted_stats.join(" "))
            }
        }
    }
}
