import { describe, expect, it } from "vitest";

import { MAX_FILL_MS, RESYNC_THRESHOLD_MS, clamp, lineIndexAt, needsResync, syncedView } from "./sync";

const lines = [
  { timeMs: 10_000, text: "first" },
  { timeMs: 14_000, text: "" },
  { timeMs: 18_000, text: "third" },
];

describe("lineIndexAt", () => {
  it("is -1 before the first line", () => {
    expect(lineIndexAt(lines, 9_999)).toBe(-1);
    expect(lineIndexAt([], 5_000)).toBe(-1);
  });

  it("moves on exactly when the next line starts", () => {
    expect(lineIndexAt(lines, 10_000)).toBe(0);
    expect(lineIndexAt(lines, 13_999)).toBe(0);
    expect(lineIndexAt(lines, 14_000)).toBe(1);
  });

  it("stays on the last line after it starts", () => {
    expect(lineIndexAt(lines, 600_000)).toBe(2);
  });
});

describe("syncedView", () => {
  it("shows the lines around the current one, with a note for empty lines", () => {
    expect(syncedView(lines, 200_000, 15_000)).toMatchObject({
      prev: "first",
      current: "♪",
      next: "third",
      index: 1,
    });
  });

  it("fills the current line over the gap to the next one", () => {
    expect(syncedView(lines, 200_000, 12_000).fill).toBeCloseTo(0.5);
    expect(syncedView(lines, 200_000, 13_999).fill).toBeLessThan(1);
  });

  it("caps the fill time when the next line is far away", () => {
    const sparse = [
      { timeMs: 0, text: "a" },
      { timeMs: 60_000, text: "b" },
    ];
    expect(syncedView(sparse, 200_000, MAX_FILL_MS / 2).fill).toBeCloseTo(0.5);
    expect(syncedView(sparse, 200_000, MAX_FILL_MS + 1).fill).toBe(1);
  });

  it("fills a note during the intro", () => {
    const view = syncedView(lines, 200_000, 5_000);
    expect(view).toMatchObject({ prev: "", current: "♪", next: "first", index: -2 });
    expect(view.fill).toBeCloseTo(0.5);
  });

  it("fills the last line over the capped time", () => {
    // BUG-03: the song's end is ignored, so this holds even when the outro is shorter.
    expect(syncedView(lines, 200_000, 18_000 + MAX_FILL_MS / 2).fill).toBeCloseTo(0.5);
    expect(syncedView(lines, 200_000, 400_000)).toMatchObject({ current: "third", next: "", fill: 1 });
  });
});

describe("clamp", () => {
  it("keeps the fill between 0 and 1", () => {
    expect(clamp(-0.5)).toBe(0);
    expect(clamp(0.25)).toBe(0.25);
    expect(clamp(3)).toBe(1);
  });

  it("treats a ratio that isn't a number as done", () => {
    expect(clamp(Number.NaN)).toBe(1);
    expect(clamp(Number.POSITIVE_INFINITY)).toBe(1);
  });
});

describe("needsResync", () => {
  const local = { positionMs: 60_000, playing: true };
  const reported = (positionMs: number, isPlaying = true) => ({ positionMs, isPlaying });

  it("ignores drift up to the threshold, so lines don't jitter", () => {
    expect(needsResync(local, reported(60_000 + RESYNC_THRESHOLD_MS), false)).toBe(false);
    expect(needsResync(local, reported(60_000 - RESYNC_THRESHOLD_MS), false)).toBe(false);
  });

  it("resyncs on larger drift in either direction", () => {
    expect(needsResync(local, reported(60_000 + RESYNC_THRESHOLD_MS + 1), false)).toBe(true);
    expect(needsResync(local, reported(60_000 - RESYNC_THRESHOLD_MS - 1), false)).toBe(true);
  });

  it("resyncs on play, pause and song changes", () => {
    expect(needsResync(local, reported(60_000, false), false)).toBe(true);
    expect(needsResync(local, reported(60_000), true)).toBe(true);
  });
});
