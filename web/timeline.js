/**
 * Correlation timeline (vertical): votes × campaign receipts × disclosure dates.
 * Cursor scrub + Gaussian weights (default σ = 4 weeks). Zoom/pan the date range.
 * Focus panel is always open (no expand/collapse jank).
 */

const MS_DAY = 86400000;
const DEFAULT_SIGMA_WEEKS = 4;
const SIGMA_MS = DEFAULT_SIGMA_WEEKS * 7 * MS_DAY;

/** @param {string|null|undefined} raw */
export function parseEventDate(raw) {
  if (raw == null) return null;
  const s = String(raw).trim();
  if (!s) return null;
  let m = s.match(/^(\d{4})-(\d{2})-(\d{2})/);
  if (m) {
    const t = Date.UTC(+m[1], +m[2] - 1, +m[3]);
    return Number.isFinite(t) ? t : null;
  }
  m = s.match(/^(\d{1,2})[\/\-.](\d{1,2})[\/\-.](\d{4})/);
  if (m) {
    const t = Date.UTC(+m[3], +m[1] - 1, +m[2]);
    return Number.isFinite(t) ? t : null;
  }
  m = s.match(/^(\d{4})[\/\-.](\d{1,2})[\/\-.](\d{1,2})/);
  if (m) {
    const t = Date.UTC(+m[1], +m[2] - 1, +m[3]);
    return Number.isFinite(t) ? t : null;
  }
  const d = Date.parse(s);
  if (!Number.isFinite(d)) return null;
  const dt = new Date(d);
  return Date.UTC(dt.getUTCFullYear(), dt.getUTCMonth(), dt.getUTCDate());
}

/** @param {number} ms */
export function fmtDate(ms) {
  if (!Number.isFinite(ms)) return "—";
  const d = new Date(ms);
  const y = d.getUTCFullYear();
  const mo = String(d.getUTCMonth() + 1).padStart(2, "0");
  const day = String(d.getUTCDate()).padStart(2, "0");
  return `${y}-${mo}-${day}`;
}

export function parseUsd(display) {
  if (display == null) return null;
  const s = String(display).replace(/[^0-9.\-]/g, "");
  if (!s) return null;
  const n = Number(s);
  return Number.isFinite(n) ? n : null;
}

export function gaussianWeight(dt, sigmaMs = SIGMA_MS) {
  if (!Number.isFinite(dt) || !Number.isFinite(sigmaMs) || sigmaMs <= 0) return 0;
  const z = dt / sigmaMs;
  return Math.exp(-0.5 * z * z);
}

/**
 * @param {object} c
 * @param {object} e
 */
