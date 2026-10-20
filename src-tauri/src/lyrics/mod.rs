//! Lyrics lookup via LRCLIB (https://lrclib.net), a free, open database of
//! time-synced lyrics that needs no API key.

mod lrc;
mod lrclib;

use serde::Serialize;

pub use lrclib::{fetch, http_client};

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Line {
    pub time_ms: u64,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum Lyrics {
    Loading,
    Synced {
        lines: Vec<Line>,
    },
    /// Only plain text is available, so it can't follow the song.
    Unsynced {
        text: String,
    },
    Instrumental,
    NotFound,
    Failed {
        message: String,
    },
}
