import "@fontsource-variable/bricolage-grotesque";
import "./styles.css";

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";

// ---- Payloads sent by the Rust side (src-tauri/src/lib.rs) ----------------

type Track = { title: string; artist: string; album: string; durationMs: number };
type Playback = { track: Track; positionMs: number; isPlaying: boolean };
type Line = { timeMs: number; text: string };
type Lyrics =
  | { status: "loading" | "instrumental" | "notFound" }
  | { status: "synced"; lines: Line[] }
  | { status: "unsynced"; text: string }
  | { status: "failed"; message: string };
type TrackLyrics = { track: Track; lyrics: Lyrics };
type Snapshot = { playback: Playback | null; lyrics: TrackLyrics | null; locked: boolean };

// ---- Settings ---------------------------------------------------------------

type Settings = { fontSize: number; offsetMs: number; context: boolean; locked: boolean };

const DEFAULT_SETTINGS: Settings = { fontSize: 30, offsetMs: 0, context: true, locked: false };
const MIN_FONT_SIZE = 16;
const MAX_FONT_SIZE = 72;
const OFFSET_STEP_MS = 250;

function loadSettings(): Settings {
  try {
    return { ...DEFAULT_SETTINGS, ...JSON.parse(localStorage.getItem("settings") ?? "{}") };
  } catch {
    return { ...DEFAULT_SETTINGS };
  }
}

function saveSettings() {
  try {
    localStorage.setItem("settings", JSON.stringify(settings));
  } catch {
    // Storage can be unavailable; settings then just last for this session.
  }
}

const settings = loadSettings();

// ---- Playback clock -----------------------------------------------------------

/** Only jump the local clock when it drifts this far from what Spotify reports. */
const RESYNC_THRESHOLD_MS = 400;
/** Cap on how long a line takes to fill, so lines followed by long gaps don't crawl. */
const MAX_FILL_MS = 7000;

let playback: Playback | null = null;
let trackLyrics: TrackLyrics | null = null;
let clock = { positionMs: 0, at: 0, playing: false };

function positionNow(): number {
  return clock.positionMs + (clock.playing ? performance.now() - clock.at : 0);
}

function sameSong(a: Track, b: Track): boolean {
  return a.title === b.title && a.artist === b.artist && a.album === b.album;
}

function onPlayback(next: Playback | null) {
  const songChanged = !next || !playback || !sameSong(next.track, playback.track);
  const drift = next ? Math.abs(positionNow() - next.positionMs) : 0;
  if (next && (songChanged || next.isPlaying !== clock.playing || drift > RESYNC_THRESHOLD_MS)) {
    clock = { positionMs: next.positionMs, at: performance.now(), playing: next.isPlaying };
  }
  playback = next;
  nowTitle.textContent = next?.track.title ?? "";
  nowArtist.textContent = next?.track.artist ?? "";
}

// ---- Rendering --------------------------------------------------------------

const $ = <T extends HTMLElement>(id: string) => document.getElementById(id) as T;
const stage = $("stage");
const prevEl = $("prev");
const currentEl = $("current");
const currentText = $("current-text");
const nextEl = $("next");
const nowTitle = $("now-title");
const nowArtist = $("now-artist");
const offsetEl = $("offset");
const contextButton = $("context");
const toastEl = $("toast");

type View = {
  prev: string;
  current: string;
  next: string;
  /** Lyric line index, or -1 for status messages. */
  index: number;
  fill: number;
};

function message(text: string, detail = ""): View {
  return { prev: "", current: text, next: detail, index: -1, fill: 0 };
}

function computeView(): View {
  if (!playback) {
    return message("Play a song in Spotify to see its lyrics");
  }
  const lyrics: Lyrics =
    trackLyrics && sameSong(trackLyrics.track, playback.track) ? trackLyrics.lyrics : { status: "loading" };
  const title = playback.track.title;

  switch (lyrics.status) {
    case "loading":
      return message("Finding lyrics…", title);
    case "notFound":
      return message("No lyrics found for this song", title);
    case "unsynced":
      return message("Only unsynced lyrics exist for this song", title);
    case "instrumental":
      return message("♪ Instrumental", title);
    case "failed":
      return message("Couldn't reach LRCLIB", "Check your connection, then press reload");
    case "synced":
      return syncedView(lyrics.lines, playback.track.durationMs);
  }
}

function syncedView(lines: Line[], durationMs: number): View {
  const pos = positionNow() + settings.offsetMs;
  const i = lineIndexAt(lines, pos);
  const text = (k: number) => (k < 0 || k >= lines.length ? "" : lines[k].text || "♪");

  // Intro before the first line: a note that fills until the singing starts.
  if (i < 0) {
    return { prev: "", current: "♪", next: text(0), index: -2, fill: clamp(pos / lines[0].timeMs) };
  }
  const start = lines[i].timeMs;
  const end = i + 1 < lines.length ? lines[i + 1].timeMs : Math.max(durationMs, start + MAX_FILL_MS);
  const fill = clamp((pos - start) / Math.min(end - start, MAX_FILL_MS));
  return { prev: text(i - 1), current: text(i), next: text(i + 1), index: i, fill };
}

