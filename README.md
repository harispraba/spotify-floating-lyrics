# Spotify Floating Lyrics

A small always-on-top, see-through window that shows time-synced lyrics for whatever the Spotify desktop app is playing. The current line fills in as it's sung. Built with [Tauri 2](https://tauri.app) (Rust + TypeScript), so one codebase runs on Windows, macOS and Linux.

- **No Spotify login or API key.** Playback is read from the OS media integration on the same machine.
- **Lyrics from [LRCLIB](https://lrclib.net)**, a free, open database of synced lyrics.
- **Click-through lock** so the lyrics never block what's underneath.

## How it works

| Platform | Where "now playing" comes from |
| --- | --- |
| Windows 10/11 | System Media Transport Controls (`GlobalSystemMediaTransportControlsSessionManager`) |
| Linux | MPRIS over D-Bus (`org.mpris.MediaPlayer2.spotify*`, which also covers spotifyd) |
| macOS | Spotify's AppleScript interface via `osascript` |

A background thread polls the player every 500 ms and sends the track and position to the window. When the song changes, the backend looks it up on LRCLIB: first an exact match on title, artist, album and duration, then a fuzzy search. It prefers synced lyrics and caches results in memory. The window keeps its own clock between polls, so lines change smoothly.

```
src-tauri/src/
  lib.rs                 app wiring: poller thread, tray menu, lock shortcut, commands
  player/mod.rs          Track / Playback types, picks the right backend per OS
  player/smtc.rs         Windows
  player/mpris.rs        Linux
  player/applescript.rs  macOS
  lyrics/lrclib.rs       LRCLIB client and result matching
  lyrics/lrc.rs          LRC parser
src/
  main.ts                overlay rendering, sync clock, controls
  styles.css
```

## Running it on Windows

> **Using WSL?** Build and run on the Windows side, not inside WSL. A WSL build is a Linux app and can't see the Windows Spotify app. Copy or clone the project to a Windows folder (for example `C:\dev\spotify-floating-lyrics`) and use PowerShell.

1. Install the prerequisites (details in [Tauri's guide](https://tauri.app/start/prerequisites/#windows)):
   - **Microsoft C++ Build Tools** with the "Desktop development with C++" workload
   - **WebView2**, which is already present on Windows 10 (1803+) and 11
   - **Rust**, from <https://rustup.rs>
   - **Node.js 20.19+ or 22.12+** (Vite 8 needs it)
2. Install and start it:
   ```powershell
   npm install
   npm run tauri dev
   ```
3. Play something in the Spotify desktop app.

To produce an installer, run `npm run tauri build`. The installers land in `src-tauri/target/release/bundle/` (`nsis/*.exe` and `msi/*.msi`).

## Using it

- **Move:** drag anywhere on the lyrics. **Resize:** drag the grip in the bottom-right corner. Position and size are remembered.
- **Hover** (while unlocked) to show the track name and the toolbar:
  - **A− / A+:** text size
  - **− 0.00 s +:** timing offset in 0.25 s steps. Use **+** if lyrics lag behind the singing. Click the number to reset.
  - **≡:** show or hide the previous and next lines
  - **↻:** look the lyrics up again
  - **Lock:** click-through mode. The window ignores the mouse entirely.
  - **×:** hide the window
- **Unlock or show it again** from the tray icon menu, or press **Ctrl+Alt+Shift+L** (⌘⌥⇧L on macOS) to toggle the lock.

Settings are stored per machine in the window's local storage.

## Running it on Linux and macOS

Same commands (`npm install`, `npm run tauri dev`) after installing [Tauri's prerequisites](https://tauri.app/start/prerequisites/) for your OS.

- **Linux:** needs the official Spotify client (or spotifyd) with MPRIS on the session bus. Transparency needs a compositing window manager. The global shortcut uses X11 and may not fire under Wayland; use the tray menu there.
- **macOS:** the first run asks for permission to control Spotify (System Settings → Privacy & Security → Automation). Transparency uses Tauri's `macOSPrivateApi`, which is fine for direct distribution but not allowed on the Mac App Store.

## Known limits

- Lyrics coverage is whatever LRCLIB has. For songs with only plain (unsynced) lyrics, the window says so instead of guessing timings. You can contribute synced lyrics at lrclib.net.
- Spotify on Windows only updates its reported position on play, pause and seek, so the app extrapolates from the last update. If lyrics drift on your machine, the timing offset fixes it.
- Only the Spotify desktop app on the same computer is detected. The web player and other devices aren't. Supporting them would need the Spotify Web API and a Spotify developer app.

## Development

```bash
npm run build                                                     # type-check + bundle the frontend
cargo test --manifest-path src-tauri/Cargo.toml                   # LRC parser and LRCLIB matching tests
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets
```

Open work and known bugs are tracked in [TRACKING.md](TRACKING.md). [HANDOVER.md](HANDOVER.md) describes the project's current state and what's been tested.
