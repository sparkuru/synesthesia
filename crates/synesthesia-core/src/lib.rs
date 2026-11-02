//! Platform-free configuration, subtitle state, scheduling and diagnostics.
pub mod cli;
pub mod config;
pub mod diagnostics;
pub mod domain;
pub mod pipeline;
pub mod session;

use serde::{Deserialize, Serialize};

pub const SAMPLE_RATE: u32 = 16_000;
pub const MAX_FRAME_SAMPLES: u32 = 3_200;
pub const MAX_AUDIO_SAMPLES: u64 = 32_000;
pub const MAX_TRANSLATIONS: usize = 4;
pub const MAX_EVENTS: usize = 64;
pub const MAX_TEXT_BYTES: usize = 8_192;
pub const MAX_HISTORY: usize = 20;

/// Stable, safe categories. Never embed foreign errors or user payloads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    InvalidConfig,
    UnsupportedVersion,
    WrongRole,
    InvalidLanguage,
    InvalidPath,
    InvalidToken,
    InvalidAddress,
    InvalidEvent,
    InvalidState,
    Overflow,
    Overloaded,
    InvalidProtocol,
    InvalidAudio,
    Discontinuity,
    Authentication,
    InferenceFailed,
    NotImplemented,
    StorageUnavailable,
    RecordTooLarge,
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for ErrorCode {}
