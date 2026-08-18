/**
 * Optional libcurl.js transport (Wisp WebSocket proxy).
 * E2E TLS to origin — proxy cannot read request/response bodies.
 * Used for CORS-blocked state sources (AZ/FL chambers, FL measures).
 */
import { getWispUrl } from "./settings.js";

const LIBCURL_VER = "0.7.4";
const LIBCURL_BASE = `https://cdn.jsdelivr.net/npm/libcurl.js@${LIBCURL_VER}`;

/** @type {null | { ready: boolean, fetch: Function, HTTPSession: Function, set_websocket: Function }} */
let libcurl = null;
let loadPromise = null;
let lastWisp = "";

export function hasWispConfigured() {
  // Empty string = user disabled; default Mercury URL counts as configured
  return !!getWispUrl();
}

export function isCurlReady() {
  return !!(libcurl && libcurl.ready);
}

/**
 * Load libcurl.js WASM + point it at the configured Wisp URL.
 * No-op / resolves false when Wisp URL unset.
 * @returns {Promise<boolean>}
 */
export async function ensureCurl() {
  const wisp = getWispUrl();
  if (!wisp) return false;

  if (!loadPromise) {
    loadPromise = (async () => {
      const mod = await import(`${LIBCURL_BASE}/libcurl.mjs`);
      const lc = mod.libcurl || mod.default?.libcurl || mod.default;
      if (!lc) throw new Error("libcurl.js module missing libcurl export");
      await lc.load_wasm(`${LIBCURL_BASE}/libcurl.wasm`);
      libcurl = lc;
      return lc;
    })().catch((e) => {
      loadPromise = null;
      throw e;
    });
  }

  const lc = await loadPromise;
  if (wisp !== lastWisp) {
    // Must end with trailing slash per libcurl.js docs
    const url = wisp.endsWith("/") ? wisp : wisp + "/";
    lc.set_websocket(url);
    lastWisp = wisp;
  }
  return !!(lc && lc.ready);
}

/**
 * libcurl.js writes CURLOPT_COOKIEJAR only on easy-handle cleanup, and
 * HTTPSession.remove_request defers that via setTimeout(1). Starting the next
 * request in the same turn races the jar flush — e.g. eFD agree Set-Cookie
 * sessionid never seen by the report GET → redirect to /search/home/.
 * Call after the response body is fully consumed.
 * @param {object | null | undefined} session
 */
export async function settleSessionCookies(session) {
  if (!session) return;
  // Two macrotasks: one for remove_request's setTimeout(1), one buffer.
  await new Promise((r) => setTimeout(r, 5));
  try {
    if (typeof session.export_cookies === "function") {
      session.export_cookies();
    }
  } catch {
    /* ignore */
  }
}

/**
 * @param {object | null | undefined} session
 * @param {string} url
 * @param {RequestInit & Record<string, unknown>} init
 */
async function rawFetch(session, url, init) {
  return session ? session.fetch(url, init) : libcurl.fetch(url, init);
}

/**
 * Fetch + read text + flush session cookie jar.
 * @param {object | null | undefined} session
 * @param {string} url
 * @param {RequestInit & Record<string, unknown>} init
 */
async function fetchTextSettled(session, url, init) {
  const res = await rawFetch(session, url, init);
  let text = "";
  try {
    text = await res.text();
  } catch {
    text = "";
  }
  await settleSessionCookies(session);
  return { res, text };
}

/**
 * GET absolute URL via libcurl (no CORS). Returns response text.
 * @param {string} url
 * @param {{ headers?: Record<string,string>, session?: object, redirect?: string }} [opts]
 */
export async function curlFetchText(url, opts = {}) {
  const ok = await ensureCurl();
  if (!ok || !libcurl) throw new Error("libcurl.js not ready (set Wisp URL in Settings)");

  const init = {
    method: "GET",
    headers: opts.headers || {},
  };
  if (opts.redirect) init.redirect = opts.redirect;

  const { res, text } = await fetchTextSettled(opts.session, url, init);
  if (!res.ok) {
    const err = new Error(`HTTP ${res.status} via libcurl for ${url}`);
    err.status = res.status;
    err.location = resolveUrl(url, responseLocation(res));
    err.body = text;
    throw err;
  }
  return text;
}

/**
 * Resolve relative Location against request URL.
 * @param {string} baseUrl
 * @param {string} location
 */
function resolveUrl(baseUrl, location) {
  const loc = (location || "").trim();
  if (!loc) return "";
  try {
    return new URL(loc, baseUrl).href;
  } catch {
    return loc;
  }
}

/**
 * Read Location header from a libcurl/fetch Response (header API varies).
 * @param {any} res
 */
function responseLocation(res) {
  try {
    if (res && res.headers) {
      if (typeof res.headers.get === "function") {
        return res.headers.get("Location") || res.headers.get("location") || "";
      }
      if (typeof res.headers === "object") {
        return res.headers.Location || res.headers.location || "";
      }
    }
  } catch {
    /* ignore */
  }
  try {
    const raw = res && res.raw_headers;
    if (Array.isArray(raw)) {
      for (const pair of raw) {
        if (!pair || pair.length < 2) continue;
        if (String(pair[0]).toLowerCase() === "location") return String(pair[1] || "");
      }
    }
  } catch {
    /* ignore */
  }
  return "";
}

