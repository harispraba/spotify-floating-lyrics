//! Reads what Spotify is currently playing from the operating system's media
//! integration, so no Spotify login or API key is needed:
//!
//! - Windows: System Media Transport Controls (SMTC)
//! - Linux: MPRIS over D-Bus
//! - macOS: Spotify's AppleScript dictionary

use serde::Serialize;

#[cfg(target_os = "macos")]
mod applescript;
#[cfg(target_os = "linux")]
mod mpris;
#[cfg(windows)]
mod smtc;

#[cfg(target_os = "macos")]
pub use applescript::SpotifySource;
#[cfg(target_os = "linux")]
pub use mpris::SpotifySource;
#[cfg(windows)]
pub use smtc::SpotifySource;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Track {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration_ms: u64,
}

impl Track {
    /// Whether both refer to the same song. Duration is ignored because some
    /// platforms report it a moment after the title changes.
    pub fn same_song(&self, other: &Track) -> bool {
        self.title == other.title && self.artist == other.artist && self.album == other.album
    }

    pub fn cache_key(&self) -> String {
        format!("{}\u{1f}{}\u{1f}{}", self.title, self.artist, self.album)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Playback {
    pub track: Track,
    /// Playback position at the moment this snapshot was taken.
    pub position_ms: u64,
    pub is_playing: bool,
}
