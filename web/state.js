import { cachedFetch, cacheGet, cachePut } from "./cache.js";
import {
  curlCloseSession,
  curlFetchBytes,
  curlFetchText,
  curlPostForm,
  curlPostJson,
  curlRequest,
  curlSession,
  ensureCurl,
  hasWispConfigured,
} from "./curl-transport.js";
import {
  getCorsProxy,
  getCycle,
  getFlDosTsv,
  getVoterParty,
  getVoterPrecinct,
  proxiedUrl,
} from "./settings.js";
import {
  SENATE_URL as AZ_SENATE,
  HOUSE_URL as AZ_HOUSE,
  AZ_MEASURES_ELECTIONS_URL,
  AZ_MEASURES_COUNTIES_BASE,
  AZ_MEASURES_LIST_BASE,
  AZ_OFFICIALS_LIST_BASE,
} from "./state-urls.js";

export { AZ_SENATE, AZ_HOUSE };

const FL_SENATE = "https://www.flsenate.gov/Senators";
const FL_HOUSE =
  "https://www.flhouse.gov/Sections/Representatives/representatives.aspx";
const FL_MEASURES = "https://constitutionalinitiatives.dos.fl.gov/";
const FL_DOS_EXTRACT =
  "https://dos.elections.myflorida.com/candidates/extractCanList.asp";
const FL_DOS_INDEX = "https://dos.elections.myflorida.com/candidates/";
const FL_DOS_CANTYPES = ["STA", "MUL", "LOC"];

function proxyHint() {
  if (hasWispConfigured()) return "Wisp/libcurl.js";
  if (getCorsProxy()) return "CORS proxy";
  return null;
}

function acceptTextBody(body, minLength = 200) {
  if (!body) return false;
  if (body.length >= minLength) return true;
  // Short structured payloads (API errors, tiny TSV/JSON).
  if (body.length > 20 && (body.includes("{") || body.includes("<") || body.includes("\t"))) {
    return true;
  }
  return false;
}

/** GET text: prefer libcurl+Wisp, else CORS proxy / direct + IndexedDB cache. */
export async function tryFetchText(url, cacheKey, ttlMs = 0) {
  const hit = await cacheGet(cacheKey);
  if (hit != null && acceptTextBody(hit)) return hit;

  // 1) libcurl.js via Wisp (no CORS; E2E TLS)
  if (hasWispConfigured()) {
    try {
      await ensureCurl();
      const body = await curlFetchText(url);
      if (acceptTextBody(body)) {
        await cachePut(cacheKey, body, ttlMs);
        return body;
      }
    } catch (e) {
      console.warn("[state] libcurl GET failed", cacheKey, e);
      // fall through to CORS/direct
    }
  }

  // 2) browser fetch: CORS proxy and/or direct
  const direct = url;
  const proxied = proxiedUrl(url);
  const attempts = [];
  if (getCorsProxy() && proxied !== direct) {
    attempts.push({ url: proxied, key: cacheKey + ":proxy" });
    attempts.push({ url: direct, key: cacheKey });
  } else {
    attempts.push({ url: direct, key: cacheKey });
    if (proxied !== direct) attempts.push({ url: proxied, key: cacheKey + ":proxy" });
  }

  let lastErr = null;
  for (const a of attempts) {
    try {
      const r = await cachedFetch(a.url, { key: a.key, ttlMs });
      if (acceptTextBody(r.body)) return r.body;
    } catch (e) {
      lastErr = e;
    }
  }
  if (lastErr) throw lastErr;
  return "";
}

/**
 * POST form: libcurl session first, else CORS proxy / direct.
 * @param {object} [opts]
 * @param {number} [opts.minLength=200] minimum body length to accept/cache
 * @param {object} [opts.session] libcurl session
 * @param {object} [opts.headers] extra headers
 */
export async function tryPostForm(
  url,
  formBody,
  cacheKey,
  ttlMs = 0,
  sessionOrOpts,
  extraHeaders = {}
) {
  // Back-compat: 5th arg was session, 6th extraHeaders; or opts { minLength, session, headers }.
  let session = sessionOrOpts;
  let minLength = 200;
  let headers = extraHeaders;
  if (
    sessionOrOpts &&
    typeof sessionOrOpts === "object" &&
    ("minLength" in sessionOrOpts ||
      "session" in sessionOrOpts ||
      ("headers" in sessionOrOpts && !("handle" in sessionOrOpts)))
  ) {
    minLength = sessionOrOpts.minLength ?? 200;
    session = sessionOrOpts.session;
    headers = sessionOrOpts.headers || extraHeaders || {};
  }

  const hit = await cacheGet(cacheKey);
  if (hit != null && hit.length >= minLength) return hit;

  if (hasWispConfigured()) {
    try {
      await ensureCurl();
      const body = await curlPostForm(url, formBody, {
        session,
        headers,
      });
      if (body && body.length >= minLength) {
        await cachePut(cacheKey, body, ttlMs);
        return body;
      }
      // Accept short but structured TSV/HTML (e.g. single-row contrib totals).
      if (body && body.length > 40 && (body.includes("\t") || body.includes("<"))) {
        await cachePut(cacheKey, body, ttlMs);
        return body;
      }
    } catch (e) {
      console.warn("[state] libcurl POST failed", cacheKey, e);
    }
  }

  const targets = [];
  const proxied = proxiedUrl(url);
  if (getCorsProxy() && proxied !== url) targets.push(proxied);
  targets.push(url);

  let lastErr = null;
  for (const target of targets) {
    try {
      const res = await fetch(target, {
        method: "POST",
        headers: {
          "Content-Type": "application/x-www-form-urlencoded",
          ...headers,
        },
        body: formBody,
      });
      if (!res.ok) {
        lastErr = new Error(`HTTP ${res.status} POST ${cacheKey}`);
        continue;
      }
      const body = await res.text();
      if (body && body.length >= minLength) {
        await cachePut(cacheKey, body, ttlMs);
        return body;
      }
      if (body && body.length > 40 && (body.includes("\t") || body.includes("<"))) {
        await cachePut(cacheKey, body, ttlMs);
        return body;
      }
    } catch (e) {
      lastErr = e;
    }
  }
  if (lastErr) throw lastErr;
  return "";
}

/**
 * POST application/json: libcurl/Wisp first, else CORS proxy / direct.
 * @param {object} [opts]
 * @param {number} [opts.minLength=40]
 * @param {object} [opts.headers]
 */
export async function tryPostJson(
  url,
  jsonBody,
  cacheKey,
  ttlMs = 0,
  opts = {}
) {
  const minLength = opts.minLength ?? 40;
  const headers = opts.headers || {};
  const bodyStr =
    typeof jsonBody === "string" ? jsonBody : JSON.stringify(jsonBody || {});

  const hit = await cacheGet(cacheKey);
  if (hit != null && hit.length >= minLength) return hit;

  if (hasWispConfigured()) {
    try {
      await ensureCurl();
      const body = await curlPostJson(url, bodyStr, { headers });
      if (body && body.length >= minLength) {
        await cachePut(cacheKey, body, ttlMs);
        return body;
      }
      if (body && body.length > 2 && (body.startsWith("{") || body.startsWith("["))) {
        await cachePut(cacheKey, body, ttlMs);
        return body;
      }
    } catch (e) {
      console.warn("[state] libcurl POST JSON failed", cacheKey, e);
    }
  }

  const targets = [];
  const proxied = proxiedUrl(url);
  if (getCorsProxy() && proxied !== url) targets.push(proxied);
  targets.push(url);

  let lastErr = null;
  for (const target of targets) {
    try {
      const res = await fetch(target, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          Accept: "application/json, text/plain, */*",
          ...headers,
        },
        body: bodyStr,
      });
      if (!res.ok) {
        lastErr = new Error(`HTTP ${res.status} POST JSON ${cacheKey}`);
        continue;
      }
      const body = await res.text();
      if (body && body.length >= minLength) {
        await cachePut(cacheKey, body, ttlMs);
        return body;
      }
      if (body && body.length > 2 && (body.startsWith("{") || body.startsWith("["))) {
        await cachePut(cacheKey, body, ttlMs);
        return body;
      }
    } catch (e) {
      lastErr = e;
    }
  }
  if (lastErr) throw lastErr;
  return "";
}

