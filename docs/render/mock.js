// Stands in for the Tauri runtime so the real frontend renders in a plain browser.
// Song, artist and lyrics are made up for the README renders.
(() => {
  const track = {
    title: "Lanterns on the Water",
    artist: "Juniper Static",
    album: "Night Ferries",
    durationMs: 214000,
  };
  const texts = [
    "We kept the porch light burning",
    "Long after the trains went quiet",
    "And every song you left me",
    "Still finds its way back home",
    "So sing it slow, sing it softly",
    "Till the morning pulls us under",
  ];
  const lines = texts.map((text, i) => ({ timeMs: 9000 + i * 4000, text }));
  const current = 2;
  const positionMs = lines[current].timeMs + 0.55 * 4000;
  const snapshot = {
    playback: { track, positionMs, isPlaying: true },
    lyrics: { track, lyrics: { status: "synced", lines } },
    locked: false,
  };

  // Freeze the clock so the fill is captured at the same point every time.
  const frozen = 10000;
  performance.now = () => frozen;

  window.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener() {} };
  window.__TAURI_INTERNALS__ = {
    metadata: {
      currentWindow: { label: "main" },
      currentWebview: { windowLabel: "main", label: "main" },
    },
    transformCallback(callback) {
      const id = Math.floor(Math.random() * 1e9);
      window[`_${id}`] = callback;
      return id;
    },
    unregisterCallback() {},
    convertFileSrc: (path) => path,
    async invoke(cmd) {
      if (cmd === "snapshot") return snapshot;
      if (cmd === "plugin:event|listen") return 1;
      return null;
    },
  };
})();
