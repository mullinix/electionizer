import { cachedFetch } from "./cache.js";
import {
  getCycle,
  getFecApiKey,
  getOpenStatesApiKey,
  hasOpenStatesKey,
  getCivicApiKey,
  hasCivicKey,
} from "./settings.js";
import { fetchStateBodies } from "./state.js";
import {
  parse_zippo_js,
  build_live_ballot_report,
} from "./pkg/electionizer_wasm.js";

function fecCandidatesUrl({ state, office, district, cycle, apiKey }) {
  const u = new URL("https://api.open.fec.gov/v1/candidates/");
  u.searchParams.set("api_key", apiKey);
  u.searchParams.set("state", state);
  u.searchParams.set("office", office);
  u.searchParams.set("cycle", String(cycle));
  u.searchParams.set("election_year", String(cycle));
  u.searchParams.set("candidate_status", "C");
  u.searchParams.set("per_page", "100");
  u.searchParams.set("sort", "name");
  if (district != null) {
    u.searchParams.set("district", district === 0 ? "00" : String(district));
  }
  return u.toString();
}

function fecCacheKey({ state, office, district, cycle }) {
  const d = district == null ? "none" : String(district);
  return `fec:candidates:${state}:${office}:${d}:${cycle}`;
}

/** TIGERweb Legislative identify — CORS-enabled (Census geocoder is not). */
function tigerwebIdentifyUrl(lon, lat) {
  const pad = 0.5;
  const u = new URL(
    "https://tigerweb.geo.census.gov/arcgis/rest/services/TIGERweb/Legislative/MapServer/identify"
  );
  u.searchParams.set("geometry", `${lon},${lat}`);
  u.searchParams.set("geometryType", "esriGeometryPoint");
  u.searchParams.set("sr", "4326");
  u.searchParams.set("layers", "all:0,1,2");
  u.searchParams.set("tolerance", "2");
  u.searchParams.set(
    "mapExtent",
    `${lon - pad},${lat - pad},${lon + pad},${lat + pad}`
  );
  u.searchParams.set("imageDisplay", "400,400,96");
  u.searchParams.set("returnGeometry", "false");
  u.searchParams.set("f", "json");
  return u.toString();
}

function fccAreaUrl(lat, lon) {
  return `https://geo.fcc.gov/api/census/area?lat=${lat}&lon=${lon}&format=json`;
}

function parseCdFromTiger(body) {
  try {
    const j = JSON.parse(body);
    for (const r of j.results || []) {
      const layer = r.layerName || "";
      if (!layer.includes("Congressional")) continue;
      const a = r.attributes || {};
      const raw = a.CD119 || a.BASENAME || (a.NAME || "").split(/\s+/).pop();
      const n = parseInt(String(raw).replace(/^0+/, "") || "0", 10);
      if (Number.isFinite(n)) return n;
    }
  } catch {
    /* wasm re-parses */
  }
  return 0;
}

/** State upper/lower district numbers from TIGERweb identify JSON. */
function parseStateDistrictsFromTiger(body) {
  let senate = null;
  let house = null;
  try {
    const j = JSON.parse(body);
    for (const r of j.results || []) {
      const layer = String(r.layerName || "");
      const a = r.attributes || {};
      const raw = a.BASENAME || a.DISTRICT || (a.NAME || "").split(/\s+/).pop();
      const n = parseInt(String(raw).replace(/^0+/, "") || "", 10);
      if (!Number.isFinite(n)) continue;
      if (/Upper|State Senate/i.test(layer)) senate = n;
      else if (/Lower|State (House|Assembly|Legislative Districts - Lower)/i.test(layer))
        house = n;
    }
  } catch {
    /* ignore */
  }
  return { senate, house };
}

function osPeopleCount(body) {
  if (!body || !body.trim()) return 0;
  try {
    const j = JSON.parse(body);
    return Array.isArray(j.results) ? j.results.length : 0;
  } catch {
    return 0;
  }
}

async function fetchOpenStatesPeopleGeo(lat, lon, apiKey) {
  const u = new URL("https://v3.openstates.org/people.geo");
  u.searchParams.set("lat", String(lat));
  u.searchParams.set("lng", String(lon));
  u.searchParams.set("apikey", apiKey);
  const { body } = await cachedFetch(u.toString(), {
    key: `os:people.geo:${lat.toFixed(3)},${lon.toFixed(3)}`,
    ttlMs: 24 * 60 * 60 * 1000,
    init: { headers: { "X-API-KEY": apiKey } },
  });
  return body || "";
}