/** Match native `fl_gen_elec_id` — first Tue after first Mon in Nov. */
function flGenElecId(cycle) {
  // Same as core federal_election_date → YYYYMMDD-GEN
  const y = cycle;
  // Nov 1 weekday; first Monday is day 1+(1-wd+7)%7 where wd=0 Sun
  const nov1 = new Date(Date.UTC(y, 10, 1));
  const wd = nov1.getUTCDay(); // 0=Sun
  const firstMon = 1 + ((1 - wd + 7) % 7);
  const electionDay = firstMon + 1; // Tuesday
  const mm = "11";
  const dd = String(electionDay).padStart(2, "0");
  return `${y}${mm}${dd}-GEN`;
}

function looksLikeDosTsv(body) {
  return body && body.includes("AcctNum") && body.includes("\t");
}

/**
 * Live FL DOS Candidate Tracking extract (STA+MUL+LOC) — same as native server.
 * Uses libcurl+Wisp (or CORS proxy POST). Cached in IndexedDB until reload.
 */
async function fetchFlDosLive(cycle, onProgress) {
  const elecId = flGenElecId(cycle);
  const mergeKey = `fl:dos_cts:${elecId}:STA+MUL+LOC`;
  const cached = await cacheGet(mergeKey);
  if (looksLikeDosTsv(cached)) return { tsv: cached, source: "cache", elecId };

  if (!hasWispConfigured() && !getCorsProxy()) {
    return { tsv: "", source: "", elecId };
  }

  onProgress(`Florida DOS candidate list (${elecId})…`);

  let header = null;
  const dataLines = [];
  const seenAcct = new Set();
  let anyOk = false;
  let lastErr = null;

  for (const cantype of FL_DOS_CANTYPES) {
    onProgress(`Florida DOS extract ${cantype}…`);
    const form = new URLSearchParams();
    form.set("elecID", elecId);
    form.set("office", "All");
    form.set("status", "All");
    form.set("cantype", cantype);
    form.set("FormSubmit", "Download Candidate List");
    const formBody = form.toString();
    // DOS expects + for spaces in FormSubmit (URLSearchParams uses %20)
    const bodyFixed = formBody.replace(
      "FormSubmit=Download+Candidate+List",
      "FormSubmit=Download+Candidate+List"
    ).replace(
      "FormSubmit=Download%20Candidate%20List",
      "FormSubmit=Download+Candidate+List"
    );

    const partKey = `fl:dos_cts:${elecId}:${cantype}`;
    try {
      let body = await cacheGet(partKey);
      if (!looksLikeDosTsv(body)) {
        body = "";
        if (hasWispConfigured()) {
          try {
            await ensureCurl();
            const r = await curlRequest(FL_DOS_EXTRACT, {
              method: "POST",
              headers: {
                "Content-Type": "application/x-www-form-urlencoded",
                Referer: FL_DOS_INDEX,
              },
              body: bodyFixed,
            });
            if (!r.ok) throw new Error(`HTTP ${r.status}`);
            body = r.text;
          } catch (e) {
            console.warn("[state] libcurl DOS", cantype, e);
            lastErr = e;
          }
        }
        if (!looksLikeDosTsv(body) && getCorsProxy()) {
          try {
            const res = await fetch(proxiedUrl(FL_DOS_EXTRACT), {
              method: "POST",
              headers: {
                "Content-Type": "application/x-www-form-urlencoded",
              },
              body: bodyFixed,
            });
            if (!res.ok) throw new Error(`HTTP ${res.status}`);
            body = await res.text();
          } catch (e) {
            lastErr = e;
          }
        }
        if (looksLikeDosTsv(body)) {
          await cachePut(partKey, body, 24 * 60 * 60 * 1000);
        }
      }
      if (!looksLikeDosTsv(body)) {
        lastErr = lastErr || new Error(`DOS ${cantype} empty/invalid`);
        continue;
      }
      anyOk = true;
      const lines = body.split(/\r?\n/);
      const h = lines[0] || "";
      if (!header) header = h;
      for (let i = 1; i < lines.length; i++) {
        const line = lines[i];
        if (!line || !line.trim()) continue;
        const acct = line.split("\t")[0]?.trim() || "";
        if (acct && seenAcct.has(acct)) continue;
        if (acct) seenAcct.add(acct);
        dataLines.push(line);
      }
    } catch (e) {
      lastErr = e;
    }
  }

  if (!anyOk || !header) {
    throw lastErr || new Error("DOS extract failed for all cantypes");
  }

  const merged = [header, ...dataLines].join("\n");
  await cachePut(mergeKey, merged, 24 * 60 * 60 * 1000);
  return { tsv: merged, source: "live", elecId };
}

function extractTokenJs(html) {
  const m1 = html.match(
    /name\s*=\s*"__RequestVerificationToken"[^>]*value\s*=\s*"([^"]+)"/i
  );
  if (m1) return m1[1];
  const m2 = html.match(
    /value\s*=\s*"([^"]+)"[^>]*name\s*=\s*"__RequestVerificationToken"/i
  );
  return m2 ? m2[1] : "";
}

/**
 * FL DOS constitutional initiatives: GET index → POST filter MadeBallot=Y.
 * Prefer libcurl+Wisp (cookies); else CORS proxy.
 */
async function fetchFlMeasuresHtml(cycle, onProgress) {
  const cacheKey = `fl:measures:${cycle}:ballot`;
  const cached = await cacheGet(cacheKey);
  if (cached && cached.length > 200) return cached;

  if (!hasWispConfigured() && !getCorsProxy()) {
    return "";
  }

  onProgress("Florida constitutional amendments…");

  let session = null;
  try {
    if (hasWispConfigured()) {
      try {
        await ensureCurl();
        session = await curlSession();
      } catch {
        session = null;
      }
    }

    let indexHtml = "";
    if (session) {
      try {
        indexHtml = await curlFetchText(FL_MEASURES, { session });
      } catch (e) {
        console.warn("[state] libcurl measures index", e);
      }
    }
    if (!indexHtml) {
      indexHtml = await tryFetchText(
        FL_MEASURES,
        `fl:measures:index:${cycle}`,
        6 * 60 * 60 * 1000
      );
    }
    if (!indexHtml) return "";

    let token = extractTokenJs(indexHtml);
    try {
      const { extract_verification_token_js } = await import(
        "./pkg/electionizer_wasm.js"
      );
      token = extract_verification_token_js(indexHtml) || token;
    } catch {
      /* js fallback */
    }
    if (!token) {
      throw new Error("FL measures: missing antiforgery token on index page");
    }

    const params = new URLSearchParams();
    params.set("__RequestVerificationToken", token);
    params.set("Year", String(cycle));
    params.set("Status", "ACT");
    params.set("MadeBallot", "Y");
    params.set("Sponsor", "ALL");

    return await tryPostForm(
      FL_MEASURES,
      params.toString(),
      cacheKey,
      24 * 60 * 60 * 1000,
      session
    );
  } finally {
    curlCloseSession(session);
  }
}

async function loadShippedFlDos(cycle) {
  const paths = [
    `./data/fl-dos-${cycle}.tsv`,
    `./data/fl-cts-${cycle}.tsv`,
  ];
  for (const p of paths) {
    try {
      const res = await fetch(p);
      if (res.ok) {
        const t = await res.text();
        if (t && t.length > 100) return t;
      }
    } catch {
      /* optional */
    }
  }
  return "";
}

function missingStateTransportNote(kind) {
  const hint = proxyHint();
  if (hint) {
    return `${kind} failed even with ${hint}. Check Settings or try again.`;
  }
  return `${kind} blocked (CORS). Set a Wisp URL (libcurl.js) or CORS proxy in Settings.`;
}

/**
 * Fetch FL/AZ state source bodies for the given USPS state.
 */
/**
 * Fill FL measure summaries from DOS InitDetail pages (Wisp/libcurl preferred).
 * Mutates `report.measures` in place. Best-effort; skips when transport unavailable.
 */
