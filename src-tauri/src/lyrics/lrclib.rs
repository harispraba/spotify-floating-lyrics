use std::time::Duration;

use anyhow::Result;
use reqwest::{Client, StatusCode};
use serde::Deserialize;
use serde::de::DeserializeOwned;

use super::{Lyrics, lrc};
use crate::player::Track;

const API_BASE: &str = "https://lrclib.net/api";
/// LRCLIB asks clients to identify themselves.
const USER_AGENT: &str = concat!("spotify-floating-lyrics/", env!("CARGO_PKG_VERSION"));
const RETRY_DELAY: Duration = Duration::from_millis(800);
/// Search results further off than this are most likely a different recording.
const MAX_DURATION_DIFF_MS: u64 = 4_000;

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Record {
    /// Seconds.
    #[serde(default)]
    duration: f64,
    #[serde(default)]
    instrumental: bool,
    plain_lyrics: Option<String>,
    synced_lyrics: Option<String>,
}

impl Record {
    fn synced(&self) -> Option<&str> {
        self.synced_lyrics
            .as_deref()
            .filter(|s| !s.trim().is_empty())
    }

    fn plain(&self) -> Option<&str> {
        self.plain_lyrics
            .as_deref()
            .filter(|s| !s.trim().is_empty())
    }

    fn into_lyrics(self) -> Lyrics {
        if let Some(lines) = self
            .synced()
            .map(lrc::parse)
            .filter(|lines| !lines.is_empty())
        {
            Lyrics::Synced { lines }
        } else if self.instrumental {
            Lyrics::Instrumental
        } else if let Some(text) = self.plain() {
            Lyrics::Unsynced {
                text: text.to_string(),
            }
        } else {
            Lyrics::NotFound
        }
    }
}

pub fn http_client() -> reqwest::Result<Client> {
    Client::builder()
        .user_agent(USER_AGENT)
        .timeout(Duration::from_secs(10))
        .build()
}

/// Looks up lyrics for `track`, preferring time-synced lyrics.
pub async fn fetch(http: &Client, track: &Track) -> Result<Lyrics> {
    // The exact-signature endpoint is fast but needs title/artist/album/duration
    // to match. It's only a shortcut, so errors (LRCLIB sometimes answers 503)
    // fall through to search.
    let mut exact = get_exact(http, track).await.unwrap_or_default();
    if let Some(record) = exact.take_if(|r| r.synced().is_some() || r.instrumental) {
        return Ok(record.into_lyrics());
    }

    // Fall back to fuzzy search with a simplified title, e.g. without "- Remastered 2011".
    let title = clean_title(&track.title);
    let artist = primary_artist(&track.artist);
    let mut best = pick_best(
        search(http, &[("track_name", &title), ("artist_name", artist)]).await?,
        track.duration_ms,
    );
    if best.is_none() {
        best = pick_best(
            search(http, &[("q", &format!("{title} {artist}"))]).await?,
            track.duration_ms,
        );
    }

    // A synced search result beats plain lyrics from the exact match.
    Ok(match (best, exact) {
        (Some(best), _) if best.synced().is_some() => best.into_lyrics(),
        (_, Some(exact)) => exact.into_lyrics(),
        (Some(best), None) => best.into_lyrics(),
        (None, None) => Lyrics::NotFound,
    })
}

async fn get_exact(http: &Client, track: &Track) -> Result<Option<Record>> {
    if track.duration_ms == 0 {
        return Ok(None);
    }
    let duration = ((track.duration_ms + 500) / 1000).to_string();
    let query = [
        ("track_name", track.title.as_str()),
        ("artist_name", &track.artist),
        ("album_name", &track.album),
        ("duration", &duration),
    ];
    get(http, "get", &query).await
}

async fn search(http: &Client, query: &[(&str, &str)]) -> Result<Vec<Record>> {
    Ok(get(http, "search", query).await?.unwrap_or_default())
}

/// GETs an LRCLIB endpoint, returning `None` on 404. LRCLIB sometimes answers
/// 503 for a moment, so server errors are retried once.
async fn get<T: DeserializeOwned>(
    http: &Client,
    endpoint: &str,
    query: &[(&str, &str)],
) -> Result<Option<T>> {
    let send = || {
        http.get(format!("{API_BASE}/{endpoint}"))
            .query(query)
            .send()
    };
    let mut response = send().await?;
    if response.status().is_server_error() {
        tokio::time::sleep(RETRY_DELAY).await;
        response = send().await?;
    }
    if response.status() == StatusCode::NOT_FOUND {
        return Ok(None);
    }
    Ok(Some(response.error_for_status()?.json().await?))
}