/** District-filtered /people when geo is empty (one request per chamber). */
async function fetchOpenStatesPeopleDistrict(state, org, district, apiKey) {
  const st = String(state || "").toLowerCase();
  if (st.length !== 2 || district == null || !Number.isFinite(district)) return [];
  const u = new URL("https://v3.openstates.org/people");
  u.searchParams.set("jurisdiction", st);
  u.searchParams.set("org_classification", org);
  u.searchParams.set("district", String(district));
  u.searchParams.set("per_page", "10");
  u.searchParams.set("apikey", apiKey);
  const { body } = await cachedFetch(u.toString(), {
    key: `os:people:${st}:${org}:${district}`,
    ttlMs: 24 * 60 * 60 * 1000,
    init: { headers: { "X-API-KEY": apiKey } },
  });
  try {
    const j = JSON.parse(body || "{}");
    return Array.isArray(j.results) ? j.results : [];
  } catch {
    return [];
  }
}

/**
 * Open States incumbents: people.geo first, then /people by TIGERweb district.
 * Returns JSON `{ results: [...] }` or "".
 */
const CIVIC_ROOT = "https://www.googleapis.com/civicinfo/v2";
/** VIP contest data: cache ≤24h per Google Civic developer guidelines. */
const CIVIC_TTL_MS = 12 * 60 * 60 * 1000;
const CIVIC_MAX_ELECTION_TRIES = 4;

function todayIsoDate() {
  const d = new Date();
  const y = d.getFullYear();
  const m = String(d.getMonth() + 1).padStart(2, "0");
  const day = String(d.getDate()).padStart(2, "0");
  return `${y}-${m}-${day}`;
}

function daysFromToday(isoDay) {
  if (!isoDay || !/^\d{4}-\d{2}-\d{2}/.test(isoDay)) return null;
  const t = Date.parse(isoDay.slice(0, 10) + "T12:00:00Z");
  if (!Number.isFinite(t)) return null;
  const now = Date.parse(todayIsoDate() + "T12:00:00Z");
  return Math.round((t - now) / 86400000);
}

/**
 * Rank electionQuery rows for voterinfo tries (best first).
 * Prefers upcoming state-matching races in the cycle year; skips VIP Test.
 */
export function rankCivicElections(electionsJson, { state, stateName, cycle } = {}) {
  let root;
  try {
    root = JSON.parse(electionsJson || "{}");
  } catch {
    return [];
  }
  const list = Array.isArray(root.elections) ? root.elections : [];
  if (!list.length) return [];
  const st = String(state || "").toUpperCase();
  const sn = String(stateName || "").toLowerCase();
  const year = String(cycle || "");
  const scored = list
    .map((e) => {
      const id = e && e.id != null ? String(e.id) : "";
      const name = String((e && e.name) || "");
      const day = String((e && e.electionDay) || "");
      const nl = name.toLowerCase();
      let score = 0;
      if (/vip test|test election/i.test(name)) score -= 200;
      const delta = daysFromToday(day);
      if (delta != null) {
        if (delta < 0) score -= 60; // past — API often returns electionOver
        else if (delta <= 45) score += 55; // VIP often live in this window
        else if (delta <= 120) score += 25;
        else score += 8;
        // Prefer nearer upcoming
        if (delta >= 0 && delta <= 365) score += Math.max(0, 30 - Math.floor(delta / 7));
      }
      if (year && (day.startsWith(year) || name.includes(year))) score += 35;
      // Adjacent odd year (locals) still useful mid-cycle
      if (year) {
        const y = parseInt(year, 10);
        if (Number.isFinite(y) && day.startsWith(String(y - 1))) score += 8;
      }
      if (st && (new RegExp(`\\b${st}\\b`).test(name) || nl.includes(`, ${st.toLowerCase()}`))) {
        score += 55;
      }
      if (sn && sn.length > 2 && nl.includes(sn)) score += 50;
      if (/general/i.test(name)) score += 12;
      if (/primary|runoff|run-off/i.test(name)) score += 6;
      if (/special|municipal|local|school/i.test(name)) score += 4;
      return { id, score, name, day };
    })
    .filter((x) => x.id && x.score > -100);
  scored.sort((a, b) => b.score - a.score || String(a.day).localeCompare(String(b.day)));
  return scored;
}

