import init, {
  normalize_zip_js,
  build_report_from_fixture,
  sample_ballot_ref_js,
} from "./pkg/electionizer_wasm.js";
import { buildLiveFederalBallot } from "./live.js";
import { enrichCandidate, planStages, runVerdictPass, runMeasureVerdict } from "./enrich.js";
import { renderVerdictShell, renderVerdictCard, mountVoterProfile } from "./verdict.js";
import {
  resetBoard,
  runScoreQueue,
  cancelScoring,
  pauseScoring,
  resumeScoring,
  scoresByKey,
  renderScoreTree,
  renderScorecard,
  rememberCard,
  reapplyVoterFit,
  prioritizeAndScore,
  getScoreItem,
  resetScoreItems,
  itemsForRole,
  ensureScoreQueue,
  scoreKey,
} from "./scoreboard.js";
import {
  renderBallotReport,
  renderDetailShell,
  renderMeasureDetail,
  patchDetailSections,
  indexCandidates,
  indexMeasures,
  DETAIL_TAB_IDS,
} from "./render.js";
import { defaultVotesUi, defaultFinanceUi } from "./detail-lists.js";
import { mountTimeline } from "./timeline.js";

const DETAIL_TAB_KEY = "electionizer:detailTab";

function readTabFromUrl() {
  try {
    const q = new URLSearchParams(location.search || "");
    const t = (q.get("tab") || "").trim().toLowerCase();
    if (DETAIL_TAB_IDS.includes(t)) return t;
  } catch {
    /* ignore */
  }
  return null;
}

function writeTabToUrl(id) {
  if (!DETAIL_TAB_IDS.includes(id)) return;
  try {
    const url = new URL(location.href);
    if (url.searchParams.get("tab") === id) return;
    url.searchParams.set("tab", id);
    history.replaceState(null, "", url.pathname + url.search + url.hash);
  } catch {
    /* ignore */
  }
}

function clearTabFromUrl() {
  try {
    const url = new URL(location.href);
    if (!url.searchParams.has("tab")) return;
    url.searchParams.delete("tab");
    const q = url.searchParams.toString();
    history.replaceState(
      null,
      "",
      url.pathname + (q ? `?${q}` : "") + url.hash
    );
  } catch {
    /* ignore */
  }
}

function readRememberedTab() {
  const fromUrl = readTabFromUrl();
  if (fromUrl) return fromUrl;
  try {
    const t = sessionStorage.getItem(DETAIL_TAB_KEY);
    if (DETAIL_TAB_IDS.includes(t)) return t;
  } catch {
    /* private mode */
  }
  return "dossier";
}

function rememberTab(id) {
  if (!DETAIL_TAB_IDS.includes(id)) return;
  try {
    sessionStorage.setItem(DETAIL_TAB_KEY, id);
  } catch {
    /* private mode */
  }
  writeTabToUrl(id);
}
import {
  clearBallotAndResponseCache,
  getLastBallot,
  saveLastBallot,
  withFreshCache,
  deleteAiCacheForSubject,
} from "./cache.js";
import { ensureCurl, isCurlReady, hasWispConfigured } from "./curl-transport.js";
import {
  enrichFlMeasureFinance,
  enrichMdMeasureFinance,
  enrichFtmMeasureFinance,
  enrichFlMeasureSummaries,
  enrichMeasureEndorsements,
} from "./state.js";
import {
  DEFAULT_WISP_URL,
  getCorsProxy,
  getCycle,
  getFecApiKey,
  getFlDosTsvMeta,
  getMode,
  getOpenStatesApiKey,
  getFtmApiKey,
  getCourtListenerToken,
  getWispUrl,
  isDefaultWispUrl,
  resetWispUrlToDefault,
  setCorsProxy,
  setCycle,
  setFecApiKey,
  setFlDosTsv,
  setMode,
  setOpenStatesApiKey,
  setFtmApiKey,
  setCourtListenerToken,
  setCivicApiKey,
  getCivicApiKey,
  hasCivicKey,
  setWispUrl,
  getVoterPrecinct,
  setVoterPrecinct,
  getVoterParty,
  setVoterParty,
  getLlmApiKey,
  setLlmApiKey,
  getLlmProvider,
  setLlmProvider,
  getLlmModel,
  setLlmModel,
  getScoreConcurrency,
  setScoreConcurrency,
  hasLlmKey,
} from "./settings.js";

const $ = (sel) => document.querySelector(sel);

let wasmReady = false;
/** @type {object|null} */
let lastReport = null;
/** @type {Map<string, object>} */
let candidateIndex = new Map();

function statusMessage() {
  const mode = getMode();
  const cycle = getCycle();
  if (mode === "fixture") {
    return "WASM ready · fixture mode (any ZIP → 90210 sample).";
  }
  const key = getFecApiKey();
  const keyHint = key === "DEMO_KEY" ? "DEMO_KEY" : "personal FEC key";
  const wisp = getWispUrl();
  const wispHint = !wisp
    ? "Wisp off"
    : isDefaultWispUrl()
      ? "Wisp default"
      : "Wisp custom";
  const civicHint = hasCivicKey() ? " · Civic on" : "";
  return `WASM ready · live · cycle ${cycle} · ${keyHint} · ${wispHint}${civicHint}.`;
}

function formatUserError(e) {
  let msg = (e && e.message) || String(e);
  if (/429|rate limit/i.test(msg)) {
    if (!/Settings|api\.open\.fec/.test(msg)) {
      msg +=
        " — add a personal FEC key in Settings (https://api.open.fec.gov/developers/).";
    }
  } else if (/Failed to fetch|NetworkError|network error/i.test(msg)) {
    if (!/CORS|proxy/i.test(msg)) {
      msg += " — check network; AZ/FL chamber pages need a CORS proxy in Settings.";
    }
  }
  return msg;
}

function syncModeUi() {
  const mode = getMode();
  $("#mode-badge").textContent = mode === "fixture" ? "FIXTURE" : "LIVE";
  const lede = $("#home-lede");
  if (lede) {
    lede.textContent =
      mode === "fixture"
        ? "Offline demo: any ZIP loads the 90210 sample ballot through Rust WASM."
        : "Enter your ZIP code for a summary of candidates and ballot measures in your area.";
  }
  $("#footer-note").textContent =
    mode === "fixture"
      ? "// fixture mode · WASM core · no backend"
      : "// sources cited on detail · client-only WASM · IndexedDB cache";
  if ($("#mode-live")) {
    $("#mode-live").checked = mode === "live";
    $("#mode-fixture").checked = mode === "fixture";
  }
  const demo = $("#demo-warning");
  if (demo) {
    demo.hidden = !(mode === "live" && getFecApiKey() === "DEMO_KEY");
  }
  if (window.ElectionizerTheme) {
    window.ElectionizerTheme.apply(window.ElectionizerTheme.get());
  }
}

function showView(name) {
  $("#home-view").hidden = name !== "home";
  $("#ballot-view").hidden = name !== "ballot";
  if ($("#verdict-view")) $("#verdict-view").hidden = name !== "verdict";
  $("#detail-view").hidden = name !== "detail";
  $("#settings-view").hidden = name !== "settings";
  if ($("#scorecard-view")) $("#scorecard-view").hidden = name !== "scorecard";
  if (name !== "detail") clearTabFromUrl();
  if (name === "home") document.title = "electionizer · static";
  else if (name === "ballot" && lastReport)
    document.title = `Ballot ${lastReport.zip} · electionizer`;
  else if (name === "scorecard" && lastReport)
    document.title = `Scorecard ${lastReport.zip} · electionizer`;
  else if (name === "scorecard") document.title = "Scorecard · electionizer";
  else if (name === "settings") document.title = "Settings · electionizer";
  else if (name === "verdict") document.title = "Verdict · electionizer";
  else if (name === "detail") document.title = "Candidate · electionizer";
}

