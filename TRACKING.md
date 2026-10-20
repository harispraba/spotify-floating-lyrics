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
| TASK-05 | Check Spotify detection on Windows | todo | P1 | Hovering shows the right title and artist. Spotify here is the Store version (1.305.3058) |
| TASK-06 | Check lyric sync over a whole song | todo | P1 | See Details |
| TASK-07 | Check the window on a real desktop | todo | P1 | See Details |
| TASK-08 | Check the lock and the tray menu | todo | P1 | See Details |
| BUG-01 | Position jumps to the end when SMTC has no timeline | todo | P2 | Found by reading the code. See Details |
| TASK-09 | Build and test the installer | todo | P2 | `npm run tauri build`, then install and run the NSIS build |
| TASK-10 | Opt-in debug logging | todo | P2 | Only errors are logged now. Log the detected track, the raw SMTC timeline and which LRCLIB step matched |
| TASK-11 | Replace the default Tauri icons | todo | P2 | The tray and the installer show the Tauri logo |
| FEAT-10 | Show unsynced lyrics | todo | P2 | Today it says "Only unsynced lyrics exist for this song". Could show the plain text, scrolled by song progress |
| FEAT-11 | Overlay never takes keyboard focus | todo | P2 | First check whether it does (TASK-07) |
| FEAT-12 | Retry failed lookups automatically | todo | P3 | A locked overlay can't reach the reload button, so a network blip sticks until the next song |
| FEAT-13 | Remember the timing offset per song | todo | P3 | Timing varies per LRCLIB entry, so one global offset doesn't fit every song |
| FEAT-14 | Colour and theme settings | todo | P3 | Colours are already custom properties at the top of `styles.css` |
| FEAT-15 | Start with Windows | todo | P3 | `tauri-plugin-autostart`, with a toggle in the tray |
| FEAT-16 | Spotify Web API source | todo | P3 | Covers the web player and other devices. Needs a Spotify developer app and PKCE login |
| FEAT-17 | Save looked-up lyrics to disk | todo | P3 | Fewer LRCLIB calls for songs played again |
| TASK-12 | Build and run on Linux | todo | P3 | The full app has never been compiled for Linux (needs webkit2gtk) |
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

### BUG-01 Position jumps to the end when SMTC has no timeline

In `src-tauri/src/player/smtc.rs`, `read()` adds the time since `LastUpdatedTime` to the position while a song is playing. If a session never fills in its timeline, `LastUpdatedTime` is zero, which is the year 1601. The elapsed time then comes out as centuries, the position lands at or past the end of the song, and the overlay shows the last line.

Fix: only extrapolate when `UniversalTime > UNIX_EPOCH_TICKS`, and consider capping the elapsed time. TASK-05 will show whether Spotify fills in the timeline at all.