export async function enrichFlMeasureSummaries(report, onProgress = () => {}) {
  const measures = report?.measures;
  if (!Array.isArray(measures) || !measures.length) return report;

  const need = measures.filter(
    (m) =>
      !m.summary &&
      m.source_url &&
      /InitDetail/i.test(m.source_url)
  );
  if (!need.length) return report;

  if (!hasWispConfigured() && !getCorsProxy()) return report;

  let parseSummary;
  let cacheKeyFn;
  try {
    const wasm = await import("./pkg/electionizer_wasm.js");
    parseSummary = wasm.parse_fl_measure_summary_js;
    cacheKeyFn = wasm.fl_measure_detail_cache_key_js;
  } catch (e) {
    console.warn("[state] measure summary wasm", e);
    return report;
  }

  onProgress(`Florida amendment summaries (0/${need.length})…`);
  let done = 0;
  // Cap concurrent Wisp hits; public endpoints throttle easily.
  const concurrency = 2;
  let idx = 0;
  async function worker() {
    while (idx < need.length) {
      const i = idx++;
      const m = need[i];
      try {
        const key =
          (cacheKeyFn && cacheKeyFn(m.source_url)) ||
          `fl:measures:detail:${m.id}`;
        const html = await tryFetchText(m.source_url, key, 7 * 24 * 60 * 60 * 1000);
        if (html && parseSummary) {
          const s = parseSummary(html);
          if (s) m.summary = s;
        }
      } catch (e) {
        console.warn("[state] InitDetail", m.source_url, e);
      }
      done += 1;
      onProgress(`Florida amendment summaries (${done}/${need.length})…`);
    }
  }
  await Promise.all(
    Array.from({ length: Math.min(concurrency, need.length) }, () => worker())
  );
  return report;
}

function emptyMeasureFinance(partial = {}) {
  return {
    account: "",
    contributions_sum: 0,
    contributions_sum_display: "—",
    top_contributors: [],
    line_count: 0,
    committee_url: "",
    trefin_url: "",
    note: "",
    committee_name: "",
    role: "sponsor",
    oppose: [],
    ...partial,
  };
}

async function fetchTrefinFinance(account, parseFin, trefinUrl, meta = {}) {
  const url = trefinUrl(account);
  const key = `fl:trefin:contrib:${account}`;
  const html = await tryFetchText(url, key, 24 * 60 * 60 * 1000);
  let fin = null;
  if (html && parseFin) {
    try {
      fin = parseFin(html, account, 8);
    } catch (e) {
      console.warn("[state] TreFin parse", account, e);
    }
  }
  if (fin) {
    return {
      ...fin,
      committee_name: meta.committee_name || fin.committee_name || "",
      role: meta.role || fin.role || "sponsor",
      oppose: [],
    };
  }
  return emptyMeasureFinance({
    account,
    committee_url: `https://dos.elections.myflorida.com/committees/ComDetail.asp?account=${encodeURIComponent(account)}`,
    trefin_url: url,
    note: html
      ? "TreFin response could not be parsed (HTML shape may have changed)."
      : "TreFin fetch empty — enable Wisp (Settings) or retry; public Wisp may throttle.",
    committee_name: meta.committee_name || "",
    role: meta.role || "sponsor",
  });
}

/**
 * One DOS committee name search (cached). Hits used to match “No on N” PACs.
 */
async function fetchFlOpposeCommitteeHits(wasm) {
  const url =
    (wasm.fl_com_lkup_by_name_url_js && wasm.fl_com_lkup_by_name_url_js()) ||
    "https://dos.elections.myflorida.com/committees/ComLkupByName.asp";
  const form =
    (wasm.fl_com_lkup_by_name_form_js &&
      wasm.fl_com_lkup_by_name_form_js("No on")) ||
    "searchtype=1&comName=No+on&LkupTypeName=C&NameSearchBtn=Search+by+Name";
  const cacheKey = "fl:committees:lkup:no-on";
  const html = await tryPostForm(url, form, cacheKey, 24 * 60 * 60 * 1000);
  if (!html || !wasm.parse_com_lkup_by_name_js) return [];
  try {
    const hits = wasm.parse_com_lkup_by_name_js(html);
    return Array.isArray(hits) ? hits : [];
  } catch (e) {
    console.warn("[state] parse ComLkupByName", e);
    return [];
  }
}

/**
 * FL measure PAC finance via DOS TreFin (Wisp). Attaches `m.finance` when possible.
 * Also discovers oppose PACs via committee name search (“No on N”).
 */
export async function enrichFlMeasureFinance(report, onProgress = () => {}) {
  const measures = report?.measures;
  if (!Array.isArray(measures) || !measures.length) return report;
  if (!hasWispConfigured() && !getCorsProxy()) return report;

  let wasm;
  let parseAcct;
  let trefinUrl;
  let parseFin;
  try {
    wasm = await import("./pkg/electionizer_wasm.js");
    parseAcct = wasm.parse_dos_account_js;
    trefinUrl = wasm.fl_trefin_contrib_url_js;
    parseFin = wasm.parse_trefin_finance_js;
  } catch (e) {
    console.warn("[state] measure finance wasm", e);
    return report;
  }

  const need = measures.filter((m) => m.source_url && !m.finance);
  if (!need.length) return report;

  onProgress(`Florida amendment finance (0/${need.length})…`);
  let done = 0;
  const concurrency = 2;
  let idx = 0;
  async function worker() {
    while (idx < need.length) {
      const i = idx++;
      const m = need[i];
      try {
        const account = parseAcct(m.source_url);
        if (!account || account === "10") {
          // Legislature-referred placeholder — no sponsor PAC TreFin dump
          m.finance = emptyMeasureFinance({
            account: account || "",
            committee_url: account
              ? `https://dos.elections.myflorida.com/committees/ComDetail.asp?account=${encodeURIComponent(account)}`
              : "",
            note: "Legislature-referred measure — no sponsor PAC itemized file on DOS TreFin.",
          });
        } else {
          m.finance = await fetchTrefinFinance(account, parseFin, trefinUrl, {
            role: "sponsor",
          });
        }
      } catch (e) {
        console.warn("[state] TreFin", m.source_url, e);
        m.finance = emptyMeasureFinance({
          note: `TreFin failed: ${e.message || e}`,
        });
      }
      done += 1;
      onProgress(`Florida amendment finance (${done}/${need.length})…`);
    }
  }
  await Promise.all(
    Array.from({ length: Math.min(concurrency, need.length) }, () => worker())
  );

  // Oppose PAC discovery — one name search, then TreFin per matched Active PAC.
  onProgress("Florida oppose PAC search…");
  let hits = [];
  try {
    hits = await fetchFlOpposeCommitteeHits(wasm);
  } catch (e) {
    console.warn("[state] oppose PAC search", e);
  }
  if (!hits.length || !wasm.select_oppose_committees_js) {
    for (const m of need) {
      if (m.finance && !Array.isArray(m.finance.oppose)) m.finance.oppose = [];
    }
    return report;
  }

  const amendFn = wasm.amendment_number_from_code_js;
  const hitsJson = JSON.stringify(hits);
  /** @type {Map<string, object>} */
  const trefinByAccount = new Map();
  const opposeJobs = [];

  for (const m of need) {
    if (!m.finance) continue;
    const n = amendFn ? amendFn(m.measure_code || undefined) : null;
    if (n == null) {
      m.finance.oppose = [];
      continue;
    }
    let selected = [];
    try {
      selected = wasm.select_oppose_committees_js(hitsJson, n, 2) || [];
    } catch (e) {
      console.warn("[state] select oppose", m.measure_code, e);
    }
    if (!Array.isArray(selected) || !selected.length) {
      m.finance.oppose = [];
      if (!m.finance.note) {
        m.finance.note = `No Active “No on ${n}” PAC found in DOS committee search.`;
      }
      continue;
    }
    opposeJobs.push({ m, selected });
  }

  const uniqueAccounts = [];
  const seenAcct = new Set();
  for (const job of opposeJobs) {
    for (const hit of job.selected) {
      const a = hit && hit.account;
      if (!a || seenAcct.has(a)) continue;
      seenAcct.add(a);
      uniqueAccounts.push(hit);
    }
  }

  onProgress(`Florida oppose PAC finance (0/${uniqueAccounts.length})…`);
  let odone = 0;
  let oidx = 0;
  async function oworker() {
    while (oidx < uniqueAccounts.length) {
      const j = oidx++;
      const hit = uniqueAccounts[j];
      try {
        const fin = await fetchTrefinFinance(hit.account, parseFin, trefinUrl, {
          committee_name: hit.name || "",
          role: "oppose",
        });
        trefinByAccount.set(String(hit.account), fin);
      } catch (e) {
        console.warn("[state] oppose TreFin", hit.account, e);
        trefinByAccount.set(
          String(hit.account),
          emptyMeasureFinance({
            account: hit.account,
            committee_name: hit.name || "",
            role: "oppose",
            note: `Oppose TreFin failed: ${e.message || e}`,
            committee_url: `https://dos.elections.myflorida.com/committees/ComDetail.asp?account=${encodeURIComponent(hit.account)}`,
          })
        );
      }
      odone += 1;
      onProgress(
        `Florida oppose PAC finance (${odone}/${uniqueAccounts.length})…`
      );
    }
  }
  if (uniqueAccounts.length) {
    await Promise.all(
      Array.from(
        { length: Math.min(concurrency, uniqueAccounts.length) },
        () => oworker()
      )
    );
  }

  for (const job of opposeJobs) {
    job.m.finance.oppose = job.selected
      .map((hit) => trefinByAccount.get(String(hit.account)))
      .filter(Boolean);
  }

  return report;
}