/** Index of the last line starting at or before `pos`, or -1 before the first line. */
function lineIndexAt(lines: Line[], pos: number): number {
  let lo = 0;
  let hi = lines.length - 1;
  let found = -1;
  while (lo <= hi) {
    const mid = (lo + hi) >> 1;
    if (lines[mid].timeMs <= pos) {
      found = mid;
      lo = mid + 1;
    } else {
      hi = mid - 1;
    }
  }
  return found;
}

function clamp(n: number): number {
  return Number.isFinite(n) ? Math.min(1, Math.max(0, n)) : 1;
}

let shown: View = { prev: "", current: "", next: "", index: NaN, fill: NaN };
let shownFill = "";

function render() {
  const view = computeView();
  if (
    view.index !== shown.index ||
    view.current !== shown.current ||
    view.prev !== shown.prev ||
    view.next !== shown.next
  ) {
    prevEl.textContent = view.prev;
    currentText.textContent = view.current;
    nextEl.textContent = view.next;
    currentEl.classList.toggle("status", view.index === -1);
    if (view.index >= 0 && view.index !== shown.index) {
      // Restart the CSS animation for the incoming line.
      currentEl.classList.remove("rise");
      void currentEl.offsetWidth;
      currentEl.classList.add("rise");
    }
    shown = view;
  }

  const fill = `${(view.fill * 100).toFixed(1)}%`;
  if (fill !== shownFill) {
    currentEl.style.setProperty("--fill", fill);
    shownFill = fill;
  }
  document.body.classList.toggle("paused", playback !== null && !playback.isPlaying);
  requestAnimationFrame(render);
}

// ---- Controls -----------------------------------------------------------------

const LOCK_SHORTCUT = navigator.userAgent.includes("Mac") ? "⌘⌥⇧L" : "Ctrl+Alt+Shift+L";
const appWindow = getCurrentWindow();

function applySettings() {
  document.documentElement.style.setProperty("--size", `${settings.fontSize}px`);
  const seconds = settings.offsetMs / 1000;
  offsetEl.textContent = `${seconds > 0 ? "+" : ""}${seconds.toFixed(2)} s`;
  document.body.classList.toggle("compact", !settings.context);
  contextButton.setAttribute("aria-pressed", String(settings.context));
  saveSettings();
}

function update(change: Partial<Settings>) {
  Object.assign(settings, change);
  settings.fontSize = Math.min(MAX_FONT_SIZE, Math.max(MIN_FONT_SIZE, settings.fontSize));
  applySettings();
}

let toastTimer = 0;
function toast(text: string) {
  toastEl.textContent = text;
  toastEl.classList.add("show");
  clearTimeout(toastTimer);
  toastTimer = window.setTimeout(() => toastEl.classList.remove("show"), 3500);
}

function onLocked(locked: boolean) {
  document.body.classList.toggle("locked", locked);
  update({ locked });
  toast(locked ? `Locked. Press ${LOCK_SHORTCUT} or use the tray icon to unlock.` : "Unlocked");
}

const actions: Record<string, () => void> = {
  smaller: () => update({ fontSize: settings.fontSize - 2 }),
  larger: () => update({ fontSize: settings.fontSize + 2 }),
  earlier: () => update({ offsetMs: settings.offsetMs + OFFSET_STEP_MS }),
  later: () => update({ offsetMs: settings.offsetMs - OFFSET_STEP_MS }),
  resetOffset: () => update({ offsetMs: 0 }),
  context: () => update({ context: !settings.context }),
  reload: () => void invoke("reload_lyrics"),
  lock: () => void invoke("set_locked", { locked: true }),
  hide: () => void invoke("hide_window"),
};

$("controls").addEventListener("click", (event) => {
  const button = (event.target as Element).closest<HTMLElement>("button[data-action]");
  const action = button && actions[button.dataset.action ?? ""];
  action?.();
});

// The whole window is a drag handle, except the controls and the resize grip.
stage.addEventListener("mousedown", (event) => {
  if (event.button === 0 && !(event.target as Element).closest("button, .grip")) {
    void appWindow.startDragging();
  }
});

$("grip").addEventListener("mousedown", (event) => {
  if (event.button === 0) {
    event.preventDefault();
    void appWindow.startResizeDragging("SouthEast");
  }
});

if (import.meta.env.PROD) {
  document.addEventListener("contextmenu", (event) => event.preventDefault());
}

// ---- Startup --------------------------------------------------------------------

async function start() {
  applySettings();

  // Subscribe before asking for the snapshot so no update is missed.
  await listen<Playback | null>("playback", (event) => onPlayback(event.payload));
  await listen<TrackLyrics>("lyrics", (event) => {
    trackLyrics = event.payload;
  });
  await listen<boolean>("locked", (event) => onLocked(event.payload));

  const snapshot = await invoke<Snapshot>("snapshot");
  if (playback === null) onPlayback(snapshot.playback);
  trackLyrics ??= snapshot.lyrics;
  if (snapshot.locked !== settings.locked) {
    await invoke("set_locked", { locked: settings.locked });
  }

  requestAnimationFrame(render);
}

void start();
