use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use windows::Media::Control::{
    GlobalSystemMediaTransportControlsSession as Session,
    GlobalSystemMediaTransportControlsSessionManager as SessionManager,
    GlobalSystemMediaTransportControlsSessionPlaybackStatus as PlaybackStatus,
};

use super::{Playback, Track};

/// WinRT `DateTime` counts 100ns ticks since 1601-01-01; this is the Unix epoch in those ticks.
const UNIX_EPOCH_TICKS: i64 = 116_444_736_000_000_000;

pub struct SpotifySource {
    manager: Option<SessionManager>,
}

impl SpotifySource {
    pub fn new() -> Self {
        Self { manager: None }
    }

    pub fn poll(&mut self) -> Result<Option<Playback>> {
        let manager = match &self.manager {
            Some(manager) => manager.clone(),
            None => {
                let manager = SessionManager::RequestAsync()?.join()?;
                self.manager = Some(manager.clone());
                manager
            }
        };
        match find_spotify(&manager)? {
            Some(session) => read(&session),
            None => Ok(None),
        }
    }
}

fn find_spotify(manager: &SessionManager) -> Result<Option<Session>> {
    for session in manager.GetSessions()? {
        // "Spotify.exe" for the regular installer, "SpotifyAB.SpotifyMusic_...!Spotify" for the Store app.
        let app_id = session.SourceAppUserModelId()?.to_string_lossy();
        if app_id.to_ascii_lowercase().contains("spotify") {
            return Ok(Some(session));
        }
    }
    Ok(None)
}

fn read(session: &Session) -> Result<Option<Playback>> {
    let props = session.TryGetMediaPropertiesAsync()?.join()?;
    let title = props.Title()?.to_string_lossy();
    if title.is_empty() {
        return Ok(None);
    }

    let timeline = session.GetTimelineProperties()?;
    let is_playing = session.GetPlaybackInfo()?.PlaybackStatus()? == PlaybackStatus::Playing;
    let duration_ms = ticks_to_ms(timeline.EndTime()?.Duration);
    let mut position_ms = ticks_to_ms(timeline.Position()?.Duration);
    if is_playing {
        // Spotify only refreshes the timeline on play/pause/seek, so advance it
        // by the time elapsed since that last update.
        let updated_ms = (timeline.LastUpdatedTime()?.UniversalTime - UNIX_EPOCH_TICKS) / 10_000;
        position_ms += now_ms().saturating_sub(updated_ms.max(0) as u64);
    }
    if duration_ms > 0 {
        position_ms = position_ms.min(duration_ms);
    }

    Ok(Some(Playback {
        track: Track {
            title,
            artist: props.Artist()?.to_string_lossy(),
            album: props.AlbumTitle()?.to_string_lossy(),
            duration_ms,
        },
        position_ms,
        is_playing,
    }))
}

fn ticks_to_ms(ticks: i64) -> u64 {
    (ticks.max(0) / 10_000) as u64
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_millis() as u64)
}
