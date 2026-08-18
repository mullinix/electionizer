/** IndexedDB cache for HTTP bodies + last ballot. Entries persist until explicit reload. */
const DB_NAME = "electionizer-cache";
const STORE = "responses";
const DB_VERSION = 1;
/** 0 = never expire. Civic VIP still passes a finite TTL (Google guideline). */
export const DEFAULT_TTL_MS = 0;
const LAST_BALLOT_KEY = "electionizer:last-ballot";

let bypassCount = 0;

export function isCacheBypassed() {
  return bypassCount > 0;
}

/** Skip cacheGet for the duration of `fn` (writes still land). */
export async function withFreshCache(fn) {
  bypassCount += 1;
  try {
    return await fn();
  } finally {
    bypassCount -= 1;
  }
}

function openDb() {
  return new Promise((resolve, reject) => {
    const req = indexedDB.open(DB_NAME, DB_VERSION);
    req.onupgradeneeded = () => {
      const db = req.result;
      if (!db.objectStoreNames.contains(STORE)) {
        db.createObjectStore(STORE, { keyPath: "key" });
      }
    };
    req.onsuccess = () => resolve(req.result);
    req.onerror = () => reject(req.error);
  });
}

function civicKeyExpires(key) {
  return String(key).startsWith("civic:");
}

export async function cacheGet(key) {
  if (bypassCount > 0) return null;
  try {
    const db = await openDb();
    return await new Promise((resolve, reject) => {
      const tx = db.transaction(STORE, "readonly");
      const req = tx.objectStore(STORE).get(key);
      req.onsuccess = () => {
        const row = req.result;
        if (!row) return resolve(null);
        if (row.expiresAt && Date.now() > row.expiresAt && civicKeyExpires(key)) {
          cacheDelete(key);
          return resolve(null);
        }
        resolve(row.body);
      };
      req.onerror = () => reject(req.error);
    });
  } catch {
    return null;
  }
}

export async function cachePut(key, body, ttlMs = DEFAULT_TTL_MS) {
  try {
    const db = await openDb();
    await new Promise((resolve, reject) => {
      const tx = db.transaction(STORE, "readwrite");
      tx.objectStore(STORE).put({
        key,
        body,
        expiresAt: ttlMs > 0 ? Date.now() + ttlMs : 0,
        storedAt: Date.now(),
      });
      tx.oncomplete = () => resolve();
      tx.onerror = () => reject(tx.error);
    });
  } catch {
    /* ignore quota / private mode */
  }
}

export async function cacheDelete(key) {
  try {
    const db = await openDb();
    await new Promise((resolve, reject) => {
      const tx = db.transaction(STORE, "readwrite");
      tx.objectStore(STORE).delete(key);
      tx.oncomplete = () => resolve();
      tx.onerror = () => reject(tx.error);
    });
  } catch {
    /* ignore */
  }
}

function httpError(status, cacheKey, url) {
  let msg = `HTTP ${status}`;
  if (status === 429) {
    msg =
      "Rate limited (HTTP 429). Set a personal FEC API key in Settings — DEMO_KEY is ~40 requests/hour.";
  } else if (status === 403) {
    msg = `Access denied (HTTP 403) fetching ${cacheKey}. If this is chamber HTML, set a CORS proxy in Settings.`;
  } else if (status === 404) {
    msg = `Not found (HTTP 404): ${cacheKey}`;
  } else if (status >= 500) {
    msg = `Upstream error (HTTP ${status}) for ${cacheKey}. Try again shortly.`;
  } else {
    msg = `HTTP ${status} for ${cacheKey}`;
  }
  const err = new Error(msg);
  err.status = status;
  err.cacheKey = cacheKey;
  err.url = url;
  return err;
}

/** Fetch text with IndexedDB cache. */
export async function cachedFetch(url, { key, ttlMs, init } = {}) {
  const cacheKey = key || url;
  const hit = await cacheGet(cacheKey);
  if (hit != null) return { body: hit, cached: true };

  let res;
  try {
    res = await fetch(url, init);
  } catch (e) {
    const err = new Error(
      `Network error fetching ${cacheKey}: ${e.message || e}. Check connectivity or CORS proxy.`
    );
    err.cause = e;
    err.cacheKey = cacheKey;
    throw err;
  }
  if (!res.ok) throw httpError(res.status, cacheKey, url);
  const body = await res.text();
  await cachePut(cacheKey, body, ttlMs);
  return { body, cached: false };
}

/** Persist last successful ballot report (JSON string) for offline reopen. */
export async function saveLastBallot(zip, report) {
  const payload = JSON.stringify({
    zip,
    savedAt: Date.now(),
    report,
  });
  await cachePut(LAST_BALLOT_KEY, payload, 0);
}

export async function getLastBallot() {
  const raw = await cacheGet(LAST_BALLOT_KEY);
  if (!raw) return null;
  try {
    return JSON.parse(raw);
  } catch {
    return null;
  }
}

export async function cacheDeleteWhere(pred) {
  let n = 0;
  try {
    const db = await openDb();
    const keys = await new Promise((resolve, reject) => {
      const tx = db.transaction(STORE, "readonly");
      const req = tx.objectStore(STORE).getAllKeys();
      req.onsuccess = () => resolve(req.result || []);
      req.onerror = () => reject(req.error);
    });
    for (const key of keys) {
      if (pred(String(key))) {
        await cacheDelete(key);
        n += 1;
      }
    }
  } catch {
    /* ignore */
  }
  return n;
}

/** Drop verdict + L6 contrast cache for one subject. */
export async function deleteAiCacheForSubject({ id, name } = {}) {
  const idStr = id != null && String(id) !== "" ? String(id) : "";
  const nameStr = name ? String(name).toLowerCase() : "";
  if (!idStr && !nameStr) return 0;
  return cacheDeleteWhere((key) => {
    if (key.startsWith("verdict:")) {
      if (idStr && key.includes(`:${idStr}:`)) return true;
      if (nameStr && key.toLowerCase().includes(`:${nameStr}:`)) return true;
    }
    if (key.startsWith("llm-contrast:") && nameStr && key.toLowerCase().includes(`:${nameStr}:`)) {
      return true;
    }
    return false;
  });
}

/**
 * Clear IndexedDB response cache + last ballot.
 * Does not touch localStorage credentials/settings (FEC/OS/Wisp/CORS/theme/mode).
 * @returns {{ cleared: number }}
 */
export async function clearBallotAndResponseCache() {
  let cleared = 0;
  try {
    const db = await openDb();
    cleared = await new Promise((resolve, reject) => {
      const tx = db.transaction(STORE, "readwrite");
      const store = tx.objectStore(STORE);
      const countReq = store.count();
      countReq.onsuccess = () => {
        const n = countReq.result || 0;
        const clearReq = store.clear();
        clearReq.onsuccess = () => resolve(n);
        clearReq.onerror = () => reject(clearReq.error);
      };
      countReq.onerror = () => reject(countReq.error);
    });
  } catch {
    /* ignore */
  }
  return { cleared };
}
