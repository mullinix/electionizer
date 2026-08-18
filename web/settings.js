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
const VOTER_PROFILE_AXES_KEY = "electionizer-voter-profile-axes";
const VOTER_PROFILE_META_KEY = "electionizer-voter-profile-meta";
export const VOTER_PROFILE_KIND = "electionizer-voter-profile";
const DEFAULT_VOTER_PROFILE_URL = "./voter-profile-defaults.json";

let defaultProfile = null;
let defaultProfilePromise = null;

export function loadDefaultVoterProfile() {
  if (defaultProfile) return Promise.resolve(defaultProfile);
  if (!defaultProfilePromise) {
    defaultProfilePromise = fetch(DEFAULT_VOTER_PROFILE_URL)
      .then((r) => {
        if (!r || !r.ok) throw new Error("defaults missing");
        return r.json();
      })
      .then((v) => {
        defaultProfile = v;
        return v;
      });
  }
  return defaultProfilePromise;
}

function slugAxisId(raw) {
  let s = String(raw || "")
    .normalize("NFKD")
    .replace(/[\u0300-\u036f]/g, "")
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "_")
    .replace(/^_+|_+$/g, "")
    .slice(0, 40);
  if (!s) s = "issue";
  if (/^[0-9]/.test(s)) s = `x_${s}`;
  return s;
}

function uniqAxisId(want, taken) {
  let id = want;
  let n = 2;
  while (taken.has(id)) {
    id = `${want}_${n}`.slice(0, 48);
    n += 1;
  }
  return id;
}

export function normalizeCustomAxis(raw, taken) {
  if (!raw || typeof raw !== "object") return null;
  const label = String(raw.label || raw.name || raw.id || "")
    .trim()
    .slice(0, 80);
  if (!label) return null;
  const takenSet = taken instanceof Set ? taken : new Set(taken || []);
  let id = slugAxisId(raw.id || label);
  if (takenSet.has(id) && slugAxisId(label) !== id) id = slugAxisId(label);
  if (takenSet.has(id)) id = uniqAxisId(id.startsWith("custom_") ? id : `custom_${id}`, takenSet);
  const group = String(raw.group || "Custom").trim().slice(0, 40) || "Custom";
  const low = String(raw.low_label || raw.low || "Disagree").trim().slice(0, 24) || "Disagree";
  const high = String(raw.high_label || raw.high || "Agree").trim().slice(0, 24) || "Agree";
  return {
    id,
    label,
    definition: String(raw.definition || raw.def || "").trim().slice(0, 400),
    group,
    low_label: low,
    high_label: high,
    custom: true,
  };
}

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

export function getCustomProfileAxes() {
  try {
    const raw = localStorage.getItem(VOTER_PROFILE_AXES_KEY);
    if (!raw) return [];
    const v = JSON.parse(raw);
    const rows = Array.isArray(v) ? v : v && Array.isArray(v.axes) ? v.axes : [];
    const out = [];
    const taken = new Set();
    for (const row of rows) {
      const a = normalizeCustomAxis(row, taken);
      if (!a) continue;
      taken.add(a.id);
      out.push(a);
    }
    return out;
  } catch {
    return [];
  }
}

function emptyProfileMeta() {
  return { overrides: {}, hidden: [] };
}

export function getProfileMeta() {
  try {
    const raw = localStorage.getItem(VOTER_PROFILE_META_KEY);
    if (!raw) return emptyProfileMeta();
    const o = JSON.parse(raw);
    if (!o || typeof o !== "object") return emptyProfileMeta();
    const overrides = o.overrides && typeof o.overrides === "object" ? o.overrides : {};
    const hidden = Array.isArray(o.hidden)
      ? o.hidden.map((id) => String(id || "").toLowerCase()).filter(Boolean)
      : [];
    const clean = {};
    for (const [k, v] of Object.entries(overrides)) {
      if (!k || !v || typeof v !== "object") continue;
      clean[String(k).toLowerCase()] = {
        label: v.label != null ? String(v.label).trim().slice(0, 80) : undefined,
        definition: v.definition != null ? String(v.definition).trim().slice(0, 400) : undefined,
        low_label: v.low_label != null ? String(v.low_label).trim().slice(0, 24) : undefined,
        high_label: v.high_label != null ? String(v.high_label).trim().slice(0, 24) : undefined,
      };
    }
    return { overrides: clean, hidden: [...new Set(hidden)] };
  } catch {
    return emptyProfileMeta();
  }
}

export function setProfileMeta(meta) {
  const next = {
    overrides: (meta && meta.overrides) || {},
    hidden: [...new Set(((meta && meta.hidden) || []).map((id) => String(id || "").toLowerCase()).filter(Boolean))],
  };
  const has =
    Object.keys(next.overrides).length || next.hidden.length;
  try {
    if (has) localStorage.setItem(VOTER_PROFILE_META_KEY, JSON.stringify(next));
    else localStorage.removeItem(VOTER_PROFILE_META_KEY);
  } catch {
    /* ignore */
  }
  return next;
}