async function loadFixture() {
  const res = await fetch("./fixtures/fixture_90210.json");
  if (!res.ok) throw new Error(`fixture fetch failed: ${res.status}`);
  return res.text();
}

let measureIndex = new Map();
/** Open measure detail id (string), or null when on ballot / candidate detail. */
let measureDetailId = null;

function bindBallotClicks() {
  const root = $("#ballot-root");
  root.onclick = (ev) => {
    const mA = ev.target.closest("a[data-measure-id]");
    if (mA) {
      ev.preventDefault();
      const id = mA.getAttribute("data-measure-id");
      const m = measureIndex.get(String(id));
      if (m) openVerdictMeasure(m);
      return;
    }
    const a = ev.target.closest("a[data-cand-id]");
    if (!a) return;
    ev.preventDefault();
    const id = a.getAttribute("data-cand-id");
    const c = candidateIndex.get(String(id));
    if (c) openVerdict(c);
  };
}

function ballotRenderOpts() {
  const opts = { scores: scoresByKey() };
  const precinct = getVoterPrecinct();
  const party = getVoterParty();
  if (!precinct || !party) return opts;
  try {
    const ref = sample_ballot_ref_js(precinct, party, "");
    if (ref) opts.sampleBallot = ref;
  } catch {
    /* ignore */
  }
  return opts;
}

function snapshotScoreFolds(root) {
  if (!root) return null;
  const wrap = root.querySelector(".score-progress-fold");
  return {
    wrapOpen: wrap ? wrap.open : null,
    wrapDone: !!(wrap && wrap.classList.contains("score-progress-done")),
    roles: [...root.querySelectorAll(".score-role")].map((el) => ({
      title: el.querySelector(".score-role-title")?.textContent || "",
      open: el.open,
      done: el.classList.contains("score-role-done"),
    })),
  };
}

function restoreScoreFolds(root, snap) {
  if (!root) return;
  const wrap = root.querySelector(".score-progress-fold");
  if (wrap) {
    const nowDone = wrap.classList.contains("score-progress-done");
    if (!nowDone) wrap.open = true;
    else if (!snap || snap.wrapOpen == null || !snap.wrapDone) wrap.open = false;
    else wrap.open = snap.wrapOpen;
  }
  root.querySelectorAll(".score-role").forEach((el) => {
    const t = el.querySelector(".score-role-title")?.textContent || "";
    const nowDone = el.classList.contains("score-role-done");
    const prev = snap && snap.roles.find((r) => r.title === t);
    if (!prev) {
      el.open = !nowDone;
      return;
    }
    if (nowDone && !prev.done) el.open = false;
    else el.open = prev.open;
  });
}

function paintScoreUi() {
  const noKey = !hasLlmKey();
  const noWisp = hasLlmKey() && !hasWispConfigured();
  const tree = renderScoreTree({ noKey, noWisp });
  const apply = (el) => {
    if (!el) return;
    const snap = snapshotScoreFolds(el);
    el.innerHTML = tree;
    restoreScoreFolds(el, snap);
  };
  apply($("#score-progress"));
  apply($("#scorecard-progress"));
  const host = $("#scorecard-host");
  if (host && lastReport) host.innerHTML = renderScorecard({ zip: lastReport.zip });
  wireScoreNav();
}

function wireScoreNav() {
  document.querySelectorAll("[data-score-settings]").forEach((a) => {
    a.onclick = (ev) => {
      ev.preventDefault();
      loadSettingsForm();
      showView("settings");
    };
  });
  const bind = (root) => {
    if (!root || root.dataset.scoreBound) return;
    root.dataset.scoreBound = "1";
    root.addEventListener("click", (ev) => {
      const mA = ev.target.closest("a[data-measure-id]");
      if (mA) {
        ev.preventDefault();
        const m = measureIndex.get(String(mA.getAttribute("data-measure-id")));
        if (m) openVerdictMeasure(m);
        return;
      }
      const a = ev.target.closest("a[data-cand-id]");
      if (!a) return;
      ev.preventDefault();
      const c = candidateIndex.get(String(a.getAttribute("data-cand-id")));
      if (c) openVerdict(c);
    });
  };
  bind($("#score-progress"));
  bind($("#scorecard-progress"));
  bind($("#scorecard-host"));
}

function refreshBallotList() {
  if (!lastReport) {
    paintScoreUi();
    return;
  }
  const ballotVisible = $("#ballot-view") && !$("#ballot-view").hidden;
  if (ballotVisible) {
    const y = window.scrollY;
    const openTitles = [...document.querySelectorAll(".judicial-seat[open] .judicial-seat-title")].map(
      (el) => el.textContent
    );
    const root = $("#ballot-root");
    if (root) {
      root.innerHTML = renderBallotReport(lastReport, ballotRenderOpts());
      bindBallotClicks();
      document.querySelectorAll(".judicial-seat").forEach((el) => {
        const t = el.querySelector(".judicial-seat-title")?.textContent;
        if (t && openTitles.includes(t)) el.open = true;
      });
    }
    window.scrollTo(0, y);
  }
  const scorecardVisible = $("#scorecard-view") && !$("#scorecard-view").hidden;
  const y2 = scorecardVisible ? window.scrollY : null;
  paintScoreUi();
  if (y2 != null) window.scrollTo(0, y2);
}

function startBallotScoring(report) {
  resetBoard(report);
  paintScoreUi();
  runScoreQueue({
    noKey: !hasLlmKey(),
    noWisp: hasLlmKey() && !hasWispConfigured(),
    onUpdate: () => refreshBallotList(),
  }).catch((e) => console.warn("score queue", e));
}

function presentBallot(report) {
  lastReport = report;
  candidateIndex = indexCandidates(report);
  measureIndex = indexMeasures(report);
  measureDetailId = null;
  cancelScoring();
  resetBoard(report);
  $("#ballot-root").innerHTML = renderBallotReport(report, ballotRenderOpts());
  bindBallotClicks();
  paintScoreUi();
  showView("ballot");
  saveLastBallot(report.zip, report).catch(() => {});
  startBallotScoring(report);
}

function refreshOpenMeasureDetail() {
  if (!measureDetailId || !lastReport) return;
  if ($("#detail-view")?.hidden) return;
  const m = measureIndex.get(String(measureDetailId));
  if (!m) return;
  const host = $("#detail-host");
  if (!host) return;
  // Preserve scroll if possible
  const y = window.scrollY;
  host.innerHTML = renderMeasureDetail(m, {
    election_name: lastReport.election_name,
    election_date: lastReport.election_date,
    zip: lastReport.zip,
  });
  $("#back-ballot")?.addEventListener("click", (ev) => {
    ev.preventDefault();
    measureDetailId = null;
    resumeScoring();
    showView("ballot");
    refreshBallotList();
  });
  try {
    window.scrollTo(0, y);
  } catch {
    /* ignore */
  }
}

function openMeasureDetail(raw, opts = {}) {
  const m = { ...raw };
  measureDetailId = String(m.id);
  detailCtx = null;
  const host = $("#detail-host");
  host.innerHTML = renderMeasureDetail(m, {
    election_name: lastReport?.election_name,
    election_date: lastReport?.election_date,
    zip: lastReport?.zip,
  });
  applyDetailCrumb(host, !!opts.fromVerdict);
  showView("detail");
  document.title = `${m.measure_code || m.title || "Measure"} · electionizer`;
  clearTabFromUrl();
  $("#back-ballot")?.addEventListener("click", (ev) => {
    ev.preventDefault();
    measureDetailId = null;
    resumeScoring();
    showView("ballot");
    refreshBallotList();
  });
  $("#back-verdict")?.addEventListener("click", (ev) => {
    ev.preventDefault();
    showView("verdict");
    document.title = `${m.measure_code || m.title || "Measure"} · verdict`;
  });
}