/**
 * MD MDCRIS ballot-issue committee finance (live). Prefer over FTM when MD.
 */
export async function enrichMdMeasureFinance(report, onProgress = () => {}) {
  const measures = report?.measures;
  if (!Array.isArray(measures) || !measures.length) return report;
  const st = String(report.state_code || report.state || "")
    .trim()
    .toUpperCase();
  if (st !== "MD") return report;

  const need = measures.filter((m) => {
    if (!m.finance) return true;
    if (Number(m.finance.line_count) > 0) return false;
    if (m.finance.source && m.finance.source !== "md_cf_measure") {
      if (m.finance.source === "fl_trefin" || m.finance.source === "ftm_measure")
        return false;
    }
    return true;
  });
  if (!need.length) return report;

  let wasm;
  try {
    wasm = await import("./pkg/electionizer_wasm.js");
  } catch (e) {
    console.warn("[state] MD measure finance wasm", e);
    return report;
  }
  if (
    !wasm.md_cf_committee_list_url_js ||
    !wasm.parse_md_cf_committee_list_json_js ||
    !wasm.md_measure_finance_from_hits_js
  ) {
    return report;
  }

  const cycle = Number(report.cycle) || getCycle() || 2026;
  const listUrl = wasm.md_cf_committee_list_url_js();
  const termSet = new Set(["Question"]);
  for (const m of need) {
    let terms = [];
    try {
      terms =
        (wasm.md_measure_search_terms_js &&
          wasm.md_measure_search_terms_js(
            m.measure_code || "",
            m.title || "",
            cycle
          )) ||
        [];
    } catch (_) {
      /* ignore */
    }
    for (const t of terms || []) {
      if (t) termSet.add(String(t));
    }
  }

  onProgress(`MDCRIS ballot-issue committees…`);
  /** @type {Map<string, object>} */
  const byGuid = new Map();
  const terms = Array.from(termSet).slice(0, 12);
  for (let i = 0; i < terms.length; i++) {
    const term = terms[i];
    onProgress(`MDCRIS measure search (${i + 1}/${terms.length})…`);
    const body =
      (wasm.md_cf_committee_list_body_js &&
        wasm.md_cf_committee_list_body_js(term, 50)) ||
      JSON.stringify({ pageNumber: 1, pageSize: 50, filerName: term });
    const cacheKey = `md:cf:measure:${term.toLowerCase()}`;
    let raw = "";
    try {
      raw = await tryPostJson(listUrl, body, cacheKey, 24 * 60 * 60 * 1000, {
        minLength: 20,
        headers: {
          Origin: "https://campaignfinance.maryland.gov",
          Referer: "https://campaignfinance.maryland.gov/",
        },
      });
    } catch (e) {
      console.warn("[state] MD measure list", term, e);
      continue;
    }
    if (!raw) continue;
    let hits = [];
    try {
      hits = wasm.parse_md_cf_committee_list_json_js(raw) || [];
    } catch (e) {
      console.warn("[state] MD measure parse", term, e);
      continue;
    }
    for (const h of hits) {
      const g = h.filer_registration_guid || h.filing_entity_id;
      if (g != null && g !== "") byGuid.set(String(g), h);
    }
  }

  const allHits = Array.from(byGuid.values());
  if (!allHits.length) return report;
  const hitsJson = JSON.stringify(allHits);

  let done = 0;
  for (const m of need) {
    try {
      const fin = wasm.md_measure_finance_from_hits_js(
        hitsJson,
        m.measure_code || "",
        m.title || "",
        cycle
      );
      if (fin && typeof fin === "object") {
        m.finance = fin;
      }
    } catch (e) {
      console.warn("[state] MD measure finance match", m.measure_code, e);
    }
    done += 1;
    onProgress(`MDCRIS measure finance (${done}/${need.length})…`);
  }
  return report;
}

/**
 * FTM ballot-measure finance (public HTML via Wisp). Fills `m.finance` when
 * TreFin (or other) has not already attached finance. Data through FTM max year.
 */
