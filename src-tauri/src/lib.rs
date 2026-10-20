mod lyrics;
mod player;

use std::collections::HashMap;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use serde::Serialize;
use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Emitter, Manager, State, Wry};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};
use tauri_plugin_window_state::StateFlags;

use lyrics::Lyrics;
use player::{Playback, SpotifySource, Track};

const POLL_INTERVAL: Duration = Duration::from_millis(500);
const LOCK_SHORTCUT: &str = "CommandOrControl+Alt+Shift+L";
const MAIN_WINDOW: &str = "main";

#[derive(Default)]
struct AppState {
    playback: Option<Playback>,
    lyrics: Option<TrackLyrics>,
    cache: HashMap<String, Lyrics>,
    locked: bool,
}

type SharedState = Mutex<AppState>;

#[derive(Debug, Clone, Serialize)]
struct TrackLyrics {
    track: Track,
    lyrics: Lyrics,
}

#[derive(Serialize)]
struct Snapshot {
    playback: Option<Playback>,
    lyrics: Option<TrackLyrics>,
    locked: bool,
}

struct TrayItems {
    visible: CheckMenuItem<Wry>,
    locked: CheckMenuItem<Wry>,
}

#[tauri::command]
fn snapshot(state: State<SharedState>) -> Snapshot {
    let state = state.lock().unwrap();
    Snapshot {
        playback: state.playback.clone(),
        lyrics: state.lyrics.clone(),
        locked: state.locked,
    }
}

#[tauri::command]
fn set_locked(app: AppHandle, locked: bool) -> Result<(), String> {
    apply_locked(&app, locked).map_err(|e| e.to_string())
}

#[tauri::command]
fn hide_window(app: AppHandle) -> Result<(), String> {
    apply_visible(&app, false).map_err(|e| e.to_string())
}

#[tauri::command]
fn reload_lyrics(app: AppHandle, state: State<SharedState>) {
    let mut state = state.lock().unwrap();
    if let Some(track) = state.playback.as_ref().map(|p| p.track.clone()) {
        state.cache.remove(&track.cache_key());
        load_lyrics(&app, &mut state, track);
    }
}

/// Click-through mode: the window ignores the mouse so it never gets in the way.
fn apply_locked(app: &AppHandle, locked: bool) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window(MAIN_WINDOW) {
        window.set_ignore_cursor_events(locked)?;
    }
    app.state::<SharedState>().lock().unwrap().locked = locked;
    app.state::<TrayItems>().locked.set_checked(locked)?;
    app.emit("locked", locked)
}

fn apply_visible(app: &AppHandle, visible: bool) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window(MAIN_WINDOW) {
        if visible {
            window.show()?;
        } else {
            window.hide()?;
        }
    }
    app.state::<TrayItems>().visible.set_checked(visible)
}

fn toggle_locked(app: &AppHandle) {
    let locked = app.state::<SharedState>().lock().unwrap().locked;
    if let Err(err) = apply_locked(app, !locked) {
        eprintln!("failed to toggle lock: {err}");
    }
}

fn toggle_visible(app: &AppHandle) {
    let visible = app
        .get_webview_window(MAIN_WINDOW)
        .and_then(|w| w.is_visible().ok())
        .unwrap_or(false);
    if let Err(err) = apply_visible(app, !visible) {
        eprintln!("failed to toggle visibility: {err}");
    }
}

fn setup_tray(app: &AppHandle) -> tauri::Result<()> {
    let visible = CheckMenuItem::with_id(app, "visible", "Show lyrics", true, true, None::<&str>)?;
    let locked = CheckMenuItem::with_id(
        app,
        "locked",
        "Lock (click-through)",
        true,
        false,
        Some(LOCK_SHORTCUT),
    )?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(
        app,
        &[
            &visible,
            &locked,
            &PredefinedMenuItem::separator(app)?,
            &quit,
        ],
    )?;

    let mut tray = TrayIconBuilder::with_id("main")
        .tooltip("Spotify Floating Lyrics")
        .menu(&menu)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "visible" => toggle_visible(app),
            "locked" => toggle_locked(app),
            "quit" => app.exit(0),
            _ => {}
        });
    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }
    tray.build(app)?;

    app.manage(TrayItems { visible, locked });
    Ok(())
}

