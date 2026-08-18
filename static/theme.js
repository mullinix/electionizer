(function () {
  var KEY = "electionizer-theme";
  var THEMES = [
    "sage",
    "midnight",
    "amber",
    "bipartisan",
    "violet",
    "rose",
    "ice",
    "phosphor",
  ];
  var DEFAULT = "sage";

  function valid(id) {
    return THEMES.indexOf(id) !== -1 ? id : DEFAULT;
  }

  function readCookie() {
    var m = document.cookie.match(/(?:^|;\s*)electionizer-theme=([^;]+)/);
    return m ? decodeURIComponent(m[1]) : null;
  }

  function writeCookie(id) {
    document.cookie =
      KEY +
      "=" +
      encodeURIComponent(id) +
      "; path=/; max-age=31536000; SameSite=Lax";
  }

  function getTheme() {
    try {
      var ls = localStorage.getItem(KEY);
      if (ls) return valid(ls);
    } catch (_) {}
    var c = readCookie();
    if (c) return valid(c);
    return DEFAULT;
  }

  function applyTheme(id) {
    id = valid(id);
    document.documentElement.setAttribute("data-theme", id);
    return id;
  }

  function setTheme(id) {
    id = applyTheme(id);
    try {
      localStorage.setItem(KEY, id);
    } catch (_) {}
    writeCookie(id);
    syncPicker(id);
    return id;
  }

  function syncPicker(id) {
    var nodes = document.querySelectorAll("[data-theme-id]");
    for (var i = 0; i < nodes.length; i++) {
      var el = nodes[i];
      var on = el.getAttribute("data-theme-id") === id;
      el.classList.toggle("is-active", on);
      el.setAttribute("aria-checked", on ? "true" : "false");
    }
  }

  function initPicker() {
    var root = document.querySelector("[data-theme-picker]");
    if (!root) return;
    root.addEventListener("click", function (e) {
      var btn = e.target.closest("[data-theme-id]");
      if (!btn) return;
      setTheme(btn.getAttribute("data-theme-id"));
    });
    syncPicker(getTheme());
  }

  var HEX_R = 12;
  var HEX_W = HEX_R * Math.sqrt(3);
  var HEX_H = HEX_R * 3;
  var HEX_MAX = 14;

  function snapToHex(x, y) {
    var i0 = Math.round((x - HEX_W / 2) / HEX_W);
    var j0 = Math.round((y - HEX_R) / HEX_H);
    var e = { x: i0 * HEX_W + HEX_W / 2, y: j0 * HEX_H + HEX_R };
    var i1 = Math.round(x / HEX_W);
    var j1 = Math.round((y - HEX_R * 2.5) / HEX_H);
    var o = { x: i1 * HEX_W, y: j1 * HEX_H + HEX_R * 2.5 };
    var de = (e.x - x) * (e.x - x) + (e.y - y) * (e.y - y);
    var d0 = (o.x - x) * (o.x - x) + (o.y - y) * (o.y - y);
    return de <= d0 ? e : o;
  }

  function viewportToLocal(grid, vx, vy) {
    var tr = getComputedStyle(grid).transform;
    var m = tr && tr !== "none" ? new DOMMatrix(tr) : new DOMMatrix();
    var p = m.inverse().transformPoint({
      x: vx - window.innerWidth / 2,
      y: vy - window.innerHeight / 2,
    });
    return { x: p.x + grid.clientWidth / 2, y: p.y + grid.clientHeight / 2 };
  }

  function randomVisibleCell(grid) {
    var local = viewportToLocal(
      grid,
      Math.random() * window.innerWidth,
      Math.random() * window.innerHeight
    );
    return snapToHex(local.x, local.y);
  }

  function hexPoints(cx, cy, r) {
    var pts = [];
    for (var k = 0; k < 6; k++) {
      var a = -Math.PI / 2 + (k * Math.PI) / 3;
      pts.push(
        (cx + r * Math.cos(a)).toFixed(3) + "," + (cy + r * Math.sin(a)).toFixed(3)
      );
    }
    return pts.join(" ");
  }

  function initHexGlow() {
    var grid = document.querySelector(".hex-grid");
    if (!grid) return;
    var NS = "http://www.w3.org/2000/svg";
    var rings = [0.97, 0.76, 0.56, 0.38, 0.22];
    var peaks = [0.95, 0.68, 0.42, 0.22, 0.08];
    var widths = [1.4, 1.15, 0.95, 0.8, 0.65];

    function spawnOne() {
      if (grid.querySelectorAll(".hex-pulse").length >= HEX_MAX) return;
      if (grid.clientWidth < 2 || grid.clientHeight < 2) return;
      var cell = randomVisibleCell(grid);
      var dur = 0.7 + Math.random() * 0.9;
      var beats = 1 + Math.floor(Math.random() * 3);
      var svg = document.createElementNS(NS, "svg");
      svg.setAttribute("class", "hex-pulse");
      svg.setAttribute("viewBox", "0 0 " + HEX_W + " " + HEX_R * 2);
      svg.style.left = cell.x - HEX_W / 2 + "px";
      svg.style.top = cell.y - HEX_R + "px";
      svg.style.setProperty("--dur", dur + "s");
      svg.style.setProperty("--beats", String(beats));
      var cx = HEX_W / 2;
      var cy = HEX_R;
      for (var i = 0; i < rings.length; i++) {
        var poly = document.createElementNS(NS, "polygon");
        poly.setAttribute("points", hexPoints(cx, cy, HEX_R * rings[i]));
        poly.setAttribute("stroke-width", String(widths[i]));
        poly.style.setProperty("--i", String(i));
        poly.style.setProperty("--peak", String(peaks[i]));
        svg.appendChild(poly);
      }
      grid.appendChild(svg);
      window.setTimeout(function () {
        if (svg.parentNode) svg.parentNode.removeChild(svg);
      }, (dur * beats + dur * 0.12 * (rings.length - 1)) * 1000 + 80);
    }

    function scheduleSpawn() {
      window.setTimeout(function () {
        if (!document.hidden) {
          spawnOne();
          if (Math.random() < 0.28) spawnOne();
        }
        scheduleSpawn();
      }, 160 + Math.random() * 1100);
    }

    for (var n = 0; n < 6; n++) spawnOne();
    scheduleSpawn();
  }

  window.ElectionizerTheme = {
    themes: THEMES,
    get: getTheme,
    set: setTheme,
    apply: applyTheme,
  };

  applyTheme(getTheme());

  function boot() {
    initPicker();
    initHexGlow();
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", boot);
  } else {
    boot();
  }
})();