/** Measures: paint ballot first, then FL summaries/TreFin and/or FTM measure $. */
async function progressiveMeasureEnrich(report, onProgress) {
  const measures = report?.measures;
  if (!Array.isArray(measures) || !measures.length) return;
  const st = String(report.state_code || report.state || "").toUpperCase();
  // Heuristic: FL measures carry constitutionalinitiatives / DOS InitDetail URLs
  const flish =
    st === "FL" ||
    measures.some(
      (m) =>
        m.source_url &&
        /constitutionalinitiatives|InitDetail|dos\.fl\.gov/i.test(m.source_url)
    );

  let lastPaint = 0;
  const paint = (force = false) => {
    const now = Date.now();
    if (!force && now - lastPaint < 450) return;
    lastPaint = now;
    // Always keep indexes/cache fresh.
    lastReport = report;
    candidateIndex = indexCandidates(report);
    measureIndex = indexMeasures(report);
    saveLastBallot(report.zip, report).catch(() => {});

    // Candidate detail open — don't yank ballot DOM under them; still refresh measure detail if that is open.
    if ($("#detail-view") && !$("#detail-view").hidden) {
      if (measureDetailId) refreshOpenMeasureDetail();
      return;
    }
    const root = $("#ballot-root");
    if (root) root.innerHTML = renderBallotReport(report, ballotRenderOpts());
    bindBallotClicks();
  };

  if (flish) {
    try {
      await enrichFlMeasureSummaries(report, onProgress);
      paint(true);
    } catch (e) {
      console.warn("[app] FL measure summaries", e);
    }
    try {
      await enrichFlMeasureFinance(report, (msg) => {
        onProgress(msg);
        paint(false);
      });
      paint(true);
    } catch (e) {
      console.warn("[app] FL measure finance", e);
    }
  }

  // Live state measure $ (MD MDCRIS ballot-issue committees) before FTM archive.
  try {
    await enrichMdMeasureFinance(report, (msg) => {
      onProgress(msg);
      paint(false);
    });
    paint(true);
  } catch (e) {
    console.warn("[app] MD measure finance", e);
  }

  // FTM measure $ for non-FL (and FL rows still without finance). Needs Wisp/CORS.
  try {
    await enrichFtmMeasureFinance(report, (msg) => {
      onProgress(msg);
      paint(false);
    });
    paint(true);
  } catch (e) {
    console.warn("[app] FTM measure finance", e);
  }

  try {
    await enrichMeasureEndorsements(report, (msg) => {
      onProgress(msg);
      paint(false);
    });
    paint(true);
  } catch (e) {
    console.warn("[app] measure endorsements", e);
  }

  // Never leave Sponsor $/Oppose $ on loading "…" after enrich finishes.
  let stamped = false;
  for (const m of measures) {
    if (m.finance) continue;
    m.finance = {
      account: "",
      contributions_sum: 0,
      contributions_sum_display: "—",
      top_contributors: [],
      line_count: 0,
      committee_url: "",
      trefin_url: "",
      note: "No published measure committee $ (Wisp + TreFin/FTM when available).",
      committee_name: "",
      role: "sponsor",
      oppose: [],
    };
    stamped = true;
  }
  if (stamped) paint(true);
}

function setBallotJob(pct, stage, msg) {
  const job = $("#ballot-job");
  const bar = $("#ballot-job-bar");
  const pctEl = $("#ballot-job-pct");
  const msgEl = $("#ballot-job-msg");
  if (!job) return;
  job.hidden = false;
  const p = Math.max(0, Math.min(100, Math.round(Number(pct) || 0)));
  if (bar) bar.style.width = p + "%";
  if (pctEl) {
    pctEl.innerHTML = `<strong>${p}%</strong> · ${stage || "working…"}`;
  }
  if (msgEl && msg) msgEl.textContent = msg;
}

function hideBallotJob() {
  const job = $("#ballot-job");
  if (job) job.hidden = true;
  const bar = $("#ballot-job-bar");
  if (bar) bar.style.width = "0%";
}

function clearLastBallotOffer() {
  const el = $("#last-ballot-offer");
  if (!el) return;
  el.hidden = true;
  el.innerHTML = "";
}

async function showBallot(zipRaw, streetRaw = "") {
  const status = $("#status-line");
  status.classList.remove("warn-banner");
  try {
    const zip = normalize_zip_js(zipRaw);
    const street = String(streetRaw || "").trim();
    status.textContent = `Building ballot for ${zip}…`;
    const jobEl = $("#ballot-job");
    if (jobEl) jobEl.classList.remove("status-failed");
    setBallotJob(4, "start", `Building ballot for ${zip}…`);
    let report;
    if (getMode() === "fixture") {
      setBallotJob(40, "fixture", "Loading fixture ballot…");
      const fixtureJson = await loadFixture();
      report = build_report_from_fixture(fixtureJson, zip);
      setBallotJob(70, "fixture", "Fixture report ready…");
    } else {
      let livePct = 8;
      report = await buildLiveFederalBallot(
        zip,
        (msg) => {
          const m = String(msg || "");
          status.textContent = m || `Building ballot for ${zip}…`;
          livePct = Math.min(78, livePct + 6);
          let stage = "live";
          if (/geo|zip|district|tiger|fcc/i.test(m)) stage = "geo";
          else if (/fec|candidate|house|senate/i.test(m)) stage = "fec";
          else if (/florida|arizona|carolina|maryland|state|dos|civic|open states/i.test(m))
            stage = "state";
          else if (/wasm|report|build/i.test(m)) stage = "report";
          setBallotJob(livePct, stage, m || `Building ballot for ${zip}…`);
        },
        { street }
      );
    }
    setBallotJob(82, "measures", "Enriching ballot measures…");
    presentBallot(report);
    await progressiveMeasureEnrich(report, (msg) => {
      // Quiet progress: short phase labels only
      const m = String(msg || "");
      let line = "Measures…";
      if (/summar/i.test(m)) line = "Measures · summaries…";
      else if (/oppose PAC search/i.test(m)) line = "Measures · oppose PACs…";
      else if (/oppose PAC finance/i.test(m)) {
        const frac = m.match(/\((\d+\/\d+)\)/);
        line = frac
          ? `Measures · oppose ${frac[1]}…`
          : "Measures · oppose finance…";
      } else if (/amendment finance|FollowTheMoney measure finance|MDCRIS measure finance/i.test(m)) {
        const frac = m.match(/\((\d+\/\d+)\)/);
        line = frac
          ? `Measures · finance ${frac[1]}…`
          : "Measures · finance…";
      } else if (/FollowTheMoney measure list|MDCRIS ballot-issue|MDCRIS measure search/i.test(m)) {
        line = "Measures · committees…";
      }
      status.textContent = line;
      const frac = m.match(/\((\d+)\/(\d+)\)/);
      let pct = 88;
      if (frac) {
        const a = Number(frac[1]) || 0;
        const b = Math.max(1, Number(frac[2]) || 1);
        pct = 82 + Math.round((a / b) * 14);
      }
      setBallotJob(pct, "measures", line);
    });
    setBallotJob(100, "ready", "Ready.");
    status.textContent = "Ready.";
    hideBallotJob();
  } catch (e) {
    console.error(e);
    status.textContent = formatUserError(e);
    status.classList.add("warn-banner");
    setBallotJob(100, "failed", formatUserError(e));
    const job = $("#ballot-job");
    if (job) job.classList.add("status-failed");
  }
}

function markStage(id, status, detail) {
  const li = document.querySelector(
    `#detail-stages [data-stage-id="${CSS.escape(id)}"]`
  );
  if (!li) return;
  li.className = "stage-" + status;
  const mark = li.querySelector(".stage-mark");
  if (mark) {
    if (status === "running") mark.textContent = "›";
    else if (status === "done") mark.textContent = "✓";
    else if (status === "skip") mark.textContent = "–";
    else if (status === "error") mark.textContent = "!";
    else mark.textContent = "·";
  }
  const d = li.querySelector(".stage-detail");
  if (d) d.textContent = detail ? " · " + detail : "";
}

