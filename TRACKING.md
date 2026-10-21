# Tracking

Every piece of work gets an ID. Search for one (`BUG-01`) to find its row, and start commit messages with it.

## How to use this file

| Prefix | Use for |
| --- | --- |
| `FEAT` | New or changed behaviour the user can see |
| `BUG` | Something that works wrongly |
| `TASK` | Everything else: verification, builds, docs, tooling, refactors |

- **Numbering:** use the next number after the highest existing one for that prefix, with two digits (`BUG-02`). Never reuse or renumber an ID, even for dropped items.
- **Status:** `todo`, `doing` or `blocked`. For `blocked`, say on what in Notes. Finished items move to Done with the date. Dropped items move to Dropped with the reason.
- **Priority:** P1 is needed before a first Windows release, P2 should happen soon, P3 is someday.
- Keep notes to a line. Longer context goes under Details, in a section headed by the ID.

## Open

| ID | Title | Status | Priority | Notes |
| --- | --- | --- | --- | --- |
| BUG-03 | The last line always takes 7 s to fill | todo | P3 | Found writing TASK-17's tests. `syncedView` ignores the song's end, so a short outro cuts the fill off. Pinned in `sync.test.ts` |
| BUG-01 | Position jumps to the end when SMTC has no timeline | todo | P2 | Found by reading the code. Store Spotify fills in the timeline, so not seen yet. See Details |
| TASK-10 | Opt-in debug logging | todo | P2 | Only errors are logged now. Log the detected track, the raw SMTC timeline and which LRCLIB step matched |
| TASK-11 | Replace the default Tauri icons | todo | P2 | The tray and the installer show the Tauri logo |
| TASK-20 | Fill in the installer metadata | todo | P3 | Windows lists the publisher as "harispraba", taken from the identifier. Set `publisher`, `copyright` and `licenseFile` under `bundle` in `tauri.conf.json` |
| FEAT-10 | Show unsynced lyrics | todo | P2 | Today it says "Only unsynced lyrics exist for this song". Could show the plain text, scrolled by song progress |
| FEAT-13 | Remember the timing offset per song | todo | P3 | Timing varies per LRCLIB entry, so one global offset doesn't fit every song |
| FEAT-14 | Colour and theme settings | todo | P3 | Colours are already custom properties at the top of `styles.css` |
| FEAT-15 | Start with Windows | todo | P3 | `tauri-plugin-autostart`, with a toggle in the tray |
| FEAT-16 | Spotify Web API source | todo | P3 | Covers the web player and other devices. Needs a Spotify developer app and PKCE login |
| FEAT-17 | Save looked-up lyrics to disk | todo | P3 | Fewer LRCLIB calls for songs played again |
| TASK-12 | Build and run on Linux | todo | P3 | Builds, lints and passes the tests on ubuntu-24.04 in CI (TASK-16). Never run against a real Spotify yet |
| TASK-13 | Build and run on macOS | todo | P3 | The full app has never been compiled for macOS. Check the Automation permission prompt |

## Done