export async function enrichFtmMeasureFinance(report, onProgress = () => {}) {
  const measures = report?.measures;
  if (!Array.isArray(measures) || !measures.length) return report;
  if (!hasWispConfigured() && !getCorsProxy()) return report;

  const need = measures.filter((m) => {
    if (!m.finance) return true;
    if (Number(m.finance.line_count) > 0) return false;
    // Keep FL TreFin / MDCRIS (or other official) empty notes; only fill bare rows.
    if (
      m.finance.source &&
      m.finance.source !== "ftm_measure" &&
      m.finance.source !== "md_cf_measure"
    )
      return false;
    if (
      m.finance.account &&
      m.finance.source !== "ftm_measure" &&
      m.finance.source !== "md_cf_measure"
    )
      return false;
    // FL TreFin path sets account/committee_url without source — leave those.
    if (m.finance.trefin_url || m.finance.committee_url) {
      if (/dos\.elections\.myflorida|trefin/i.test(
        `${m.finance.trefin_url || ""} ${m.finance.committee_url || ""}`
      )) {
        return false;
      }
      if (/campaignfinance\.maryland/i.test(m.finance.committee_url || "")) {
        return false;
      }
    }
    if (m.finance.note && /TreFin|Legislature-referred|DOS|MDCRIS/i.test(m.finance.note)) {
      return false;
    }
    return m.finance.source !== "ftm_measure";
  });
  if (!need.length) return report;

  let wasm;
  try {
    wasm = await import("./pkg/electionizer_wasm.js");
  } catch (e) {
    console.warn("[state] FTM measure finance wasm", e);
    return report;
  }
  if (
    !wasm.ftm_measures_list_url_js ||
    !wasm.parse_ftm_measures_list_html_js ||
    !wasm.match_ftm_measure_js
  ) {
    return report;
  }

  const st = String(report.state_code || report.state || "")
    .trim()
    .toUpperCase();
  if (!st || st.length !== 2) return report;

  const cycle = Number(report.cycle) || getCycle() || 2026;
  const year =
    (typeof wasm.ftm_data_year_js === "function" &&
      wasm.ftm_data_year_js(cycle)) ||
    Math.min(cycle, 2024);

  if (year < cycle) {
    onProgress(
      `FollowTheMoney measure list ${st} ${year} (FTM through ${year}; ballot cycle ${cycle})…`
    );
  } else {
    onProgress(`FollowTheMoney measure list ${st} ${year}…`);
  }
  const listUrl = wasm.ftm_measures_list_url_js(st, year);
  const listKey = `ftm:measures:${st}:${year}`;
  let listHtml = "";
  try {
    listHtml = await tryFetchText(listUrl, listKey, 24 * 60 * 60 * 1000);
  } catch (e) {
    console.warn("[state] FTM measures list", e);
    return report;
  }
  if (!listHtml) {
    for (const m of need) {
      if (!m.finance) {
        m.finance = emptyMeasureFinance({
          note: `FollowTheMoney measure list empty for ${st} ${year} — enable Wisp or retry.`,
          source: "ftm_measure",
        });
      }
    }
    return report;
  }

  let hits = [];
  try {
    hits = wasm.parse_ftm_measures_list_html_js(listHtml) || [];
  } catch (e) {
    console.warn("[state] parse FTM measures list", e);
    return report;
  }
  if (!Array.isArray(hits) || !hits.length) {
    for (const m of need) {
      if (!m.finance) {
        m.finance = emptyMeasureFinance({
          note: `No FollowTheMoney ballot measures published for ${st} ${year} (FTM through 2024).`,
          source: "ftm_measure",
        });
      }
    }
    return report;
  }
  const hitsJson = JSON.stringify(hits);

  const jobs = [];
  for (const m of need) {
    let matchOut = null;
    try {
      matchOut = wasm.match_ftm_measure_js(
        hitsJson,
        m.measure_code || "",
        m.title || ""
      );
    } catch (e) {
      console.warn("[state] match FTM measure", m.measure_code, e);
    }
    const kind = matchOut && matchOut.kind;
    if (kind === "unique" && matchOut.hit) {
      jobs.push({ m, hit: matchOut.hit });
    } else if (!m.finance) {
      m.finance = emptyMeasureFinance({
        note:
          kind === "ambiguous"
            ? "FollowTheMoney measure match ambiguous — omitted."
            : `No FollowTheMoney measure match for ${year} (data through 2024).`,
        source: "ftm_measure",
      });
    }
  }
  if (!jobs.length) return report;

  onProgress(`FollowTheMoney measure finance (0/${jobs.length})…`);
  let done = 0;
  const concurrency = 2;
  let idx = 0;
  async function worker() {
    while (idx < jobs.length) {
      const i = idx++;
      const { m, hit } = jobs[i];
      try {
        const eid = hit.eid;
        const ovUrl = wasm.ftm_measure_overview_url_js(eid);
        const ovKey = `ftm:measure:ov:${eid}`;
        const ovHtml = await tryFetchText(ovUrl, ovKey, 24 * 60 * 60 * 1000);
        let overview = null;
        if (ovHtml && wasm.parse_ftm_measure_overview_html_js) {
          try {
            overview = wasm.parse_ftm_measure_overview_html_js(ovHtml);
          } catch (e) {
            console.warn("[state] FTM overview parse", eid, e);
          }
        }
        if (!overview) {
          m.finance = emptyMeasureFinance({
            account: eid,
            committee_url:
              (wasm.ftm_measure_show_me_url_js &&
                wasm.ftm_measure_show_me_url_js(eid)) ||
              "",
            note: "FollowTheMoney measure overview could not be parsed.",
            source: "ftm_measure",
            committee_name: hit.name || "",
          });
        } else {
          let supportDonors = [];
          let opposeCmtes = [];
          try {
            const dUrl = wasm.ftm_measure_donors_url_js(eid, true);
            const dHtml = await tryFetchText(
              dUrl,
              `ftm:measure:sup-donors:${eid}`,
              24 * 60 * 60 * 1000
            );
            if (dHtml && wasm.parse_ftm_measure_entity_table_html_js) {
              supportDonors =
                wasm.parse_ftm_measure_entity_table_html_js(dHtml, 8) || [];
            }
          } catch (e) {
            console.warn("[state] FTM support donors", eid, e);
          }
          try {
            const oUrl = wasm.ftm_measure_committees_url_js(eid, false);
            const oHtml = await tryFetchText(
              oUrl,
              `ftm:measure:opp-cmtes:${eid}`,
              24 * 60 * 60 * 1000
            );
            if (oHtml && wasm.parse_ftm_measure_entity_table_html_js) {
              opposeCmtes =
                wasm.parse_ftm_measure_entity_table_html_js(oHtml, 5) || [];
            }
          } catch (e) {
            console.warn("[state] FTM oppose committees", eid, e);
          }
          if (wasm.ftm_measure_finance_from_parts_js) {
            m.finance = wasm.ftm_measure_finance_from_parts_js(
              JSON.stringify(hit),
              JSON.stringify(overview),
              JSON.stringify(supportDonors),
              JSON.stringify(opposeCmtes),
              8
            );
          } else {
            m.finance = emptyMeasureFinance({
              account: eid,
              contributions_sum: overview.support_total || 0,
              contributions_sum_display: overview.support_display || "—",
              line_count: overview.support_total > 0 ? 1 : 0,
              committee_url:
                (wasm.ftm_measure_show_me_url_js &&
                  wasm.ftm_measure_show_me_url_js(eid)) ||
                "",
              note: overview.note || "",
              source: "ftm_measure",
              committee_name: hit.name || "",
              role: "sponsor",
              oppose:
                overview.oppose_total > 0
                  ? [
                      {
                        contributions_sum: overview.oppose_total,
                        contributions_sum_display:
                          overview.oppose_display || "—",
                        line_count: 1,
                        committee_name: "Oppose (FTM total)",
                        role: "oppose",
                        committee_url:
                          (wasm.ftm_measure_show_me_url_js &&
                            wasm.ftm_measure_show_me_url_js(eid)) ||
                          "",
                      },
                    ]
                  : [],
            });
          }
        }
      } catch (e) {
        console.warn("[state] FTM measure finance", hit && hit.eid, e);
        m.finance = emptyMeasureFinance({
          note: `FollowTheMoney measure finance failed: ${e.message || e}`,
          source: "ftm_measure",
        });
      }
      done += 1;
      onProgress(`FollowTheMoney measure finance (${done}/${jobs.length})…`);
    }
  }
  await Promise.all(
    Array.from({ length: Math.min(concurrency, jobs.length) }, () => worker())
  );
  return report;
}

function measureCycleYear(report) {
  const d = String(report?.election_date || "");
  const y = parseInt(d.slice(0, 4), 10);
  if (Number.isFinite(y) && y >= 2000 && y <= 2100) return y;
  return getCycle();
}

function financeSourceLabel(f) {
  if (!f) return "public filings";
  return (
    f.source_label ||
    (f.source === "ftm_measure"
      ? "FollowTheMoney"
      : f.source === "md_cf_measure"
        ? "MDCRIS"
        : f.source === "fl_trefin"
          ? "FL TreFin"
          : f.source || "public filings")
  );
}

/**
 * L8 — Ballotpedia Support/Oppose lists + already-loaded committee sides.
 * Mutates `report.measures[].endorsements`. Best-effort; unique title match only.
 */
