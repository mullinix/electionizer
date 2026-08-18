const FEC_KEY = "electionizer-fec-api-key";
const OS_KEY = "electionizer-openstates-api-key";
const FTM_KEY = "electionizer-ftm-api-key";
const CL_KEY = "electionizer-courtlistener-token";
const CIVIC_KEY = "electionizer-civic-api-key";
const MODE_KEY = "electionizer-mode"; // "live" | "fixture"
const CYCLE_KEY = "electionizer-cycle";
const CORS_PROXY_KEY = "electionizer-cors-proxy";
const WISP_URL_KEY = "electionizer-wisp-url";
const FL_DOS_TSV_KEY = "electionizer-fl-dos-tsv";
const FL_DOS_TSV_META = "electionizer-fl-dos-tsv-meta";
const PRECINCT_KEY = "electionizer-voter-precinct";
const VOTER_PARTY_KEY = "electionizer-voter-party";
const LLM_KEY = "electionizer-llm-api-key";
const LLM_PROVIDER_KEY = "electionizer-llm-provider";
const LLM_MODEL_KEY = "electionizer-llm-model";
const SCORE_CONCURRENCY_KEY = "electionizer-score-concurrency";

export function getFecApiKey() {
  try {
    return localStorage.getItem(FEC_KEY) || "DEMO_KEY";
  } catch {
    return "DEMO_KEY";
  }
}

export function setFecApiKey(key) {
  const v = (key || "").trim();
  try {
    if (!v || v === "DEMO_KEY") localStorage.removeItem(FEC_KEY);
    else localStorage.setItem(FEC_KEY, v);
  } catch {
    /* ignore */
  }
}

export function getOpenStatesApiKey() {
  try {
    return localStorage.getItem(OS_KEY) || "";
  } catch {
    return "";
  }
}

export function setOpenStatesApiKey(key) {
  const v = (key || "").trim();
  try {
    if (!v) localStorage.removeItem(OS_KEY);
    else localStorage.setItem(OS_KEY, v);
  } catch {
    /* ignore */
  }
}

export function hasOpenStatesKey() {
  return !!getOpenStatesApiKey();
}

/** FollowTheMoney / OpenSecrets state CF API key (free myFollowTheMoney account). */
export function getFtmApiKey() {
  try {
    return localStorage.getItem(FTM_KEY) || "";
  } catch {
    return "";
  }
}

export function setFtmApiKey(key) {
  const v = (key || "").trim();
  try {
    if (!v) localStorage.removeItem(FTM_KEY);
    else localStorage.setItem(FTM_KEY, v);
  } catch {
    /* ignore */
  }
}

export function hasFtmKey() {
  return !!getFtmApiKey();
}

export function getCourtListenerToken() {
  try {
    return localStorage.getItem(CL_KEY) || "";
  } catch {
    return "";
  }
}

export function setCourtListenerToken(key) {
  try {
    const v = (key || "").trim();
    if (!v) localStorage.removeItem(CL_KEY);
    else localStorage.setItem(CL_KEY, v);
  } catch {
    /* ignore */
  }
}

export function hasCourtListenerToken() {
  return !!getCourtListenerToken();
}

export function getCivicApiKey() {
  try {
    return (localStorage.getItem(CIVIC_KEY) || "").trim();
  } catch {
    return "";
  }
}

export function setCivicApiKey(key) {
  try {
    const t = String(key || "").trim();
    if (t) localStorage.setItem(CIVIC_KEY, t);
    else localStorage.removeItem(CIVIC_KEY);
  } catch {
    /* ignore */
  }
}

export function hasCivicKey() {
  return !!getCivicApiKey();
}

/** Optional CORS proxy prefix, e.g. `https://corsproxy.io/?` */
export function getCorsProxy() {
  try {
    return (localStorage.getItem(CORS_PROXY_KEY) || "").trim();
  } catch {
    return "";
  }
}

export function setCorsProxy(url) {
  try {
    const v = (url || "").trim();
    if (!v) localStorage.removeItem(CORS_PROXY_KEY);
    else localStorage.setItem(CORS_PROXY_KEY, v);
  } catch {
    /* ignore */
  }
}

