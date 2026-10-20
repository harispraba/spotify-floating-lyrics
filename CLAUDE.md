# Spotify Floating Lyrics

A Tauri 2 desktop app: a transparent, always-on-top window that shows time-synced lyrics for whatever the Spotify desktop app is playing. The backend is Rust (`src-tauri/`) and the frontend is vanilla TypeScript with Vite (`src/`). Windows comes first; Linux and macOS share the same codebase.

- **Work items** are in [TRACKING.md](TRACKING.md). Check it before you start and update it when you finish (rules below). Its Done rows record what has been tested and how.
- [README.md](README.md) has the user-facing docs, plus the troubleshooting table and the design notes. Read the design notes before changing how playback or lookups work.
- The repo is public at <https://github.com/harispraba/spotify-floating-lyrics>, so keep personal details (paths, emails) out of files and commits.

## Commands

Run these from the repo root. They behave the same in PowerShell, cmd and Git Bash.

| Command | What it does |
| --- | --- |
| `npm install` | Installs the frontend dependencies and the Tauri CLI |
| `npm run tauri dev` | Runs the app with hot reload. It keeps running until you quit from the tray, so start it in the background. Rust `eprintln!` output shows up here |
| `npm run build` | Type-checks (tsc) and bundles the frontend |
| `cargo test --manifest-path src-tauri/Cargo.toml` | Rust unit tests (LRC parser, LRCLIB matching), 12 tests |
| `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets` | Lint. Keep it warning-free |
| `cargo fmt --manifest-path src-tauri/Cargo.toml` | Formats the Rust code (default rustfmt settings) |
| `npm run tauri build` | Release build. Installers land in `src-tauri/target/release/bundle/` |

A change is done when `npm run build`, the tests, clippy and fmt are all clean, and the user has checked anything visual.

## Architecture

```
src-tauri/src/
  lib.rs                 poller thread, AppState, tray, lock shortcut, commands
  player/mod.rs          Track / Playback types; picks the backend per OS
  player/smtc.rs         Windows: System Media Transport Controls
  player/mpris.rs        Linux: MPRIS over D-Bus
  player/applescript.rs  macOS: osascript
  lyrics/lrclib.rs       LRCLIB client and result matching
  lyrics/lrc.rs          LRC parser
src/
  main.ts                rendering, local playback clock, controls, settings
  styles.css             all colours and sizes as custom properties at the top
```

A plain `std::thread` in `lib.rs` calls `SpotifySource::poll()` every 500 ms. It stores the result in `AppState` and emits it to the window. Once a song has been seen on two polls in a row, it looks up lyrics on LRCLIB on Tauri's async runtime, and keeps retrying a failed lookup while that song plays. The window keeps its own clock between polls.

### Rust ↔ window contract

The Rust payloads serialize as camelCase. They're mirrored by hand as TS types at the top of `src/main.ts`, so change both together.

- Events (Rust → window): `playback` (`Playback | null`), `lyrics` (`TrackLyrics`), `locked` (`boolean`)
- Commands (window → Rust): `snapshot`, `set_locked { locked }`, `hide_window`, `reload_lyrics`
- `Lyrics` is tagged by `status`: `loading`, `synced`, `unsynced`, `instrumental`, `notFound`, `failed`

A window API called from the frontend needs a permission in `src-tauri/capabilities/default.json`.

## Conventions

- Platform-specific code lives in `src-tauri/src/player/<backend>.rs` behind `cfg`. Each backend exposes `SpotifySource` with `new()` and `poll(&mut self) -> anyhow::Result<Option<Playback>>`. Nothing outside `player/` is platform-specific.
- Comments explain why, not what, and are sparse. Match the density of the surrounding code.
- No frontend framework. User-facing text is short, plain and in sentence case.
- Settings live in the window's localStorage under the `settings` key.
- Think twice before adding a dependency. In particular, reqwest uses `native-tls` with default features off, which keeps aws-lc-rs and its C build out of the tree. Keep it that way.

## Gotchas

- Vite 8 needs Node 20.19+ or 22.12+. If `npm install` crashes with `Cannot read properties of null (reading 'edgesOut')`, run `npx -y npm@11 install`.
- `package-lock.json` was written by npm 11. npm 10 strips its `libc` fields on install, which is churn, not a real change. Don't commit it; `git checkout package-lock.json` undoes it.
- Never build or run inside WSL. A WSL build is a Linux app and can't see the Windows Spotify app.
- In the `windows` 0.62 crate, you block on WinRT async with `.join()`. It used to be `.get()`.
- Spotify on Windows only refreshes its reported position on play, pause and seek, so `smtc.rs` extrapolates from `LastUpdatedTime`. Suspect this first if lyrics drift.
- On quit, WebView2 logs `Failed to unregister class Chrome_WidgetWin_0. Error = 1412`. It's harmless.
- Release builds have no console (`windows_subsystem = "windows"`), so logs only appear under `npm run tauri dev`. For webview devtools, right-click → Inspect in a dev build (the context menu is disabled in release builds).
- `npm run tauri dev` fails with "Port 1420 is already in use" when an earlier dev server is still running. Stop it first.
- Lock mode makes the window ignore the mouse, and it persists across restarts. Unlock with Ctrl+Alt+Shift+L or the tray menu.
- You can't see the overlay. For anything visual, ask the user to look, or to take a screenshot with Win+Shift+S and paste it in.

## Tracking rules

- Every piece of work has an ID in [TRACKING.md](TRACKING.md) (`FEAT-xx`, `BUG-xx`, `TASK-xx`). If there's no ID yet, add a row before you start.
- Set the status to `doing` when you start. When it's finished, move the row to Done with the date. Log new bugs as you find them, even ones you don't fix.
- Start commit messages with the ID, for example `BUG-01: skip SMTC extrapolation without a timeline`.