/// Polls Spotify on a background thread and pushes changes to the window.
fn spawn_player_poller(app: AppHandle) {
    thread::spawn(move || {
        let mut source = SpotifySource::new();
        let mut last_error = None;
        let mut pending: Option<Track> = None;
        loop {
            let playback = match source.poll() {
                Ok(playback) => {
                    last_error = None;
                    playback
                }
                Err(err) => {
                    let message = format!("{err:#}");
                    if last_error.as_ref() != Some(&message) {
                        eprintln!("failed to read Spotify playback: {message}");
                        last_error = Some(message);
                    }
                    None
                }
            };
            on_playback(&app, playback, &mut pending);
            thread::sleep(POLL_INTERVAL);
        }
    });
}

fn on_playback(app: &AppHandle, playback: Option<Playback>, pending: &mut Option<Track>) {
    let state = app.state::<SharedState>();
    let mut state = state.lock().unwrap();

    if let Some(track) = playback.as_ref().map(|p| &p.track) {
        let has_lyrics = state
            .lyrics
            .as_ref()
            .is_some_and(|l| l.track.same_song(track));
        if has_lyrics {
            *pending = None;
        } else if pending.as_ref().is_some_and(|p| p.same_song(track)) {
            // Only look up a song once it's been seen on two polls in a row, so
            // its duration has settled and skipping through songs stays cheap.
            *pending = None;
            load_lyrics(app, &mut state, track.clone());
        } else {
            *pending = Some(track.clone());
        }
    }

    state.playback = playback.clone();
    drop(state);
    let _ = app.emit("playback", playback);
}

fn load_lyrics(app: &AppHandle, state: &mut AppState, track: Track) {
    if let Some(cached) = state.cache.get(&track.cache_key()).cloned() {
        set_lyrics(
            app,
            state,
            TrackLyrics {
                track,
                lyrics: cached,
            },
        );
        return;
    }
    set_lyrics(
        app,
        state,
        TrackLyrics {
            track: track.clone(),
            lyrics: Lyrics::Loading,
        },
    );

    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        let http = app.state::<reqwest::Client>();
        let lyrics = lyrics::fetch(&http, &track)
            .await
            .unwrap_or_else(|err| Lyrics::Failed {
                message: format!("{err:#}"),
            });

        let state = app.state::<SharedState>();
        let mut state = state.lock().unwrap();
        if !matches!(lyrics, Lyrics::Failed { .. }) {
            state.cache.insert(track.cache_key(), lyrics.clone());
        }
        // The song may have changed while the request was in flight.
        if state
            .lyrics
            .as_ref()
            .is_some_and(|l| l.track.same_song(&track))
        {
            set_lyrics(&app, &mut state, TrackLyrics { track, lyrics });
        }
    });
}

fn set_lyrics(app: &AppHandle, state: &mut AppState, lyrics: TrackLyrics) {
    let _ = app.emit("lyrics", &lyrics);
    state.lyrics = Some(lyrics);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_window_state::Builder::default()
                .with_state_flags(StateFlags::POSITION | StateFlags::SIZE)
                .build(),
        )
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, event| {
                    if event.state() == ShortcutState::Pressed {
                        toggle_locked(app);
                    }
                })
                .build(),
        )
        .manage(SharedState::default())
        .manage(lyrics::http_client().expect("failed to create HTTP client"))
        .invoke_handler(tauri::generate_handler![
            snapshot,
            set_locked,
            hide_window,
            reload_lyrics
        ])
        .setup(|app| {
            setup_tray(app.handle())?;
            if let Err(err) = app.global_shortcut().register(LOCK_SHORTCUT) {
                eprintln!("could not register {LOCK_SHORTCUT}: {err}");
            }
            spawn_player_poller(app.handle().clone());
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
