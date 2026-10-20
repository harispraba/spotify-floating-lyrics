# Handover

Written 2026-10-20 at the end of the first build session. The app was written and cross-checked from WSL, and it has **never been run on Windows**. This page is for whoever picks it up in PowerShell next, whether that's a person or Claude Code. The WSL session's memory doesn't carry over, so everything worth knowing is in this file, [CLAUDE.md](CLAUDE.md) and [TRACKING.md](TRACKING.md).

## Where things stand

- **v0.1.0 has every feature planned for a first try:** the overlay, Spotify detection on all three OSes, LRCLIB lookup, click-through lock, hover toolbar and tray menu. The full list is FEAT-01 to FEAT-09 in TRACKING.md.
- **The project is on Windows and committed** (TASK-03), and it builds, passes its checks and runs there (TASK-04). It reads the Store version of Spotify and keeps lyrics in sync (TASK-05, TASK-06). `C:\dev\spotify-floating-lyrics` is the real copy now.
- **Next job:** check the behaviour that couldn't be checked from WSL (TASK-07 and TASK-08).

## 1. Move the project to Windows

The project has to be built on the Windows side. Copy it out of WSL and leave behind `node_modules` and the build output, which hold Linux binaries:

```powershell
robocopy \\wsl.localhost\Ubuntu\home\haris\workspaces\harispraba\spotify-floating-lyrics C:\dev\spotify-floating-lyrics /E /XD node_modules dist target
cd C:\dev\spotify-floating-lyrics
claude
```

robocopy exits with code 1 when it copied files, which means success. From then on, the Windows copy is the real one. Don't keep editing the WSL copy, or the two will drift apart.

## 2. Install the prerequisites

| Tool | Notes |
| --- | --- |
| Microsoft C++ Build Tools | Pick the "Desktop development with C++" workload. See [Tauri's guide](https://tauri.app/start/prerequisites/#windows) |
| WebView2 | Already part of Windows 10 (1803+) and 11 |
| Rust (stable, MSVC toolchain) | `winget install Rustlang.Rustup`, or <https://rustup.rs> |
| Node.js 20.19+ or 22.12+ | `winget install OpenJS.NodeJS.LTS`. Older versions fail on Vite 8 |

Open a new terminal after installing so the updated `PATH` is picked up.

## 3. First run

Do these in order:

1. Check the versions: `node -v`, `rustc -V`, `cargo -V`.
2. Run `npm install`. If it crashes with `reading 'edgesOut'`, run `npx -y npm@11 install` instead.
3. Run `cargo test --manifest-path src-tauri/Cargo.toml`. All 12 tests should pass. The first build compiles a few hundred crates and takes several minutes.
4. Run `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets`. It should report no warnings, since it was clean when cross-checked for the Windows target.
5. ~~Make the initial commit (TASK-03).~~ Done on 2026-10-20, before steps 1 to 4.
6. Start Spotify and play a song, then run `npm run tauri dev`. Work through TASK-05 to TASK-08 with the user. The checklists are under Details in TRACKING.md.

## What was verified, and how

| Check | How | Result |
| --- | --- | --- |
| Whole app compiles for Windows | `cargo clippy --target x86_64-pc-windows-msvc --all-targets` from WSL, with the link step stubbed out | Clean |
| Rust unit tests | Run on Linux | 12/12 pass |
| Linux playback reader | Against a fake Spotify on a private D-Bus | Correct title, artists and duration |
| LRCLIB lookups | Live requests | Synced lyrics found for "Get Lucky" and "Bohemian Rhapsody - Remastered 2011" |
| macOS playback reader | Clippy for `aarch64-apple-darwin`, backend code only | Clean |
| Frontend | `npm run build`, plus headless Chromium screenshots with a mocked Tauri API | Builds; readable over light and dark backgrounds |
| Windows build | `npm run build`, `cargo test`, clippy and fmt on Windows, then `npm run tauri dev` | All clean, 12/12 tests pass, the window shows |
| Windows playback reader | The Store version of Spotify (1.305.3058) through SMTC | Right title and artist; synced lyrics load and advance |
| Lyric sync on Windows | A whole song with pause, seek and skip, watched by the user | On time |

Not done yet:

- Transparency, always-on-top and click-through on a real desktop
- Building the installer
- Building the full app for Linux or macOS

The WSL cross-check setup (a throwaway Rust toolchain and a test harness) lived in a temporary folder and is gone. Windows doesn't need it, because the real toolchain runs everything directly.

## If something's wrong

The likeliest problems come first.

| Symptom | Likely cause | What to do |
| --- | --- | --- |
| Says "Play a song in Spotify…" while a song is playing | No SMTC session found, or polling fails | Look in the terminal for `failed to read Spotify playback: …`. `find_spotify` in `smtc.rs` matches any app ID containing "spotify". If the error mentions `CoInitialize`, call `RoInitialize(RO_INIT_MULTITHREADED)` at the start of the poller thread in `lib.rs` |
| Every line is early or late by the same amount | That LRCLIB entry's timing | Try another song. The toolbar's offset buttons compensate |
| Lyrics drift during a song, or jump after pause or seek | `LastUpdatedTime` doesn't behave the way `smtc.rs` assumes | Log the raw `Position`, `LastUpdatedTime` and current time on each poll (TASK-10) |
| Shows the last line as soon as a song starts | BUG-01 | See BUG-01 in TRACKING.md |
| White or black box behind the text | Transparency isn't working | Check `"transparent": true` in `tauri.conf.json` and the transparent `body` background in `styles.css`. Update WebView2 |
| Ctrl+Alt+Shift+L does nothing | Another app has claimed the shortcut | The terminal shows `could not register …`. Use the tray menu |
| The window opens off-screen | Saved position from a monitor that's gone | Delete `%APPDATA%\com.harispraba.spotifyfloatinglyrics\.window-state.json` |
| Settings seem stuck | Bad value in localStorage | In a dev build, open devtools and run `localStorage.clear()` |

## Design decisions

These choices were deliberate. Don't undo them without a reason:

- **Playback comes from the OS media integration, not the Spotify Web API.** That way there's no login, no Spotify developer app and no rate limits. The cost is that only the desktop app on the same machine is seen. Web API support is FEAT-16.
- **Lyrics come from LRCLIB**, which is free, needs no key and has synced lyrics.
  - The lookup tries the exact `/get` endpoint first, then `/search` with a cleaned title and the first artist, then a free-text search.
  - Results whose length is more than 4 s off are rejected, because they're most likely a different recording.
  - Synced lyrics beat plain ones.
  - Server errors (5xx) are retried once after 800 ms.
  - Failed lookups aren't cached, so reload tries again.
- **Lookups wait until a song shows up on two polls in a row.** Skipping through songs then doesn't fire a request for each one, and the duration has time to settle.
- **The window runs its own clock.** It only resyncs when it's more than 400 ms away from what Spotify reports, so lines don't jitter with poll timing. A line takes at most 7 s to fill.
- **The poller is a plain thread, not a tokio task**, because the SMTC, D-Bus and osascript calls all block. The lyrics fetch is async.
- **Only window position and size are restored.** Restoring visibility as well could start the app hidden.
- **Unsynced lyrics show a message** instead of guessed timings. Showing them properly is FEAT-10.
- **reqwest uses native-tls**, which is SChannel on Windows. This keeps aws-lc-rs out of the build.