export function buildTimelineEvents(c, e) {
  const out = [];
  let seq = 0;
  const push = (ev) => {
    if (!ev || !Number.isFinite(ev.t)) return;
    out.push({
      id: `ev-${seq++}`,
      t: ev.t,
      date: fmtDate(ev.t),
      kind: ev.kind,
      lane: ev.lane,
      label: ev.label || "—",
      detail: ev.detail || "",
      amount: ev.amount != null ? ev.amount : null,
      amount_display: ev.amount_display || null,
      url: ev.url || null,
      source: ev.source || null,
    });
  };

  for (const v of e.votes || []) {
    const t = parseEventDate(v.date);
    if (t == null) continue;
    const pos = (v.position || "").trim();
    push({
      t,
      kind: "vote",
      lane: "votes",
      label: v.question || "Vote",
      detail: [pos, v.result].filter(Boolean).join(" · "),
      url: v.url || null,
      source: e.votes_source || "GovTrack / Open States",
    });
  }

  const moneyRows = [];
  for (const r of e.timeline_receipts || [])
    moneyRows.push({ ...r, _stream: r.stream || "receipt" });
  if (!moneyRows.length) {
    for (const r of e.top_individuals || [])
      moneyRows.push({ ...r, _stream: "individual" });
    for (const r of e.top_committees || [])
      moneyRows.push({ ...r, _stream: "committee" });
  }
  for (const r of moneyRows) {
    const t = parseEventDate(r.date);
    if (t == null) continue;
    const stream = r._stream || r.stream || "receipt";
    const kind =
      stream === "committee" || stream === "pac" ? "committee" : "receipt";
    push({
      t,
      kind,
      lane: "money",
      label: r.name || "Contribution",
      detail:
        stream === "committee"
          ? "Committee / PAC gift (itemized)"
          : stream === "individual"
            ? "Individual gift (itemized)"
            : "Campaign receipt (itemized)",
      amount: parseUsd(r.amount_display),
      amount_display: r.amount_display || null,
      url: r.url || null,
      source: "Campaign finance (itemized)",
    });
  }

  const efdT = parseEventDate(e.efd_report_date);
  if (efdT != null) {
    const n = (e.dossier && e.dossier.holdings && e.dossier.holdings.length) || 0;
    push({
      t: efdT,
      kind: "disclosure",
      lane: "personal",
      label: "Senate eFD annual filed",
      detail: n
        ? `${n} holding${n === 1 ? "" : "s"} on Part 3 (as-filed ranges)`
        : "Personal financial disclosure (annual)",
      url: e.efd_report_url || null,
      source: "Senate eFD",
    });
  }
  const hcy = e.house_clerk_filing_year;
  if (hcy != null && String(hcy).trim()) {
    const y = parseInt(String(hcy).replace(/\D/g, "").slice(0, 4), 10);
    if (y >= 1990 && y <= 2100) {
      const t = Date.UTC(y, 5, 30);
      const n = (e.dossier && e.dossier.holdings && e.dossier.holdings.length) || 0;
      push({
        t,
        kind: "disclosure",
        lane: "personal",
        label: `House Clerk FD · ${y}`,
        detail: n
          ? `${n} Schedule A holding${n === 1 ? "" : "s"} (year midpoint — exact day unknown)`
          : "Personal FD filing year (midpoint)",
        url: e.house_clerk_pdf_url || null,
        source: "House Clerk FD",
      });
    }
  }

  // Newest first for vertical (top = recent)
  out.sort((a, b) => b.t - a.t || a.kind.localeCompare(b.kind));
  return out;
}