/** @deprecated use rankCivicElections */
export function pickCivicElectionId(electionsJson, opts) {
  const ranked = rankCivicElections(electionsJson, opts);
  return ranked[0] ? ranked[0].id : null;
}

/** True if a Civic contest is federal (FEC owns those). */
export function civicContestIsFederal(c) {
  if (!c || typeof c !== "object") return true;
  const levels = Array.isArray(c.level)
    ? c.level.map((x) => String(x).toLowerCase())
    : [];
  if (levels.includes("country") || levels.includes("international")) return true;
  const scope = String((c.district && c.district.scope) || "").toLowerCase();
  if (scope === "congressional" || scope === "national") return true;
  const office = String(c.ballotTitle || c.office || "").toLowerCase();
  if (
    /united states|u\.s\. house|u\.s\. senate|us house|us senate|u\.s\. representative|u\.s\. senator|president of the united states/.test(
      office
    )
  ) {
    return true;
  }
  return false;
}

/** State/local contests (or measures) present — worth keeping this voterinfo body. */
export function civicBodyHasStateContests(body) {
  try {
    const j = JSON.parse(body || "{}");
    if (j.error) return false;
    const contests = Array.isArray(j.contests) ? j.contests : [];
    return contests.some((c) => !civicContestIsFederal(c));
  } catch {
    return false;
  }
}

function civicElectionLabel(body) {
  try {
    const j = JSON.parse(body || "{}");
    const name = j.election && j.election.name;
    const day = j.election && j.election.electionDay;
    if (name && day) return `${name} (${day})`;
    if (name) return String(name);
  } catch {
    /* ignore */
  }
  return "";
}

/**
 * Fetch Google Civic voterInfo for street (optional) + city/ST/ZIP.
 * Tries top-ranked elections until one has state/local contests.
 * @returns {Promise<{ body: string, note?: string, electionLabel?: string }>}
 */
export async function fetchCivicVoterInfo({
  street = "",
  city,
  state,
  stateName = "",
  zip,
  cycle,
  onProgress = () => {},
}) {
  const apiKey = getCivicApiKey();
  if (!apiKey) return { body: "" };
  const streetLine = String(street || "").trim();
  const address = [streetLine, city, state, zip].filter(Boolean).join(", ");
  if (!address) return { body: "" };

  onProgress("Google Civic elections list…");
  const electionsUrl = new URL(`${CIVIC_ROOT}/elections`);
  electionsUrl.searchParams.set("key", apiKey);
  let electionsBody = "";
  try {
    const r = await cachedFetch(electionsUrl.toString(), {
      key: `civic:elections:${apiKey.slice(0, 8)}`,
      ttlMs: CIVIC_TTL_MS,
    });
    electionsBody = r.body || "";
  } catch (e) {
    throw new Error(`Google Civic elections: ${e.message || e}`);
  }

  const ranked = rankCivicElections(electionsBody, {
    state,
    stateName,
    cycle,
  });
  // null id = let API pick default (earliest); then try ranked ids
  const tryIds = [null, ...ranked.map((r) => r.id)].filter(
    (id, i, a) => a.indexOf(id) === i
  ).slice(0, CIVIC_MAX_ELECTION_TRIES);

  let lastBody = "";
  let lastErr = "";
  const streetKey = streetLine
    ? streetLine.toLowerCase().replace(/\s+/g, " ").slice(0, 48)
    : "";

  for (const electionId of tryIds) {
    onProgress(
      electionId
        ? `Google Civic voterinfo (election ${electionId})…`
        : "Google Civic voterinfo…"
    );
    const vi = new URL(`${CIVIC_ROOT}/voterinfo`);
    vi.searchParams.set("key", apiKey);
    vi.searchParams.set("address", address);
    if (electionId) vi.searchParams.set("electionId", electionId);

    const cacheKey = `civic:voterinfo:${state}:${zip}:${streetKey}:${electionId || "auto"}:${cycle}`;
    try {
      const r = await cachedFetch(vi.toString(), {
        key: cacheKey,
        ttlMs: CIVIC_TTL_MS,
      });
      const body = r.body || "";
      if (!body || body.length < 20) continue;
      let j;
      try {
        j = JSON.parse(body);
      } catch {
        continue;
      }
      if (j.error) {
        lastErr = j.error.message || JSON.stringify(j.error);
        continue;
      }
      lastBody = body;
      if (civicBodyHasStateContests(body)) {
        return {
          body,
          electionLabel: civicElectionLabel(body),
        };
      }
    } catch (e) {
      lastErr = e.message || String(e);
    }
  }

  if (lastBody) {
    return {
      body: lastBody,
      electionLabel: civicElectionLabel(lastBody),
      note:
        "Google Civic returned no state/local contests for this address (VIP is seasonal — often only ~2–4 weeks before election day). Federal still from FEC; other sources used when available.",
    };
  }
  if (lastErr) throw new Error(lastErr);
  return {
    body: "",
    note: "Google Civic: no voterinfo for this address/election list.",
  };
}