export async function enrichMeasureEndorsements(report, onProgress = () => {}) {
  const measures = report?.measures;
  if (!Array.isArray(measures) || !measures.length) return report;

  let wasm;
  try {
    wasm = await import("./pkg/electionizer_wasm.js");
  } catch (e) {
    console.warn("[state] measure endorsements wasm", e);
    return report;
  }

  const need = measures.filter((m) => !Array.isArray(m.endorsements));
  if (!need.length) return report;

  const year = measureCycleYear(report);
  const state = String(report.state_code || report.state || "").trim();
  let links = [];
  const indexUrl =
    wasm.ballotpedia_state_measures_url_js &&
    wasm.ballotpedia_state_measures_url_js(state, year);
  if (indexUrl && (hasWispConfigured() || getCorsProxy())) {
    try {
      onProgress(`Ballotpedia measure list (${state} ${year})…`);
      const html = await tryFetchText(
        indexUrl,
        `bp:measures:${state}:${year}`,
        24 * 60 * 60 * 1000
      );
      if (html && wasm.ballotpedia_measure_links_from_index_js) {
        links = wasm.ballotpedia_measure_links_from_index_js(html) || [];
      }
    } catch (e) {
      console.warn("[state] BP measure index", indexUrl, e);
    }
  }

  const linksJson = JSON.stringify(links);
  onProgress(`Measure endorsements (0/${need.length})…`);
  let done = 0;
  const concurrency = 2;
  let idx = 0;
  async function worker() {
    while (idx < need.length) {
      const i = idx++;
      const m = need[i];
      let ends = [];
      const f = m.finance;
      if (f && wasm.endorsements_from_measure_sides_js) {
        try {
          ends =
            wasm.endorsements_from_measure_sides_js(
              f.committee_name || "",
              f.committee_url || f.profile_url || f.show_me_url || undefined,
              financeSourceLabel(f),
              JSON.stringify(f.oppose || [])
            ) || [];
        } catch (e) {
          console.warn("[state] measure committee endorsements", e);
        }
      }
      let hit = null;
      if (links.length && wasm.match_ballotpedia_measure_js) {
        try {
          hit = wasm.match_ballotpedia_measure_js(
            linksJson,
            m.title || "",
            m.measure_code || undefined
          );
        } catch (e) {
          console.warn("[state] BP measure match", m.title, e);
        }
      }
      const pageUrl = hit && hit.url;
      if (pageUrl && wasm.endorsements_from_ballotpedia_measure_html_js) {
        try {
          const html = await tryFetchText(
            pageUrl,
            `bp:measure:${pageUrl}`,
            24 * 60 * 60 * 1000
          );
          const ok =
            html &&
            (!wasm.ballotpedia_html_matches_measure_js ||
              wasm.ballotpedia_html_matches_measure_js(
                html,
                m.title || "",
                m.measure_code || undefined
              ));
          if (ok) {
            const extra =
              wasm.endorsements_from_ballotpedia_measure_html_js(html, pageUrl) ||
              [];
            if (wasm.merge_endorsement_lists_js) {
              ends = wasm.merge_endorsement_lists_js(
                JSON.stringify(ends),
                JSON.stringify(extra)
              );
            } else {
              ends = ends.concat(extra);
            }
            m.ballotpedia_url = pageUrl;
          }
        } catch (e) {
          console.warn("[state] BP measure page", pageUrl, e);
        }
      }
      m.endorsements = Array.isArray(ends) ? ends : [];
      done += 1;
      onProgress(`Measure endorsements (${done}/${need.length})…`);
    }
  }
  await Promise.all(
    Array.from({ length: Math.min(concurrency, need.length) }, () => worker())
  );
  return report;
}

const NC_CANDIDATE_LISTS_PAGE =
  "https://www.ncsbe.gov/results-data/candidate-lists";

/**
 * Pick NCSBE referendum PDF URL(s) for a cycle from candidate-lists HTML.
 * Prefer November/general (month 11) over primary (03) when both exist.
 * @param {string} html
 * @param {number} cycle
 * @returns {string[]}
 */