| ID | Title | Done | Notes |
| --- | --- | --- | --- |
| FEAT-01 | Transparent always-on-top overlay | 2026-10-20 | Previous, current and next lines. The current line fills amber as it's sung |
| FEAT-02 | Windows playback via SMTC | 2026-10-20 | Compiles for Windows. Not tried against real Spotify yet (TASK-05) |
| FEAT-03 | Linux playback via MPRIS | 2026-10-20 | Tested against a fake Spotify on D-Bus |
| FEAT-04 | macOS playback via AppleScript | 2026-10-20 | Type-checks; never run |
| FEAT-05 | LRCLIB lookup | 2026-10-20 | Exact match, then search. Prefers synced lyrics, retries server errors, caches in memory |
| FEAT-06 | Click-through lock | 2026-10-20 | From the toolbar, the tray or Ctrl+Alt+Shift+L. Survives restarts |
| FEAT-07 | Hover toolbar | 2026-10-20 | Text size, timing offset, context lines, reload, lock, hide |
| FEAT-08 | Move, resize and remember the window | 2026-10-20 | Drag anywhere; resize from the corner grip |
| FEAT-09 | Tray menu | 2026-10-20 | Show lyrics, lock, quit |
| TASK-01 | README | 2026-10-20 | Setup, usage, known limits |
| TASK-02 | Handover docs and this tracker | 2026-10-20 | CLAUDE.md, HANDOVER.md, TRACKING.md |
| TASK-03 | Move the project to Windows and make the initial commit | 2026-10-20 | Copy matches the WSL tree. Committed before the first build, so Windows fixes show up as their own diffs |
| TASK-04 | First Windows build: install, test, lint, launch | 2026-10-20 | Rust 1.99.1, VS 2022 Build Tools, Node 23.1 through fnm. 12/12 tests, clippy and fmt clean, window shows |
| TASK-05 | Check Spotify detection on Windows | 2026-10-20 | Store version 1.305.3058. Right title and artist on hover; synced lyrics load and advance |
| TASK-06 | Check lyric sync over a whole song | 2026-10-20 | Checked by the user with the Store app: on time through a whole song, pause, seek and skip |
| TASK-07 | Check the window on a real desktop | 2026-10-20 | All items pass, including Alt+Tab and an exact position restore. Focus at launch is under FEAT-11 |
| TASK-08 | Check the lock and the tray menu | 2026-10-20 | All items pass. Starting locked was also confirmed from the window's click-through style |
| TASK-14 | Publish the repo on GitHub | 2026-10-20 | Public at harispraba/spotify-floating-lyrics. MIT license, commits rewritten to the noreply email, HANDOVER.md folded into the README |
| TASK-16 | CI on every PR | 2026-10-20 | Windows and Linux jobs, about 5 min each on a cold cache. The Linux job is the first full Linux build |
| TASK-17 | Frontend unit tests | 2026-10-20 | 13 Vitest tests for `src/sync.ts`. Breaking the resync threshold or the fill cap on purpose makes them fail |
| TASK-19 | Protect main | 2026-10-20 | Ruleset "Protect main": PR with green Windows and Linux checks, squash only, no bypass. A test push straight to main was rejected with GH013 |
| TASK-18 | Release workflow and v0.1.0 | 2026-10-21 | Tag `v0.1.0` built both installers in CI into a draft release, which was published once TASK-09 passed |
| TASK-09 | Build and test the installer | 2026-10-21 | The NSIS installer downloaded from the v0.1.0 draft installs per user into `%LOCALAPPDATA%`, with a Start menu shortcut, and runs. Checked by the user |
| TASK-15 | Add renders of the overlay to the README | 2026-10-21 | `docs/render` renders the real frontend with a mocked Tauri API and made-up lyrics. Output is byte-identical between runs. Checked by the user |
| FEAT-11 | Overlay never takes keyboard focus | 2026-10-21 | `"focus": false` and `"focusable": false` in `tauri.conf.json`; no platform code. Checked by the user. See Details |
| BUG-02 | A wrapped current line fills on every row at once | 2026-10-20 | The gradient moved to an inline span, so it runs through the rows in reading order. Checked by the user |
| FEAT-12 | Retry failed lookups automatically | 2026-10-20 | After 3, 10 and 30 s, then every minute, while the song plays. Failures are now logged. See Details |

## Dropped

| ID | Title | Dropped | Reason |
| --- | --- | --- | --- |

## Details

### TASK-06 Check lyric sync over a whole song

Spotify on Windows only updates its reported position on play, pause and seek, and `smtc.rs` adds the time elapsed since `LastUpdatedTime`. Check that:

- Lines change on time, within about half a second, from the start to the end of a song that's around four minutes long.
- Pausing for 10 s or more and then resuming doesn't make the lyrics jump.
- After a seek forward or back, the overlay catches up within a second.
- Skipping to the next or previous track loads that song's lyrics.

