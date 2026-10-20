use std::process::Command;

use anyhow::{Result, bail};

use super::{Playback, Track};

/// Prints title, artist, album, duration (ms), position (ms) and state on
/// separate lines, or nothing when Spotify isn't running or is stopped.
/// Position is converted to an integer here so the output doesn't depend on
/// the user's locale decimal separator.
const SCRIPT: &str = r#"
if application "Spotify" is running then
    tell application "Spotify"
        if player state is stopped then return ""
        set t to current track
        return (name of t) & linefeed & (artist of t) & linefeed & (album of t) & linefeed & (duration of t) & linefeed & ((player position * 1000) as integer) & linefeed & (player state as string)
    end tell
end if
return ""
"#;

pub struct SpotifySource;

impl SpotifySource {
    pub fn new() -> Self {
        Self
    }

    pub fn poll(&mut self) -> Result<Option<Playback>> {
        let output = Command::new("osascript").args(["-e", SCRIPT]).output()?;
        if !output.status.success() {
            bail!(
                "osascript failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            );
        }
        Ok(parse(&String::from_utf8_lossy(&output.stdout)))
    }
}

fn parse(output: &str) -> Option<Playback> {
    let mut fields = output.trim_end_matches(['\r', '\n']).split('\n');
    let title = fields.next()?.to_string();
    if title.is_empty() {
        return None;
    }
    let artist = fields.next()?.to_string();
    let album = fields.next()?.to_string();
    let duration_ms = fields.next()?.trim().parse().unwrap_or(0);
    let position_ms = fields.next()?.trim().parse().unwrap_or(0);
    let is_playing = fields.next()?.trim() == "playing";

    Some(Playback {
        track: Track {
            title,
            artist,
            album,
            duration_ms,
        },
        position_ms,
        is_playing,
    })
}