export function pickNcReferendumPdfUrls(html, cycle) {
  const year = String(cycle);
  const found = new Set();
  const re =
    /https?:\/\/s3\.amazonaws\.com\/dl\.ncsbe\.gov\/[^"'\\\s>]+referendums_[^"'\\\s>]+\.pdf/gi;
  const hrefRe = /href=["']([^"']*referendums_[^"']+\.pdf)["']/gi;
  let m;
  while ((m = re.exec(html || ""))) found.add(m[0].replace(/&amp;/g, "&"));
  while ((m = hrefRe.exec(html || ""))) {
    let u = m[1].replace(/&amp;/g, "&");
    if (u.startsWith("//")) u = `https:${u}`;
    else if (u.startsWith("/")) u = `https://s3.amazonaws.com${u}`;
    else if (!/^https?:/i.test(u)) {
      u = `https://s3.amazonaws.com/dl.ncsbe.gov/${u.replace(/^\//, "")}`;
    }
    found.add(u);
  }
  const list = [...found].filter((u) => u.includes(year) || u.includes(`_${year}`));
  const score = (u) => {
    const low = u.toLowerCase();
    let s = 0;
    if (/referendums_20\d{2}11/.test(low) || /_11\d{2}20\d{2}/.test(low)) s += 20;
    if (low.includes("general")) s += 10;
    if (/referendums_20\d{2}03/.test(low) || low.includes("primary")) s += 1;
    return s;
  };
  list.sort((a, b) => score(b) - score(a));
  return list;
}

/**
 * NCSBE referendum list → plain text via WASM PDF extract.
 * @param {number} cycle
 * @param {(msg: string) => void} [onProgress]
 * @returns {Promise<{ text: string, url: string, note: string }>}
 */
async function fetchNcReferendumText(cycle, onProgress = () => {}) {
  onProgress("North Carolina ballot measures (NCSBE)…");
  const textKey = `nc:measures:text:${cycle}`;
  const urlKey = `nc:measures:url:${cycle}`;
  const cachedText = await cacheGet(textKey);
  const cachedUrl = await cacheGet(urlKey);
  if (cachedText && cachedText.length > 80 && /REFERENDUM|CONSTITUTIONAL|BONDS/i.test(cachedText)) {
    return {
      text: cachedText,
      url: cachedUrl || "",
      note: "Using cached NCSBE referendum list text.",
    };
  }

  /** @type {string[]} */
  let urls = [];
  try {
    const page = await tryFetchText(
      NC_CANDIDATE_LISTS_PAGE,
      `nc:candidate-lists-page`
    ).catch(() => "");
    urls = pickNcReferendumPdfUrls(page || "", cycle);
  } catch {
    /* ignore */
  }
  // Common general-election layout (works for 2026).
  const fallback = `https://s3.amazonaws.com/dl.ncsbe.gov/Elections/${cycle}/Candidate%20Filing/referendums_${cycle}1103.pdf`;
  if (!urls.includes(fallback)) urls.push(fallback);

  let lastErr = null;
  for (const pdfUrl of urls) {
    try {
      const resp = await fetch(pdfUrl);
      if (!resp.ok) {
        lastErr = new Error(`HTTP ${resp.status} ${pdfUrl}`);
        continue;
      }
      const buf = new Uint8Array(await resp.arrayBuffer());
      if (buf.length < 500) {
        lastErr = new Error("PDF too small");
        continue;
      }
      const wasm = await import("./pkg/electionizer_wasm.js");
      if (typeof wasm.extract_pdf_text_js !== "function") {
        return {
          text: "",
          url: "",
          note: "NC measures: WASM PDF extract unavailable (rebuild pkg).",
        };
      }
      const text = wasm.extract_pdf_text_js(buf);
      if (!text || text.length < 80) {
        lastErr = new Error("PDF text extract empty");
        continue;
      }
      await cachePut(textKey, text, 24 * 60 * 60 * 1000);
      await cachePut(urlKey, pdfUrl, 24 * 60 * 60 * 1000);
      return {
        text,
        url: pdfUrl,
        note: `Using NCSBE referendum list PDF (${pdfUrl.split("/").pop()}).`,
      };
    } catch (e) {
      lastErr = e;
    }
  }
  return {
    text: "",
    url: "",
    note: `NC measures: referendum PDF fetch/extract failed${
      lastErr ? ` (${lastErr.message || lastErr})` : ""
    }.`,
  };
}

/**
 * Prefer `{cycle} - General`, else `{cycle} - Primary`, else first label containing year.
 * @param {unknown} electionsJson
 * @param {number} cycle
 * @returns {number|null}
 */
export function pickAzMeasuresElectionId(electionsJson, cycle) {
  let arr;
  try {
    arr = typeof electionsJson === "string" ? JSON.parse(electionsJson) : electionsJson;
  } catch {
    return null;
  }
  if (!Array.isArray(arr)) return null;
  const year = String(cycle);
  let general = null;
  let primary = null;
  let anyYear = null;
  for (const item of arr) {
    const label = String(item?.value ?? "").toLowerCase();
    if (!label.includes(year)) continue;
    const key = item?.key;
    const id = typeof key === "number" ? key : parseInt(String(key ?? ""), 10);
    if (!Number.isFinite(id)) continue;
    if (label.includes("general")) general = id;
    else if (label.includes("primary")) primary = id;
    else if (anyYear == null) anyYear = id;
  }
  return general ?? primary ?? anyYear;
}

/**
 * @param {unknown} countiesJson
 * @param {string} countyName
 * @returns {number|null}
 */
export function pickAzMeasuresCountyId(countiesJson, countyName) {
  const want = String(countyName || "")
    .trim()
    .toLowerCase()
    .replace(/\s+county\s*$/i, "")
    .replace(/[^a-z0-9]/g, "");
  if (!want) return null;
  let arr;
  try {
    arr = typeof countiesJson === "string" ? JSON.parse(countiesJson) : countiesJson;
  } catch {
    return null;
  }
  if (!Array.isArray(arr)) return null;
  for (const item of arr) {
    const label = String(item?.value ?? "")
      .trim()
      .toLowerCase()
      .replace(/\s+county\s*$/i, "")
      .replace(/[^a-z0-9]/g, "");
    if (label !== want) continue;
    const key = item?.key;
    const id = typeof key === "number" ? key : parseInt(String(key ?? ""), 10);
    if (Number.isFinite(id)) return id;
  }
  return null;
}

/**
 * Clean Elections BallotMeasures HTML (statewide + ZIP county when known).
 * @param {number} cycle
 * @param {string} [countyName]
 * @param {(msg: string) => void} [onProgress]
 * @returns {Promise<{ html: string, note: string }>}
 */
async function fetchAzMeasuresHtml(cycle, countyName = "", onProgress = () => {}) {
  onProgress("Arizona ballot measures (Clean Elections)…");
  const electionsBody = await tryFetchText(
    AZ_MEASURES_ELECTIONS_URL,
    `az:measures:elections`
  ).catch(() => "");
  if (!electionsBody || electionsBody.length < 10) {
    return { html: "", note: "AZ measures: ElectionsForBM fetch failed." };
  }
  const electionId = pickAzMeasuresElectionId(electionsBody, cycle);
  if (electionId == null) {
    return {
      html: "",
      note: `AZ measures: no Clean Elections election for cycle ${cycle}.`,
    };
  }

  let countyId = 0;
  if (countyName) {
    const countiesUrl = `${AZ_MEASURES_COUNTIES_BASE}?election=${electionId}`;
    const countiesBody = await tryFetchText(
      countiesUrl,
      `az:measures:counties:${electionId}`
    ).catch(() => "");
    const cid = pickAzMeasuresCountyId(countiesBody, countyName);
    if (cid != null) countyId = cid;
  }

  const listUrl = `${AZ_MEASURES_LIST_BASE}?election=${electionId}&county=${countyId}&lang=en&mode=all`;
  const html = await tryFetchText(
    listUrl,
    `az:measures:${electionId}:${countyId}`
  ).catch(() => "");
  if (!html || html.length < 80 || !/<details/i.test(html)) {
    return {
      html: "",
      note: `AZ measures: BallotMeasures empty for election ${electionId} (county ${countyId}).`,
    };
  }
  return {
    html,
    note: `Using AZ Clean Elections measures (election ${electionId}, county ${countyId}).`,
  };
}

async function loadFlWasm() {
  try {
    return await import("./pkg/electionizer_wasm.js");
  } catch {
    return null;
  }
}

async function flSoeListUrl(county) {
  const wasm = await loadFlWasm();
  if (wasm && typeof wasm.fl_soe_candidate_list_url_js === "function") {
    return wasm.fl_soe_candidate_list_url_js(county) || "";
  }
  const c = String(county || "")
    .replace(/\s+County$/i, "")
    .trim();
  if (!c) return "";
  return `https://www.voterfocus.com/CampaignFinance/candidate_pr.php?c=${encodeURIComponent(c)}`;
}

async function fetchFlSoeList(county, onProgress) {
  const url = await flSoeListUrl(county);
  if (!url) return { html: "", note: "" };
  onProgress(`Florida county SOE candidate list (${county})…`);
  try {
    const html = await tryFetchText(url, `fl:soe:vf:${county.toLowerCase()}:list`);
    if (html && html.length > 400 && /officename|candidate_pr\.php\?op=cv/i.test(html)) {
      return { html, note: "Using county Supervisor of Elections (VoterFocus) candidate list." };
    }
    if (html) {
      return { html: "", note: "FL SOE candidate list returned unexpected content." };
    }
    return { html: "", note: missingStateTransportNote("FL SOE VoterFocus list") };
  } catch (e) {
    return { html: "", note: `FL SOE candidate list failed: ${e.message || e}` };
  }
}

async function fetchFlSampleBallotText(precinct, party, onProgress) {
  const wasm = await loadFlWasm();
  if (!wasm || typeof wasm.brevard_sample_ballot_url_js !== "function") {
    return { text: "", note: "" };
  }
  let url = "";
  try {
    url = wasm.brevard_sample_ballot_url_js(precinct, party, "") || "";
  } catch {
    return { text: "", note: "" };
  }
  if (!url) return { text: "", note: "" };
  const cacheKey = `fl:sample_ballot:${url}`;
  const cached = await cacheGet(cacheKey);
  if (cached && String(cached).length > 80) {
    return { text: cached, note: "Using cached official sample-ballot text (precinct)." };
  }
  onProgress("Official sample ballot (precinct filter)…");
  try {
    let buf = null;
    if (hasWispConfigured()) {
      buf = await curlFetchBytes(url, { headers: { Accept: "application/pdf,*/*" } });
    } else {
      const res = await fetch(url);
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      buf = new Uint8Array(await res.arrayBuffer());
    }
    if (!buf || buf.length < 500) {
      return { text: "", note: "Sample ballot PDF too small to extract." };
    }
    if (typeof wasm.extract_pdf_text_js !== "function") {
      return { text: "", note: "Sample ballot: WASM PDF extract unavailable." };
    }
    const text = wasm.extract_pdf_text_js(buf);
    if (!text || text.length < 80) {
      return { text: "", note: "Sample ballot PDF extract empty (image-only or blocked)." };
    }
    await cachePut(cacheKey, text, 24 * 60 * 60 * 1000);
    return { text, note: "Using official sample-ballot text to filter county districts." };
  } catch (e) {
    return { text: "", note: `Sample ballot fetch skipped: ${e.message || e}` };
  }
}

/**
 * @param {string} state
 * @param {(msg: string) => void} [onProgress]
 * @param {{ county?: string, cd?: number, legDistrict?: number, precinct?: string, party?: string }} [opts]
 */
export async function fetchStateBodies(state, onProgress = () => {}, opts = {}) {
  const st = (state || "").toUpperCase();
  const cycle = getCycle();
  const countyName = String(opts.county || "").trim();
  const cdNum = Number(opts.cd) || 0;
  const legDistrict = Number(opts.legDistrict) || 0;
  const out = {
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

  if (hasWispConfigured()) {
    onProgress("Loading libcurl.js (Wisp)…");
    try {
      await ensureCurl();
      out.notes.push("Using libcurl.js + Wisp for state HTTP.");
    } catch (e) {
      out.notes.push(
        `libcurl.js failed to load: ${e.message || e}. Falling back to CORS proxy/direct.`
      );
    }
  }

  if (st === "NC") {
    onProgress("North Carolina candidate filings (NCSBE)…");
    try {
      const url = `https://s3.amazonaws.com/dl.ncsbe.gov/Elections/${cycle}/Candidate%20Filing/Candidate_Listing_${cycle}.csv`;
      // CORS-open S3 — direct fetch preferred (no Wisp required).
      const { body } = await cachedFetch(url, {
        key: `nc:candidates:${cycle}`,
        ttlMs: 24 * 60 * 60 * 1000,
      });
      if (body && body.length > 200 && /contest_name/i.test(body)) {
        out.ncCandidates = body;
        out.notes.push("Using NCSBE candidate listing CSV (direct).");
      } else {
        out.notes.push("NC candidate listing CSV empty or unexpected.");
      }
    } catch (e) {
      out.notes.push(`NC candidate listing failed: ${e.message || e}`);
    }
    try {
      const m = await fetchNcReferendumText(cycle, onProgress);
      out.ncMeasures = m.text || "";
      out.ncMeasuresUrl = m.url || "";
      if (m.note) out.notes.push(m.note);
    } catch (e) {
      out.notes.push(`NC measures failed: ${e.message || e}`);
    }
    return out;
  }

  if (st === "MD") {
    onProgress("Maryland candidate filings (SBE)…");
    // elections.maryland.gov is not CORS-open — Wisp/libcurl preferred.
    const mdBase = `https://elections.maryland.gov/elections/${cycle}`;
    const tryMdPair = async (phase, label) => {
      const folder =
        phase === "GG" ? "general_candidates" : "primary_candidates";
      const stateUrl = `${mdBase}/${folder}/${cycle}_${phase}_statewide_candidatelist.csv`;
      const localUrl = `${mdBase}/${folder}/${cycle}_${phase}_all_counties_candidatelist.csv`;
      const [statewide, local] = await Promise.all([
        tryFetchText(stateUrl, `md:${phase}:statewide:${cycle}`).catch(() => ""),
        tryFetchText(localUrl, `md:${phase}:local:${cycle}`).catch(() => ""),
      ]);
      const okState =
        statewide &&
        statewide.length > 200 &&
        /Office Name/i.test(statewide);
      const okLocal =
        local && local.length > 200 && /Office Name/i.test(local);
      if (okState || okLocal) {
        return {
          statewide: okState ? statewide : "",
          local: okLocal ? local : "",
          label,
        };
      }
      return null;
    };
    try {
      let pair = await tryMdPair("GG", "general");
      if (!pair) pair = await tryMdPair("GP", "primary");
      if (pair) {
        out.mdStatewide = pair.statewide;
        out.mdLocal = pair.local;
        out.mdPhase = pair.label === "general" ? "GG" : "GP";
        out.notes.push(
          `Using Maryland SBE ${pair.label} candidate CSVs (statewide ${
            pair.statewide ? "ok" : "—"
          }, local ${pair.local ? "ok" : "—"}).`
        );
      } else {
        out.notes.push(
          missingStateTransportNote("MD SBE candidate CSV fetch")
        );
      }
    } catch (e) {
      out.notes.push(`MD candidate listings failed: ${e.message || e}`);
    }
    try {
      onProgress("Maryland ballot questions (SBE)…");
      const qUrl = `${mdBase}/ballot_questions.html`;
      const qHtml = await tryFetchText(
        qUrl,
        `md:measures:${cycle}`
      ).catch(() => "");
      if (
        qHtml &&
        qHtml.length > 400 &&
        /ballot\s+question/i.test(qHtml) &&
        /<details/i.test(qHtml)
      ) {
        out.mdMeasures = qHtml;
        out.notes.push("Using Maryland SBE ballot questions page.");
      } else if (qHtml && qHtml.length > 200) {
        out.notes.push(
          "MD ballot questions page returned unexpected content (no question sections)."
        );
      } else {
        out.notes.push(
          missingStateTransportNote("MD SBE ballot questions fetch")
        );
      }
    } catch (e) {
      out.notes.push(`MD ballot questions failed: ${e.message || e}`);
    }
    return out;
  }

  if (st === "AZ") {
    onProgress("Arizona Legislature rosters…");
    try {
      const [s, h] = await Promise.all([
        tryFetchText(AZ_SENATE, "az:senate_roster"),
        tryFetchText(AZ_HOUSE, "az:house_roster"),
      ]);
      out.azSenate = s;
      out.azHouse = h;
      if (!s && !h) {
        out.notes.push(missingStateTransportNote("AZ roster fetch"));
      } else if (!s || !h) {
        out.notes.push(
          `AZ partial rosters (senate ${s ? "ok" : "fail"}, house ${h ? "ok" : "fail"}).`
        );
      }
    } catch (e) {
      out.notes.push(`AZ rosters failed: ${e.message || e}`);
    }
    try {
      const m = await fetchAzMeasuresHtml(cycle, countyName, onProgress);
      out.azMeasures = m.html || "";
      if (m.note) out.notes.push(m.note);
    } catch (e) {
      out.notes.push(`AZ measures failed: ${e.message || e}`);
    }
    try {
      onProgress("Arizona statewide officials (Clean Elections)…");
      const countyTok = countyName
        .replace(/\s+County$/i, "")
        .trim();
      if (cdNum > 0 && legDistrict > 0 && countyTok) {
        const packed = `AZ-${cdNum}-${legDistrict}-${countyTok}~County-0--`;
        const url = `${AZ_OFFICIALS_LIST_BASE}?location=${encodeURIComponent(packed)}`;
        const html = await tryFetchText(url, `az:officials:${packed}`).catch(
          () => ""
        );
        if (
          html &&
          html.length > 200 &&
          /class=["']office["']/i.test(html) &&
          /Governor|Senator|Representative/i.test(html)
        ) {
          out.azOfficials = html;
          out.notes.push(
            `Using Clean Elections OfficialList incumbents (${packed}).`
          );
        } else if (html) {
          out.notes.push(
            "AZ OfficialList returned unexpected content (no office rows)."
          );
        } else {
          out.notes.push(
            missingStateTransportNote("AZ Clean Elections OfficialList fetch")
          );
        }
      } else {
        out.notes.push(
          "AZ OfficialList skipped (need congressional + legislative district + county)."
        );
      }
    } catch (e) {
      out.notes.push(`AZ OfficialList failed: ${e.message || e}`);
    }
    return out;
  }

  if (st === "FL") {
    onProgress("Florida chamber rosters…");
    try {
      const [s, h] = await Promise.all([
        tryFetchText(FL_SENATE, "fl:senate_roster").catch(() => ""),
        tryFetchText(FL_HOUSE, "fl:house_roster").catch(() => ""),
      ]);
      out.flSenate = s || "";
      out.flHouse = h || "";
    } catch (e) {
      out.notes.push(`FL chamber rosters failed: ${e.message || e}`);
    }
    if (!out.flSenate && !out.flHouse) {
      out.notes.push(missingStateTransportNote("FL chamber HTML"));
    }

    // Live DOS extract via Wisp/libcurl (same as native) — TSV upload is optional fallback
    let tsv = "";
    let tsvSource = "";
    try {
      const live = await fetchFlDosLive(cycle, onProgress);
      if (live.tsv) {
        tsv = live.tsv;
        tsvSource = live.source === "cache" ? "live-cache" : "live";
      }
    } catch (e) {
      console.warn("[state] FL DOS live fetch", e);
      out.notes.push(`FL DOS live fetch failed: ${e.message || e}`);
    }
    if (!tsv) {
      tsv = getFlDosTsv();
      if (tsv) tsvSource = "upload";
    }
    if (!tsv) {
      tsv = await loadShippedFlDos(cycle);
      if (tsv) tsvSource = "shipped";
    }
    if (tsv) {
      out.flDosTsv = tsv;
      // informational only — live.js filters /^Using /
      const labels = {
        live: "Using live FL DOS Candidate Tracking extract (via Wisp).",
        "live-cache": "Using cached FL DOS Candidate Tracking extract.",
        upload: "Using FL DOS TSV from Settings upload.",
        shipped: `Using shipped web/data/fl-dos-${cycle}.tsv.`,
      };
      out.notes.push(labels[tsvSource] || "Using FL DOS extract.");
    } else if (!out.notes.some((n) => n.includes("DOS live"))) {
      out.notes.push(
        hasWispConfigured()
          ? "FL DOS extract unavailable — state/judicial/local filings missing. Try again or upload a TSV in Settings."
          : "FL DOS extract needs Wisp (libcurl.js) enabled, or upload a TSV in Settings."
      );
    }

    try {
      out.flMeasures = await fetchFlMeasuresHtml(cycle, onProgress);
      if (out.flMeasures) {
        out.notes.push("Using live FL constitutional amendments list.");
      }
      // empty → core adds link-only fallback; no banner
    } catch (e) {
      console.warn("[state] FL amendments", e);
    }

    if (countyName) {
      try {
        const soe = await fetchFlSoeList(countyName, onProgress);
        out.flSoe = soe.html || "";
        if (soe.note) out.notes.push(soe.note);
      } catch (e) {
        out.notes.push(`FL SOE list failed: ${e.message || e}`);
      }
    }

    const precinct = String(opts.precinct || getVoterPrecinct() || "").trim();
    const party = String(opts.party || getVoterParty() || "").trim();
    if (precinct && party && /brevard/i.test(countyName)) {
      try {
        const sample = await fetchFlSampleBallotText(precinct, party, onProgress);
        out.flSampleBallot = sample.text || "";
        if (sample.note) out.notes.push(sample.note);
      } catch (e) {
        out.notes.push(`FL sample ballot skipped: ${e.message || e}`);
      }
    }

    return out;
  }

  return out;
}
