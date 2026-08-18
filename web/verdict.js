import { apply_voter_profile_js, voter_profile_axes_js } from "./pkg/electionizer_wasm.js";
import { getVoterProfile, setVoterPref, clearVoterProfile } from "./settings.js";

function esc(s) {
  return String(s ?? "")
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

const ICON_DATA = `<svg class="reload-ico" viewBox="0 0 16 16" aria-hidden="true">
  <ellipse cx="8" cy="3.6" rx="5.1" ry="2" fill="none" stroke="currentColor" stroke-width="1.3"/>
  <path d="M2.9 3.6v8.2c0 1.15 2.25 2.05 5.1 2.05s5.1-.9 5.1-2.05V3.6" fill="none" stroke="currentColor" stroke-width="1.3"/>
  <path d="M2.9 7.6c0 1.15 2.25 2.05 5.1 2.05s5.1-.9 5.1-2.05" fill="none" stroke="currentColor" stroke-width="1.3"/>
</svg>`;

const ICON_SCORE = `<svg class="reload-ico" viewBox="0 0 16 16" aria-hidden="true">
  <path d="M3 12.4V8.2h2.2v4.2zM6.9 12.4V4.4h2.2v8zM10.8 12.4V6.6H13v5.8z" fill="currentColor"/>
</svg>`;

export function reloadButtonsHtml(scope, id) {
  const target =
    scope === "ballot"
      ? "the whole ballot"
      : scope === "role"
        ? "this race"
        : scope === "measure"
          ? "this measure"
          : "this candidate";
  const dataTip = `Refresh data for ${target}: refetch filings, bios, votes, and finance.`;
  const scoreTip = `Refresh score for ${target}: rerun the AI verdict and voter-fit.`;
  return `<span class="reload-btns" data-reload-scope="${esc(scope)}" data-reload-id="${esc(id ?? "")}">
    <span class="reload-label">refresh:</span>
    <button type="button" class="reload-btn" data-reload="scrape" data-tip="${esc(dataTip)}" aria-label="${esc(dataTip)}">${ICON_DATA}</button>
    <button type="button" class="reload-btn" data-reload="ai" data-tip="${esc(scoreTip)}" aria-label="${esc(scoreTip)}">${ICON_SCORE}</button>
  </span>`;
}

/** Cividis-like stops — blue (poor) → gold (good). Safe for deuteranopia/protanopia. */
const FIT_STOPS = [
  [0, 0, 32, 76],
  [25, 48, 75, 107],
  [50, 124, 123, 120],
  [75, 190, 163, 82],
  [100, 253, 231, 37],
];

function lerp(a, b, t) {
  return a + (b - a) * t;
}

export function fitRgb(n) {
  const x = Math.max(0, Math.min(100, Number(n)));
  if (!Number.isFinite(x)) return [124, 123, 120];
  let i = 0;
  while (i < FIT_STOPS.length - 2 && x > FIT_STOPS[i + 1][0]) i += 1;
  const a = FIT_STOPS[i];
  const b = FIT_STOPS[i + 1];
  const t = (x - a[0]) / Math.max(1, b[0] - a[0]);
  return [lerp(a[1], b[1], t), lerp(a[2], b[2], t), lerp(a[3], b[3], t)].map((v) =>
    Math.round(v)
  );
}

export function fitCss(n) {
  const [r, g, b] = fitRgb(n);
  return `rgb(${r}, ${g}, ${b})`;
}

function srgbLum(r, g, b) {
  const lin = (c) => {
    const x = c / 255;
    return x <= 0.04045 ? x / 12.92 : ((x + 0.055) / 1.055) ** 2.4;
  };
  return 0.2126 * lin(r) + 0.7152 * lin(g) + 0.0722 * lin(b);
}

function contrastRatio(l1, l2) {
  const a = Math.max(l1, l2);
  const b = Math.min(l1, l2);
  return (a + 0.05) / (b + 0.05);
}

/** Ink that contrasts with the heatmap fill (chips). */
export function fitTextCss(n) {
  const [r, g, b] = fitRgb(n);
  const bg = srgbLum(r, g, b);
  const darkL = srgbLum(11, 18, 32);
  const lightL = srgbLum(244, 241, 232);
  return contrastRatio(bg, darkL) >= contrastRatio(bg, lightL) ? "#0b1220" : "#f4f1e8";
}

function unwrapJs(raw) {
  if (raw == null || raw === "") return null;
  let v = raw;
  if (typeof raw === "string") {
    try {
      v = JSON.parse(raw);
    } catch {
      return null;
    }
  }
  if (v instanceof Map) {
    try {
      v = Object.fromEntries(v);
    } catch {
      return null;
    }
  }
  return v;
}

export function applyVoterFit(card) {
  if (!card) return card;
  const profile = getVoterProfile();
  if (!Object.keys(profile).length) return card;
  try {
    const out = apply_voter_profile_js(JSON.stringify(card), JSON.stringify(profile));
    return unwrapJs(out) || card;
  } catch {
    return card;
  }
}

function profileAxisList() {
  try {
    const raw = voter_profile_axes_js();
    const v = unwrapJs(raw);
    if (Array.isArray(v)) return v;
    if (v && typeof v === "object") return Object.values(v);
  } catch {
    /* wasm not ready */
  }
  return [];
}

function profileSummaryText(prefs) {
  const n = Object.keys(prefs || {}).length;
  if (!n) return `Voter profile <span class="muted">(optional · 1–5 · this browser)</span>`;
  return `Voter profile <span class="muted">· ${n} rated · this browser</span>`;
}

export function mountVoterProfile(root, opts = {}) {
  const host = (root || document).querySelector("#voter-profile-axes");
  const box = (root || document).querySelector("#voter-profile");
  const sum = (root || document).querySelector("#voter-profile-summary");
  if (!host) return;
  const axes = profileAxisList();
  if (!axes.length) return;
  const prefs = getVoterProfile();
  const groups = [];
  for (const a of axes) {
    const g = a.group || "Other";
    let bucket = groups.find((x) => x.name === g);
    if (!bucket) {
      bucket = { name: g, rows: [] };
      groups.push(bucket);
    }
    bucket.rows.push(a);
  }
  host.innerHTML = groups
    .map((g) => {
      const rows = g.rows
        .map((a) => {
          const cur = prefs[a.id];
          const radios = [1, 2, 3, 4, 5]
            .map(
              (n) =>
                `<label class="likert-n"><input type="radio" name="vp-${esc(a.id)}" value="${n}"${
                  cur === n ? " checked" : ""
                }><span>${n}</span></label>`
            )
            .join("");
          const poles = `<span class="likert-poles muted">${esc(a.low_label || "Disagree")} · ${esc(
            a.high_label || "Agree"
          )}</span>`;
          return `<div class="likert-row" data-axis="${esc(a.id)}" title="${esc(a.definition || "")}">
            <span class="likert-label">${esc(a.label)}</span>
            ${radios}
            <button type="button" class="likert-clear" data-clear="${esc(a.id)}" aria-label="Clear ${esc(
              a.label
            )}">×</button>
            ${poles}
          </div>`;
        })
        .join("");
      return `<div class="voter-profile-group"><h3>${esc(g.name)}</h3>${rows}</div>`;
    })
    .join("");
  if (sum) sum.innerHTML = profileSummaryText(prefs);
  if (box && Object.keys(prefs).length) box.open = true;

  const onChange = () => {
    if (typeof opts.onChange === "function") opts.onChange(getVoterProfile());
    const s = (root || document).querySelector("#voter-profile-summary");
    if (s) s.innerHTML = profileSummaryText(getVoterProfile());
    const st = (root || document).querySelector("#voter-profile-status");
    if (st) st.textContent = "Saved in this browser.";
  };

  host.onchange = (ev) => {
    const input = ev.target && ev.target.closest && ev.target.closest("input[type=radio]");
    if (!input) return;
    const id = String(input.name || "").replace(/^vp-/, "");
    setVoterPref(id, input.value);
    onChange();
  };
  host.onclick = (ev) => {
    const btn = ev.target && ev.target.closest && ev.target.closest("[data-clear]");
    if (!btn) return;
    ev.preventDefault();
    const id = btn.getAttribute("data-clear");
    setVoterPref(id, 0);
    host.querySelectorAll(`input[name="vp-${id}"]`).forEach((el) => {
      el.checked = false;
    });
    onChange();
  };
  const wipe = (root || document).querySelector("#voter-profile-clear");
  if (wipe) {
    wipe.onclick = () => {
      clearVoterProfile();
      host.querySelectorAll("input[type=radio]").forEach((el) => {
        el.checked = false;
      });
      onChange();
    };
  }
}

function partyChip(party, partyClass) {
  const cls = partyClass || "o";
  return `<span class="party-chip party-${esc(cls)}">${esc(party || "")}</span>`;
}

function subjectTitle(s) {
  if (!s) return "Subject";
  if (s.kind === "measure") {
    const code = s.measure_code ? `${s.measure_code}: ` : "";
    return `${code}${s.title || s.name || "Ballot measure"}`;
  }
  return s.name || "Candidate";
}

export function renderVerdictShell(subject, opts = {}) {
  const s = subject || {};
  const title = subjectTitle(s);
  const isMeasure = s.kind === "measure";
  const line = isMeasure
    ? [s.jurisdiction, s.election_name, s.election_date].filter(Boolean).join(" · ")
    : `${partyChip(s.party, s.party_class)} · ${esc(s.office || "")} · ${esc(s.jurisdiction || "")}`;
  const extra = isMeasure
    ? ""
    : `<p class="meta muted detail-header-line">${esc(s.election_name || "")}${
        s.election_date ? ` · ${esc(s.election_date)}` : ""
      }${s.is_incumbent ? " · Incumbent" : ""}${s.is_judge ? " · Judicial" : ""}</p>`;
  const noKey = !!opts.noKey;
  const noWisp = !!opts.noWisp;
  let cta = "";
  if (noKey) {
    cta = `<p class="warn-banner" id="verdict-cta">Add an xAI or OpenAI key in <a href="#settings" id="verdict-settings">Settings</a> to generate a scored verdict. Details still work.</p>`;
  } else if (noWisp) {
    cta = `<p class="warn-banner" id="verdict-cta">Wisp is required — API hosts block CORS. Set a Wisp URL in <a href="#settings" id="verdict-settings">Settings</a>.</p>`;
  }
  return `
    <article class="verdict-page" id="verdict-page" data-kind="${esc(s.kind || "candidate")}" data-subject-key="${esc(
      s.subject_key || (isMeasure ? `m:${s.id ?? ""}` : `c:${s.id ?? ""}`)
    )}">
      <header class="detail-header verdict-header">
        <p class="crumb"><a href="#" id="back-ballot">← Ballot</a></p>
        <h1 class="detail-name">${esc(title)}</h1>
        <p class="meta detail-header-line">${line}</p>
        ${extra}
        <p class="verdict-actions">
          <button type="button" class="verdict-details-btn" id="verdict-details">Details →</button>
          ${reloadButtonsHtml(isMeasure ? "measure" : "candidate", s.id)}
        </p>
        ${cta}
      </header>
      <section class="card verdict-card" id="verdict-card">
        <p class="empty muted" id="verdict-status">${
          noKey
            ? "No LLM key — open Details for filings, or add a key in Settings."
            : "Gathering sources and writing a verdict…"
        }</p>
      </section>
    </article>`;
}

function citeChip(c) {
  if (!c) return "";
  const label = c.label || c.tab || "source";
  if (c.url) {
    return `<a class="verdict-cite" href="${esc(c.url)}" target="_blank" rel="noopener noreferrer">${esc(label)}</a>`;
  }
  if (c.tab) {
    return `<button type="button" class="verdict-cite verdict-tab-cite" data-tab="${esc(c.tab)}">${esc(label)}</button>`;
  }
  return `<span class="verdict-cite">${esc(label)}</span>`;
}

function citeList(cites) {
  const rows = (cites || []).map(citeChip).filter(Boolean);
  if (!rows.length) return "";
  return `<span class="verdict-cites">${rows.join(" ")}</span>`;
}

function scoreBar(score, signed, opts = {}) {
  if (score == null || score === "") {
    return `<div class="verdict-bar-wrap"><div class="verdict-bar verdict-bar-empty"></div><span class="muted">—</span></div>`;
  }
  const n = Number(score);
  if (!Number.isFinite(n)) {
    return `<div class="verdict-bar-wrap"><div class="verdict-bar verdict-bar-empty"></div><span class="muted">—</span></div>`;
  }
  if (opts.fit) {
    const pct = Math.max(0, Math.min(100, n));
    const color = fitCss(pct);
    return `<div class="verdict-bar-wrap verdict-bar-fit" style="--fit:${color}"><div class="verdict-bar verdict-bar-heat" style="width:${pct}%;background:${color}"></div><span class="verdict-score-n">${esc(
      String(n)
    )}</span></div>`;
  }
  if (signed) {
    const pct = Math.min(100, Math.abs(n));
    const side = n < 0 ? "neg" : n > 0 ? "pos" : "zero";
    return `<div class="verdict-bar-wrap verdict-bar-signed"><div class="verdict-bar verdict-bar-${side}" style="width:${pct}%"></div><span class="verdict-score-n">${esc(String(n))}</span></div>`;
  }
  const pct = Math.max(0, Math.min(100, n));
  return `<div class="verdict-bar-wrap"><div class="verdict-bar" style="width:${pct}%"></div><span class="verdict-score-n">${esc(String(n))}</span></div>`;
}

function sentenceHtml(s) {
  if (!s || !s.text) return "";
  return `<p class="verdict-sentence">${esc(s.text)} ${citeList(s.cites)}</p>`;
}

export function renderVerdictCard(card, opts = {}) {
  if (!card) {
    return `<p class="muted empty">${esc(opts.empty || "No verdict yet.")}</p>`;
  }
  const fitted = opts.skipFit ? card : applyVoterFit(card);
  const signedIds = new Set(
    profileAxisList()
      .filter((a) => a.signed)
      .map((a) => a.id)
      .concat(["tax_direction", "restriction_direction", "abortion", "health_insurance", "h1b", "border"])
  );
  const overall = fitted.overall instanceof Map ? Object.fromEntries(fitted.overall) : fitted.overall || {};
  const profiled = !!(overall.profiled || opts.profiled);
  const axes = Array.isArray(fitted.axes) ? fitted.axes : [];
  const found = Array.isArray(fitted.found) ? fitted.found : [];
  const tabs = Array.isArray(fitted.tab_cites) ? fitted.tab_cites : [];
  const summary = (Array.isArray(fitted.summary) ? fitted.summary : []).map(sentenceHtml).join("");
  const axisHtml = axes.length
    ? `<ul class="verdict-axes">${axes
        .map((a) => {
          const ev = (a.evidence || []).map(sentenceHtml).join("");
          const hasFit = profiled && a.raw_score != null && a.score != null;
          const rawBit =
            hasFit && a.raw_score !== a.score
              ? `<span class="muted verdict-raw">raw ${esc(String(a.raw_score))}${
                  a.inverted ? " · inverted" : ""
                }</span>`
              : "";
          return `<li>
            <div class="verdict-axis-head">
              <strong>${esc(a.label || a.id)}</strong>
              ${scoreBar(a.score, signedIds.has(a.id) && !hasFit, { fit: hasFit || (profiled && a.score != null && !signedIds.has(a.id)) })}
            </div>
            ${rawBit}
            ${a.verdict ? `<p class="verdict-axis-verdict">${esc(a.verdict)}</p>` : ""}
            ${ev}
          </li>`;
        })
        .join("")}</ul>`
    : `<p class="muted empty">No scored axes (unsourced scores are dropped).</p>`;
  const foundHtml = found.length
    ? `<ul class="verdict-found">${found
        .map((f) => {
          const label = f.org || f.text || f.kind;
          const href = f.url
            ? `<a href="${esc(f.url)}" target="_blank" rel="noopener noreferrer">${esc(label)}</a>`
            : esc(label);
          const stance = f.stance
            ? `<span class="endorsement-stance endorsement-${esc(f.stance)}">${esc(f.stance)}</span> `
            : "";
          return `<li>${stance}${href} <span class="muted">· ${esc(f.kind || "")}${
            f.trust ? ` · ${esc(f.trust)}` : ""
          }</span></li>`;
        })
        .join("")}</ul>`
    : `<p class="muted empty">No extra live-search finds yet.</p>`;
  const tabHtml = tabs.length
    ? `<p class="verdict-tabs">${tabs
        .map(
          (t) =>
            `<button type="button" class="verdict-tab-cite" data-tab="${esc(t.tab)}">${esc(
              t.label || t.tab
            )}</button>`
        )
        .join(" ")}</p>`
    : "";
  const tools = (fitted.tools || []).length ? fitted.tools.join(" + ") : "packed filings";
  const meta = [
    fitted.provider,
    fitted.model,
    tools,
    fitted.generated_at,
  ]
    .filter(Boolean)
    .map(esc)
    .join(" · ");
  const rawOverall =
    profiled && overall.raw_score != null && overall.raw_score !== overall.score
      ? `<span class="muted verdict-raw">alignment ${esc(String(overall.raw_score))}</span>`
      : "";
  const kicker = profiled
    ? "Your fit · remapped from profile · cited"
    : "AI verdict · scored + cited";
  const legend = profiled
    ? `<div class="fit-legend-row"><span class="muted">Poor fit</span><span class="fit-legend" aria-hidden="true"></span><span class="muted">Good fit</span></div>`
    : "";
  return `
    <p class="verdict-kicker muted">${kicker}</p>
    ${fitted.headline ? `<h2 class="verdict-headline">${esc(fitted.headline)}</h2>` : ""}
    <div class="verdict-overall"${profiled ? ` data-fit="${esc(String(overall.score ?? ""))}"` : ""}>
      ${scoreBar(overall.score, false, { fit: profiled && overall.score != null })}
      ${overall.label ? `<strong>${esc(overall.label)}</strong>` : ""}
      ${rawOverall}
      ${legend}
      ${overall.verdict ? `<p>${esc(overall.verdict)}</p>` : ""}
    </div>
    <div class="verdict-summary">${summary || `<p class="muted empty">No cited summary.</p>`}</div>
    <h3>${profiled ? "Fit vs your profile" : "Fit scores"}</h3>
    ${axisHtml}
    <h3>Found at call time</h3>
    ${foundHtml}
    <h3>From this report</h3>
    ${tabHtml || `<p class="muted empty">Open Details for filings.</p>`}
    <p class="meta muted verdict-model">${meta}</p>`;
}

export function mergeFoundEndorsements(existing, extra) {
  const list = Array.isArray(existing) ? existing.slice() : [];
  const seen = new Set(list.map((e) => `${e.stance}|${e.org}`.toLowerCase()));
  for (const r of extra || []) {
    if (!r || !r.org) continue;
    const k = `${r.stance}|${r.org}`.toLowerCase();
    if (seen.has(k)) continue;
    seen.add(k);
    list.push(r);
  }
  return list;
}