function setDetailPct(completed, total, label) {
  const p = total ? Math.min(100, Math.round((completed * 100) / total)) : 0;
  const bar = $("#detail-bar");
  const pctEl = $("#detail-pct");
  const msgEl = $("#detail-msg");
  if (bar) bar.style.width = p + "%";
  if (pctEl) {
    pctEl.innerHTML = `<strong>${completed} / ${total}</strong> · ${label || "working…"}`;
  }
  if (msgEl && label) msgEl.textContent = label;
}

function wireDetailTabs(root) {
  const tablist = root.querySelector('[role="tablist"]');
  if (!tablist) return;
  const tabs = [...tablist.querySelectorAll('[role="tab"]')];
  const panels = [...root.querySelectorAll('[role="tabpanel"]')];

  function selectTab(tab, { focus } = { focus: false }) {
    const id = tab.getAttribute("data-tab");
    tabs.forEach((t) => {
      const on = t === tab;
      t.setAttribute("aria-selected", on ? "true" : "false");
      t.tabIndex = on ? 0 : -1;
      t.classList.toggle("is-active", on);
    });
    panels.forEach((p) => {
      const on = p.getAttribute("data-tab-panel") === id;
      p.hidden = !on;
      p.setAttribute("aria-hidden", on ? "false" : "true");
      if (on) p.removeAttribute("inert");
      else p.setAttribute("inert", "");
    });
    rememberTab(id);
    if (typeof tab.scrollIntoView === "function") {
      try {
        tab.scrollIntoView({ block: "nearest", inline: "nearest", behavior: "smooth" });
      } catch {
        tab.scrollIntoView(false);
      }
    }
    if (focus) tab.focus();
    // Timeline SVG needs a laid-out width; remount when the tab becomes visible.
    if (id === "timeline") {
      requestAnimationFrame(() => {
        const sec = root.querySelector("#detail-sec-timeline");
        if (sec) {
          try {
            mountTimeline(sec);
          } catch (e) {
            console.warn("timeline remount", e);
          }
        }
      });
    }
  }

  tablist.addEventListener("click", (ev) => {
    const tab = ev.target.closest('[role="tab"]');
    if (!tab || !tablist.contains(tab)) return;
    selectTab(tab);
  });

  tablist.addEventListener("keydown", (ev) => {
    const i = tabs.indexOf(document.activeElement);
    if (i < 0) return;
    let next = -1;
    if (ev.key === "ArrowRight" || ev.key === "ArrowDown") {
      next = (i + 1) % tabs.length;
    } else if (ev.key === "ArrowLeft" || ev.key === "ArrowUp") {
      next = (i - 1 + tabs.length) % tabs.length;
    } else if (ev.key === "Home") {
      next = 0;
    } else if (ev.key === "End") {
      next = tabs.length - 1;
    } else if (ev.key === "Enter" || ev.key === " ") {
      ev.preventDefault();
      selectTab(tabs[i]);
      return;
    } else {
      return;
    }
    ev.preventDefault();
    selectTab(tabs[next], { focus: true });
  });

  const want = readRememberedTab();
  const initial =
    tabs.find((t) => t.getAttribute("data-tab") === want) ||
    tabs.find((t) => t.getAttribute("aria-selected") === "true") ||
    tabs[0];
  if (initial) selectTab(initial);
}

/** Detail list UI state (votes filter/page) — survives progressive patches. */
let detailCtx = null;
/** @type {{ key: string|null, promise: Promise<object>|null, enrich: object, listeners: Function[] }} */
let sharedEnrich = { key: null, promise: null, enrich: {}, listeners: [] };
/** @type {{ kind: string, candidate?: object, measure?: object, card?: object }|null} */
let verdictCtx = null;
let verdictGen = 0;
let activeVerdictKey = "";

function subjectKey(kind, id) {
  return `${kind === "measure" ? "m" : "c"}:${id}`;
}

function stampCard(card, key, person) {
  if (!card || typeof card !== "object") return card;
  return {
    ...card,
    subject_key: key,
    subject_name: person?.name || person?.title || card.subject_name || "",
    subject_office: person?.office || card.subject_office || "",
  };
}

function beginVerdict(key) {
  verdictGen += 1;
  activeVerdictKey = key;
  return { gen: verdictGen, key };
}

function verdictLive(token) {
  return !!token && token.gen === verdictGen && activeVerdictKey === token.key;
}

function withElectionMeta(raw) {
  return {
    ...raw,
    election_name: lastReport?.election_name || raw.election_name,
    election_date: lastReport?.election_date || raw.election_date,
    state_code: lastReport?.state_code || raw.state_code,
    zip: lastReport?.zip,
  };
}

function attachEnrichListener(fn) {
  if (typeof fn === "function") sharedEnrich.listeners.push(fn);
}

function notifyEnrichSession(session, u) {
  for (const fn of session.listeners.slice()) {
    try {
      fn(u);
    } catch (e) {
      console.warn(e);
    }
  }
}

function startSharedEnrich(c, opts = {}) {
  const key = `c:${c.id}`;
  if (!opts.fresh && sharedEnrich.key === key && sharedEnrich.promise) {
    return sharedEnrich.promise;
  }
  const prevListeners = sharedEnrich.key === key ? sharedEnrich.listeners.slice() : [];
  const session = { key, promise: null, enrich: {}, listeners: prevListeners };
  sharedEnrich = session;
  session.promise = enrichCandidate(
    c,
    (u) => {
      if (sharedEnrich !== session) return;
      if (u.enrich) session.enrich = u.enrich;
      notifyEnrichSession(session, u);
    },
    { fresh: !!opts.fresh, skipAi: !!opts.skipAi }
  ).then((r) => {
    if (sharedEnrich === session) session.enrich = r.enrich;
    return r;
  });
  return session.promise;
}

function setReloadStatus(msg) {
  const el = $("#status-line");
  if (el) el.textContent = msg;
}

function scoreQueueOpts() {
  return {
    noKey: !hasLlmKey(),
    noWisp: hasLlmKey() && !hasWispConfigured(),
    onUpdate: () => refreshBallotList(),
  };
}

function resetDetailStagesUi() {
  document.querySelectorAll("#detail-stages li").forEach((li) => {
    li.className = "stage-pending";
    const d = li.querySelector(".stage-detail");
    if (d) d.textContent = "";
  });
  const pct = $("#detail-pct");
  if (pct) pct.innerHTML = "<strong>0</strong> · reloading…";
  const bar = $("#detail-bar");
  if (bar) bar.style.width = "0%";
  const msg = $("#detail-msg");
  if (msg) msg.textContent = "Reloading sources…";
  const progress = $("#detail-progress");
  if (progress) {
    progress.classList.remove("status-done", "status-failed");
    progress.open = true;
  }
}

