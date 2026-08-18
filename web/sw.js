/* electionizer static shell + fixture offline cache */
const CACHE = "electionizer-shell-v50";
const PRECACHE = [
  "./",
  "./index.html",
  "./app.css",
  "./app.js",
  "./cache.js",
  "./detail-lists.js",
  "./enrich.js",
  "./verdict.js",
  "./scoreboard.js",
  "./live.js",
  "./render.js",
  "./timeline.js",
  "./settings.js",
  "./voter-profile-defaults.json",
  "./state.js",
  "./state-urls.js",
  "./curl-transport.js",
  "./theme.js",
  "./fixtures/fixture_90210.json",
  // WASM required for fixture mode offline
  "./pkg/electionizer_wasm.js",
  "./pkg/electionizer_wasm_bg.wasm",
];

async function precacheAll(cache) {
  // Prefer individual puts so one failure does not abort the whole install.
  await Promise.all(
    PRECACHE.map(async (url) => {
      try {
        const res = await fetch(url, { cache: "reload" });
        if (res && res.ok) await cache.put(url, res);
      } catch {
        /* optional asset */
      }
    })
  );
}

self.addEventListener("install", (event) => {
  event.waitUntil(
    caches
      .open(CACHE)
      .then((cache) => precacheAll(cache))
      .then(() => self.skipWaiting())
      .catch(() => self.skipWaiting())
  );
});

self.addEventListener("activate", (event) => {
  event.waitUntil(
    caches
      .keys()
      .then((keys) =>
        Promise.all(keys.filter((k) => k !== CACHE).map((k) => caches.delete(k)))
      )
      .then(() => self.clients.claim())
  );
});

self.addEventListener("fetch", (event) => {
  const req = event.request;
  if (req.method !== "GET") return;

  const url = new URL(req.url);
  // Same-origin app shell + wasm pkg + data only
  if (url.origin !== self.location.origin) return;

  event.respondWith(
    caches.open(CACHE).then(async (cache) => {
      const cached = await cache.match(req);
      if (cached) {
        // Stale-while-revalidate for shell assets
        event.waitUntil(
          fetch(req)
            .then((res) => {
              if (res && res.ok) cache.put(req, res.clone());
            })
            .catch(() => {})
        );
        return cached;
      }
      try {
        const res = await fetch(req);
        if (res && res.ok) {
          const path = url.pathname;
          if (
            path.endsWith(".js") ||
            path.endsWith(".css") ||
            path.endsWith(".html") ||
            path.endsWith(".json") ||
            path.endsWith(".wasm") ||
            path.endsWith(".tsv") ||
            path.endsWith("/")
          ) {
            cache.put(req, res.clone());
          }
        }
        return res;
      } catch (e) {
        // Offline: try path variants for navigation / module requests
        const fallbacks = [
          req,
          url.pathname,
          url.pathname.replace(/^\//, "./"),
          "./index.html",
        ];
        for (const key of fallbacks) {
          const hit = await cache.match(key);
          if (hit) return hit;
        }
        if (req.mode === "navigate") {
          const index = await cache.match("./index.html");
          if (index) return index;
        }
        throw e;
      }
    })
  );
});