export function getHiddenProfileIds() {
  return new Set(getProfileMeta().hidden);
}

export function hideProfileAxis(id, builtinIds) {
  const key = String(id || "").trim().toLowerCase();
  if (!key) return;
  const builtin = new Set((builtinIds || []).map((x) => String(x || "").toLowerCase()));
  if (!builtin.has(key)) {
    removeCustomProfileAxis(key);
    return;
  }
  const meta = getProfileMeta();
  if (!meta.hidden.includes(key)) meta.hidden.push(key);
  setProfileMeta(meta);
  const prefs = getVoterProfile();
  if (prefs[key] != null) {
    delete prefs[key];
    setVoterProfile(prefs);
  }
}

export function restoreHiddenProfileAxes() {
  const meta = getProfileMeta();
  meta.hidden = [];
  return setProfileMeta(meta);
}

export function updateProfileAxisFields(id, patch, builtinIds) {
  const key = String(id || "").trim().toLowerCase();
  if (!key || !patch || typeof patch !== "object") return null;
  const builtin = new Set((builtinIds || []).map((x) => String(x || "").toLowerCase()));
  const fields = {};
  if ("label" in patch) fields.label = String(patch.label || "").trim().slice(0, 80);
  if ("definition" in patch) fields.definition = String(patch.definition || "").trim().slice(0, 400);
  if ("low_label" in patch) fields.low_label = String(patch.low_label || "").trim().slice(0, 24) || "Disagree";
  if ("high_label" in patch) fields.high_label = String(patch.high_label || "").trim().slice(0, 24) || "Agree";
  if (builtin.has(key)) {
    const meta = getProfileMeta();
    meta.overrides[key] = { ...(meta.overrides[key] || {}), ...fields };
    if (fields.label === "") delete meta.overrides[key].label;
    setProfileMeta(meta);
    return { id: key, ...meta.overrides[key], builtin: true };
  }
  const existing = getCustomProfileAxes();
  const i = existing.findIndex((a) => a.id === key);
  if (i < 0) return null;
  if (fields.label === "") return existing[i];
  existing[i] = { ...existing[i], ...fields, custom: true };
  setCustomProfileAxes(existing);
  return existing[i];
}

export function resolveProfileAxes(builtin) {
  const rows = Array.isArray(builtin) ? builtin : [];
  const builtinIds = new Set(rows.map((a) => String(a.id || "").toLowerCase()));
  const meta = getProfileMeta();
  const hidden = new Set(meta.hidden);
  const out = [];
  for (const a of rows) {
    const id = String(a.id || "").toLowerCase();
    if (!id || hidden.has(id)) continue;
    const o = meta.overrides[id] || {};
    out.push({
      id,
      label: o.label || a.label || id,
      definition: o.definition != null ? o.definition : a.definition || "",
      group: a.group || "Other",
      low_label: o.low_label || a.low_label || "Disagree",
      high_label: o.high_label || a.high_label || "Agree",
      signed: !!a.signed,
      custom: false,
    });
  }
  for (const a of getCustomProfileAxes()) {
    if (!a.id || builtinIds.has(a.id) || hidden.has(a.id)) continue;
    out.push({ ...a, custom: true });
  }
  return out;
}

export function setCustomProfileAxes(axes) {
  const out = [];
  const taken = new Set();
  for (const row of axes || []) {
    const a = normalizeCustomAxis(row, taken);
    if (!a) continue;
    taken.add(a.id);
    out.push(a);
  }
  try {
    if (out.length) localStorage.setItem(VOTER_PROFILE_AXES_KEY, JSON.stringify(out));
    else localStorage.removeItem(VOTER_PROFILE_AXES_KEY);
  } catch {
    /* ignore */
  }
  return out;
}

export function addCustomProfileAxis(raw, builtinIds) {
  const builtin = (builtinIds || []).map((id) => String(id || "").toLowerCase());
  const want = slugAxisId((raw && (raw.id || raw.label)) || "");
  if (want && builtin.includes(want) && getHiddenProfileIds().has(want)) {
    const meta = getProfileMeta();
    meta.hidden = meta.hidden.filter((id) => id !== want);
    const patch = {};
    if (raw.label) patch.label = raw.label;
    if (raw.definition) patch.definition = raw.definition;
    if (raw.low_label) patch.low_label = raw.low_label;
    if (raw.high_label) patch.high_label = raw.high_label;
    if (Object.keys(patch).length) {
      meta.overrides[want] = { ...(meta.overrides[want] || {}), ...patch };
    }
    setProfileMeta(meta);
    return { id: want, label: String((raw && raw.label) || want), restored: true };
  }
  const taken = new Set(builtin);
  const existing = getCustomProfileAxes();
  for (const a of existing) taken.add(a.id);
  const axis = normalizeCustomAxis(raw, taken);
  if (!axis) return null;
  existing.push(axis);
  setCustomProfileAxes(existing);
  return axis;
}

