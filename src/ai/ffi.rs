use std::os::raw::{c_char, c_double, c_int};

#[repr(C)]
pub struct EgaroucidEngine {
    _private: [u8; 0],
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct EgaroucidSearchOptions {
    pub size: u32,
    pub level: c_int,
    pub use_book: c_int,
    pub book_accuracy_level: c_int,
    pub use_multi_thread: c_int,
    pub show_log: c_int,
    pub time_limit_ms: c_int,
}

impl Default for EgaroucidSearchOptions {
    fn default() -> Self {
        Self {
            size: std::mem::size_of::<Self>() as u32,
            level: 3,
            use_book: 0,
            book_accuracy_level: 1,
            use_multi_thread: 1,
            show_log: 0,
            time_limit_ms: 1000,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct EgaroucidSearchResult {
    pub size: u32,
    pub r#move: c_int,
    pub value: c_int,
    pub depth: c_int,
    pub nodes: u64,
    pub nps: c_double,
    pub is_end_search: c_int,
}

impl Default for EgaroucidSearchResult {
    fn default() -> Self {
        Self {
            size: std::mem::size_of::<Self>() as u32,
            r#move: -1,
            value: 0,
            depth: 0,
            nodes: 0,
            nps: 0.0,
            is_end_search: 0,
        }
    }
}

extern "C" {
    pub fn egaroucid_global_init(resource_dir: *const c_char) -> c_int;
    pub fn egaroucid_create() -> *mut EgaroucidEngine;
    pub fn egaroucid_destroy(engine: *mut EgaroucidEngine);
    pub fn egaroucid_search_array(
        engine: *mut EgaroucidEngine,
        board: *const c_int,
        player: c_int,
        options: *const EgaroucidSearchOptions,
        result: *mut EgaroucidSearchResult,
    ) -> c_int;
}
