// Lyric timing, kept free of the DOM, Tauri and the clock so it can be unit tested.

/** Only jump the local clock when it drifts this far from what Spotify reports. */
export const RESYNC_THRESHOLD_MS = 400;
/** Cap on how long a line takes to fill, so lines followed by long gaps don't crawl. */
export const MAX_FILL_MS = 7000;

export type TimedLine = { timeMs: number; text: string };

export type View = {
  prev: string;
  current: string;
  next: string;
  /** Lyric line index, -1 for status messages, or -2 for the intro before the first line. */
  index: number;
  fill: number;
};

export function message(text: string, detail = ""): View {
  return { prev: "", current: text, next: detail, index: -1, fill: 0 };
}

/** What to show `pos` ms into a song. `lines` must not be empty. */
export function syncedView(lines: TimedLine[], durationMs: number, pos: number): View {
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
export function lineIndexAt(lines: { timeMs: number }[], pos: number): number {
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

export function clamp(n: number): number {
  return Number.isFinite(n) ? Math.min(1, Math.max(0, n)) : 1;
}

/** Whether the local clock should jump to the position Spotify reports. */
export function needsResync(
  local: { positionMs: number; playing: boolean },
  reported: { positionMs: number; isPlaying: boolean },
  songChanged: boolean,
): boolean {
  return (
    songChanged ||
    reported.isPlaying !== local.playing ||
    Math.abs(local.positionMs - reported.positionMs) > RESYNC_THRESHOLD_MS
  );
}