async function fetchOpenStatesLegislators(
  lat,
  lon,
  state,
  tigerBody,
  onProgress
) {
  if (!hasOpenStatesKey()) return "";
  const key = getOpenStatesApiKey();
  onProgress("Open States legislature incumbents…");

  let geoBody = "";
  try {
    geoBody = await fetchOpenStatesPeopleGeo(lat, lon, key);
  } catch (e) {
    console.warn("[live] Open States people.geo", e);
    // Fall through to district search unless hard rate-limit
    if (/429|rate/i.test(String(e.message || e))) throw e;
  }
  if (osPeopleCount(geoBody) > 0) return geoBody;

  const { senate, house } = parseStateDistrictsFromTiger(tigerBody);
  if (senate == null && house == null) return geoBody || "";

  onProgress("Open States district lookup…");
  const results = [];
  const seen = new Set();
  try {
    const batches = await Promise.all([
      senate != null
        ? fetchOpenStatesPeopleDistrict(state, "upper", senate, key)
        : Promise.resolve([]),
      house != null
        ? fetchOpenStatesPeopleDistrict(state, "lower", house, key)
        : Promise.resolve([]),
    ]);
    for (const batch of batches) {
      for (const p of batch) {
        const id = p?.id || p?.name;
        if (!id || seen.has(id)) continue;
        seen.add(id);
        results.push(p);
      }
    }
  } catch (e) {
    console.warn("[live] Open States /people district", e);
    if (/429|rate/i.test(String(e.message || e))) throw e;
    return geoBody || "";
  }
  if (!results.length) return geoBody || "";
  return JSON.stringify({ results });
}

function pushHardNotes(notes, clientWarnings) {
  for (const n of notes || []) {
    if (/^Using /i.test(n)) continue;
    if (/libcurl\.js \+ Wisp/i.test(n)) continue;
    if (/live-cache|cached FL DOS/i.test(n)) continue;
    if (/amendments scrape returned empty/i.test(n)) continue;
    if (/amendments need Wisp/i.test(n)) continue;
    if (/partial rosters/i.test(n)) continue;
    if (/State legislature incumbents via Open States/i.test(n)) continue;
    if (/NCSBE candidate listing/i.test(n)) continue;
    if (/Parsed \d+ NCSBE/i.test(n)) continue;
    if (/Source: https:\/\/s3\.amazonaws\.com\/dl\.ncsbe/i.test(n)) continue;
    if (/Maryland SBE/i.test(n)) continue;
    if (/Parsed \d+ MD SBE/i.test(n)) continue;
    if (/Parsed \d+ MD ballot question/i.test(n)) continue;
    if (/Using Maryland SBE ballot questions/i.test(n)) continue;
    if (/elections\.maryland\.gov/i.test(n)) continue;
    if (/AZ Clean Elections measures/i.test(n)) continue;
    if (/Using AZ Clean Elections/i.test(n)) continue;
    if (/Using Clean Elections OfficialList/i.test(n)) continue;
    if (/AZ statewide executive incumbent/i.test(n)) continue;
    if (/AZ coverage is incumbents only/i.test(n)) continue;
    if (/NCSBE referendum list/i.test(n)) continue;
    if (/Using cached NCSBE referendum/i.test(n)) continue;
    clientWarnings.push(n);
  }
}

/**
 * @param {string} zip 5-digit
 * @param {(msg: string) => void} onProgress
 * @returns {Promise<object>} ballot report (may include client_warnings)
 */
/**
 * @param {string} zip
 * @param {(msg: string) => void} [onProgress]
 * @param {{ street?: string }} [opts]
 */
