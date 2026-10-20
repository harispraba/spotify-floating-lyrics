use std::collections::HashMap;

use anyhow::{Context, Result};
use zbus::blocking::{Connection, fdo::DBusProxy};
use zbus::zvariant::{OwnedValue, Value};

use super::{Playback, Track};

/// Bus name prefix used by the official client (`...spotify`) and spotifyd (`...spotifyd`).
const BUS_PREFIX: &str = "org.mpris.MediaPlayer2.spotify";
const OBJECT_PATH: &str = "/org/mpris/MediaPlayer2";
const PLAYER_INTERFACE: &str = "org.mpris.MediaPlayer2.Player";

pub struct SpotifySource {
    connection: Option<Connection>,
}

impl SpotifySource {
    pub fn new() -> Self {
        Self { connection: None }
    }

    pub fn poll(&mut self) -> Result<Option<Playback>> {
        if self.connection.is_none() {
            self.connection =
                Some(Connection::session().context("connecting to the D-Bus session bus")?);
        }
        let connection = self.connection.as_ref().unwrap();

        let names = DBusProxy::new(connection)?.list_names()?;
        let Some(name) = names.iter().find(|n| n.as_str().starts_with(BUS_PREFIX)) else {
            return Ok(None);
        };

        // Query the properties directly instead of through a caching proxy:
        // `Position` never emits change signals, so a cached value would go stale.
        let reply = connection.call_method(
            Some(name.as_str()),
            OBJECT_PATH,
            Some("org.freedesktop.DBus.Properties"),
            "GetAll",
            &PLAYER_INTERFACE,
        )?;
        let props: HashMap<String, OwnedValue> = reply.body().deserialize()?;
        Ok(parse(&props))
    }
}

fn parse(props: &HashMap<String, OwnedValue>) -> Option<Playback> {
    let prop = |key: &str| props.get(key).map(|v| unwrap_variant(v));
    let Some(Value::Dict(metadata)) = prop("Metadata") else {
        return None;
    };
    let field = |key: &str| {
        metadata
            .iter()
            .find(|(k, _)| matches!(k, Value::Str(s) if s.as_str() == key))
            .map(|(_, v)| unwrap_variant(v))
    };

    let title = field("xesam:title").and_then(as_string)?;
    if title.is_empty() {
        return None;
    }
    let artist = match field("xesam:artist") {
        Some(Value::Array(artists)) => artists
            .iter()
            .filter_map(|a| as_string(unwrap_variant(a)))
            .collect::<Vec<_>>()
            .join(", "),
        Some(other) => as_string(other).unwrap_or_default(),
        None => String::new(),
    };
    let album = field("xesam:album").and_then(as_string).unwrap_or_default();
    let duration_us = field("mpris:length").and_then(as_i64).unwrap_or(0);
    let position_us = prop("Position").and_then(as_i64).unwrap_or(0);
    let is_playing = prop("PlaybackStatus").and_then(as_string).as_deref() == Some("Playing");

    Some(Playback {
        track: Track {
            title,
            artist,
            album,
            duration_ms: (duration_us.max(0) / 1000) as u64,
        },
        position_ms: (position_us.max(0) / 1000) as u64,
        is_playing,
    })
}

/// Values inside `a{sv}` dictionaries arrive wrapped in a variant.
fn unwrap_variant<'a>(value: &'a Value<'a>) -> &'a Value<'a> {
    match value {
        Value::Value(inner) => unwrap_variant(inner),
        other => other,
    }
}

fn as_string(value: &Value) -> Option<String> {
    match value {
        Value::Str(s) => Some(s.to_string()),
        _ => None,
    }
}

/// Spotify has shipped `mpris:length` as both signed and unsigned integers over the years.
fn as_i64(value: &Value) -> Option<i64> {
    match *value {
        Value::I64(n) => Some(n),
        Value::U64(n) => i64::try_from(n).ok(),
        Value::I32(n) => Some(n.into()),
        Value::U32(n) => Some(n.into()),
        _ => None,
    }
}