/// Picks the closest-length result that has lyrics, preferring synced ones.
fn pick_best(records: Vec<Record>, duration_ms: u64) -> Option<Record> {
    let diff = |r: &Record| {
        if duration_ms == 0 {
            0
        } else {
            (r.duration * 1000.0 - duration_ms as f64).abs() as u64
        }
    };
    records
        .into_iter()
        .filter(|r| diff(r) <= MAX_DURATION_DIFF_MS)
        .filter(|r| r.synced().is_some() || r.instrumental || r.plain().is_some())
        .min_by_key(|r| (r.synced().is_none(), diff(r)))
}

/// Drops version suffixes and featured artists that LRCLIB entries usually
/// don't have: "Song - Remastered 2011" and "Song (feat. X)" become "Song".
fn clean_title(title: &str) -> String {
    let mut out = title.split(" - ").next().unwrap_or(title).to_string();
    for (open, close) in [('(', ')'), ('[', ']')] {
        let mut from = 0;
        while let Some(start) = out[from..].find(open).map(|i| i + from) {
            let Some(len) = out[start..].find(close) else {
                break;
            };
            let inner = out[start + 1..start + len].trim_start().to_lowercase();
            if inner.starts_with("feat") || inner.starts_with("ft.") || inner.starts_with("with ") {
                out.replace_range(start..=start + len, "");
            } else {
                from = start + 1;
            }
        }
    }
    let cleaned = out.split_whitespace().collect::<Vec<_>>().join(" ");
    if cleaned.is_empty() {
        title.trim().to_string()
    } else {
        cleaned
    }
}

/// "Artist A, Artist B" -> "Artist A".
fn primary_artist(artist: &str) -> &str {
    artist.split(", ").next().unwrap_or(artist).trim()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(duration: f64, synced: Option<&str>, plain: Option<&str>) -> Record {
        Record {
            duration,
            synced_lyrics: synced.map(str::to_string),
            plain_lyrics: plain.map(str::to_string),
            ..Default::default()
        }
    }

    #[test]
    fn cleans_titles() {
        assert_eq!(clean_title("Yesterday - Remastered 2009"), "Yesterday");
        assert_eq!(clean_title("Song (feat. Someone)"), "Song");
        assert_eq!(clean_title("Song [with Someone] (Live)"), "Song (Live)");
        assert_eq!(
            clean_title("(I Can't Get No) Satisfaction"),
            "(I Can't Get No) Satisfaction"
        );
        assert_eq!(clean_title(" - Intro"), "- Intro");
    }

    #[test]
    fn takes_first_artist() {
        assert_eq!(primary_artist("Daft Punk, Pharrell Williams"), "Daft Punk");
        assert_eq!(primary_artist("Simon & Garfunkel"), "Simon & Garfunkel");
    }

    #[test]
    fn prefers_synced_then_closest_duration() {
        let records = vec![
            record(200.0, None, Some("plain, exact length")),
            record(203.0, Some("[00:01.00]synced, 3s off"), None),
            record(201.0, Some("[00:01.00]synced, 1s off"), None),
            record(260.0, Some("[00:01.00]other version"), None),
        ];
        let best = pick_best(records, 200_000).unwrap();
        assert_eq!(best.synced(), Some("[00:01.00]synced, 1s off"));
    }

    #[test]
    fn rejects_results_without_lyrics_or_with_wrong_length() {
        let records = vec![
            record(200.0, None, None),
            record(300.0, Some("[00:01.00]x"), None),
        ];
        assert!(pick_best(records, 200_000).is_none());
    }

    #[test]
    fn converts_records_to_lyrics() {
        assert!(matches!(
            record(1.0, Some("[00:01.00]hi"), None).into_lyrics(),
            Lyrics::Synced { .. }
        ));
        assert!(matches!(
            record(1.0, Some("  "), Some("hi")).into_lyrics(),
            Lyrics::Unsynced { .. }
        ));
        let instrumental = Record {
            instrumental: true,
            ..Default::default()
        };
        assert_eq!(instrumental.into_lyrics(), Lyrics::Instrumental);
        assert_eq!(record(1.0, None, None).into_lyrics(), Lyrics::NotFound);
    }
}