export async function buildLiveFederalBallot(zip, onProgress = () => {}, opts = {}) {
  const cycle = getCycle();
  const apiKey = getFecApiKey();
  const street = String(opts.street || "").trim();
  /** @type {string[]} */
  const clientWarnings = [];

  onProgress("Looking up ZIP…");
  const zippoUrl = `https://api.zippopotam.us/us/${zip}`;
  let zippoBody;
  try {
    const r = await cachedFetch(zippoUrl, {
      key: `zippo:${zip}`,
      ttlMs: 7 * 24 * 60 * 60 * 1000,
    });
    zippoBody = r.body;
  } catch (e) {
    if (e.status === 404) throw new Error(`ZIP ${zip} not found`);
    throw e;
  }

  const place = parse_zippo_js(zippoBody);
  const lon = place.longitude;
  const lat = place.latitude;
  const state = String(place.state_abbr || "").toUpperCase();

  onProgress(`District lookup ${place.city}, ${state}…`);
  const [tiger, fcc] = await Promise.all([
    cachedFetch(tigerwebIdentifyUrl(lon, lat), {
      key: `tiger:${lon.toFixed(4)},${lat.toFixed(4)}`,
      ttlMs: 7 * 24 * 60 * 60 * 1000,
    }),
    cachedFetch(fccAreaUrl(lat, lon), {
      key: `fcc:${lat.toFixed(4)},${lon.toFixed(4)}`,
      ttlMs: 7 * 24 * 60 * 60 * 1000,
    }).catch(() => ({ body: "", cached: false })),
  ]);

  const cdNum = parseCdFromTiger(tiger.body);

  onProgress(`Fetching FEC House candidates (${state}-${cdNum || "AL"})…`);
  let house;
  try {
    house = await cachedFetch(
      fecCandidatesUrl({
        state,
        office: "H",
        district: cdNum,
        cycle,
        apiKey,
      }),
      { key: fecCacheKey({ state, office: "H", district: cdNum, cycle }) }
    );
  } catch (e) {
    if (e.status === 429 && apiKey === "DEMO_KEY") throw e;
    throw e;
  }

  onProgress(`Fetching FEC Senate candidates (${state})…`);
  const senate = await cachedFetch(
    fecCandidatesUrl({
      state,
      office: "S",
      district: null,
      cycle,
      apiKey,
    }),
    { key: fecCacheKey({ state, office: "S", district: null, cycle }) }
  );

  let stateBodies = {
    flDosTsv: "",
    flSenate: "",
    flHouse: "",
    azSenate: "",
    azHouse: "",
    azMeasures: "",
    azOfficials: "",
    flMeasures: "",
    flSoe: "",
    flSampleBallot: "",
    ncCandidates: "",
    ncMeasures: "",
    ncMeasuresUrl: "",
    mdStatewide: "",
    mdLocal: "",
    mdPhase: "",
    mdMeasures: "",
    notes: [],
  };
  if (state === "FL" || state === "AZ" || state === "NC" || state === "MD") {
    try {
      let countyHint = "";
      try {
        const fccJson = JSON.parse(fcc.body || "{}");
        countyHint = String(fccJson?.results?.[0]?.county_name || "").trim();
      } catch {
        /* ignore */
      }
      const sd = parseStateDistrictsFromTiger(tiger.body);
      stateBodies = await fetchStateBodies(state, onProgress, {
        county: countyHint,
        cd: cdNum,
        legDistrict: sd.senate || sd.house || 0,
      });
      pushHardNotes(stateBodies.notes, clientWarnings);
    } catch (e) {
      clientWarnings.push(
        `State enrichment failed: ${e.message || e}. Federal races still shown.`
      );
    }
  }

  // Official rich filings — skip Civic (and often OS) when present.
  const richFl = state === "FL" && !!(stateBodies.flDosTsv && stateBodies.flDosTsv.length > 100);
  const richNc =
    state === "NC" &&
    !!(stateBodies.ncCandidates && stateBodies.ncCandidates.length > 500);
  const richMd =
    state === "MD" &&
    !!((stateBodies.mdStatewide && stateBodies.mdStatewide.length > 200) ||
      (stateBodies.mdLocal && stateBodies.mdLocal.length > 200));
  // AZ rosters are incumbents-only — still try Civic for full contests when keyed.
  const richAz =
    state === "AZ" &&
    !!(
      (stateBodies.azSenate && stateBodies.azSenate.length > 200) ||
      (stateBodies.azHouse && stateBodies.azHouse.length > 200)
    );

  let civicVoterinfo = "";
  let civicHasStateContests = false;
  if (hasCivicKey() && !richFl && !richNc && !richMd) {
    try {
      const civicResult = await fetchCivicVoterInfo({
        street,
        city: place.city || "",
        state,
        stateName: place.state_name || "",
        zip,
        cycle,
        onProgress,
      });
      civicVoterinfo = civicResult.body || "";
      civicHasStateContests = civicBodyHasStateContests(civicVoterinfo);
      if (civicHasStateContests) {
        const label = civicResult.electionLabel || "live VIP election";
        clientWarnings.push(
          `Google Civic / VIP ballot: ${label}` +
            (street
              ? " · street address used"
              : " · ZIP centroid (optional street improves precinct match)")
        );
      } else if (civicResult.note) {
        clientWarnings.push(civicResult.note);
      } else if (!civicVoterinfo) {
        clientWarnings.push(
          "Google Civic: no voterinfo body (check key, or no live VIP election for this address)."
        );
      }
    } catch (e) {
      const msg = e.message || String(e);
      if (/API key|invalid|403|401/i.test(msg)) {
        clientWarnings.push(
          `Google Civic key rejected: ${msg}. Check Settings → Google Civic API key.`
        );
      } else {
        clientWarnings.push(`Google Civic skipped: ${msg}`);
      }
    }
  } else if (hasCivicKey() && (richFl || richNc || richMd)) {
    // Quiet: official filings preferred; no Civic call
  }

  // Open States people.geo — skip when rich filings or Civic already returned state contests
  let osPeopleGeo = "";
  if (
    hasOpenStatesKey() &&
    !richFl &&
    !richAz &&
    !richNc &&
    !richMd &&
    !civicHasStateContests
  ) {
    try {
      osPeopleGeo = await fetchOpenStatesLegislators(
        lat,
        lon,
        state,
        tiger.body,
        onProgress
      );
    } catch (e) {
      const msg = e.message || String(e);
      if (/429|rate/i.test(msg)) {
        clientWarnings.push(
          "Open States rate limit — state legislature incumbents skipped. Retry in a minute."
        );
      } else {
        clientWarnings.push(`Open States incumbents failed: ${msg}`);
      }
    }
  }

  onProgress("Building ballot…");
  const statePayload = {};
  if (stateBodies.flDosTsv) statePayload["fl:dos"] = stateBodies.flDosTsv;
  if (stateBodies.flSenate) statePayload["fl:senate"] = stateBodies.flSenate;
  if (stateBodies.flHouse) statePayload["fl:house"] = stateBodies.flHouse;
  if (stateBodies.flMeasures) statePayload["fl:measures"] = stateBodies.flMeasures;
  if (stateBodies.flSoe) statePayload["fl:soe"] = stateBodies.flSoe;
  if (stateBodies.flSampleBallot) statePayload["fl:sample_ballot"] = stateBodies.flSampleBallot;
  if (stateBodies.azSenate) statePayload["az:senate"] = stateBodies.azSenate;
  if (stateBodies.azHouse) statePayload["az:house"] = stateBodies.azHouse;
  if (stateBodies.azMeasures) statePayload["az:measures"] = stateBodies.azMeasures;
  if (stateBodies.azOfficials) statePayload["az:officials"] = stateBodies.azOfficials;
  if (stateBodies.ncCandidates) statePayload["nc:candidates"] = stateBodies.ncCandidates;
  if (stateBodies.ncMeasures) statePayload["nc:measures"] = stateBodies.ncMeasures;
  if (stateBodies.ncMeasuresUrl) statePayload["nc:measures_url"] = stateBodies.ncMeasuresUrl;
  if (stateBodies.mdStatewide) statePayload["md:statewide"] = stateBodies.mdStatewide;
  if (stateBodies.mdLocal) statePayload["md:local"] = stateBodies.mdLocal;
  if (stateBodies.mdPhase) statePayload["md:phase"] = stateBodies.mdPhase;
  if (stateBodies.mdMeasures) statePayload["md:measures"] = stateBodies.mdMeasures;
  if (civicVoterinfo) statePayload["civic:voterinfo"] = civicVoterinfo;
  if (osPeopleGeo) statePayload["os:people.geo"] = osPeopleGeo;
  const report = build_live_ballot_report(
    zip,
    zippoBody,
    tiger.body,
    fcc.body || "",
    house.body,
    senate.body,
    cycle,
    JSON.stringify(statePayload)
  );

  // FL measure summary/finance enrich runs after first paint (app.js progressive).

  if (clientWarnings.length) {
    report.client_warnings = clientWarnings;
  }
  return report;
}