async function reloadCandidate(id, what) {
  const raw = candidateIndex.get(String(id));
  if (!raw) return;
  const c = withElectionMeta(raw);
  if (what === "ai") {
    await deleteAiCacheForSubject({ id: c.id, name: c.name });
    resetScoreItems([scoreKey("candidate", c.id)]);
    refreshBallotList();
    const open =
      (verdictCtx && verdictCtx.candidate && String(verdictCtx.candidate.id) === String(id)) ||
      (detailCtx && detailCtx.candidate && String(detailCtx.candidate.id) === String(id));
    if (open && hasLlmKey() && hasWispConfigured()) {
      if (verdictCtx && String(verdictCtx.candidate?.id) === String(id)) {
        verdictCtx.card = null;
        paintVerdict(null, "Re-scoring…");
      }
      const snap = sharedEnrich.key === `c:${c.id}` ? sharedEnrich.enrich || {} : {};
      const early = await runVerdictPass(c, snap, { pass: "early", fresh: true });
      if (early && early.card) {
        const key = subjectKey("candidate", c.id);
        const stamped = stampCard(early.card, key, c);
        rememberCard("candidate", c.id, stamped);
        if (sharedEnrich.key === key && sharedEnrich.enrich) {
          sharedEnrich.enrich.verdict = stamped;
        }
        if (verdictCtx && String(verdictCtx.candidate?.id) === String(id)) {
          verdictCtx.card = stamped;
          paintVerdict(stamped);
        }
      } else if (verdictCtx && String(verdictCtx.candidate?.id) === String(id)) {
        paintVerdict(null, (early && early.skip) || "No verdict.");
      }
    } else {
      await prioritizeAndScore("candidate", c.id, refreshBallotList);
      await ensureScoreQueue(scoreQueueOpts());
    }
    refreshBallotList();
    return;
  }
  setReloadStatus(`Reloading sources for ${c.name}…`);
  if (detailCtx && String(detailCtx.candidate?.id) === String(id)) resetDetailStagesUi();
  await startSharedEnrich(c, { fresh: true, skipAi: true });
  if (detailCtx && String(detailCtx.candidate?.id) === String(id)) {
    detailCtx.enrich = sharedEnrich.enrich || {};
    patchDetail(null, true);
  }
  setReloadStatus(`Sources reloaded for ${c.name}.`);
}

async function reloadMeasure(id, what) {
  const m = measureIndex.get(String(id));
  if (!m) return;
  if (what === "ai") {
    await deleteAiCacheForSubject({ id: m.id, name: m.title || m.name });
    resetScoreItems([scoreKey("measure", m.id)]);
    refreshBallotList();
    const open = verdictCtx && verdictCtx.measure && String(verdictCtx.measure.id) === String(id);
    if (open && hasLlmKey() && hasWispConfigured()) {
      verdictCtx.card = null;
      paintVerdict(null, "Re-scoring…");
      const result = await runMeasureVerdict(m, { fresh: true });
      if (result && result.card) {
        const key = subjectKey("measure", m.id);
        const stamped = stampCard(result.card, key, m);
        verdictCtx.card = stamped;
        paintVerdict(stamped);
        rememberCard("measure", m.id, stamped);
      } else {
        paintVerdict(null, (result && result.skip) || "No verdict.");
      }
    } else {
      await prioritizeAndScore("measure", m.id, refreshBallotList);
      await ensureScoreQueue(scoreQueueOpts());
    }
    refreshBallotList();
    return;
  }
  if (!lastReport) return;
  setReloadStatus(`Reloading measure sources…`);
  await withFreshCache(() => progressiveMeasureEnrich(lastReport, (msg) => setReloadStatus(msg)));
  refreshOpenMeasureDetail();
  refreshBallotList();
  setReloadStatus("Measure sources reloaded.");
}

async function reloadRole(roleKey, what) {
  const items = itemsForRole(roleKey);
  if (!items.length) return;
  if (what === "ai") {
    for (const it of items) {
      await deleteAiCacheForSubject({ id: it.id, name: it.name });
    }
    resetScoreItems(items.map((it) => it.key));
    refreshBallotList();
    await ensureScoreQueue(scoreQueueOpts());
    setReloadStatus(`Re-queued scores for ${items.length} item${items.length === 1 ? "" : "s"}.`);
    return;
  }
  pauseScoring();
  try {
    let i = 0;
    let didMeasures = false;
    for (const it of items) {
      i += 1;
      setReloadStatus(`Reloading sources ${i}/${items.length} · ${it.name}`);
      if (it.kind === "measure") {
        if (!didMeasures && lastReport) {
          await withFreshCache(() =>
            progressiveMeasureEnrich(lastReport, (msg) => setReloadStatus(msg))
          );
          didMeasures = true;
        }
        continue;
      }
      const raw = candidateIndex.get(String(it.id));
      if (!raw) continue;
      const c = withElectionMeta(raw);
      if (detailCtx && String(detailCtx.candidate?.id) === String(c.id)) resetDetailStagesUi();
      await startSharedEnrich(c, { fresh: true, skipAi: true });
      if (detailCtx && String(detailCtx.candidate?.id) === String(c.id)) {
        detailCtx.enrich = sharedEnrich.enrich || {};
        patchDetail(null, true);
      }
    }
    refreshOpenMeasureDetail();
    refreshBallotList();
    setReloadStatus("Sources reloaded.");
  } finally {
    resumeScoring();
  }
}

async function handleReload(btn) {
  const what = btn.getAttribute("data-reload");
  const wrap = btn.closest("[data-reload-scope]");
  const scope = wrap?.getAttribute("data-reload-scope") || "";
  const id = wrap?.getAttribute("data-reload-id") || "";
  if (!what || !scope) return;
  const all = document.querySelectorAll(".reload-btn");
  all.forEach((b) => {
    b.disabled = true;
  });
  btn.classList.add("is-busy");
  try {
    if (scope === "candidate") await reloadCandidate(id, what);
    else if (scope === "measure") await reloadMeasure(id, what);
    else if (scope === "role") await reloadRole(id, what);
    else if (scope === "ballot") await reloadRole("all", what);
  } catch (e) {
    console.warn("reload", e);
    setReloadStatus(e && e.message ? String(e.message) : "Reload failed.");
  } finally {
    all.forEach((b) => {
      b.disabled = false;
    });
    btn.classList.remove("is-busy");
  }
}

function paintVerdict(card, status) {
  const host = $("#verdict-card");
  if (!host) return;
  const page = $("#verdict-page");
  const want = (page && page.getAttribute("data-subject-key")) || activeVerdictKey;
  if (card) {
    const got = card.subject_key != null ? String(card.subject_key) : "";
    if (want && got && want !== got) return;
    host.innerHTML = renderVerdictCard(card);
    return;
  }
  const el = $("#verdict-status");
  if (el && status) el.textContent = status;
}

function wireVerdictPage(kind) {
  const back = $("#back-ballot");
  if (back) {
    back.onclick = (ev) => {
      ev.preventDefault();
      measureDetailId = null;
      resumeScoring();
      showView("ballot");
      refreshBallotList();
    };
  }
  const details = $("#verdict-details");
  if (details) {
    details.onclick = (ev) => {
      ev.preventDefault();
      if (kind === "measure" && verdictCtx?.measure) {
        openMeasureDetail(verdictCtx.measure, { fromVerdict: true });
      } else if (verdictCtx?.candidate) {
        openDetail(verdictCtx.candidate, { fromVerdict: true });
      }
    };
  }
  const settings = $("#verdict-settings");
  if (settings) {
    settings.onclick = (ev) => {
      ev.preventDefault();
      loadSettingsForm();
      showView("settings");
    };
  }
  const cardHost = $("#verdict-card");
  if (cardHost) {
    cardHost.onclick = (ev) => {
      const btn = ev.target.closest("[data-tab]");
      if (!btn) return;
      ev.preventDefault();
      const tab = btn.getAttribute("data-tab");
      if (kind === "measure" && verdictCtx?.measure) {
        openMeasureDetail(verdictCtx.measure, { fromVerdict: true });
        return;
      }
      if (verdictCtx?.candidate) {
        if (tab) rememberTab(tab);
        openDetail(verdictCtx.candidate, { fromVerdict: true, tab });
      }
    };
  }
}

function applyDetailCrumb(host, fromVerdict) {
  if (!fromVerdict) return;
  const crumb = host.querySelector(".crumb");
  if (!crumb) return;
  crumb.innerHTML = `<a href="#" id="back-verdict">← Verdict</a> · <a href="#" id="back-ballot">← Ballot</a>`;
}