/**
 * POST application/x-www-form-urlencoded via libcurl.
 * Prefer a cookie-enabled session for ASP.NET antiforgery / Django CSRF.
 *
 * Uses redirect:"manual" then one GET follow so Set-Cookie on 302 (e.g.
 * efdsearch.senate.gov agree → sessionid) is jar-flushed before the next hop.
 * Default libcurl FOLLOWLOCATION + CUSTOMREQUEST POST can also 411 on the
 * redirected GET when the body is replayed.
 */
export async function curlPostForm(url, formBody, opts = {}) {
  const ok = await ensureCurl();
  if (!ok || !libcurl) throw new Error("libcurl.js not ready (set Wisp URL in Settings)");

  const headers = {
    "Content-Type": "application/x-www-form-urlencoded",
    ...(opts.headers || {}),
  };
  const init = {
    method: "POST",
    headers,
    body: formBody,
    redirect: "manual",
  };

  let { res, text } = await fetchTextSettled(opts.session, url, init);

  // POST → 302/303 with empty body: cookies are set; GET the Location.
  if (res.status >= 300 && res.status < 400) {
    const next = resolveUrl(url, responseLocation(res));
    if (next) {
      ({ res, text } = await fetchTextSettled(opts.session, next, {
        method: "GET",
        headers: opts.headers || {},
        redirect: "manual",
      }));
      // Rare second hop (trailing slash etc.)
      if (res.status >= 300 && res.status < 400) {
        const next2 = resolveUrl(next, responseLocation(res));
        if (next2) {
          ({ res, text } = await fetchTextSettled(opts.session, next2, {
            method: "GET",
            headers: opts.headers || {},
          }));
        }
      }
    }
  }
  if (!res.ok) {
    const err = new Error(`HTTP ${res.status} POST via libcurl for ${url}`);
    err.status = res.status;
    throw err;
  }
  return text;
}

/** POST application/json via libcurl (e.g. MDCRIS public API). */
export async function curlPostJson(url, jsonBody, opts = {}) {
  const ok = await ensureCurl();
  if (!ok || !libcurl) throw new Error("libcurl.js not ready (set Wisp URL in Settings)");

  const init = {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Accept: "application/json, text/plain, */*",
      ...(opts.headers || {}),
    },
    body: typeof jsonBody === "string" ? jsonBody : JSON.stringify(jsonBody),
  };
  const { res, text } = await fetchTextSettled(opts.session, url, init);
  if (!res.ok) {
    const snippet = String(text || "")
      .replace(/\s+/g, " ")
      .slice(0, 180);
    const err = new Error(
      snippet
        ? `HTTP ${res.status}: ${snippet}`
        : `HTTP ${res.status} POST JSON via libcurl`
    );
    err.status = res.status;
    err.body = text;
    throw err;
  }
  return text;
}

/**
 * Low-level libcurl fetch (any method). Returns { ok, status, text, location, url }.
 * Does not auto-follow redirects (caller may GET location).
 */
export async function curlRequest(url, init = {}, session = null) {
  const ok = await ensureCurl();
  if (!ok || !libcurl) throw new Error("libcurl.js not ready (set Wisp URL in Settings)");
  const reqInit = { ...init, redirect: init.redirect || "manual" };
  const { res, text } = await fetchTextSettled(session, url, reqInit);
  return {
    ok: res.ok,
    status: res.status,
    text,
    location: resolveUrl(url, responseLocation(res)),
    url: res.url || url,
  };
}

/**
 * GET binary body via libcurl (PDF etc.). Returns Uint8Array.
 * @param {string} url
 * @param {{ headers?: Record<string,string>, session?: object }} [opts]
 */
export async function curlFetchBytes(url, opts = {}) {
  const ok = await ensureCurl();
  if (!ok || !libcurl) throw new Error("libcurl.js not ready (set Wisp URL in Settings)");
  const init = {
    method: "GET",
    headers: opts.headers || {},
  };
  const res = await rawFetch(opts.session, url, init);
  let bytes;
  if (typeof res.arrayBuffer === "function") {
    bytes = new Uint8Array(await res.arrayBuffer());
  } else {
    // Fallback: text → bytes (may corrupt binary; prefer arrayBuffer)
    const t = await res.text();
    bytes = new Uint8Array(t.length);
    for (let i = 0; i < t.length; i++) bytes[i] = t.charCodeAt(i) & 0xff;
  }
  await settleSessionCookies(opts.session);
  if (!res.ok) {
    const err = new Error(`HTTP ${res.status} via libcurl for ${url}`);
    err.status = res.status;
    throw err;
  }
  return bytes;
}

/** Cookie-jar session for multi-step scrapes (e.g. FL measures GET+POST). */
export async function curlSession() {
  const ok = await ensureCurl();
  if (!ok || !libcurl) throw new Error("libcurl.js not ready");
  return new libcurl.HTTPSession({ enable_cookies: true });
}

/**
 * True if session cookie jar has a non-empty named cookie.
 * @param {object | null | undefined} session
 * @param {string} [name]
 */
export function sessionHasCookie(session, name = "sessionid") {
  if (!session || typeof session.export_cookies !== "function") return false;
  try {
    const jar = session.export_cookies() || "";
    const re = new RegExp(
      `(?:^|\\n)[^\\n]*\\t${name.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}\\t([^\\n\\t]+)`,
      "i"
    );
    const m = jar.match(re);
    return !!(m && m[1] && m[1].trim());
  } catch {
    return false;
  }
}

export function curlCloseSession(session) {
  try {
    if (session && typeof session.close === "function") session.close();
  } catch {
    /* ignore */
  }
}
