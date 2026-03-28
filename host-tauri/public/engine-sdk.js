/**
 * Ring Engine — Mini-Game JS SDK
 *
 * Include this script in your mini-game's HTML page.
 * Once loaded, `window.engine` provides the API to communicate
 * with the host visual novel engine.
 *
 * Lifecycle:
 *   1. SDK loaded → listens for "engine:init" from parent
 *   2. Parent sends init message → engine._ready = true, "engine:ready" event fires
 *   3. Game reads params via engine.getInfo()
 *   4. Game completes → engine.complete(result)
 *
 * Usage:
 *   <script src="engine-sdk.js"><\/script>
 *   <script>
 *     window.addEventListener("engine:ready", () => {
 *       const info = engine.getInfo();
 *       // ... game logic ...
 *       engine.complete("win");
 *     });
 *   <\/script>
 */
(function () {
  "use strict";

  var engine = {
    _ready: false,
    _info: null,
    _assetBase: "",

    /** Whether the SDK has received init data from the host. */
    isReady: function () {
      return this._ready;
    },

    /** Returns { game_id, params } passed from the VN script. */
    getInfo: function () {
      return this._info;
    },

    /**
     * Signal that the game is complete and pass the result back.
     * @param {*} result — The value stored in the VN script variable.
     */
    complete: function (result) {
      parent.postMessage(
        { type: "engine:complete", result: result !== undefined ? result : "" },
        "*",
      );
    },

    /**
     * Resolve a logical path relative to the game directory into an accessible URL.
     * @param {string} path — Relative path (e.g. "sfx/click.mp3").
     * @returns {string} Full URL usable in <img>, <audio>, fetch, etc.
     */
    assetUrl: function (path) {
      return this._assetBase + "/" + path;
    },

    /**
     * Log a message to the host engine's logging system.
     * @param {string} level — "debug" | "info" | "warn" | "error"
     * @param {string} message
     */
    log: function (level, message) {
      parent.postMessage(
        { type: "engine:log", level: level, message: message },
        "*",
      );
    },
  };

  window.addEventListener("message", function (e) {
    if (e.data && e.data.type === "engine:init") {
      engine._info = e.data.info || null;
      engine._assetBase = e.data.assetBase || "";
      engine._ready = true;
      window.dispatchEvent(new Event("engine:ready"));
    }
  });

  window.engine = engine;
})();
