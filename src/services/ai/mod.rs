pub mod service;
pub mod types;
pub mod utils;

pub const TEMPERATURE: f64 = 0.7;
pub const SEED: u64 = 299792458;
pub const TOP_P: f64 = 0.8;

/// The context size to consider for the repeat penalty.
pub const REPEAT_LAST_N: usize = 128;

/// Penalty to apply for repeating tokens
pub const REPEAT_PENALTY: f32 = 1.1;

/// Only supported on Nvidia cards?
pub const USE_FLASH_ATTN: bool = false;

pub const MAX_TOKENS: usize = 10_000;

pub use service::AIService;