async function openVerdict(raw) {
  if (readTabFromUrl()) {
    await openDetail(raw);
    return;
  }
  const c = withElectionMeta(raw);
  const key = subjectKey("candidate", c.id);
  const token = beginVerdict(key);
  verdictCtx = { kind: c.is_judge ? "judge" : "candidate", candidate: c, measure: null, card: null };
  measureDetailId = null;
  const host = $("#verdict-host");
  if (!host) {
    await openDetail(c);
    return;
  }
  host.innerHTML = renderVerdictShell(
    {
      kind: c.is_judge ? "judge" : "candidate",
      subject_key: key,
      ...c,
    },
    { noKey: !hasLlmKey(), noWisp: hasLlmKey() && !hasWispConfigured() }
  );
  showView("verdict");
  document.title = `${c.name || "Candidate"} · verdict`;
  wireVerdictPage("candidate");
  pauseScoring();
  const accept = (card, status) => {
    if (!verdictLive(token)) return false;
    if (card) {
      const stamped = stampCard(card, key, c);
      verdictCtx.card = stamped;
      paintVerdict(stamped);
      return true;
    }
    paintVerdict(null, status);
    return false;
  };
  const cached = getScoreItem("candidate", c.id);
  if (cached && cached.card) {
    accept(cached.card);
  } else if (hasLlmKey() && hasWispConfigured() && cached && cached.status !== "done") {
    accept(null, "Scoring this candidate…");
    try {
      const item = await prioritizeAndScore("candidate", c.id, refreshBallotList);
      if (item && item.card && verdictLive(token) && !verdictCtx.card) {
        accept(item.card);
      }
    } catch (e) {
      console.warn("prioritize verdict", e);
    }
  }

  let lastSkip = "";
  const onStage = (u) => {
    if (!verdictLive(token)) return;
    if (u.enrich && u.enrich.verdict) {
      accept(u.enrich.verdict);
    } else if (u.status === "skip" && u.id === "ai_verdict" && !verdictCtx.card) {
      lastSkip = u.detail || lastSkip;
      if (lastSkip) accept(null, lastSkip);
    } else if (u.status === "running" && !verdictCtx.card) {
      accept(null, (u.label || "Sources") + "…");
    }
  };
  startSharedEnrich(c);
  attachEnrichListener(onStage);

  if (hasLlmKey() && hasWispConfigured()) {
    try {
      const enrichSnap = sharedEnrich.key === key ? sharedEnrich.enrich || {} : {};
      const early = await runVerdictPass(c, enrichSnap, { pass: "early" });
      if (!verdictLive(token)) return;
      if (early && early.card) {
        accept(early.card);
        if (sharedEnrich.key === key && sharedEnrich.enrich) {
          sharedEnrich.enrich.verdict = verdictCtx.card;
        }
        rememberCard("candidate", c.id, verdictCtx.card);
      } else if (early && early.skip && !verdictCtx.card) {
        lastSkip = early.skip;
        accept(null, early.skip);
      }
    } catch (e) {
      lastSkip = formatUserError(e);
      console.warn("verdict early", e);
    }
  }

  try {
    const { enrich } = await startSharedEnrich(c);
    if (!verdictLive(token)) return;
    if (enrich && enrich.verdict) {
      accept(enrich.verdict);
      rememberCard("candidate", c.id, verdictCtx.card);
    } else if (!verdictCtx.card && !hasLlmKey()) {
      accept(null, "No LLM key — open Details for filings, or add a key in Settings.");
    } else if (!verdictCtx.card) {
      accept(null, lastSkip || "No verdict from this pass. Open Details for filings.");
    }
  } catch (e) {
    if (verdictLive(token) && !verdictCtx.card) accept(null, formatUserError(e));
  }
}

async function openVerdictMeasure(raw) {
  if (readTabFromUrl()) {
    openMeasureDetail(raw);
    return;
  }
  const m = { ...raw };
  const key = subjectKey("measure", m.id);
  const token = beginVerdict(key);
  verdictCtx = { kind: "measure", measure: m, candidate: null, card: null };
  measureDetailId = String(m.id);
  const host = $("#verdict-host");
  if (!host) {
    openMeasureDetail(m);
    return;
  }
  host.innerHTML = renderVerdictShell(
    {
      kind: "measure",
      subject_key: key,
      ...m,
      election_name: lastReport?.election_name,
      election_date: lastReport?.election_date,
    },
    { noKey: !hasLlmKey(), noWisp: hasLlmKey() && !hasWispConfigured() }
  );
  showView("verdict");
  document.title = `${m.measure_code || m.title || "Measure"} · verdict`;
  wireVerdictPage("measure");
  pauseScoring();
  const accept = (card, status) => {
    if (!verdictLive(token)) return;
    if (card) {
      const stamped = stampCard(card, key, m);
      verdictCtx.card = stamped;
      paintVerdict(stamped);
      return;
    }
    paintVerdict(null, status);
  };
  const cachedM = getScoreItem("measure", m.id);
  if (cachedM && cachedM.card) {
    accept(cachedM.card);
  }
  if (!hasLlmKey()) return;
  if (!hasWispConfigured()) {
    accept(null, "Wisp required for live verdict.");
    return;
  }
  if (!verdictCtx.card) accept(null, "Searching and scoring this measure…");
  try {
    const result = await runMeasureVerdict(m);
    if (!verdictLive(token)) return;
    if (result && result.card) {
      accept(result.card);
      rememberCard("measure", m.id, verdictCtx.card);
    } else {
      accept(null, (result && result.skip) || "No verdict.");
    }
  } catch (e) {
    if (verdictLive(token)) accept(null, formatUserError(e));
  }
}

function patchDetail(stageId, final, only) {
  if (!detailCtx) return;
  const opts = {
    votesUi: detailCtx.votesUi,
    financeUi: detailCtx.financeUi,
  };
  if (stageId) opts.stageId = stageId;
  if (final) opts.final = true;
  if (only) opts.only = only;
  patchDetailSections(detailCtx.candidate, detailCtx.enrich, opts);
}

function readVotesUiFromDom(root) {
  const q = root.querySelector('[name="votes-q"]');
  const year = root.querySelector('[name="votes-year"]');
  const pos = root.querySelector('[name="votes-pos"]');
  const size = root.querySelector('[name="votes-size"]');
  if (!detailCtx) return;
  const ui = detailCtx.votesUi;
  if (q) ui.query = q.value;
  if (year) ui.year = year.value;
  if (pos) ui.position = pos.value;
  if (size) ui.pageSize = Number(size.value) || 10;
}

function readFinanceUiFromDom(root) {
  const q = root.querySelector('[name="fin-q"]');
  const kind = root.querySelector('[name="fin-kind"]');
  const size = root.querySelector('[name="fin-size"]');
  if (!detailCtx) return;
  const ui = detailCtx.financeUi;
  if (q) ui.query = q.value;
  if (kind) ui.kind = kind.value || "all";
  if (size) ui.pageSize = Number(size.value) || 10;
}