function esc(s) {
  return String(s ?? "")
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

function kindColor(kind) {
  if (kind === "vote") return "var(--tl-vote, #6af)";
  if (kind === "receipt") return "var(--tl-receipt, #6c8)";
  if (kind === "committee") return "var(--tl-committee, #c8a)";
  if (kind === "disclosure") return "var(--tl-disclosure, #db6)";
  return "var(--text-dim)";
}

function kindLabel(kind) {
  if (kind === "vote") return "Vote";
  if (kind === "receipt") return "Receipt";
  if (kind === "committee") return "Committee $";
  if (kind === "disclosure") return "Disclosure";
  return kind || "Event";
}

/**
 * @param {object} c
 * @param {object} e
 * @param {{ sigmaWeeks?: number }} [opts]
 */
export function renderTimelineBody(c, e, opts = {}) {
  const events = buildTimelineEvents(c, e);
  const sigmaWeeks =
    opts.sigmaWeeks != null ? opts.sigmaWeeks : DEFAULT_SIGMA_WEEKS;
  const counts = { vote: 0, receipt: 0, committee: 0, disclosure: 0 };
  for (const ev of events) {
    if (counts[ev.kind] != null) counts[ev.kind] += 1;
  }

  const judicial =
    !!(c && c.is_judge) || /courtlistener/i.test((e && e.votes_source) || "");
  const voteLane = judicial ? "Decisions" : "Votes";
  const votesNote =
    e.votes_total_available != null &&
    e.votes_fetch_cap != null &&
    e.votes_total_available > (e.votes || []).length
      ? ` · ${judicial ? "opinions" : "votes"} loaded ${(e.votes || []).length} of ${e.votes_total_available} (cap ${e.votes_fetch_cap})`
      : e.votes_fetch_cap
        ? ` · ${judicial ? "opinions" : "votes"} cap ${e.votes_fetch_cap}`
        : "";

  if (!events.length) {
    return `<h2>Correlation timeline</h2>
      <p class="meta muted">Vertical timeline of ${
        judicial ? "decisions" : "votes"
      } and dated campaign receipts. Gaussian σ = ${sigmaWeeks} weeks. Zoom the date range; scrub with the cursor.</p>
      <p class="empty muted">No dated events yet. Open // Detail sources // — need ${
        judicial ? "opinions" : "votes"
      } and/or itemized contributions with dates.</p>`;
  }

  const payload = esc(JSON.stringify({ events, sigmaWeeks }));

  return `<h2>Correlation timeline</h2>
    <p class="meta muted">Vertical axis = time (recent at top). Move the <strong>cursor</strong> to weight nearby events (1σ = ${sigmaWeeks} weeks by default). Click a mark to pin. Scroll-wheel zooms dates; drag pans. Proximity only — not causation.${esc(votesNote)}</p>
    <div class="finance-grid timeline-stats">
      <div class="finance-stat"><span class="finance-label">Events</span><span class="finance-value">${events.length}</span></div>
      <div class="finance-stat"><span class="finance-label">${voteLane}</span><span class="finance-value">${counts.vote}</span></div>
      <div class="finance-stat"><span class="finance-label">Receipts</span><span class="finance-value">${counts.receipt}</span></div>
      <div class="finance-stat"><span class="finance-label">Committee $</span><span class="finance-value">${counts.committee}</span></div>
      <div class="finance-stat"><span class="finance-label">Disclosure</span><span class="finance-value">${counts.disclosure}</span></div>
    </div>
    <div class="timeline-root" data-timeline-root data-timeline-payload="${payload}">
      <div class="timeline-controls">
        <label class="timeline-sigma-label">σ
          <input type="range" class="timeline-sigma" name="tl-sigma" min="1" max="12" step="1" value="${sigmaWeeks}" />
          <span class="timeline-sigma-val mono">${sigmaWeeks}w</span>
        </label>
        <div class="timeline-zoom-btns" role="group" aria-label="Zoom">
          <button type="button" class="list-page-btn" data-tl-zoom="out" title="Zoom out">−</button>
          <span class="timeline-zoom-val mono" data-tl-zoom-val>100%</span>
          <button type="button" class="list-page-btn" data-tl-zoom="in" title="Zoom in">+</button>
          <button type="button" class="list-page-btn" data-tl-zoom="reset" title="Reset zoom">Reset</button>
        </div>
        <div class="timeline-legend" aria-hidden="true">
          <span class="tl-leg tl-leg-vote">${judicial ? "Decision" : "Vote"}</span>
          <span class="tl-leg tl-leg-receipt">Receipt</span>
          <span class="tl-leg tl-leg-committee">Committee</span>
          <span class="tl-leg tl-leg-disclosure">Disclosure</span>
        </div>
      </div>
      <div class="timeline-layout">
        <div class="timeline-svg-wrap">
          <div class="timeline-col-heads" aria-hidden="true">
            <span class="tl-col-head tl-col-date">Date</span>
            <span class="tl-col-head">${voteLane}</span>
            <span class="tl-col-head">Campaign $</span>
            <span class="tl-col-head">Personal</span>
          </div>
          <svg class="timeline-svg" role="img" aria-label="Vertical event timeline"></svg>
        </div>
        <aside class="timeline-focus" data-timeline-focus>
          <h3 class="timeline-focus-h">Cursor</h3>
          <p class="timeline-focus-main muted" data-timeline-focus-main>Move over the chart to place the cursor.</p>
          <h4 class="timeline-corr-h">Nearby (Gaussian weight)</h4>
          <ul class="timeline-corr-list" data-timeline-corr></ul>
        </aside>
      </div>
    </div>`;
}

/**
 * @param {HTMLElement} root
 */
export function mountTimeline(root) {
  const host = root?.matches?.("[data-timeline-root]")
    ? root
    : root?.querySelector?.("[data-timeline-root]");
  if (!host) return null;

  let payload;
  try {
    payload = JSON.parse(host.getAttribute("data-timeline-payload") || "{}");
  } catch {
    return null;
  }
  const events = Array.isArray(payload.events) ? payload.events : [];
  if (!events.length) return null;

  let sigmaWeeks =
    payload.sigmaWeeks != null ? Number(payload.sigmaWeeks) : DEFAULT_SIGMA_WEEKS;
  if (!Number.isFinite(sigmaWeeks) || sigmaWeeks < 1) sigmaWeeks = DEFAULT_SIGMA_WEEKS;

  const svg = host.querySelector(".timeline-svg");
  const focusMain = host.querySelector("[data-timeline-focus-main]");
  const corrList = host.querySelector("[data-timeline-corr]");
  const sigmaInput = host.querySelector(".timeline-sigma");
  const sigmaVal = host.querySelector(".timeline-sigma-val");
  const zoomVal = host.querySelector("[data-tl-zoom-val]");
  const wrap = host.querySelector(".timeline-svg-wrap");
  if (!svg || !wrap) return null;

  // Full data range (newest → oldest in events array)
  const tNewest = events[0].t;
  const tOldest = events[events.length - 1].t;
  const fullSpan = Math.max(tNewest - tOldest, MS_DAY * 14);
  const fullPad = fullSpan * 0.04;
  const dataTMax = tNewest + fullPad; // top of chart (recent)
  const dataTMin = tOldest - fullPad; // bottom

  // View window [viewMin, viewMax] in time; top = viewMax
  let viewMax = dataTMax;
  let viewMin = dataTMin;

  let cursorT = events[Math.min(3, events.length - 1)]?.t ?? tNewest;
  let pinnedId = null;
  let pinned = false;

  const lanes = [
    { id: "votes", label: "Votes" },
    { id: "money", label: "Campaign $" },
    { id: "personal", label: "Personal" },
  ];

  const padL = 72;
  const padR = 12;
  const padT = 8;
  const padB = 12;
  const VIEW_H = 420;
  let W = Math.max(280, wrap.clientWidth || 480);
  const H = VIEW_H;

  const maxAmt = events.reduce((m, e) => Math.max(m, e.amount || 0), 0) || 1;

  function viewSpan() {
    return Math.max(viewMax - viewMin, MS_DAY);
  }

  function zoomPct() {
    const full = dataTMax - dataTMin;
    const cur = viewSpan();
    return Math.round((full / cur) * 100);
  }

  function yOf(t) {
    // top = viewMax (recent), bottom = viewMin (old)
    const u = (viewMax - t) / viewSpan();
    return padT + u * (H - padT - padB);
  }

  function tOfY(y) {
    const u = (y - padT) / Math.max(1, H - padT - padB);
    return viewMax - u * viewSpan();
  }

  function xOfLane(laneId) {
    const i = Math.max(0, lanes.findIndex((l) => l.id === laneId));
    const colW = (W - padL - padR) / lanes.length;
    return padL + colW * (i + 0.5);
  }

  function markR(ev) {
    if (ev.kind === "vote") return 4.5;
    if (ev.kind === "disclosure") return 5.5;
    const a = ev.amount || 0;
    if (a <= 0) return 4;
    const u = Math.log10(1 + a) / Math.log10(1 + maxAmt);
    return 3.2 + u * 6;
  }

  function clampView() {
    const full = dataTMax - dataTMin;
    let span = viewSpan();
    const minSpan = MS_DAY * 7; // 1 week max zoom-in
    const maxSpan = full;
    span = Math.min(maxSpan, Math.max(minSpan, span));
    if (viewMax - viewMin !== span) {
      const mid = (viewMax + viewMin) / 2;
      viewMax = mid + span / 2;
      viewMin = mid - span / 2;
    }
    if (viewMax > dataTMax) {
      const d = viewMax - dataTMax;
      viewMax -= d;
      viewMin -= d;
    }
    if (viewMin < dataTMin) {
      const d = dataTMin - viewMin;
      viewMax += d;
      viewMin += d;
    }
  }

  function setZoomAround(centerT, factor) {
    const span = viewSpan();
    const next = span * factor;
    const mid = Number.isFinite(centerT) ? centerT : (viewMax + viewMin) / 2;
    viewMax = mid + next / 2;
    viewMin = mid - next / 2;
    clampView();
  }

  function weightsAt(tCenter) {
    const sigmaMs = sigmaWeeks * 7 * MS_DAY;
    return events.map((ev) => ({
      ev,
      w: gaussianWeight(ev.t - tCenter, sigmaMs),
    }));
  }

  function updateFocusPanel() {
    const ranked = weightsAt(cursorT)
      .filter((x) => x.w >= 0.05)
      .sort((a, b) => b.w - a.w || b.ev.t - a.ev.t)
      .slice(0, 16);

    const primary =
      (pinned && pinnedId && events.find((e) => e.id === pinnedId)) ||
      ranked[0]?.ev ||
      null;

    if (focusMain) {
      const pinNote = pinned ? " · pinned" : "";
      if (primary) {
        focusMain.innerHTML = `<span class="muted mono">${esc(fmtDate(cursorT))}${esc(pinNote)}</span><br/>
          <span class="tl-pill tl-pill-${esc(primary.kind)}">${esc(kindLabel(primary.kind))}</span>
          <strong>${esc(primary.date)}</strong> — ${esc(primary.label)}${
            primary.detail
              ? ` <span class="muted">(${esc(primary.detail)})</span>`
              : ""
          }${
            primary.amount_display
              ? ` <span class="mono">${esc(primary.amount_display)}</span>`
              : ""
          }${
            primary.url
              ? ` · <a href="${esc(primary.url)}" rel="noopener noreferrer" target="_blank">source</a>`
              : ""
          }`;
      } else {
        focusMain.innerHTML = `<span class="mono">${esc(fmtDate(cursorT))}</span>${esc(pinNote)} — no nearby events above 5% weight.`;
      }
    }

    if (corrList) {
      corrList.innerHTML = ranked.length
        ? ranked
            .map(({ ev, w }) => {
              const pct = Math.round(w * 100);
              const amt = ev.amount_display
                ? `<span class="mono">${esc(ev.amount_display)}</span> · `
                : "";
              const href = ev.url
                ? `<a href="${esc(ev.url)}" rel="noopener noreferrer" target="_blank">${esc(ev.label)}</a>`
                : esc(ev.label);
              return `<li style="--w:${w.toFixed(3)}" data-corr-id="${esc(ev.id)}">
                <span class="tl-w mono" title="Gaussian weight">${pct}%</span>
                <span class="tl-pill tl-pill-${esc(ev.kind)}">${esc(kindLabel(ev.kind))}</span>
                <span class="muted">${esc(ev.date)}</span>
                <span class="tl-corr-body">${amt}${href}</span>
              </li>`;
            })
            .join("")
        : `<li class="muted">Nothing within ~2σ of the cursor.</li>`;
    }

    if (zoomVal) zoomVal.textContent = `${zoomPct()}%`;
  }

  function renderSvgOrdered() {
    W = Math.max(280, wrap.clientWidth || 480);
    svg.setAttribute("viewBox", `0 0 ${W} ${H}`);
    svg.setAttribute("width", "100%");
    svg.setAttribute("height", String(H));

    const parts = [];
    const colW = (W - padL - padR) / lanes.length;
    const plotTop = padT;
    const plotBot = H - padB;

    for (let i = 0; i < lanes.length; i++) {
      const x = padL + i * colW;
      parts.push(
        `<rect class="tl-col-bg" x="${x}" y="${plotTop}" width="${colW}" height="${
          plotBot - plotTop
        }" fill-opacity="${i % 2 === 0 ? 0.22 : 0.08}" />`
      );
      parts.push(
        `<line class="tl-col-rule" x1="${x}" y1="${plotTop}" x2="${x}" y2="${plotBot}" />`
      );
    }
    parts.push(
      `<line class="tl-col-rule" x1="${padL + lanes.length * colW}" y1="${plotTop}" x2="${
        padL + lanes.length * colW
      }" y2="${plotBot}" />`
    );

    const span = viewSpan();
    const tickMs = niceTimeStep(span / 6);
    const firstTick = Math.ceil(viewMin / tickMs) * tickMs;
    for (let t = firstTick; t <= viewMax + 1; t += tickMs) {
      const y = yOf(t);
      if (y < plotTop - 2 || y > plotBot + 2) continue;
      parts.push(
        `<line class="tl-tick" x1="${padL - 6}" y1="${y}" x2="${W - padR}" y2="${y}" />`
      );
      parts.push(
        `<text class="tl-tick-label" x="${padL - 10}" y="${y}" text-anchor="end" dominant-baseline="middle">${esc(
          tickLabel(t, tickMs)
        )}</text>`
      );
    }

    // scrub under everything interactive
    parts.push(
      `<rect class="tl-scrub" data-tl-scrub x="0" y="${plotTop}" width="${W}" height="${
        plotBot - plotTop
      }" fill="transparent" />`
    );

    const sigmaMs = sigmaWeeks * 7 * MS_DAY;
    const steps = 20;
    for (let i = 0; i < steps; i++) {
      const u0 = -2 + (4 * i) / steps;
      const u1 = -2 + (4 * (i + 1)) / steps;
      const w0 = Math.exp(-0.5 * u0 * u0);
      const w1 = Math.exp(-0.5 * u1 * u1);
      const w = (w0 + w1) / 2;
      const yA = yOf(cursorT + u0 * sigmaMs);
      const yB = yOf(cursorT + u1 * sigmaMs);
      const y = Math.min(yA, yB);
      const h = Math.max(0.5, Math.abs(yB - yA));
      if (y + h < plotTop || y > plotBot) continue;
      const op = (0.04 + 0.26 * w).toFixed(3);
      parts.push(
        `<rect class="tl-gauss" x="${padL}" y="${y}" width="${
          W - padL - padR
        }" height="${h}" fill="var(--glow)" fill-opacity="${op}" pointer-events="none" />`
      );
    }

    const cy = yOf(cursorT);
    parts.push(
      `<line class="tl-cursor" x1="${padL - 4}" y1="${cy}" x2="${W - padR}" y2="${cy}" pointer-events="none" />`
    );
    parts.push(
      `<polygon class="tl-cursor-head" points="${padL - 10},${cy - 5} ${padL - 2},${cy} ${padL - 10},${
        cy + 5
      }" pointer-events="none" />`
    );
    parts.push(
      `<text class="tl-cursor-date" x="${W - padR - 4}" y="${cy - 6}" text-anchor="end" pointer-events="none">${esc(
        fmtDate(cursorT)
      )}${pinned ? " · pin" : ""}</text>`
    );

    const wmap = new Map(weightsAt(cursorT).map(({ ev, w }) => [ev.id, w]));
    const margin = sigmaMs * 0.25;
    for (const ev of events) {
      if (ev.t < viewMin - margin || ev.t > viewMax + margin) continue;
      const x = xOfLane(ev.lane);
      const y = yOf(ev.t);
      if (y < plotTop - 8 || y > plotBot + 8) continue;
      const r = markR(ev);
      const w = wmap.get(ev.id) || 0;
      const active = pinnedId === ev.id;
      const dim = w < 0.05 && !active;
      const glow = Math.min(1, w);
      const op = dim ? 0.2 : 0.5 + 0.5 * (active ? 1 : glow);
      const rr = r * (active ? 1.4 : 1 + 0.3 * glow);
      parts.push(
        `<circle class="tl-mark tl-mark-${esc(ev.kind)}" data-id="${esc(ev.id)}"
          cx="${x}" cy="${y}" r="${rr}"
          fill="${kindColor(ev.kind)}" fill-opacity="${op.toFixed(3)}"
          stroke="var(--bg)" stroke-width="1" />`
      );
    }

    svg.innerHTML = parts.join("");
    if (zoomVal) zoomVal.textContent = `${zoomPct()}%`;
  }

  function svgPoint(clientX, clientY) {
    const rect = svg.getBoundingClientRect();
    const x = ((clientX - rect.left) / Math.max(1, rect.width)) * W;
    const y = ((clientY - rect.top) / Math.max(1, rect.height)) * H;
    return { x, y };
  }

  function moveCursorToClient(clientY, opts = {}) {
    if (pinned && !opts.force) return;
    const { y } = svgPoint(0, clientY);
    cursorT = Math.min(viewMax, Math.max(viewMin, tOfY(y)));
    renderSvgOrdered();
    updateFocusPanel();
  }

  if (host._tlAbort) host._tlAbort.abort();
  const ac = new AbortController();
  host._tlAbort = ac;
  const sig = { signal: ac.signal };

  let dragging = false;
  let dragLastY = 0;
  let panning = false;

  svg.addEventListener(
    "pointerdown",
    (ev) => {
      const mark = ev.target.closest?.(".tl-mark");
      if (mark) {
        const id = mark.getAttribute("data-id");
        const row = events.find((e) => e.id === id);
        if (row) {
          pinned = true;
          pinnedId = row.id;
          cursorT = row.t;
          // If pin is outside view, center on it
          if (cursorT > viewMax || cursorT < viewMin) {
            const span = viewSpan();
            viewMax = cursorT + span / 2;
            viewMin = cursorT - span / 2;
            clampView();
          }
          renderSvgOrdered();
          updateFocusPanel();
        }
        return;
      }
      // pan with shift or middle button; else scrub + unpin
      if (ev.shiftKey || ev.button === 1) {
        panning = true;
        dragLastY = ev.clientY;
        svg.setPointerCapture?.(ev.pointerId);
        return;
      }
      pinned = false;
      pinnedId = null;
      dragging = true;
      svg.setPointerCapture?.(ev.pointerId);
      moveCursorToClient(ev.clientY, { force: true });
    },
    sig
  );

  svg.addEventListener(
    "pointermove",
    (ev) => {
      if (panning) {
        const rect = svg.getBoundingClientRect();
        const dy = ev.clientY - dragLastY;
        dragLastY = ev.clientY;
        const dt = (dy / Math.max(1, rect.height)) * viewSpan();
        // drag down → see older (decrease times)
        viewMax -= dt;
        viewMin -= dt;
        clampView();
        renderSvgOrdered();
        updateFocusPanel();
        return;
      }
      if (dragging) {
        moveCursorToClient(ev.clientY, { force: true });
        return;
      }
      // free scrub when not pinned
      if (!pinned) moveCursorToClient(ev.clientY);
    },
    sig
  );

  const endDrag = (ev) => {
    dragging = false;
    panning = false;
    try {
      svg.releasePointerCapture?.(ev.pointerId);
    } catch {
      /* ignore */
    }
  };
  svg.addEventListener("pointerup", endDrag, sig);
  svg.addEventListener("pointercancel", endDrag, sig);
  svg.addEventListener(
    "pointerleave",
    () => {
      if (!dragging && !panning) {
        /* keep cursor where it was — no collapse */
      }
    },
    sig
  );

  // wheel zoom toward pointer
  svg.addEventListener(
    "wheel",
    (ev) => {
      ev.preventDefault();
      const { y } = svgPoint(ev.clientX, ev.clientY);
      const center = tOfY(y);
      const factor = ev.deltaY > 0 ? 1.18 : 1 / 1.18;
      setZoomAround(center, factor);
      if (!pinned) cursorT = Math.min(viewMax, Math.max(viewMin, cursorT));
      renderSvgOrdered();
      updateFocusPanel();
    },
    { ...sig, passive: false }
  );

  host.querySelectorAll("[data-tl-zoom]").forEach((btn) => {
    btn.addEventListener(
      "click",
      () => {
        const z = btn.getAttribute("data-tl-zoom");
        if (z === "in") setZoomAround(cursorT, 1 / 1.35);
        else if (z === "out") setZoomAround(cursorT, 1.35);
        else if (z === "reset") {
          viewMax = dataTMax;
          viewMin = dataTMin;
        }
        clampView();
        renderSvgOrdered();
        updateFocusPanel();
      },
      sig
    );
  });

  if (sigmaInput) {
    sigmaInput.addEventListener(
      "input",
      () => {
        sigmaWeeks = Number(sigmaInput.value) || DEFAULT_SIGMA_WEEKS;
        if (sigmaVal) sigmaVal.textContent = `${sigmaWeeks}w`;
        renderSvgOrdered();
        updateFocusPanel();
      },
      sig
    );
  }

  // Click nearby list → pin that event
  corrList?.addEventListener(
    "click",
    (ev) => {
      const li = ev.target.closest?.("[data-corr-id]");
      if (!li) return;
      const id = li.getAttribute("data-corr-id");
      const row = events.find((e) => e.id === id);
      if (!row) return;
      pinned = true;
      pinnedId = row.id;
      cursorT = row.t;
      if (cursorT > viewMax || cursorT < viewMin) {
        const span = viewSpan();
        viewMax = cursorT + span / 2;
        viewMin = cursorT - span / 2;
        clampView();
      }
      renderSvgOrdered();
      updateFocusPanel();
    },
    sig
  );

  const ro =
    typeof ResizeObserver !== "undefined"
      ? new ResizeObserver(() => {
          renderSvgOrdered();
        })
      : null;
  if (ro) ro.observe(wrap);
  ac.signal.addEventListener("abort", () => ro?.disconnect());

  renderSvgOrdered();
  updateFocusPanel();
  return { destroy: () => ac.abort() };
}

function niceTimeStep(raw) {
  const day = MS_DAY;
  const candidates = [
    day,
    day * 2,
    day * 7,
    day * 14,
    day * 30,
    day * 60,
    day * 91,
    day * 182,
    day * 365,
    day * 365 * 2,
  ];
  let best = candidates[0];
  for (const c of candidates) {
    best = c;
    if (c >= raw) break;
  }
  return best;
}

function tickLabel(t, stepMs) {
  const d = new Date(t);
  const y = d.getUTCFullYear();
  const mo = d.getUTCMonth() + 1;
  if (stepMs >= MS_DAY * 300) return String(y);
  if (stepMs >= MS_DAY * 40) return `${y}-${String(mo).padStart(2, "0")}`;
  return fmtDate(t);
}