export function removeCustomProfileAxis(id) {
  const key = String(id || "").trim().toLowerCase();
  if (!key) return getCustomProfileAxes();
  const next = getCustomProfileAxes().filter((a) => a.id !== key);
  setCustomProfileAxes(next);
  const prefs = getVoterProfile();
  if (prefs[key] != null) {
    delete prefs[key];
    setVoterProfile(prefs);
  }
  return next;
}

export function exportVoterProfileCatalog(axes) {
  const seen = new Set();
  const out = [];
  for (const a of axes || []) {
    const id = String(a.id || "").trim().toLowerCase();
    if (!id || seen.has(id)) continue;
    seen.add(id);
    out.push({
      id,
      label: String(a.label || id),
      definition: String(a.definition || ""),
      group: String(a.group || ""),
      low_label: a.low_label || "Disagree",
      high_label: a.high_label || "Agree",
    });
  }
  return {
    kind: VOTER_PROFILE_KIND,
    version: 1,
    axes: out,
  };
}

export function parseVoterProfileImport(raw) {
  let v = raw;
  if (typeof raw === "string") {
    try {
      v = JSON.parse(raw);
    } catch {
      return { error: "Not valid JSON." };
    }
  }
  if (Array.isArray(v)) return { axes: v };
  if (!v || typeof v !== "object") return { error: "Not a voter profile list." };
  if (v.kind && v.kind !== VOTER_PROFILE_KIND) {
    return { error: "Not an electionizer voter profile file." };
  }
  if (Array.isArray(v.axes)) return { axes: v.axes };
  return { error: "No axes list in file." };
}

export function isVoterProfileCatalogEmpty() {
  try {
    const axes = localStorage.getItem(VOTER_PROFILE_AXES_KEY);
    const meta = localStorage.getItem(VOTER_PROFILE_META_KEY);
    return !axes && !meta;
  } catch {
    return true;
  }
}

export function importVoterProfileCatalog(raw, builtinIds, opts = {}) {
  const parsed = parseVoterProfileImport(raw);
  if (parsed.error) return parsed;
  const fillOnly = !!opts.fillOnly;
  const builtinList = (builtinIds || []).map((id) => String(id || "").toLowerCase());
  const builtin = new Set(builtinList);
  const existing = getCustomProfileAxes();
  const have = new Set(existing.map((a) => a.id));
  let added = 0;
  let updated = 0;
  let skipped = 0;
  const meta = getProfileMeta();
  const hidden = new Set(meta.hidden);
  for (const row of parsed.axes) {
    const label = String((row && (row.label || row.name || row.id)) || "").trim();
    if (!label) {
      skipped += 1;
      continue;
    }
    const want = slugAxisId(row.id || label);
    if (builtin.has(want)) {
      if (fillOnly) {
        skipped += 1;
        continue;
      }
      meta.hidden = meta.hidden.filter((id) => id !== want);
      meta.overrides[want] = {
        ...(meta.overrides[want] || {}),
        label: String(row.label || label).trim().slice(0, 80) || label,
        definition: String(row.definition || row.def || "").trim().slice(0, 400),
        low_label: String(row.low_label || row.low || "Disagree").trim().slice(0, 24) || "Disagree",
        high_label: String(row.high_label || row.high || "Agree").trim().slice(0, 24) || "Agree",
      };
      updated += 1;
      continue;
    }
    if (have.has(want)) {
      if (fillOnly) {
        skipped += 1;
        continue;
      }
      const i = existing.findIndex((a) => a.id === want);
      if (i >= 0) {
        existing[i] = {
          ...existing[i],
          label: String(row.label || existing[i].label).trim().slice(0, 80) || existing[i].label,
          definition: String(row.definition || row.def || existing[i].definition || "").trim().slice(0, 400),
          group: String(row.group || existing[i].group || "Custom").trim().slice(0, 40) || "Custom",
          low_label: String(row.low_label || row.low || existing[i].low_label || "Disagree").trim().slice(0, 24),
          high_label: String(row.high_label || row.high || existing[i].high_label || "Agree").trim().slice(0, 24),
          custom: true,
        };
        updated += 1;
      } else {
        skipped += 1;
      }
      continue;
    }
    if (fillOnly && hidden.has(want)) {
      skipped += 1;
      continue;
    }
    const axis = normalizeCustomAxis(row, new Set([...builtin, ...have]));
    if (!axis || builtin.has(axis.id)) {
      skipped += 1;
      continue;
    }
    existing.push(axis);
    have.add(axis.id);
    added += 1;
  }
  setProfileMeta(meta);
  setCustomProfileAxes(existing);
  return { added, updated, skipped, axes: existing };
}

export async function seedVoterProfileFromDefaults(builtinIds) {
  const ids = (builtinIds || []).map((id) => String(id || "").toLowerCase()).filter(Boolean);
  if (!ids.length) return { seeded: false };
  const empty = isVoterProfileCatalogEmpty();
  try {
    const defaults = await loadDefaultVoterProfile();
    return {
      seeded: empty,
      ...importVoterProfileCatalog(defaults, ids, { fillOnly: !empty }),
    };
  } catch {
    return { seeded: false, error: "Could not load defaults." };
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