function wireDetailLists(root) {
  let searchTimer = null;
  root.addEventListener("input", (ev) => {
    const t = ev.target;
    if (!t || !detailCtx) return;
    if (t.name === "votes-q") {
      clearTimeout(searchTimer);
      searchTimer = setTimeout(() => {
        detailCtx.votesUi.query = t.value;
        detailCtx.votesUi.page = 1;
        patchDetail(null, false, "votes");
      }, 120);
      return;
    }
    if (t.name === "fin-q") {
      clearTimeout(searchTimer);
      searchTimer = setTimeout(() => {
        detailCtx.financeUi.query = t.value;
        detailCtx.financeUi.page = 1;
        patchDetail(null, false, "extra");
      }, 120);
    }
  });
  root.addEventListener("change", (ev) => {
    const t = ev.target;
    if (!t || !detailCtx) return;
    if (t.name === "votes-year") {
      detailCtx.votesUi.year = t.value;
      detailCtx.votesUi.page = 1;
      patchDetail(null, false, "votes");
    } else if (t.name === "votes-pos") {
      detailCtx.votesUi.position = t.value;
      detailCtx.votesUi.page = 1;
      patchDetail(null, false, "votes");
    } else if (t.name === "votes-size") {
      detailCtx.votesUi.pageSize = Number(t.value) || 10;
      detailCtx.votesUi.page = 1;
      patchDetail(null, false, "votes");
    } else if (t.name === "fin-kind") {
      detailCtx.financeUi.kind = t.value || "all";
      detailCtx.financeUi.page = 1;
      patchDetail(null, false, "extra");
    } else if (t.name === "fin-size") {
      detailCtx.financeUi.pageSize = Number(t.value) || 10;
      detailCtx.financeUi.page = 1;
      patchDetail(null, false, "extra");
    }
  });
  root.addEventListener("click", (ev) => {
    const votesBtn = ev.target.closest("[data-votes-page]");
    if (votesBtn && detailCtx) {
      const dir = votesBtn.getAttribute("data-votes-page");
      readVotesUiFromDom(root);
      if (dir === "prev") detailCtx.votesUi.page = Math.max(1, (detailCtx.votesUi.page || 1) - 1);
      else if (dir === "next") detailCtx.votesUi.page = (detailCtx.votesUi.page || 1) + 1;
      patchDetail(null, false, "votes");
      return;
    }
    const finBtn = ev.target.closest("[data-fin-page]");
    if (finBtn && detailCtx) {
      const dir = finBtn.getAttribute("data-fin-page");
      readFinanceUiFromDom(root);
      if (dir === "prev") detailCtx.financeUi.page = Math.max(1, (detailCtx.financeUi.page || 1) - 1);
      else if (dir === "next") detailCtx.financeUi.page = (detailCtx.financeUi.page || 1) + 1;
      patchDetail(null, false, "extra");
    }
  });
}

async function openDetail(raw, opts = {}) {
  const c = withElectionMeta(raw);
  const stages = planStages(c);
  const host = $("#detail-host");
  measureDetailId = null;
  const reuse =
    sharedEnrich.key === `c:${c.id}` && sharedEnrich.enrich
      ? sharedEnrich.enrich
      : {};
  detailCtx = {
    candidate: c,
    enrich: reuse,
    votesUi: defaultVotesUi(),
    financeUi: defaultFinanceUi(),
    fromVerdict: !!opts.fromVerdict,
  };
  const activeTab = opts.tab || readRememberedTab();
  host.innerHTML = renderDetailShell(c, stages, {
    activeTab,
  });
  applyDetailCrumb(host, !!opts.fromVerdict);
  showView("detail");
  document.title = `${c.name || "Candidate"} · electionizer`;

  $("#back-ballot")?.addEventListener("click", (ev) => {
    ev.preventDefault();
    resumeScoring();
    showView("ballot");
    refreshBallotList();
  });
  $("#back-verdict")?.addEventListener("click", (ev) => {
    ev.preventDefault();
    showView("verdict");
    document.title = `${c.name || "Candidate"} · verdict`;
  });
  writeTabToUrl(activeTab);
  wireDetailTabs(host);
  wireDetailLists(host);

  const detailId = String(c.id);
  const onStage = (u) => {
    if (!detailCtx || String(detailCtx.candidate.id) !== detailId) return;
    markStage(u.id, u.status, u.detail);
    if (u.status === "running") {
      setDetailPct(u.completed, u.total, u.label + "…");
    } else {
      setDetailPct(
        u.completed,
        u.total,
        u.label + (u.detail ? " — " + u.detail : "")
      );
    }
    if (
      u.enrich &&
      (u.status === "done" || u.status === "skip" || u.status === "error")
    ) {
      detailCtx.enrich = u.enrich;
      patchDetail(u.id, false);
    }
  };
  startSharedEnrich(c);
  attachEnrichListener(onStage);
  if (reuse && Object.keys(reuse).length) {
    patchDetail(null, false);
  }

  try {
    const { enrich } = await startSharedEnrich(c);
    if (!detailCtx || String(detailCtx.candidate.id) !== detailId) return;
    detailCtx.enrich = enrich;
    patchDetail(null, true);
    setDetailPct(stages.length, stages.length, "complete");
    const progress = $("#detail-progress");
    if (progress) {
      progress.classList.add("status-done");
      progress.open = false;
      const msg = $("#detail-msg");
      if (msg) msg.textContent = "All stages finished.";
    }
  } catch (e) {
    if (!detailCtx || String(detailCtx.candidate.id) !== detailId) return;
    console.error(e);
    const msg = $("#detail-msg");
    if (msg) msg.textContent = formatUserError(e);
    const progress = $("#detail-progress");
    if (progress) {
      progress.classList.add("status-failed");
      progress.open = true;
    }
  }
}

async function refreshWispStatus() {
  const el = $("#wisp-status");
  if (!el) return;
  const url = getWispUrl();
  if (!url) {
    el.textContent =
      "Disabled — state scrapes use CORS proxy or fail on CORS. Paste a Wisp URL or restore default.";
    return;
  }
  const src = isDefaultWispUrl() ? "default" : "custom";
  el.textContent = `Loading libcurl.js (${src})…`;
  try {
    await ensureCurl();
    el.textContent = isCurlReady()
      ? `libcurl.js ready · ${url} (${src})`
      : `libcurl.js loaded but not ready · ${url}`;
  } catch (e) {
    el.textContent = `libcurl.js error: ${e.message || e}`;
  }
}

function loadSettingsForm() {
  const key = getFecApiKey();
  $("#fec-key").value = key === "DEMO_KEY" ? "" : key;
  $("#os-key").value = getOpenStatesApiKey();
  if ($("#ftm-key")) $("#ftm-key").value = getFtmApiKey();
  if ($("#cl-token")) $("#cl-token").value = getCourtListenerToken();
  if ($("#civic-key")) $("#civic-key").value = getCivicApiKey();
  if ($("#llm-key")) $("#llm-key").value = getLlmApiKey();
  if ($("#llm-provider")) $("#llm-provider").value = getLlmProvider();
  if ($("#llm-model")) $("#llm-model").value = getLlmModel();
  if ($("#score-concurrency")) {
    $("#score-concurrency").value = String(getScoreConcurrency());
    const lab = $("#score-concurrency-val");
    if (lab) {
      const n = getScoreConcurrency();
      lab.textContent = `${n} at a time. 1 = sequential. Local Wisp handles more than the public instance.`;
    }
  }
  if ($("#settings-precinct")) $("#settings-precinct").value = getVoterPrecinct();
  if ($("#settings-voter-party")) $("#settings-voter-party").value = getVoterParty();
  if ($("#precinct-input")) $("#precinct-input").value = getVoterPrecinct();
  if ($("#voter-party-input")) $("#voter-party-input").value = getVoterParty();
  if (getVoterPrecinct() || getVoterParty()) {
    const det = $("#street-details");
    if (det) det.open = true;
  }
  $("#cycle-input").value = String(getCycle());
  if ($("#cors-proxy")) $("#cors-proxy").value = getCorsProxy();
  if ($("#wisp-url")) $("#wisp-url").value = getWispUrl();
  const meta = getFlDosTsvMeta();
  if ($("#fl-dos-meta")) {
    $("#fl-dos-meta").textContent = meta
      ? `Stored: ${meta}`
      : "No TSV stored.";
  }
  syncModeUi();
  $("#settings-status").textContent = "";
  refreshWispStatus();
}

function registerServiceWorker() {
  if (!("serviceWorker" in navigator)) return;
  if (!/^https?:$/.test(location.protocol)) return;
  navigator.serviceWorker.register("./sw.js").catch((e) => {
    console.info("SW register skipped:", e.message || e);
  });
}

async function offerLastBallot() {
  try {
    const last = await getLastBallot();
    if (!last || !last.report) return;
    const el = $("#last-ballot-offer");
    if (!el) return;
    const when = last.savedAt
      ? new Date(last.savedAt).toLocaleString()
      : "earlier";
    el.hidden = false;
    el.innerHTML = `Last ballot <strong>${last.zip || "?"}</strong> (${when}).
      <a href="#" id="reopen-last-ballot">Reopen</a>`;
    $("#reopen-last-ballot")?.addEventListener("click", (ev) => {
      ev.preventDefault();
      presentBallot(last.report);
      $("#status-line").textContent = `Reopened cached ballot for ${last.zip}.`;
      $("#status-line").classList.remove("warn-banner");
    });
  } catch {
    /* ignore */
  }
}

