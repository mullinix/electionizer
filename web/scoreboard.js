import { runVerdictPass, runMeasureVerdict } from "./enrich.js";
import { applyVoterFit, fitCss, fitTextCss, reloadButtonsHtml } from "./verdict.js";
import { getScoreConcurrency } from "./settings.js";

function esc(s) {
  return String(s ?? "")
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

/** @type {{ items: object[], byKey: Map<string, object>, gen: number, running: boolean, paused: boolean } | null} */
let board = null;
let queueLoop = null;

function overallScore(card) {
  if (!card) return null;
  const fitted = applyVoterFit(card);
  const overall =
    fitted.overall instanceof Map ? Object.fromEntries(fitted.overall) : fitted.overall || {};
  const n = Number(overall.score);
  return Number.isFinite(n) ? n : null;
}

export function fitFromCard(card) {
  return overallScore(card);
}

export function getBoard() {
  return board;
}

export function scoreKey(kind, id) {
  return `${kind === "measure" ? "m" : "c"}:${String(id)}`;
}

export function getScoreItem(kind, id) {
  if (!board) return null;
  return board.byKey.get(scoreKey(kind, id)) || null;
}

export function scoresByKey() {
  return board ? board.byKey : new Map();
}

export function compareFit(aRec, bRec) {
  const a = aRec && aRec.fit;
  const b = bRec && bRec.fit;
  const aN = a == null;
  const bN = b == null;
  if (aN && bN) return 0;
  if (aN) return 1;
  if (bN) return -1;
  return b - a;
}

export function sortCandidatesByFit(candidates, scores) {
  const list = Array.isArray(candidates) ? candidates.slice() : [];
  if (!scores || !scores.size) return list;
  return list.sort((a, b) =>
    compareFit(scores.get(scoreKey("candidate", a.id)), scores.get(scoreKey("candidate", b.id)))
  );
}

export function sortMeasuresByFit(measures, scores) {
  const list = Array.isArray(measures) ? measures.slice() : [];
  if (!scores || !scores.size) return list;
  return list.sort((a, b) =>
    compareFit(scores.get(scoreKey("measure", a.id)), scores.get(scoreKey("measure", b.id)))
  );
}

export function renderFitChip(rec) {
  if (!rec || rec.status === "queued") {
    return `<span class="fit-chip fit-pending" title="Queued">…</span>`;
  }
  if (rec.status === "scraping" || rec.status === "scoring") {
    return `<span class="fit-chip fit-run" title="${esc(rec.stage || "Scoring")}">…</span>`;
  }
  if (rec.status === "skip" || rec.status === "error") {
    return `<span class="fit-chip fit-na" title="${esc(rec.skip || rec.status)}">—</span>`;
  }
  if (rec.fit == null) {
    return `<span class="fit-chip fit-na" title="No score">—</span>`;
  }
  const color = fitCss(rec.fit);
  const text = fitTextCss(rec.fit);
  const tip = rec.card && rec.card.headline ? rec.card.headline : `Fit ${rec.fit}`;
  return `<span class="fit-chip" style="background:${color};color:${text}" title="${esc(tip)}">${esc(
    String(rec.fit)
  )}</span>`;
}

function makeCandidateItem(c, office, jurisdiction, roleKey, roleLabel) {
  return {
    key: scoreKey("candidate", c.id),
    kind: "candidate",
    id: String(c.id),
    name: c.name || "",
    office: office || c.office || "",
    jurisdiction: jurisdiction || c.jurisdiction || "",
    party: c.party || "",
    party_class: c.party_class || "o",
    is_judge: !!c.is_judge,
    roleKey,
    roleLabel: roleLabel || office || c.office || "Office",
    raw: c,
    status: "queued",
    stage: "Queued",
    card: null,
    fit: null,
    skip: null,
  };
}

function makeMeasureItem(m) {
  const label = m.measure_code ? `${m.measure_code}: ${m.title || ""}` : m.title || "Measure";
  return {
    key: scoreKey("measure", m.id),
    kind: "measure",
    id: String(m.id),
    name: label,
    office: "Ballot measure",
    jurisdiction: m.jurisdiction || "",
    party: "",
    party_class: "o",
    is_judge: false,
    roleKey: "measures",
    roleLabel: "Ballot measures",
    raw: m,
    status: "queued",
    stage: "Queued",
    card: null,
    fit: null,
    skip: null,
  };
}

export function flattenBallotItems(report) {
  const items = [];
  for (const sec of report?.ballot_sections || []) {
    if (sec.kind === "judicial") {
      const block = sec.title || "Judicial";
      for (const g of sec.seats || []) {
        const office = g.office || block;
        const roleKey = `j:${office}`;
        for (const c of g.candidates || []) {
          items.push(makeCandidateItem(c, office, g.jurisdiction, roleKey, office));
        }
      }
    } else if (sec.group) {
      const g = sec.group;
      const office = g.office || "Office";
      const roleKey = `o:${office}:${g.jurisdiction || ""}`;
      for (const c of g.candidates || []) {
        items.push(makeCandidateItem(c, office, g.jurisdiction, roleKey, office));
      }
    }
  }
  for (const g of report?.office_groups || []) {
    const office = g.office || "Office";
    const roleKey = `o:${office}:${g.jurisdiction || ""}`;
    if (items.some((it) => it.roleKey === roleKey)) continue;
    for (const c of g.candidates || []) {
      items.push(makeCandidateItem(c, office, g.jurisdiction, roleKey, office));
    }
  }
  for (const m of report?.measures || []) {
    if (m && m.id != null) items.push(makeMeasureItem(m));
  }
  return items;
}

function recount(b) {
  const items = b.items || [];
  const done = items.filter((it) => it.status === "done").length;
  const skip = items.filter((it) => it.status === "skip" || it.status === "error").length;
  const active = items.find((it) => it.status === "scraping" || it.status === "scoring");
  return {
    total: items.length,
    done,
    skip,
    pending: items.length - done - skip,
    active,
    pct: items.length ? Math.round(((done + skip) / items.length) * 100) : 0,
  };
}

export function boardStats() {
  return board ? recount(board) : { total: 0, done: 0, skip: 0, pending: 0, active: null, pct: 0 };
}

function mark(item, patch) {
  Object.assign(item, patch);
  if (board) board.byKey.set(item.key, item);
}

export function rememberCard(kind, id, card) {
  if (!board || !card) return;
  const item = board.byKey.get(scoreKey(kind, id));
  if (!item) return;
  const skey = kind === "measure" ? `m:${id}` : `c:${id}`;
  const stamped = {
    ...card,
    subject_key: card.subject_key || skey,
    subject_name: card.subject_name || item.name,
    subject_office: card.subject_office || item.office,
  };
  mark(item, {
    status: "done",
    stage: "Scored",
    card: stamped,
    fit: overallScore(stamped),
    skip: null,
  });
}

export function reapplyVoterFit() {
  if (!board) return;
  for (const item of board.items) {
    if (item.card) item.fit = overallScore(item.card);
  }
}

export function resetScoreItems(keys) {
  if (!board) return;
  const set = new Set((keys || []).map(String));
  for (const item of board.items) {
    if (!set.has(item.key)) continue;
    mark(item, {
      status: "queued",
      stage: "Queued",
      card: null,
      fit: null,
      skip: null,
    });
  }
}

export function itemsForRole(roleKey) {
  if (!board) return [];
  if (roleKey === "all" || roleKey === "ballot") return board.items.slice();
  if (roleKey === "judicial") {
    return board.items.filter((it) => String(it.roleKey).startsWith("j:"));
  }
  return board.items.filter((it) => it.roleKey === roleKey);
}

export async function ensureScoreQueue(opts = {}) {
  if (!board) return;
  if (board.running) return;
  return runScoreQueue(opts);
}

export function resetBoard(report) {
  if (board) board.gen += 1;
  const items = flattenBallotItems(report);
  const byKey = new Map(items.map((it) => [it.key, it]));
  board = {
    items,
    byKey,
    gen: board ? board.gen : 1,
    running: false,
    paused: false,
    zip: report?.zip || "",
  };
  return board;
}

export function cancelScoring() {
  if (board) {
    board.gen += 1;
    board.running = false;
    board.paused = false;
  }
}

export function pauseScoring() {
  if (board) board.paused = true;
}

export function resumeScoring() {
  if (board) board.paused = false;
}

function stageMark(status) {
  if (status === "done") return "✓";
  if (status === "scraping" || status === "scoring") return "›";
  if (status === "skip") return "–";
  if (status === "error") return "!";
  return "·";
}

export function renderScoreTree(opts = {}) {
  const noKey = !!opts.noKey;
  const noWisp = !!opts.noWisp;
  const b = board;
  if (!b || !b.items.length) {
    return `<section class="card job-status score-progress" id="score-progress-card" hidden></section>`;
  }
  const st = recount(b);
  let cta = "";
  if (noKey) {
    cta = `<p class="warn-banner">Add an xAI or OpenAI key in <a href="#settings" data-score-settings>Settings</a> to score this ballot.</p>`;
  } else if (noWisp) {
    cta = `<p class="warn-banner">Wisp is required for live scoring. Set a Wisp URL in <a href="#settings" data-score-settings>Settings</a>.</p>`;
  }
  const roles = [];
  for (const it of b.items) {
    let role = roles.find((r) => r.key === it.roleKey);
    if (!role) {
      role = { key: it.roleKey, label: it.roleLabel, jurisdiction: it.jurisdiction, rows: [] };
      roles.push(role);
    }
    role.rows.push(it);
  }
  const roleHtml = roles
    .map((role) => {
      const nSettled = role.rows.filter(
        (r) => r.status === "done" || r.status === "skip" || r.status === "error"
      ).length;
      const complete = nSettled === role.rows.length;
      const open = complete ? "" : " open";
      const doneClass = complete ? " score-role-done" : "";
      const people = role.rows
        .map((it) => {
          const href =
            it.kind === "measure"
              ? `data-measure-id="${esc(it.id)}"`
              : `data-cand-id="${esc(it.id)}"`;
          const scrape =
            it.status === "queued"
              ? "queued"
              : it.status === "scraping"
                ? it.stage || "packing…"
                : "identity + search";
          const score =
            it.status === "done" && it.fit != null
              ? String(it.fit)
              : it.status === "scoring"
                ? it.stage || "model…"
                : it.status === "skip" || it.status === "error"
                  ? it.skip || it.status
                  : "queued";
          return `<li class="stage-${esc(it.status)}" data-score-key="${esc(it.key)}">
            <span class="stage-mark">${stageMark(it.status)}</span>
            <a href="#" ${href} class="score-tree-name">${esc(it.name)}</a>
            ${it.party ? `<span class="party-chip party-${esc(it.party_class)}">${esc(it.party)}</span>` : ""}
            ${renderFitChip(it)}
            ${reloadButtonsHtml(it.kind, it.id)}
            <span class="stage-detail">scrape · ${esc(scrape)} · score · ${esc(score)}</span>
          </li>`;
        })
        .join("");
      return `<details class="score-role${doneClass}"${open}>
        <summary>
          <span class="score-role-title">${esc(role.label)}</span>
          ${role.jurisdiction ? `<span class="office-jurisdiction">${esc(role.jurisdiction)}</span>` : ""}
          <span class="score-role-count">${nSettled}/${role.rows.length}</span>
          ${reloadButtonsHtml("role", role.key)}
        </summary>
        <ul class="stage-list score-tree-list">${people}</ul>
      </details>`;
    })
    .join("");
  const line = st.active
    ? `${st.active.name} · ${st.active.stage || st.active.status}`
    : st.pending
      ? b.paused
        ? "Paused"
        : "Waiting…"
      : st.skip && st.done === 0
        ? "No scores"
        : "Done";
  const settled = st.pending === 0 && !st.active;
  const foldOpen = settled ? "" : " open";
  const foldDone = settled ? " score-progress-done" : "";
  return `<section class="card job-status score-progress" id="score-progress-card" role="status" aria-live="polite">
    <details class="score-progress-fold${foldDone}"${foldOpen}>
      <summary class="score-progress-head">
        <p class="pct"><strong>${st.done + st.skip} / ${st.total}</strong> · ${esc(line)}</p>
      </summary>
      <p class="meta muted">Early verdict per item (identity + live search). Details still run on click. Cached until you refresh data or score.</p>
      <div class="bar-wrap" aria-hidden="true"><div class="bar" style="width:${st.pct}%"></div></div>
      ${cta}
      <div class="score-tree">${roleHtml}</div>
    </details>
  </section>`;
}

function partyColumns(rows) {
  const cols = [];
  const order = (cls) => (cls === "r" ? 0 : cls === "d" ? 1 : 2);
  for (const it of rows) {
    const cls = it.party_class || "o";
    let col = cols.find((c) => c.cls === cls && c.party === (it.party || "—"));
    if (!col) {
      col = { cls, party: it.party || "Nonpartisan", rows: [] };
      cols.push(col);
    }
    col.rows.push(it);
  }
  cols.sort((a, b) => order(a.cls) - order(b.cls) || a.party.localeCompare(b.party));
  for (const col of cols) col.rows.sort((a, b) => compareFit(a, b));
  return cols;
}

function partyAvg(rows) {
  const nums = rows.map((r) => r.fit).filter((n) => n != null);
  if (!nums.length) return null;
  return Math.round(nums.reduce((a, b) => a + b, 0) / nums.length);
}

export function renderScorecard(opts = {}) {
  const b = board;
  const zip = b?.zip || opts.zip || "";
  if (!b || !b.items.length) {
    return `<h1>Scorecard</h1><p class="empty">No ballot loaded.</p>`;
  }
  const st = recount(b);
  const roles = [];
  for (const it of b.items) {
    let role = roles.find((r) => r.key === it.roleKey);
    if (!role) {
      role = {
        key: it.roleKey,
        label: it.roleLabel,
        jurisdiction: it.jurisdiction,
        kind: it.kind,
        rows: [],
      };
      roles.push(role);
    }
    role.rows.push(it);
  }
  const body = roles
    .map((role) => {
      if (role.kind === "measure" || role.key === "measures") {
        const sorted = role.rows.slice().sort((a, b) => compareFit(a, b));
                const rows = sorted
                  .map(
                    (it) => `<tr>
              <td><a href="#" data-measure-id="${esc(it.id)}">${esc(it.name)}</a></td>
              <td>${esc(it.jurisdiction || "—")}</td>
              <td class="fit-cell">${renderFitChip(it)}${reloadButtonsHtml("measure", it.id)}</td>
              <td class="muted">${esc((it.card && it.card.headline) || it.stage || "")}</td>
            </tr>`
                  )
          .join("");
        return `<section class="card scorecard-office" data-role-key="${esc(role.key)}">
          <header class="office-head">
            <div class="office-head-text"><h2>${esc(role.label)}</h2></div>
            ${reloadButtonsHtml("role", role.key)}
          </header>
          <table class="scorecard-table">
            <thead><tr><th>Measure</th><th>Jurisdiction</th><th>Fit</th><th>Verdict</th></tr></thead>
            <tbody>${rows}</tbody>
          </table>
        </section>`;
      }
      const cols = partyColumns(role.rows);
      const colHtml = cols
        .map((col) => {
          const avg = partyAvg(col.rows);
          const people = col.rows
            .map((it) => {
              const head = it.card && it.card.headline ? it.card.headline : it.stage || "";
              return `<li>
                <a href="#" data-cand-id="${esc(it.id)}">${esc(it.name)}</a>
                ${renderFitChip(it)}
                ${reloadButtonsHtml("candidate", it.id)}
                ${head ? `<span class="muted scorecard-head">${esc(head)}</span>` : ""}
              </li>`;
            })
            .join("");
          const avgChip =
            avg != null
              ? `<span class="fit-chip" style="background:${fitCss(avg)};color:${fitTextCss(avg)}">${avg}</span>`
              : `<span class="fit-chip fit-na">—</span>`;
          return `<section class="scorecard-party">
            <h3>${`<span class="party-chip party-${esc(col.cls)}">${esc(col.party)}</span>`}
              <span class="muted">avg</span> ${avgChip}</h3>
            <ol class="scorecard-people">${people}</ol>
          </section>`;
        })
        .join("");
      return `<section class="card scorecard-office" data-role-key="${esc(role.key)}">
        <header class="office-head">
          <div class="office-head-text">
            <h2>${esc(role.label)}</h2>
            ${role.jurisdiction ? `<p class="office-jurisdiction">${esc(role.jurisdiction)}</p>` : ""}
          </div>
          ${reloadButtonsHtml("role", role.key)}
        </header>
        <div class="scorecard-parties">${colHtml}</div>
      </section>`;
    })
    .join("");
  return `
    <header class="ballot-head">
      <h1>Scorecard${zip ? ` · ${esc(zip)}` : ""}</h1>
      ${reloadButtonsHtml("ballot", "all")}
    </header>
    <p class="meta">${st.done} scored · ${st.skip} skipped · ${st.pending} pending · best fit at the top of each party</p>
    <p class="meta muted">Click a name for the AI verdict. List scoring is the early pass (identity + live search).</p>
    ${body}`;
}

async function scoreOne(item, notify) {
  if (item.status === "done" && item.card) return;
  if (item.status !== "scraping" && item.status !== "scoring") {
    mark(item, { status: "scraping", stage: "Packing identity…", skip: null });
    notify();
  }
  mark(item, { status: "scoring", stage: "Live search + model…" });
  notify();
  let result;
  if (item.kind === "measure") {
    result = await runMeasureVerdict(item.raw);
  } else {
    result = await runVerdictPass(item.raw, {}, { pass: "early" });
  }
  if (result && result.card) {
    const skey = item.kind === "measure" ? `m:${item.id}` : `c:${item.id}`;
    mark(item, {
      status: "done",
      stage: "Scored",
      card: {
        ...result.card,
        subject_key: skey,
        subject_name: item.name,
        subject_office: item.office,
      },
      fit: overallScore(result.card),
      skip: null,
    });
  } else {
    mark(item, {
      status: "skip",
      stage: "Skipped",
      skip: (result && result.skip) || "No verdict",
    });
  }
}

export async function prioritizeAndScore(kind, id, notify) {
  if (!board) return getScoreItem(kind, id);
  const item = board.byKey.get(scoreKey(kind, id));
  if (!item) return null;
  if (item.status === "done" && item.card) return item;
  if (item.status === "scraping" || item.status === "scoring") {
    const started = Date.now();
    while (
      (item.status === "scraping" || item.status === "scoring") &&
      Date.now() - started < 180000
    ) {
      await new Promise((r) => setTimeout(r, 200));
    }
    return item;
  }
  try {
    await scoreOne(item, notify || (() => {}));
  } catch (e) {
    mark(item, {
      status: "error",
      stage: "Error",
      skip: e && e.message ? String(e.message) : "score failed",
    });
    if (notify) notify();
  }
  return item;
}

export async function runScoreQueue(opts = {}) {
  const notify = typeof opts.onUpdate === "function" ? opts.onUpdate : () => {};
  if (!board || !board.items.length) return;
  if (opts.noKey || opts.noWisp) {
    notify();
    return;
  }
  const gen = board.gen;
  board.running = true;
  board.paused = false;
  queueLoop = gen;
  notify();
  const n = Math.max(1, Math.min(8, Number(opts.concurrency) || getScoreConcurrency()));
  const takeNext = () => {
    if (!board || board.gen !== gen) return null;
    const item = board.items.find((it) => it.status === "queued");
    if (!item) return null;
    mark(item, { status: "scraping", stage: "Packing identity…", skip: null });
    return item;
  };
  const worker = async (slot) => {
    if (slot) await new Promise((r) => setTimeout(r, slot * 80));
    while (board && board.gen === gen) {
      while (board.paused && board.gen === gen) {
        await new Promise((r) => setTimeout(r, 250));
      }
      if (!board || board.gen !== gen) return;
      let item = takeNext();
      if (!item) {
        await new Promise((r) => setTimeout(r, 120));
        if (!board || board.gen !== gen) return;
        if (!board.items.some((it) => it.status === "queued")) return;
        continue;
      }
      try {
        await scoreOne(item, notify);
      } catch (e) {
        mark(item, {
          status: "error",
          stage: "Error",
          skip: e && e.message ? String(e.message) : "score failed",
        });
      }
      notify();
    }
  };
  try {
    await Promise.all(Array.from({ length: n }, (_, i) => worker(i)));
  } finally {
    if (board && board.gen === gen) board.running = false;
    if (queueLoop === gen) queueLoop = null;
    notify();
    if (board && board.gen === gen && board.items.some((it) => it.status === "queued")) {
      runScoreQueue(opts).catch(() => {});
    }
  }
}
