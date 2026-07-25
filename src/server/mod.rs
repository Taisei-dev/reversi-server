pub mod active_matches;
pub mod ai_match;
pub mod freematch;
pub mod history;
pub mod match_runner;
pub mod room;
pub mod session;
pub mod wait_human;
pub mod ws_bridge;

pub const DEFAULT_TIME_MS: i32 = 60_000;
pub const DEFAULT_COOLDOWN_SEC: u64 = 300;

