// Runs before first paint: applies the cached appearance so the page never
// flashes the wrong theme. The cache key and the allowed values are pinned to
// src/app/theme/apply.ts by appearance-boot.test.ts.
(function () {
  var CACHE_KEY = "vitarchy.appearance";
  var THEMES = ["system", "light", "dark"];
  var MOTIONS = ["system", "reduced", "full"];

  var theme = "system";
  var motion = "system";
  try {
    var cached = JSON.parse(localStorage.getItem(CACHE_KEY) || "null");
    if (cached && typeof cached === "object" && !Array.isArray(cached)) {
      // Own, known values only: a corrupt cache must never reach the DOM.
      if (Object.prototype.hasOwnProperty.call(cached, "theme") && THEMES.indexOf(cached.theme) !== -1) {
        theme = cached.theme;
      }
      if (Object.prototype.hasOwnProperty.call(cached, "motion") && MOTIONS.indexOf(cached.motion) !== -1) {
        motion = cached.motion;
      }
    }
  } catch (e) {
    // No usable cache: fall back to following the system.
  }
  var dark = false;
  try {
    dark = window.matchMedia("(prefers-color-scheme: dark)").matches;
  } catch (e) {
    // No matchMedia: the design's home theme is dark.
    dark = true;
  }
  var resolved = theme === "system" ? (dark ? "dark" : "light") : theme;
  var root = document.documentElement;
  root.dataset.theme = resolved;
  root.dataset.motion = motion;
  root.style.colorScheme = resolved;
})();