/** Apply CORS proxy to an absolute URL when configured. */
export function proxiedUrl(url) {
  const p = getCorsProxy();
  if (!p) return url;
  if (p.includes("?") || p.endsWith("=")) {
    return p + encodeURIComponent(url);
  }
  if (p.endsWith("/")) return p + url.replace(/^https?:\/\//, "");
  return p + url;
}

/** Public default Wisp endpoint (overridable in Settings). */
export const DEFAULT_WISP_URL = "wss://wisp.mercurywork.shop/";

/**
 * Wisp WebSocket URL for libcurl.js (E2E TLS tunnel).
 * Defaults to Mercury Workshop public instance; clear/override in Settings.
 */
export function getWispUrl() {
  try {
    const v = localStorage.getItem(WISP_URL_KEY);
    // null = never set → default; "" = user cleared → off
    if (v === null) return DEFAULT_WISP_URL;
    return v.trim();
  } catch {
    return DEFAULT_WISP_URL;
  }
}

export function setWispUrl(url) {
  try {
    const v = (url || "").trim();
    // Always persist so we can distinguish "cleared" from "default"
    localStorage.setItem(WISP_URL_KEY, v);
  } catch {
    /* ignore */
  }
}

/** True when using the built-in default (not a custom/cleared value). */
export function isDefaultWispUrl() {
  try {
    return localStorage.getItem(WISP_URL_KEY) === null;
  } catch {
    return true;
  }
}

/** Forget override so getWispUrl() returns DEFAULT_WISP_URL again. */
export function resetWispUrlToDefault() {
  try {
    localStorage.removeItem(WISP_URL_KEY);
  } catch {
    /* ignore */
  }
}

export function getFlDosTsv() {
  try {
    return localStorage.getItem(FL_DOS_TSV_KEY) || "";
  } catch {
    return "";
  }
}

export function setFlDosTsv(text, meta = "") {
  try {
    const t = text || "";
    if (!t) {
      localStorage.removeItem(FL_DOS_TSV_KEY);
      localStorage.removeItem(FL_DOS_TSV_META);
    } else {
      localStorage.setItem(FL_DOS_TSV_KEY, t);
      localStorage.setItem(
        FL_DOS_TSV_META,
        meta || `${t.length} bytes · ${new Date().toISOString()}`
      );
    }
  } catch (e) {
    throw new Error(
      "Could not store FL DOS TSV in localStorage (file may be too large). Try a smaller extract or CORS proxy."
    );
  }
}

export function getFlDosTsvMeta() {
  try {
    return localStorage.getItem(FL_DOS_TSV_META) || "";
  } catch {
    return "";
  }
}

export function getMode() {
  try {
    const m = localStorage.getItem(MODE_KEY);
    return m === "fixture" ? "fixture" : "live";
  } catch {
    return "live";
  }
}

export function setMode(mode) {
  try {
    localStorage.setItem(MODE_KEY, mode === "fixture" ? "fixture" : "live");
  } catch {
    /* ignore */
  }
}

export function getCycle() {
  try {
    const n = parseInt(localStorage.getItem(CYCLE_KEY) || "2026", 10);
    return Number.isFinite(n) && n >= 2000 && n <= 2100 ? n : 2026;
  } catch {
    return 2026;
  }
}

export function setCycle(cycle) {
  try {
    localStorage.setItem(CYCLE_KEY, String(cycle));
  } catch {
    /* ignore */
  }
}

export function getVoterPrecinct() {
  try {
    return (localStorage.getItem(PRECINCT_KEY) || "").trim();
  } catch {
    return "";
  }
}

export function setVoterPrecinct(precinct) {
  try {
    const v = String(precinct || "").trim();
    if (!v) localStorage.removeItem(PRECINCT_KEY);
    else localStorage.setItem(PRECINCT_KEY, v);
  } catch {
    /* ignore */
  }
}

/** `Rep` | `Dem` | `Non` | raw user string */
export function getVoterParty() {
  try {
    return (localStorage.getItem(VOTER_PARTY_KEY) || "").trim();
  } catch {
    return "";
  }
}

export function setVoterParty(party) {
  try {
    const v = String(party || "").trim();
    if (!v) localStorage.removeItem(VOTER_PARTY_KEY);
    else localStorage.setItem(VOTER_PARTY_KEY, v);
  } catch {
    /* ignore */
  }
}

export function getLlmProvider() {
  try {
    const v = (localStorage.getItem(LLM_PROVIDER_KEY) || "xai").trim().toLowerCase();
    if (v === "openai" || v === "xai") return v;
    return "xai";
  } catch {
    return "xai";
  }
}

export function setLlmProvider(provider) {
  try {
    const v = String(provider || "").trim().toLowerCase();
    if (v === "openai") localStorage.setItem(LLM_PROVIDER_KEY, "openai");
    else if (v === "xai" || v === "grok") localStorage.setItem(LLM_PROVIDER_KEY, "xai");
    else localStorage.removeItem(LLM_PROVIDER_KEY);
  } catch {
    /* ignore */
  }
}

export function getLlmApiKey() {
  try {
    return (localStorage.getItem(LLM_KEY) || "").trim();
  } catch {
    return "";
  }
}

export function setLlmApiKey(key) {
  try {
    const v = String(key || "").trim();
    if (!v) localStorage.removeItem(LLM_KEY);
    else localStorage.setItem(LLM_KEY, v);
  } catch {
    /* ignore */
  }
}

export function hasLlmKey() {
  return !!getLlmApiKey();
}

export function getLlmModel() {
  try {
    return (localStorage.getItem(LLM_MODEL_KEY) || "").trim();
  } catch {
    return "";
  }
}

const VOTER_PROFILE_KEY = "electionizer-voter-profile";

export function getVoterProfile() {
  try {
    const raw = localStorage.getItem(VOTER_PROFILE_KEY);
    if (!raw) return {};
    const o = JSON.parse(raw);
    if (!o || typeof o !== "object") return {};
    const out = {};
    for (const [k, v] of Object.entries(o)) {
      const n = Number(v);
      if (k && n >= 1 && n <= 5) out[String(k).toLowerCase()] = n;
    }
    return out;
  } catch {
    return {};
  }
}

export function setVoterProfile(profile) {
  const out = {};
  for (const [k, v] of Object.entries(profile || {})) {
    const n = Number(v);
    if (k && n >= 1 && n <= 5) out[String(k).toLowerCase()] = n;
  }
  try {
    if (Object.keys(out).length) localStorage.setItem(VOTER_PROFILE_KEY, JSON.stringify(out));
    else localStorage.removeItem(VOTER_PROFILE_KEY);
  } catch {
    /* ignore */
  }
  return out;
}

export function setVoterPref(id, likert) {
  const p = getVoterProfile();
  const key = String(id || "").trim().toLowerCase();
  if (!key) return p;
  const n = Number(likert);
  if (n >= 1 && n <= 5) p[key] = n;
  else delete p[key];
  return setVoterProfile(p);
}

export function clearVoterProfile() {
  return setVoterProfile({});
}

export function voterProfileFingerprint(profile) {
  const p = profile || getVoterProfile();
  const keys = Object.keys(p).sort();
  if (!keys.length) return "none";
  return keys.map((k) => `${k}${p[k]}`).join(".");
}

export function getScoreConcurrency() {
  try {
    const n = parseInt(localStorage.getItem(SCORE_CONCURRENCY_KEY) || "3", 10);
    if (!Number.isFinite(n)) return 3;
    return Math.max(1, Math.min(8, n));
  } catch {
    return 3;
  }
}

export function setScoreConcurrency(n) {
  const v = Math.max(1, Math.min(8, parseInt(n, 10) || 3));
  try {
    localStorage.setItem(SCORE_CONCURRENCY_KEY, String(v));
  } catch {
    /* ignore */
  }
  return v;
}

export function setLlmModel(model) {
  try {
    const v = String(model || "").trim();
    if (!v) localStorage.removeItem(LLM_MODEL_KEY);
    else localStorage.setItem(LLM_MODEL_KEY, v);
  } catch {
    /* ignore */
  }
}