async function main() {
  const status = $("#status-line");
  const sys = $("#sys-status");
  registerServiceWorker();
  try {
    await init();
    wasmReady = true;
    sys.textContent = "ONLINE";
    syncModeUi();
    status.textContent = statusMessage();
    mountVoterProfile(document, {
      onChange: () => {
        reapplyVoterFit();
        if (lastReport) refreshBallotList();
        if (verdictCtx && verdictCtx.card) paintVerdict(verdictCtx.card);
      },
    });
    document.addEventListener(
      "click",
      (ev) => {
        const btn = ev.target.closest("button[data-reload]");
        if (!btn) return;
        ev.preventDefault();
        ev.stopPropagation();
        handleReload(btn);
      },
      true
    );
    await offerLastBallot();
    // Warm libcurl.js in background when Wisp is on (default Mercury or custom)
    if (getWispUrl()) {
      ensureCurl()
        .then(() => {
          if (isCurlReady() && status.textContent === statusMessage()) {
            /* leave status; optional quiet warm */
          }
        })
        .catch(() => {});
    }
  } catch (e) {
    console.error(e);
    sys.textContent = "ERROR";
    status.textContent =
      "Failed to load WASM. Run: ./scripts/build-wasm.sh then serve web/";
    status.classList.add("warn-banner");
    await offerLastBallot();
    return;
  }

  function goHome(ev) {
    if (ev) ev.preventDefault();
    cancelScoring();
    showView("home");
    status.textContent = statusMessage();
    status.classList.remove("warn-banner");
    hideBallotJob();
    syncModeUi();
  }

  function goScorecard(ev) {
    if (ev) ev.preventDefault();
    if (!lastReport) return;
    paintScoreUi();
    showView("scorecard");
  }

  function goSettings(ev) {
    if (ev) ev.preventDefault();
    loadSettingsForm();
    showView("settings");
    if (window.ElectionizerTheme) {
      const id = window.ElectionizerTheme.get();
      window.ElectionizerTheme.set(id);
    }
  }

  if ($("#precinct-input")) $("#precinct-input").value = getVoterPrecinct();
  if ($("#voter-party-input")) $("#voter-party-input").value = getVoterParty();
  if (getVoterPrecinct() || getVoterParty()) {
    const det = $("#street-details");
    if (det) det.open = true;
  }

  $("#zip-form").addEventListener("submit", (ev) => {
    ev.preventDefault();
    setVoterPrecinct($("#precinct-input")?.value || "");
    setVoterParty($("#voter-party-input")?.value || "");
    showBallot($("#zip-input").value, $("#street-input")?.value || "");
  });

  $("#back-home").addEventListener("click", goHome);
  $("#nav-home").addEventListener("click", goHome);
  $("#nav-settings").addEventListener("click", goSettings);
  $("#demo-settings-link")?.addEventListener("click", goSettings);
  $("#back-from-settings").addEventListener("click", goHome);
  $("#nav-scorecard")?.addEventListener("click", goScorecard);
  $("#back-scorecard-ballot")?.addEventListener("click", (ev) => {
    ev.preventDefault();
    showView("ballot");
    refreshBallotList();
  });

  $("#wisp-use-origin")?.addEventListener("click", () => {
    const proto = location.protocol === "https:" ? "wss:" : "ws:";
    const url = `${proto}//${location.host}/`;
    if ($("#wisp-url")) $("#wisp-url").value = url;
    setWispUrl(url);
    refreshWispStatus();
    $("#settings-status").textContent = `Wisp set to ${url}.`;
  });

  $("#wisp-use-default")?.addEventListener("click", () => {
    resetWispUrlToDefault();
    if ($("#wisp-url")) $("#wisp-url").value = DEFAULT_WISP_URL;
    refreshWispStatus();
    $("#settings-status").textContent = `Wisp restored to default ${DEFAULT_WISP_URL}`;
  });

  $("#score-concurrency")?.addEventListener("input", () => {
    const n = setScoreConcurrency($("#score-concurrency").value);
    const lab = $("#score-concurrency-val");
    if (lab) {
      lab.textContent = `${n} at a time. 1 = sequential. Local Wisp handles more than the public instance.`;
    }
  });

  $("#settings-form").addEventListener("submit", async (ev) => {
    ev.preventDefault();
    const mode = $("#mode-fixture").checked ? "fixture" : "live";
    setMode(mode);
    setFecApiKey($("#fec-key").value);
    setOpenStatesApiKey($("#os-key").value);
    setFtmApiKey($("#ftm-key")?.value || "");
    setCourtListenerToken($("#cl-token")?.value || "");
    setCivicApiKey($("#civic-key")?.value || "");
    setLlmApiKey($("#llm-key")?.value || "");
    setLlmProvider($("#llm-provider")?.value || "xai");
    setLlmModel($("#llm-model")?.value || "");
    if ($("#score-concurrency")) setScoreConcurrency($("#score-concurrency").value);
    setVoterPrecinct($("#settings-precinct")?.value || "");
    setVoterParty($("#settings-voter-party")?.value || "");
    if ($("#precinct-input")) $("#precinct-input").value = getVoterPrecinct();
    if ($("#voter-party-input")) $("#voter-party-input").value = getVoterParty();
    setCorsProxy($("#cors-proxy")?.value || "");
    setWispUrl($("#wisp-url")?.value || "");
    const cycle = parseInt($("#cycle-input").value, 10);
    if (Number.isFinite(cycle)) setCycle(cycle);
    syncModeUi();
    $("#settings-status").textContent = "Saved.";
    status.textContent = statusMessage();
    await refreshWispStatus();
  });

  $("#fl-dos-file")?.addEventListener("change", async (ev) => {
    const file = ev.target.files && ev.target.files[0];
    if (!file) return;
    try {
      const text = await file.text();
      setFlDosTsv(text, `${file.name} · ${text.length} bytes`);
      loadSettingsForm();
      $("#settings-status").textContent = `Loaded ${file.name}.`;
    } catch (e) {
      $("#settings-status").textContent = e.message || String(e);
    }
  });

  $("#fl-dos-clear")?.addEventListener("click", () => {
    setFlDosTsv("");
    if ($("#fl-dos-file")) $("#fl-dos-file").value = "";
    loadSettingsForm();
    $("#settings-status").textContent = "Cleared FL DOS TSV.";
  });

  $("#clear-ballot-cache")?.addEventListener("click", async () => {
    const statusEl = $("#clear-cache-status");
    const btn = $("#clear-ballot-cache");
    if (btn) btn.disabled = true;
    try {
      const { cleared } = await clearBallotAndResponseCache();
      lastReport = null;
      candidateIndex = new Map();
      measureIndex = new Map();
      measureDetailId = null;
      detailCtx = null;
      verdictCtx = null;
      sharedEnrich = { key: null, promise: null, enrich: {}, listeners: [] };
      const root = $("#ballot-root");
      if (root) root.innerHTML = "";
      clearLastBallotOffer();
      hideBallotJob();
      if (statusEl) {
        statusEl.textContent =
          cleared > 0
            ? `Cleared ${cleared} cached item(s). Last-ballot offer removed. Next ZIP is a live refetch. Keys & settings kept.`
            : "Cache was already empty. Last-ballot offer cleared. Keys & settings kept.";
      }
      $("#settings-status").textContent = "Ballot & response cache cleared.";
    } catch (e) {
      if (statusEl) statusEl.textContent = e.message || String(e);
    } finally {
      if (btn) btn.disabled = false;
    }
  });

  if (location.hash === "#settings") {
    goSettings();
  } else {
    syncModeUi();
  }
}

main();