If the lyrics gradually run ahead or fall behind over a song, the extrapolation is wrong, so log the raw values (TASK-10). If every line is off by the same amount from the first line on, the cause is usually the LRCLIB entry, and the offset buttons fix that.

### TASK-07 Check the window on a real desktop

- There's no white or black box behind the text; the desktop shows through.
- It stays on top of normal windows, including while another app is focused.
- It doesn't appear in the taskbar. Check Alt+Tab too, because hiding from the taskbar may not hide it there.
- Dragging anywhere moves it, the corner grip resizes it, and both survive a restart.
- Hovering shows the track and the toolbar, and every button works.
- The text is readable over a white window and over a dark one.
- Note whether it takes keyboard focus at launch (FEAT-11).

### TASK-08 Check the lock and the tray menu

- The lock button makes clicks pass through to the window underneath, and the toast says how to unlock.
- Ctrl+Alt+Shift+L toggles the lock, and the tray's "Lock (click-through)" check mark follows it.
- After restarting while locked, the app starts locked.
- The tray's "Show lyrics" hides and shows the window. The toolbar's × hides it and clears that check mark. Quit exits.

### FEAT-11 Overlay never takes keyboard focus

Checked in TASK-07. The window's extended style is `0x40118` (`TOPMOST`, `APPWINDOW`), with no `WS_EX_NOACTIVATE`, and `tauri.conf.json` leaves `focus` at its default of on. When it was launched from a background shell, it didn't take focus from the app in front, but Windows' foreground lock may be what stopped it there. In TASK-08 the user confirmed that clicking the overlay takes keyboard focus from the field they were typing in.

Fixed with two window options in `tauri.conf.json`, so no platform code was needed. `"focusable": false` makes tao add `WS_EX_NOACTIVATE` on Windows (the style became `0x8040118`) and show the window without activating it, and `"focus": false` stops it asking for focus when it's created. The foreground app stayed in front through launch. The user then typed in another app while clicking, dragging and resizing the overlay and pressing its toolbar buttons, and the keystrokes never left that app. The toolbar works by mouse only now, which suits an overlay.

To read the overlay's styles from outside, find the window with class `Tauri Window`. During startup, `Process.MainWindowHandle` can return tao's `Tao Thread Event Target` instead, which is visible, 18×18 and click-through by design.

### FEAT-12 Retry failed lookups automatically

In TASK-08, the lookup for "And the Hero Will Drown (10 Year Version)" failed and stayed failed. Replaying the requests showed `/search` answering 503 `ServerOverloaded` ("The server is busy, please retry in a moment"). Seconds later, eight requests in a row got 200, so the overload is brief but long enough to beat the single 800 ms retry in `lrclib.rs`.

`load_lyrics` in `lib.rs` now keeps retrying while that song is playing. A newer lookup, for another song or from reload, stops the older one through the `lookup` counter in `AppState`. Each failure is logged with its cause, and the window says "Couldn't get lyrics from LRCLIB" and "Trying again shortly".

To test it, the app was run with `HTTPS_PROXY` pointing at a local proxy that refused its first four tunnels. The lookups failed at 0 s and 3 s and succeeded at 13 s, as planned.

### BUG-01 Position jumps to the end when SMTC has no timeline

In `src-tauri/src/player/smtc.rs`, `read()` adds the time since `LastUpdatedTime` to the position while a song is playing. If a session never fills in its timeline, `LastUpdatedTime` is zero, which is the year 1601. The elapsed time then comes out as centuries, the position lands at or past the end of the song, and the overlay shows the last line.

Fix: only extrapolate when `UniversalTime > UNIX_EPOCH_TICKS`, and consider capping the elapsed time. TASK-05 showed that the Store version of Spotify fills in the timeline while playing, so this only hits sessions that don't.
