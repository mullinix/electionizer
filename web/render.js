import {
  filterVotes,
  filterContributors,
  mergeContributorLists,
  pageSlice,
  voteYears,
  votePositions,
} from "./detail-lists.js";
import { mountTimeline, renderTimelineBody } from "./timeline.js";
import { sortCandidatesByFit, sortMeasuresByFit, renderFitChip, scoreKey } from "./scoreboard.js";
import { reloadButtonsHtml } from "./verdict.js";

function esc(s) {
  if (s == null) return "";
  return String(s)
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

export function candidateRows(candidates, opts = {}) {
  const clickable = opts.clickable !== false;
  const scores = opts.scores;
  if (!candidates || !candidates.length) {
    return `<p class="empty">No candidates found.</p>`;
  }
  const list = sortCandidatesByFit(candidates, scores);
  const body = list
    .map((c) => {
      const notes = [
        c.is_judge ? "Judge" : "",
        c.is_incumbent ? "Incumbent" : "",
      ]
        .filter(Boolean)
        .join(" · ");
      const nameCell = clickable
        ? `<a href="#" data-cand-id="${esc(c.id)}">${esc(c.name)}</a>`
        : esc(c.name);
      const rec = scores ? scores.get(scoreKey("candidate", c.id)) : null;
      return `<tr data-cand-id="${esc(c.id)}">
        <td>${nameCell}</td>
        <td><span class="party-chip party-${esc(c.party_class)}">${esc(c.party)}</span></td>
        <td>${esc(notes)}</td>
         <td class="fit-cell">${renderFitChip(rec)}${reloadButtonsHtml("candidate", c.id)}</td>
       </tr>`;
    })
    .join("");
  return `<table>
    <thead><tr><th>Name</th><th>Party</th><th>Notes</th><th>Fit</th></tr></thead>
    <tbody>${body}</tbody>
  </table>`;
}

export function renderOfficeGroup(g, scores) {
  const emptyClass = g.empty_message ? " office-empty" : "";
  const roleKey = `o:${g.office || "Office"}:${g.jurisdiction || ""}`;
  let body;
  if (g.empty_message) {
    body = `<p class="empty">${esc(g.empty_message)}</p>`;
  } else if (g.candidates && g.candidates.length) {
    body = candidateRows(g.candidates, { clickable: true, scores });
  } else {
    body = `<p class="empty">No candidates found.</p>`;
  }
  return `<section class="card office-card${emptyClass}" data-role-key="${esc(roleKey)}">
    <header class="office-head">
      <div class="office-head-text">
        <h2>${esc(g.office)}</h2>
        ${g.jurisdiction ? `<p class="office-jurisdiction">${esc(g.jurisdiction)}</p>` : ""}
      </div>
      ${reloadButtonsHtml("role", roleKey)}
    </header>
    ${body}
  </section>`;
}

function unopposedJudgeRows(seats, scores) {
  const rows = [];
  const flat = [];
  for (const g of seats) {
    for (const c of g.candidates || []) {
      flat.push({ g, c });
    }
  }
  flat.sort((a, b) =>
    compareFitSafe(scores, a.c.id, b.c.id)
  );
  for (const { g, c } of flat) {
    const notes = [c.is_judge ? "Judge" : "", c.is_incumbent ? "Incumbent" : "", "Unopposed"]
      .filter(Boolean)
      .join(" · ");
    const nameCell = `<a href="#" data-cand-id="${esc(c.id)}">${esc(c.name)}</a>`;
    const rec = scores ? scores.get(scoreKey("candidate", c.id)) : null;
    rows.push(`<tr data-cand-id="${esc(c.id)}">
      <td>${esc(g.office)}</td>
      <td>${nameCell}</td>
      <td><span class="party-chip party-${esc(c.party_class)}">${esc(c.party)}</span></td>
      <td>${esc(notes)}</td>
      <td class="fit-cell">${renderFitChip(rec)}${reloadButtonsHtml("candidate", c.id)}</td>
    </tr>`);
  }
  if (!rows.length) return `<p class="empty">No unopposed seats.</p>`;
  return `<table class="judicial-unopposed-table">
    <thead><tr><th>Seat</th><th>Name</th><th>Party</th><th>Notes</th><th>Fit</th></tr></thead>
    <tbody>${rows.join("")}</tbody>
  </table>`;
}

function compareFitSafe(scores, aId, bId) {
  if (!scores) return 0;
  const a = scores.get(scoreKey("candidate", aId));
  const b = scores.get(scoreKey("candidate", bId));
  const an = a && a.fit;
  const bn = b && b.fit;
  const aN = an == null;
  const bN = bn == null;
  if (aN && bN) return 0;
  if (aN) return 1;
  if (bN) return -1;
  return bn - an;
}

export function renderJudicialBlock(sec, scores) {
  const seats = sec.seats || [];
  const onBallot = [];
  const unopposed = [];
  for (const g of seats) {
    if (g.default_open === false) unopposed.push(g);
    else onBallot.push(g);
  }

  const seatHtml = onBallot
    .map((g) => {
      const n = (g.candidates || []).length;
      const emptyClass = g.empty_message ? " office-empty" : "";
      let count = "";
      if (n > 1) count = `${n} candidates · contested`;
      else if (n === 1) count = "retention / single name";
      let body;
      if (g.empty_message) body = `<p class="empty">${esc(g.empty_message)}</p>`;
      else body = candidateRows(g.candidates, { clickable: true, scores });
      return `<details class="judicial-seat${emptyClass}" open>
        <summary>
          <span class="judicial-seat-title">${esc(g.office)}</span>
          ${g.jurisdiction ? `<span class="office-jurisdiction">${esc(g.jurisdiction)}</span>` : ""}
          ${count ? `<span class="judicial-seat-count">${esc(count)}</span>` : ""}
          ${reloadButtonsHtml("role", `j:${g.office}`)}
        </summary>
        ${body}
      </details>`;
    })
    .join("");

  let unopposedHtml = "";
  if (unopposed.length) {
    const nPeople = unopposed.reduce((a, g) => a + (g.candidates || []).length, 0);
    unopposedHtml = `<details class="judicial-seat judicial-seat-unopposed">
      <summary>
        <span class="judicial-seat-title">Unopposed (takes bench)</span>
        <span class="judicial-seat-count">${unopposed.length} seat${unopposed.length !== 1 ? "s" : ""} · ${nPeople} judge${nPeople !== 1 ? "s" : ""}</span>
      </summary>
      ${unopposedJudgeRows(unopposed, scores)}
    </details>`;
  }

  let sub = `${seats.length} seat${seats.length !== 1 ? "s" : ""}`;
  if (unopposed.length && onBallot.length) {
    sub = `${onBallot.length} on ballot · ${unopposed.length} unopposed`;
  } else if (unopposed.length) {
    sub = `${unopposed.length} unopposed`;
  }

  return `<section class="card judicial-block" data-role-key="judicial">
    <header class="office-head">
      <div class="office-head-text">
        <h2>Judicial</h2>
        <p class="office-jurisdiction">${esc(sub)}</p>
      </div>
      ${reloadButtonsHtml("role", "judicial")}
    </header>
    ${sec.explainer ? `<p class="judicial-explainer">${esc(sec.explainer)}</p>` : ""}
    <div class="judicial-seats">${seatHtml}${unopposedHtml}</div>
  </section>`;
}

export function renderBallotReport(report, opts = {}) {
  const scores = opts.scores || null;
  const sections = report.ballot_sections || [];
  let officesHtml;
  if (sections.length) {
    officesHtml = `<div class="office-list">${sections
      .map((sec) => {
        if (sec.kind === "judicial") return renderJudicialBlock(sec, scores);
        if (sec.group) return renderOfficeGroup(sec.group, scores);
        return "";
      })
      .join("")}</div>`;
  } else {
    officesHtml = `<section class="card"><p class="empty">No candidates found for this ZIP.</p></section>`;
  }

  const measures = sortMeasuresByFit(report.measures || [], scores);
  let measuresHtml;
  if (measures.length) {
    const rows = measures
      .map((m) => {
        const label = m.measure_code
          ? `${esc(m.measure_code)}: ${esc(m.title)}`
          : esc(m.title);
        const title = `<a href="#" data-measure-id="${esc(String(m.id))}">${label}</a>`;
        const sum = m.summary
          ? esc(m.summary.length > 280 ? m.summary.slice(0, 277) + "…" : m.summary)
          : "—";
        const cells = measureFinanceTableCells(m.finance);
        const rec = scores ? scores.get(scoreKey("measure", m.id)) : null;
        return `<tr data-measure-id="${esc(String(m.id))}">
          <td>${title}</td>
          <td>${esc(m.jurisdiction)}</td>
          <td>${sum}</td>
          <td>${cells.sponsor}</td>
          <td>${cells.oppose}</td>
          <td class="fit-cell">${renderFitChip(rec)}${reloadButtonsHtml("measure", m.id)}</td>
        </tr>`;
      })
      .join("");
    measuresHtml = `<table class="measures-table">
      <thead><tr>
        <th>Measure</th><th>Jurisdiction</th><th>Summary</th>
        <th>Support / backers</th><th>Oppose / backers</th><th>Fit</th>
      </tr></thead>
      <tbody>${rows}</tbody>
    </table>
    <p class="meta muted">Backers are public campaign committees and itemized donors (cited TreFin / MDCRIS / FTM) — not a complete endorsement list. Click a measure for full detail.</p>`;
  } else {
    measuresHtml = `<p class="empty">No ballot measures found.</p>`;
  }

  let portal = "";
  if (report.voter_portal) {
    portal = `
      <p class="section-label">// Your registration //</p>
      <section class="card muted-card">
        <h2>Check voter registration</h2>
        <p class="meta">Confirm your status with the official portal for this ballot’s state.</p>
        <p><a href="${esc(report.voter_portal.url)}" rel="noopener noreferrer" target="_blank">${esc(report.voter_portal.label)} →</a></p>
      </section>`;
  }

  const notices = (report.client_warnings || []).filter(Boolean);
  const isWarnNotice = (w) =>
    /failed|rejected|error|skipped:|rate limit|no voterinfo|no state\/local|seasonal|check key|unavailable/i.test(
      String(w)
    ) && !/VIP ballot:/i.test(String(w));
  const warns = notices.filter(isWarnNotice);
  const infos = notices.filter((w) => !isWarnNotice(w));
  const noticeBlock = (cls, label, items) =>
    items.length
      ? `<div class="${cls}" role="status">
        <p class="meta" style="margin:0 0 0.35rem;letter-spacing:0.12em;text-transform:uppercase;font-size:0.7rem">${label}</p>
        ${items.map((w) => `<p style="margin:0.35rem 0">${esc(w)}</p>`).join("")}
      </div>`
      : "";
  const warnHtml =
    noticeBlock("info-banner", "// source //", infos) +
    noticeBlock("warn-banner", "// notice //", warns);

  return `
    <header class="ballot-head">
      <h1>Ballot for ${esc(report.zip)}</h1>
      ${reloadButtonsHtml("ballot", "all")}
    </header>
    ${report.geo_summary ? `<p class="meta">${esc(report.geo_summary)}</p>` : ""}
    ${
      report.election_name
        ? `<p class="meta">${esc(report.election_name)}${
            report.election_date ? ` · ${esc(report.election_date)}` : ""
          }</p>`
        : ""
    }
    ${report.coverage_note ? `<p class="badge">${esc(report.coverage_note)}</p>` : ""}
    ${warnHtml}
    ${sampleBallotBanner(opts.sampleBallot)}
    <p class="section-label">// Candidates //</p>
    ${officesHtml}
    <p class="section-label">// Measures //</p>
    <section class="card" data-role-key="measures">
      <header class="office-head">
        <div class="office-head-text"><h2>Ballot measures</h2></div>
        ${reloadButtonsHtml("role", "measures")}
      </header>
      ${measuresHtml}
    </section>
    ${portal}`;
}

function sampleBallotBanner(ref) {
  if (!ref || !ref.url) {
    return `<section class="card muted-card">
      <h2>Official sample ballot</h2>
      <p class="meta">Set <strong>precinct + party</strong> on the home form (or Settings) for the VoteBrevard / VoterFocus sample ballot. ZIP centroid is not precinct-accurate.</p>
      <p class="meta muted"><a href="https://www.votebrevard.gov/Election-Information/2026-Primary-Sample-Ballots" rel="noopener noreferrer" target="_blank">2026 primary sample ballots →</a></p>
    </section>`;
  }
  return `<section class="card">
    <h2>Your official sample ballot</h2>
    <p class="meta">Brevard precinct <strong>${esc(ref.precinct)}</strong> · ${esc(ref.party)} primary
      ${ref.election_id ? ` · election ${esc(ref.election_id)}` : ""}</p>
    <p><a href="${esc(ref.url)}" rel="noopener noreferrer" target="_blank">Open VoteBrevard / VoterFocus sample ballot (PDF) →</a></p>
    <p class="meta muted">${esc(ref.note || "Official PDF. Florida closed primary: your party races plus every nonpartisan contest.")}</p>
  </section>`;
}

function partyChip(party, partyClass) {
  const cls = partyClass || "o";
  return `<span class="party-chip party-${esc(cls)}">${esc(party || "")}</span>`;
}

function isFecCandidateId(ext) {
  const id = (ext || "").trim();
  return /^[HSP][A-Z0-9]{8,}$/i.test(id);
}

/** Human label for external_id (FEC vs namespaced state ids). */
function externalIdMeta(ext) {
  const id = (ext || "").trim();
  if (!id) return "";
  if (isFecCandidateId(id)) return ` · FEC ${esc(id)}`;
  if (id.startsWith("openstates:")) {
    return ` · Open States ${esc(id.slice("openstates:".length))}`;
  }
  if (id.startsWith("azleg:")) return ` · AZ Leg ${esc(id.slice(6))}`;
  if (id.startsWith("fl:acct:")) return ` · FL DOS #${esc(id.slice("fl:acct:".length))}`;
  if (id.startsWith("fl:")) return ` · FL DOS ${esc(id.slice(3))}`;
  return ` · id ${esc(id)}`;
}

function fecProfileLink(ext) {
  if (!isFecCandidateId(ext)) return "";
  return `<p class="meta"><a href="https://www.fec.gov/data/candidate/${esc(ext.trim())}/" rel="noopener noreferrer" target="_blank">Open FEC profile →</a></p>`;
}

function sourceLinkHtml(c, e) {
  const url = (e && e.profile_url) || c.source_url || c.votes_url;
  if (!url || !/^https?:\/\//i.test(url)) return "";
  return `<p class="meta"><a href="${esc(url)}" rel="noopener noreferrer" target="_blank">Source profile →</a></p>`;
}

function headerLinkBits(c) {
  const bits = [];
  if (c.source_url && /^https?:\/\//i.test(c.source_url)) {
    bits.push(
      `<a href="${esc(c.source_url)}" rel="noopener noreferrer" target="_blank">Source profile</a>`
    );
  }
  if (isFecCandidateId(c.external_id)) {
    bits.push(
      `<a href="https://www.fec.gov/data/candidate/${esc(c.external_id.trim())}/" rel="noopener noreferrer" target="_blank">FEC</a>`
    );
  }
  if (c.votes_url && /^https?:\/\//i.test(c.votes_url)) {
    bits.push(
      `<a href="${esc(c.votes_url)}" rel="noopener noreferrer" target="_blank">Votes profile</a>`
    );
  }
  return bits.length
    ? `<p class="detail-header-links meta">${bits.join(" · ")}</p>`
    : "";
}

/** Compact header teaser from loaded dossier only (cite-or-omit; never invent). */
export function renderDetailHeaderTeaser(enrich, name = "") {
  const d = enrich && enrich.dossier;
  if (!d) {
    return { photoHtml: "", teaserHtml: "" };
  }
  const alt = name ? `Photo of ${name}` : "Candidate photo";
  const photoHtml = d.photo_url
    ? `<img src="${esc(d.photo_url)}" alt="${esc(alt)}" width="72" height="90" loading="lazy" referrerpolicy="no-referrer" />`
    : "";
  const parts = [];
  const career = d.career || {};
  if (career.birth_year != null) parts.push(`Born ${career.birth_year}`);
  const fam = d.family_summary || {};
  if (fam.disclosed && fam.display) parts.push(fam.display);
  const facts = d.facts || [];
  const work = facts.find((f) =>
    ["work", "business", "legal"].includes(f.kind)
  );
  if (work && work.text) {
    const t = String(work.text).replace(/^[^:]+:\s*/, "").trim();
    if (t) parts.push(t.length > 72 ? t.slice(0, 69) + "…" : t);
  }
  const teaserHtml = parts.length
    ? `<p class="detail-header-teaser muted">${esc(parts.join(" · "))}</p>`
    : "";
  return { photoHtml, teaserHtml };
}

export const DETAIL_TAB_IDS = [
  "dossier",
  "scrutiny",
  "votes",
  "finance",
  "personal",
  "timeline",
  "party",
  "more",
];

export function renderDetailShell(c, stages, opts = {}) {
  const stageLis = (stages || [])
    .map(
      (s) => `<li data-stage-id="${esc(s.id)}" class="stage-pending">
      <span class="stage-mark" aria-hidden="true">·</span>
      <span class="stage-label">${esc(s.label)}</span>
      <span class="stage-detail muted"></span>
    </li>`
    )
    .join("");

  const tabs = [
    { id: "dossier", label: "Dossier" },
    { id: "scrutiny", label: "Scrutiny" },
    { id: "votes", label: c.is_judge ? "Decisions" : "Votes" },
    { id: "finance", label: "Finance" },
    { id: "personal", label: "Personal finance" },
    { id: "timeline", label: "Timeline" },
    { id: "party", label: "Party" },
    { id: "more", label: "More" },
  ];
  const activeTab = DETAIL_TAB_IDS.includes(opts.activeTab)
    ? opts.activeTab
    : "dossier";
  const tabBtns = tabs
    .map((t) => {
      const on = t.id === activeTab;
      return `<button type="button" role="tab" id="tab-${t.id}"
          class="detail-tab${on ? " is-active" : ""}"
          aria-controls="panel-${t.id}"
          aria-selected="${on ? "true" : "false"}"
          tabindex="${on ? "0" : "-1"}"
          data-tab="${t.id}">${esc(t.label)}</button>`;
    })
    .join("");

  const panel = (id, inner) => {
    const on = id === activeTab;
    return `<div role="tabpanel" id="panel-${id}" class="detail-panel"
          aria-labelledby="tab-${id}" data-tab-panel="${id}"
          ${on ? "" : "hidden"}
          tabindex="0">${inner}</div>`;
  };

  return `
    <div id="detail-shell">
      <header class="detail-header" id="detail-header">
        <p class="crumb"><a href="#" id="back-ballot">← Back to ballot</a></p>
        <div class="detail-header-main">
          <div class="detail-header-photo" id="detail-header-photo" hidden></div>
          <div class="detail-header-body">
            <h1 class="detail-name" id="detail-name">${esc(c.name)}</h1>
            <p class="meta detail-header-line">${partyChip(c.party, c.party_class)} · ${esc(c.office)} · ${esc(c.jurisdiction)}</p>
            <p class="meta muted detail-header-line">${esc(c.election_name || "")}${
              c.election_date ? ` · ${esc(c.election_date)}` : ""
            }${c.is_incumbent ? " · Incumbent" : ""}${c.is_judge ? " · Judicial" : ""}${
              externalIdMeta(c.external_id)
            }</p>
            <p class="detail-header-summary">${esc(c.summary || "No summary available yet.")}</p>
            <div id="detail-header-teaser"></div>
            ${headerLinkBits(c)}
            <p class="verdict-actions">${reloadButtonsHtml("candidate", c.id)}</p>
          </div>
        </div>
        <details class="card job-status detail-progress" id="detail-progress" open>
          <summary class="detail-progress-summary">
            <span class="detail-progress-title">// Detail sources //</span>
            <span class="pct" id="detail-pct" role="status" aria-live="polite"><strong>0 / ${stages.length}</strong> · starting…</span>
          </summary>
          <div class="detail-progress-body">
            <div class="bar-wrap" aria-hidden="true">
              <div class="bar" id="detail-bar" style="width: 0%"></div>
            </div>
            <p class="msg" id="detail-msg">Connecting to data sources…</p>
            <ul class="stage-list" id="detail-stages">${stageLis}</ul>
          </div>
        </details>
      </header>

      <div class="detail-tabs">
        <div role="tablist" class="detail-tablist" aria-label="Candidate detail sections" aria-orientation="horizontal">
          ${tabBtns}
        </div>
        <div class="detail-panels">
          ${panel(
            "dossier",
            `<section class="card dossier-card" id="detail-sec-dossier">
              <p class="empty muted">Loading public bio signals…</p>
            </section>`
          )}
          ${panel(
            "scrutiny",
            `<section class="card" id="detail-sec-scrutiny">
              <h2>Scrutiny</h2>
              <p class="empty muted">Loading money signals, endorsements, news, and stated positions…</p>
            </section>`
          )}
          ${panel(
            "votes",
            `<section class="card" id="detail-sec-votes">
              <h2>${c.is_judge ? "Decisions & opinions" : "Voting record"}</h2>
              <p class="empty muted">Loading…</p>
            </section>`
          )}
          ${panel(
            "finance",
            `<section class="card" id="detail-sec-finance">
              <h2>Campaign finance</h2>
              <p class="empty muted">Loading…</p>
            </section>
            <div id="detail-sec-extra"></div>`
          )}
          ${panel(
            "personal",
            `<section class="card" id="detail-sec-personal">
              <h2>Personal finance</h2>
              <p class="empty muted">Loading personal holdings…</p>
            </section>`
          )}
          ${panel(
            "timeline",
            `<section class="card" id="detail-sec-timeline">
              <h2>Correlation timeline</h2>
              <p class="empty muted">Loading dated votes and money…</p>
            </section>`
          )}
          ${panel(
            "party",
            `<section class="card" id="detail-sec-aff">
              <h2>Party / affiliation</h2>
              <p class="empty muted">Loading…</p>
            </section>`
          )}
          ${panel(
            "more",
            `<section class="card" id="detail-sec-more">
              <h2>More</h2>
              <p class="empty muted">Extra notes and disclosure portals appear here when available.</p>
            </section>`
          )}
        </div>
      </div>
    </div>`;
}

function judgeDecisionsPortals(c, e) {
  const bits = [];
  const seen = new Set();
  const push = (label, url) => {
    if (!url || seen.has(url)) return;
    seen.add(url);
    bits.push(
      `<a href="${esc(url)}" rel="noopener noreferrer" target="_blank">${esc(label)}</a>`
    );
  };
  const cl =
    e.votes_url ||
    e.courtlistener_url ||
    (e.courtlistener_id
      ? `https://www.courtlistener.com/person/${e.courtlistener_id}/`
      : null);
  if (cl) push("CourtListener profile", cl);
  else if (e.courtlistener_search_url) {
    push("CourtListener search", e.courtlistener_search_url);
  }
  if (e.fl_courts_url) push("FL courts bio", e.fl_courts_url);
  const portalLists = [
    e.judge_decision_portals,
    e.dossier && e.dossier.disclosure_portals,
  ];
  for (const list of portalLists) {
    if (!Array.isArray(list)) continue;
    for (const p of list) {
      if (!p || !p.url) continue;
      const lab = String(p.label || "");
      if (
        /bar|opinion|dca|supreme|records|directory|fl courts|flcourts/i.test(
          `${lab} ${p.url}`
        )
      ) {
        push(lab || "Portal", p.url);
      }
    }
  }
  if (!bits.length) return "";
  return `<p class="meta muted">Portals: ${bits.join(" · ")}</p>`;
}

function renderVotesBody(c, e, ui = {}) {
  const judicial = !!(c.is_judge || c.chamber === "judicial");
  const heading = judicial ? "Decisions & opinions" : "Voting record";
  const isCl = /courtlistener/i.test(e.votes_source || "");
  if (e.votes && e.votes.length) {
    const all = e.votes;
    const query = ui.query != null ? String(ui.query) : "";
    const position = ui.position != null ? String(ui.position) : "";
    const year = ui.year != null ? String(ui.year) : "";
    const pageSize = ui.pageSize || 10;
    const filtered = filterVotes(all, { query, position, year });
    const paged = pageSlice(filtered, ui.page || 1, pageSize);
    const years = voteYears(all);
    const positions = votePositions(all);
    const yearOpts = years
      .map(
        (y) =>
          `<option value="${esc(y)}"${y === year ? " selected" : ""}>${esc(y)}</option>`
      )
      .join("");
    const posOpts = positions
      .map(
        (p) =>
          `<option value="${esc(p)}"${p === position ? " selected" : ""}>${esc(p)}</option>`
      )
      .join("");
    const rows = paged.rows
      .map(
        (v) => `<tr>
        <td class="nowrap">${esc(v.date)}</td>
        <td><a href="${esc(v.url)}" rel="noopener noreferrer" target="_blank">${esc(v.question)}</a></td>
        <td>${esc(v.position)}</td>
        <td>${esc(v.result || "—")}</td>
      </tr>`
      )
      .join("");
    const emptyFilter =
      !paged.total &&
      `<tr><td colspan="4" class="empty muted">No ${
        judicial ? "decisions" : "votes"
      } match this filter.</td></tr>`;
    const from = paged.total ? (paged.page - 1) * paged.pageSize + 1 : 0;
    const to = Math.min(paged.page * paged.pageSize, paged.total);
    const sizeOpts = [10, 25, 50]
      .map(
        (n) =>
          `<option value="${n}"${n === paged.pageSize ? " selected" : ""}>${n}/page</option>`
      )
      .join("");
    const unit = judicial || isCl ? "opinion" : "roll-call vote";
    const colMatter = judicial || isCl ? "Matter" : "Question";
    const colRole = judicial || isCl ? "Role" : "Vote";
    const profileLabel = judicial || isCl ? "CourtListener →" : "member profile →";
    return `
      <h2>${heading}</h2>
      <p class="meta muted">${all.length} ${unit}${all.length === 1 ? "" : "s"}${
        e.votes_source ? ` · ${esc(e.votes_source)}` : ""
      }${
        e.votes_total_available != null &&
        Number(e.votes_total_available) > all.length
          ? ` · showing ${all.length} of ${Number(e.votes_total_available).toLocaleString()} available${
              e.votes_fetch_cap ? ` (client cap ${e.votes_fetch_cap})` : ""
            }`
          : e.votes_fetch_cap && all.length >= e.votes_fetch_cap
            ? ` · client cap ${e.votes_fetch_cap}`
            : ""
      }${
        e.votes_url
          ? ` · <a href="${esc(e.votes_url)}" rel="noopener noreferrer" target="_blank">${profileLabel}</a>`
          : ""
      }</p>
      <div class="list-toolbar" data-list="votes" role="search">
        <label class="list-field list-field-grow">
          <span class="list-field-label">Search</span>
          <input type="search" class="list-input" name="votes-q" value="${esc(query)}"
            placeholder="${judicial || isCl ? "Case, cite, role…" : "Question, position…"}" autocomplete="off" />
        </label>
        <label class="list-field">
          <span class="list-field-label">Year</span>
          <select class="list-select" name="votes-year">
            <option value="">All years</option>
            ${yearOpts}
          </select>
        </label>
        <label class="list-field">
          <span class="list-field-label">${colRole}</span>
          <select class="list-select" name="votes-pos">
            <option value="">All</option>
            ${posOpts}
          </select>
        </label>
        <label class="list-field">
          <span class="list-field-label">Page size</span>
          <select class="list-select" name="votes-size">${sizeOpts}</select>
        </label>
      </div>
      <table class="votes-table">
        <thead><tr><th>Date</th><th>${colMatter}</th><th>${colRole}</th><th>Result</th></tr></thead>
        <tbody>${rows || emptyFilter}</tbody>
      </table>
      <div class="list-pager" data-list="votes" aria-label="${judicial ? "Decisions" : "Votes"} pagination">
        <span class="list-pager-meta muted">${
          paged.total
            ? `${from}–${to} of ${paged.total}${
                filtered.length !== all.length ? ` (filtered from ${all.length})` : ""
              }`
            : "0 matches"
        }</span>
        <div class="list-pager-btns">
          <button type="button" class="list-page-btn" data-votes-page="prev"
            ${paged.page <= 1 ? "disabled" : ""}>Prev</button>
          <span class="list-page-num">Page ${paged.page} / ${paged.totalPages}</span>
          <button type="button" class="list-page-btn" data-votes-page="next"
            ${paged.page >= paged.totalPages ? "disabled" : ""}>Next</button>
        </div>
      </div>`;
  }
  if (judicial) {
    const checked = e.courtlistener_checked
      ? "Checked CourtListener — no authored opinions found."
      : e.courtlistener_skip
        ? `CourtListener: ${esc(e.courtlistener_skip)}`
        : "No published opinions loaded yet (trial benches and challengers are often empty).";
    return `<h2>${heading}</h2><p class="empty">${checked}</p>${judgeDecisionsPortals(c, e)}`;
  }
  if (c.chamber === "state_senate" || c.chamber === "state_house") {
    if (e.openstates_configured) {
      return e.votes_rate_limited
        ? `<h2>Voting record</h2><p class="empty">Open States rate limit hit (free tier is 10 requests/min). Try again in a minute.</p>`
        : `<h2>Voting record</h2><p class="empty">No state roll-call votes yet.</p>`;
    }
    return `<h2>Voting record</h2><p class="empty">State legislature votes require an Open States API key in Settings.</p>`;
  }
  if (isFecCandidateId(c.external_id)) {
    return `<h2>Voting record</h2><p class="empty">No recent roll-call votes found (challengers and non-members usually have none).</p>`;
  }
  return `<h2>Voting record</h2><p class="empty">Roll-call history is available for sitting members of Congress (via FEC) and state legislators (via Open States).</p>`;
}

function affSourceCell(a) {
  if (!a.source) return "—";
  if (a.source_url) {
    return `<a href="${esc(a.source_url)}" rel="noopener noreferrer" target="_blank">${esc(a.source)}</a>`;
  }
  return esc(a.source);
}

function affIdLinks(e) {
  const bits = [];
  if (e.bioguide_url && e.bioguide) {
    bits.push(
      `<a href="${esc(e.bioguide_url)}" rel="noopener noreferrer" target="_blank">Bioguide ${esc(e.bioguide)}</a>`
    );
  }
  if (e.votes_url && e.govtrack_id) {
    bits.push(
      `<a href="${esc(e.votes_url)}" rel="noopener noreferrer" target="_blank">GovTrack</a>`
    );
  } else if (e.votes_url && !e.bioguide_url) {
    bits.push(
      `<a href="${esc(e.votes_url)}" rel="noopener noreferrer" target="_blank">member profile</a>`
    );
  }
  if (e.congress_gov_url) {
    bits.push(
      `<a href="${esc(e.congress_gov_url)}" rel="noopener noreferrer" target="_blank">congress.gov</a>`
    );
  }
  if (e.wikidata_url && e.wikidata) {
    bits.push(
      `<a href="${esc(e.wikidata_url)}" rel="noopener noreferrer" target="_blank">Wikidata ${esc(e.wikidata)}</a>`
    );
  }
  if (!bits.length) return "";
  return `<p class="meta muted">IDs: ${bits.join(" · ")}</p>`;
}

function renderAffBody(e) {
  if (e.affiliations && e.affiliations.length) {
    const hasRowSrc = e.affiliations.some((a) => a.source);
    const rows = e.affiliations
      .map((a) => {
        const period = a.start
          ? `${esc(a.start)}${a.end ? ` → ${esc(a.end)}` : " → present"}`
          : "—";
        return `<tr>
          <td>${esc(a.party)}</td>
          <td>${esc(a.role)}</td>
          <td class="nowrap">${period}</td>
          <td>${affSourceCell(a)}</td>
        </tr>`;
      })
      .join("");
    return `
      <h2>Party / affiliation</h2>
      ${
        e.affiliations_source && !hasRowSrc
          ? `<p class="meta muted">Source: ${esc(e.affiliations_source)}</p>`
          : ""
      }
      ${affIdLinks(e)}
      <table>
        <thead><tr><th>Party</th><th>Role</th><th>Period</th><th>Source</th></tr></thead>
        <tbody>${rows}</tbody>
      </table>
      <p class="meta muted" style="margin-top:0.75rem">Public citable signals only (ballot filings, legislative service, official records) — not voter-registration history.${
        (e.affiliations || []).some(
          (a) => a.party === "Committee" || (a.role && a.role.includes("≠ voter affiliation"))
        )
          ? " Rows labeled Committee are campaign-finance entities, not the candidate’s voter-party registration."
          : ""
      }</p>`;
  }
  const idBits = affIdLinks(e);
  return `<h2>Party / affiliation</h2>${idBits}<p class="empty">No public affiliation signals on file. We show only citable public records (ballot filings, legislative service) — not voter-registration data.</p>`;
}

function renderFinanceBody(c, e) {
  if (e.finance) {
    const f = e.finance;
    if (
      f.source === "fl_trefin" ||
      f.source === "fl_contrib_search" ||
      f.source === "ftm" ||
      f.source === "nc_cf" ||
      f.source === "az_cf" ||
      f.source === "md_cf" ||
      f.source === "fl_soe"
    ) {
      const lines = Number(f.line_count) || 0;
      const isTrefin = f.source === "fl_trefin";
      const isFtm = f.source === "ftm";
      const isNc = f.source === "nc_cf";
      const isAz = f.source === "az_cf";
      const isMd = f.source === "md_cf";
      const isSoe = f.source === "fl_soe";
      const head = isTrefin
        ? "FL DOS TreFin"
        : isFtm
          ? "FollowTheMoney / OpenSecrets"
          : isNc
            ? "NCSBE campaign finance"
            : isAz
              ? "AZ SeeTheMoney"
              : isMd
                ? "MD MDCRIS campaign finance"
                : isSoe
                  ? "FL county SOE (VoterFocus)"
                  : "FL DOS contributions search";
      return `
      <h2>Campaign finance</h2>
      <p class="meta muted">${esc(head)} · cycle context ${esc(f.cycle || "")}${
        f.account ? ` · account ${esc(f.account)}` : ""
      }${
        f.candidate_id && isFtm ? ` · eid ${esc(f.candidate_id)}` : ""
      }${
        f.report_name && isNc ? ` · ${esc(f.report_name)}` : ""
      }${
        f.match_name ? ` · matched ${esc(f.match_name)}` : ""
      }</p>
      <div class="finance-grid">
        <div class="finance-stat"><span class="finance-label">${
          isTrefin
            ? "Itemized contributions"
            : isFtm
              ? "Total contributions"
              : isNc || isAz || isMd || isSoe
                ? "Cycle receipts"
                : "Reported contributions"
        }</span><span class="finance-value">${esc(f.receipts_display || "—")}</span></div>
        ${
          isNc || isAz || isMd || isSoe
            ? `<div class="finance-stat"><span class="finance-label">${isAz || isMd || isSoe ? "Expenses" : "Cycle expenditures"}</span><span class="finance-value">${esc(f.disbursements_display || "—")}</span></div>
        ${
          isSoe && f.in_kind_display
            ? `<div class="finance-stat"><span class="finance-label">In-kind</span><span class="finance-value">${esc(f.in_kind_display)}</span></div>`
            : `<div class="finance-stat"><span class="finance-label">Cash on hand</span><span class="finance-value">${esc(f.cash_on_hand_display || "—")}</span></div>`
        }`
            : isTrefin
              ? `<div class="finance-stat"><span class="finance-label">Itemized lines</span><span class="finance-value">${esc(String(lines))}</span></div>`
              : f.match_office
                ? `<div class="finance-stat"><span class="finance-label">Office</span><span class="finance-value">${esc(f.match_office)}${
                    f.match_district ? ` · dist ${esc(f.match_district)}` : ""
                  }</span></div>`
                : ""
        }
      </div>
      ${
        f.note
          ? `<p class="meta muted" style="margin-top:0.75rem">${esc(f.note)}</p>`
          : isTrefin
            ? `<p class="meta muted" style="margin-top:0.75rem">Sum of itemized TreFin contribution lines — not a certified cash-on-hand total.</p>`
            : ""
      }
      <p class="meta muted" style="margin-top:0.85rem">
        Source: ${esc(
          f.source_label ||
            (isTrefin
              ? "FL DOS TreFin"
              : isFtm
                ? "FollowTheMoney / OpenSecrets"
                : isNc
                  ? "N.C. State Board of Elections"
                  : isAz
                    ? "Arizona SeeTheMoney"
                    : isMd
                      ? "Maryland MDCRIS"
                      : isSoe
                        ? "County SOE (VoterFocus)"
                        : "FL DOS contrib.exe")
        )}
        ${
          f.profile_url
            ? ` · <a href="${esc(f.profile_url)}" rel="noopener noreferrer" target="_blank">${
                isTrefin
                  ? "DOS candidate"
                  : isFtm
                    ? "FTM profile"
                    : isNc
                      ? "NCSBE committee"
                      : isAz
                        ? "SeeTheMoney"
                        : isMd
                          ? "MDCRIS profile"
                          : isSoe
                            ? "SOE candidate"
                            : "DOS search"
              } →</a>`
            : ""
        }
        ${
          f.report_url
            ? ` · <a href="${esc(f.report_url)}" rel="noopener noreferrer" target="_blank">Report summary →</a>`
            : ""
        }
        ${
          f.trefin_url
            ? ` · <a href="${esc(f.trefin_url)}" rel="noopener noreferrer" target="_blank">TreFin dump →</a>`
            : ""
        }
        ${
          f.show_me_url
            ? ` · <a href="${esc(f.show_me_url)}" rel="noopener noreferrer" target="_blank">FTM show-me →</a>`
            : ""
        }
      </p>`;
    }
    return `
      <h2>Campaign finance</h2>
      <p class="meta muted">FEC cycle ${esc(f.cycle)}${
        f.coverage_end_date ? ` · as of ${esc(f.coverage_end_date)}` : ""
      }</p>
      <div class="finance-grid">
        <div class="finance-stat"><span class="finance-label">Raised</span><span class="finance-value">${esc(f.receipts_display)}</span></div>
        <div class="finance-stat"><span class="finance-label">Spent</span><span class="finance-value">${esc(f.disbursements_display)}</span></div>
        <div class="finance-stat"><span class="finance-label">Cash on hand</span><span class="finance-value">${esc(f.cash_on_hand_display)}</span></div>
        ${
          f.debts_display
            ? `<div class="finance-stat"><span class="finance-label">Debts</span><span class="finance-value">${esc(f.debts_display)}</span></div>`
            : ""
        }
      </div>
      ${
        f.individual_display || f.pac_display || f.party_display
          ? `<p class="meta muted" style="margin-top:0.75rem">Receipts breakdown</p>
        <div class="finance-grid finance-breakdown">
          ${f.individual_display ? `<div class="finance-stat"><span class="finance-label">Individual</span><span class="finance-value">${esc(f.individual_display)}</span></div>` : ""}
          ${f.pac_display ? `<div class="finance-stat"><span class="finance-label">Other committees (PACs)</span><span class="finance-value">${esc(f.pac_display)}</span></div>` : ""}
          ${f.party_display ? `<div class="finance-stat"><span class="finance-label">Party</span><span class="finance-value">${esc(f.party_display)}</span></div>` : ""}
        </div>`
          : ""
      }
      ${
        e.principal_committee
          ? `<p class="meta" style="margin-top:0.85rem">${esc(e.principal_committee.designation)}:
          <a href="${esc(e.principal_committee.url)}" rel="noopener noreferrer" target="_blank">${esc(e.principal_committee.name)}</a>
          <span class="muted">(${esc(e.principal_committee.committee_id)})</span></p>`
          : ""
      }
      <p class="meta muted" style="margin-top:0.85rem">
        Source: ${esc(f.source_label)} ·
        <a href="${esc(f.profile_url)}" rel="noopener noreferrer" target="_blank">FEC profile →</a>
      </p>`;
  }
  if (e.finance_error) {
    const flHint =
      (c.external_id || "").startsWith("fl:acct:") ||
      /CanDetail\.asp/i.test(c.source_url || "") ||
      /myflorida\.com|flsenate\.gov|flhouse\.gov/i.test(c.source_url || "") ||
      /florida/i.test(c.source_publisher || "");
    return `
      <h2>Campaign finance</h2>
      <p class="error">${esc(e.finance_error)}</p>
      <p class="hint">${
        flHint
          ? "FL DOS finance loads via Wisp/libcurl.js (TreFin or name-search). Set Wisp in Settings (This origin via run-static.sh preferred). Ambiguous name matches are skipped on purpose."
          : "Totals load live from OpenFEC. Check your API key in Settings if you see rate limits."
      }</p>
      ${fecProfileLink(c.external_id)}
      ${sourceLinkHtml(c, e)}`;
  }
  if (e.finance_unavailable) {
    const why =
      e.finance_note ||
      (c.chamber === "state_senate" || c.chamber === "state_house"
        ? "No state campaign-finance row linked (FL: DOS AcctNum/name-search; other states: FollowTheMoney key in Settings). OpenFEC does not cover state legislature."
        : c.is_judge
          ? "No campaign-finance row linked for this judicial filing (FL DOS account or FollowTheMoney when keyed)."
          : "No live campaign-finance account linked for this candidate (federal needs FEC id; FL needs DOS; other states need a FollowTheMoney key).");
    const soe =
      e.soe_contact_url
        ? `<p class="meta muted" style="margin-top:0.75rem"><a href="${esc(e.soe_contact_url)}" rel="noopener noreferrer" target="_blank">Contact county Supervisor of Elections →</a></p>`
        : "";
    return `<h2>Campaign finance</h2><p class="empty">${esc(why)}</p>${soe}${sourceLinkHtml(c, e)}`;
  }
  return `<h2>Campaign finance</h2><p class="empty">No finance data.</p>`;
}

function contributorRowHtml(row, showKind) {
  const kind =
    showKind && row._kind
      ? `<span class="list-kind-chip">${row._kind === "committees" ? "PAC" : "Indiv"}</span> `
      : "";
  return `<tr>
    <td>${kind}<a href="${esc(row.url)}" rel="noopener noreferrer" target="_blank">${esc(row.name)}</a>
      ${row.location ? `<div class="muted note">${esc(row.location)}</div>` : ""}${
        row.gift_count
          ? `<div class="muted note">${esc(String(row.gift_count))} itemized gifts</div>`
          : ""
      }</td>
    <td class="nowrap">${esc(row.amount_display)}</td>
    <td class="nowrap">${esc(row.date || "—")}</td>
  </tr>`;
}

function renderContributorListBlock(e, ui = {}) {
  const hasInd = e.top_individuals && e.top_individuals.length;
  const hasCmte = e.top_committees && e.top_committees.length;
  if (!hasInd && !hasCmte) return "";

  const query = ui.query != null ? String(ui.query) : "";
  const kind = ui.kind || "all";
  const pageSize = ui.pageSize || 10;
  const all = mergeContributorLists(e.top_individuals, e.top_committees, kind);
  const filtered = filterContributors(all, { query });
  const paged = pageSlice(filtered, ui.page || 1, pageSize);
  const showKind = !!(hasInd && hasCmte && kind === "all");
  const flTrefin = e.finance && e.finance.source === "fl_trefin";
  const from = paged.total ? (paged.page - 1) * paged.pageSize + 1 : 0;
  const to = Math.min(paged.page * paged.pageSize, paged.total);
  const sizeOpts = [10, 25, 50]
    .map(
      (n) =>
        `<option value="${n}"${n === paged.pageSize ? " selected" : ""}>${n}/page</option>`
    )
    .join("");
  const kindSelect =
    hasInd && hasCmte
      ? `<label class="list-field">
          <span class="list-field-label">Type</span>
          <select class="list-select" name="fin-kind">
            <option value="all"${kind === "all" ? " selected" : ""}>All</option>
            <option value="individuals"${kind === "individuals" ? " selected" : ""}>Individuals</option>
            <option value="committees"${kind === "committees" ? " selected" : ""}>Committees</option>
          </select>
        </label>`
      : "";
  const rows = paged.rows.map((r) => contributorRowHtml(r, showKind)).join("");
  const emptyFilter =
    !paged.total &&
    `<tr><td colspan="3" class="empty muted">No contributors match this filter.</td></tr>`;

  const capNote =
    e.contributors_fetch_cap && all.length >= e.contributors_fetch_cap
      ? ` · client cap ${e.contributors_fetch_cap}`
      : e.contributors_fetch_cap
        ? ` · up to ${e.contributors_fetch_cap}`
        : "";
  const depthNote = e.contributors_note
    ? `<p class="meta muted contributors-note">${esc(e.contributors_note)}</p>`
    : "";
  return `<section class="card" id="detail-sec-contributors">
    <h2>Top itemized contributors</h2>
    <p class="meta muted">${
      flTrefin
        ? "Unique contributors by summed TreFin contribution lines (not certified COH)"
        : `Unique donors/committees by summed Schedule A itemized lines, cycle ${esc(e.finance_cycle || "")}`
    } · ${all.length} loaded${
      hasInd && hasCmte
        ? ` (${e.top_individuals.length} indiv · ${e.top_committees.length} cmte)`
        : ""
    }${capNote}</p>
    ${depthNote}
    <div class="list-toolbar" data-list="finance" role="search">
      <label class="list-field list-field-grow">
        <span class="list-field-label">Search</span>
        <input type="search" class="list-input" name="fin-q" value="${esc(query)}"
          placeholder="Name, location…" autocomplete="off" />
      </label>
      ${kindSelect}
      <label class="list-field">
        <span class="list-field-label">Page size</span>
        <select class="list-select" name="fin-size">${sizeOpts}</select>
      </label>
    </div>
    <table class="contrib-table">
      <thead><tr><th>Contributor</th><th>Total</th><th>Last gift</th></tr></thead>
      <tbody>${rows || emptyFilter}</tbody>
    </table>
    <div class="list-pager" data-list="finance" aria-label="Contributors pagination">
      <span class="list-pager-meta muted">${
        paged.total
          ? `${from}–${to} of ${paged.total}${
              filtered.length !== all.length ? ` (filtered from ${all.length})` : ""
            }`
          : "0 matches"
      }</span>
      <div class="list-pager-btns">
        <button type="button" class="list-page-btn" data-fin-page="prev"
          ${paged.page <= 1 ? "disabled" : ""}>Prev</button>
        <span class="list-page-num">Page ${paged.page} / ${paged.totalPages}</span>
        <button type="button" class="list-page-btn" data-fin-page="next"
          ${paged.page >= paged.totalPages ? "disabled" : ""}>Next</button>
      </div>
    </div>
  </section>`;
}

function renderExtraFinance(e, ui = {}) {
  let html = "";
  if (e.size_buckets && e.size_buckets.length) {
    const rows = e.size_buckets
      .map(
        (row) => `<tr>
        <td>${esc(row.label)}</td>
        <td>${esc(row.total_display)}</td>
        <td>${esc(row.count_display)}</td>
      </tr>`
      )
      .join("");
    html += `<section class="card">
      <h2>Where money came from (by size)</h2>
      <p class="meta muted">Itemized individual contribution sizes, FEC cycle ${esc(e.finance_cycle || "")}</p>
      <table>
        <thead><tr><th>Size</th><th>Total</th><th>Contributions</th></tr></thead>
        <tbody>${rows}</tbody>
      </table>
    </section>`;
  }
  html += renderContributorListBlock(e, ui);
  if (e.outside_spending && e.outside_spending.length) {
    const rows = e.outside_spending
      .map(
        (row) => `<tr>
        <td><a href="${esc(row.url)}" rel="noopener noreferrer" target="_blank">${esc(row.committee)}</a></td>
        <td>${esc(row.amount_display)}</td>
        <td>${esc(row.support_oppose)}</td>
      </tr>`
      )
      .join("");
    html += `<section class="card">
      <h2>Outside spending (top)</h2>
      <p class="meta muted">Independent expenditures reported to the FEC (Schedule E), cycle ${esc(e.finance_cycle || "")}</p>
      <table>
        <thead><tr><th>Committee</th><th>Amount</th><th>Support / Oppose</th></tr></thead>
        <tbody>${rows}</tbody>
      </table>
    </section>`;
  }
  return html;
}

/**
 * Patch progressive detail sections from current enrich state.
 * @param {object} c candidate
 * @param {object} enrich
 * @param {{ final?: boolean, stageId?: string }} [opts]
 */
function factSource(f) {
  if (!f) return "";
  const list =
    Array.isArray(f.sources) && f.sources.length
      ? f.sources
      : f.source
        ? [{ name: f.source, url: f.source_url || null }]
        : [];
  if (!list.length) return "";
  const chips = list
    .map((s) => {
      const name = s.name || s.source || "source";
      const url = s.url || s.source_url;
      if (url) {
        return `<a class="dossier-cite" href="${esc(url)}" rel="noopener noreferrer" target="_blank">${esc(name)}</a>`;
      }
      return `<span class="dossier-cite">${esc(name)}</span>`;
    })
    .join('<span class="muted"> · </span>');
  return ` <span class="muted dossier-cites">(${chips})</span>`;
}

function renderDossierBody(e) {
  const d = e && e.dossier;
  if (!d) {
    return `<p class="empty muted">No dossier yet.</p>`;
  }
  const career = d.career || {};
  const photo = d.photo_url
    ? `<figure class="dossier-photo">
        <img src="${esc(d.photo_url)}" alt="" width="112" height="140" loading="lazy" referrerpolicy="no-referrer" />
        <figcaption class="muted">${
          d.photo_source_url
            ? `<a href="${esc(d.photo_source_url)}" rel="noopener noreferrer" target="_blank">${esc(d.photo_source || "photo source")}</a>`
            : esc(d.photo_source || "public image")
        }</figcaption>
      </figure>`
    : `<div class="dossier-photo dossier-photo-empty muted" aria-hidden="true">No photo</div>`;

  const facts = d.facts || [];
  const family = facts.filter((f) => f.kind === "family");
  const education = facts.filter((f) => f.kind === "education");
  const workish = facts.filter((f) =>
    ["work", "business", "legal"].includes(f.kind)
  );
  const otherFacts = facts.filter((f) => f.kind === "other");
  const bornBits = otherFacts.filter((f) =>
    /^(born|birthplace|birth name|religion|residence)\b/i.test(
      String(f.text || "").split(":")[0] || ""
    )
  );
  const blurbs = otherFacts.filter(
    (f) => !bornBits.includes(f) && String(f.text || "").length >= 40
  );
  const familySum = d.family_summary || {};
  const orient = d.orientation || {};
  const sourcesChecked = d.sources_checked || [];
  const notFound =
    sourcesChecked.length > 0
      ? `Checked ${sourcesChecked.join(" / ")} — not found.`
      : "Not disclosed in sources we check.";

  const dlRows = (items, opts = {}) =>
    items
      .map((f) => {
        const raw = String(f.text || "");
        const colon = raw.indexOf(": ");
        const hasLabel = colon > 0 && colon < 28;
        let label = hasLabel ? raw.slice(0, colon) : "";
        const value = hasLabel ? raw.slice(colon + 2) : raw;
        // Glance Education block already has <h4>Education — skip redundant dt.
        if (
          opts.omitLabel &&
          label &&
          label.toLowerCase() === String(opts.omitLabel).toLowerCase()
        ) {
          label = "";
        }
        if (!label) {
          return `<div class="dossier-fact-line">${esc(value)}${factSource(f)}</div>`;
        }
        return `<div class="dossier-kv">
          <dt>${esc(label)}</dt>
          <dd>${esc(value)}${factSource(f)}</dd>
        </div>`;
      })
      .join("");

  const citesFromList = (sources) => {
    if (!sources || !sources.length) return "";
    return factSource({ sources });
  };

  const snapBlock = (title, items, empty, rowOpts) => {
    if (items && items.length) {
      const bare = !!(rowOpts && rowOpts.omitLabel);
      const body = bare
        ? `<div class="dossier-fact-list">${dlRows(items, rowOpts)}</div>`
        : `<dl class="dossier-kv-list">${dlRows(items, rowOpts)}</dl>`;
      return `<div class="dossier-snap-block">
        <h4>${esc(title)}</h4>
        ${body}
      </div>`;
    }
    return `<div class="dossier-snap-block dossier-snap-empty">
      <h4>${esc(title)}</h4>
      <p class="muted empty">${esc(empty)}</p>
    </div>`;
  };

  const snapHtmlBlock = (title, innerHtml, empty) => {
    if (innerHtml) {
      return `<div class="dossier-snap-block">
        <h4>${esc(title)}</h4>
        ${innerHtml}
      </div>`;
    }
    return `<div class="dossier-snap-block dossier-snap-empty">
      <h4>${esc(title)}</h4>
      <p class="muted empty">${esc(empty)}</p>
    </div>`;
  };

  // Family glance: married + kids summary only (never invent unmarried/childless).
  // Prefer named children in the one-liner when core provided them.
  let familyGlanceHtml = "";
  if (familySum.disclosed && familySum.display) {
    let glance = String(familySum.display || "");
    const names = String(familySum.children_detail || "").trim();
    const n = familySum.children_count;
    if (names && n != null && n > 0 && !/\(/.test(glance)) {
      const kidWord = n === 1 ? "child" : "children";
      const bare = new RegExp(
        String.raw`\b${n}\s+${kidWord}\b`,
        "i"
      );
      if (bare.test(glance) && !glance.toLowerCase().includes(names.toLowerCase())) {
        glance = glance.replace(bare, `${n} ${kidWord} (${names})`);
      }
    } else if (names && (n == null || n === 0) && !/children:/i.test(glance)) {
      glance = glance
        ? `${glance} · Children: ${names}`
        : `Children: ${names}`;
    }
    familyGlanceHtml = `<p class="dossier-family-summary">${esc(
      glance
    )}${citesFromList(familySum.sources)}</p>`;
  }

  // Personal: birthplace/religion/residence + orientation (explicit only)
  const personalRows = [];
  for (const f of bornBits) {
    personalRows.push(f);
  }
  let personalHtml = personalRows.length
    ? `<dl class="dossier-kv-list">${dlRows(personalRows)}</dl>`
    : "";
  if (orient.disclosed && orient.label) {
    personalHtml += `<dl class="dossier-kv-list"><div class="dossier-kv">
      <dt>Orientation</dt>
      <dd>${esc(orient.label)}${citesFromList(orient.sources)}</dd>
    </div></dl>`;
  } else {
    personalHtml += `<p class="muted dossier-orientation-nd">Orientation: ${esc(notFound)}</p>`;
  }
  if (!personalRows.length && !(orient.disclosed && orient.label)) {
    personalHtml = `<p class="muted empty">${esc(
      sourcesChecked.length
        ? notFound
        : "Birthplace, religion, residence not found yet."
    )}</p>
      <p class="muted dossier-orientation-nd">Orientation: ${esc(notFound)}</p>`;
  }

  // Work: profession lines first, then employers
  const workSorted = workish.slice().sort((a, b) => {
    const al = String(a.text || "").toLowerCase();
    const bl = String(b.text || "").toLowerCase();
    const ae = al.includes("employer") ? 1 : 0;
    const be = bl.includes("employer") ? 1 : 0;
    return ae - be;
  });

  const birthLine =
    career.birth_year != null
      ? `<p class="dossier-birth muted">Born ${esc(String(career.birth_year))}${
          career.adult_years != null
            ? ` · ~${esc(String(Math.round(career.adult_years)))} adult years (as of assessment)`
            : ""
        }</p>`
      : "";

  const blurbHtml = blurbs.length
    ? `<div class="dossier-blurbs">${blurbs
        .slice(0, 2)
        .map((f) => `<p>${esc(f.text)}${factSource(f)}</p>`)
        .join("")}</div>`
    : "";

  const banner = career.is_career_politician
    ? `<div class="career-politician-banner" role="status">
        <strong>${esc(career.banner || "CAREER POLITICIAN")}</strong>
        <p>${esc(career.blurb || "")}</p>
      </div>`
    : career.blurb
      ? `<p class="dossier-career-blurb">${esc(career.blurb)}</p>`
      : "";

  const fracRows = (career.fractions || [])
    .map((f) => {
      const pct =
        f.fraction != null ? Math.round(Number(f.fraction) * 100) : null;
      const bar =
        pct != null
          ? `<div class="life-frac-bar" aria-hidden="true"><span style="width:${Math.max(pct, 0)}%"></span></div>`
          : `<div class="life-frac-bar life-frac-bar-empty" aria-hidden="true"></div>`;
      return `<tr>
        <th scope="row">${esc(f.category_label || f.category)}</th>
        <td class="nowrap dossier-frac-num">${esc(f.display || "—")}</td>
        <td class="life-frac-cell">${bar}</td>
      </tr>`;
    })
    .join("");

  const catLabel = (c) => {
    const m = {
      political: "Political",
      education: "Education",
      work: "Work",
      business: "Business",
      legal: "Legal",
    };
    return m[c] || c || "—";
  };

  const spanRows = (career.spans || [])
    .slice()
    .sort((a, b) => {
      const ay = a.start_year != null ? a.start_year : 9999;
      const by = b.start_year != null ? b.start_year : 9999;
      return ay - by;
    })
    .slice(0, 28)
    .map((s) => {
      const yrs =
        s.start_year != null
          ? `${s.start_year}–${s.end_year != null ? s.end_year : "present"}`
          : "—";
      const src = s.source_url
        ? `<a href="${esc(s.source_url)}" rel="noopener noreferrer" target="_blank">${esc(s.source)}</a>`
        : esc(s.source || "—");
      return `<tr>
        <td class="nowrap muted">${esc(catLabel(s.category))}</td>
        <td>${esc(s.label)}</td>
        <td class="nowrap">${esc(yrs)}</td>
        <td class="dossier-src">${src}</td>
      </tr>`;
    })
    .join("");

  const ends = d.endorsements || [];
  const endHtml = ends.length
    ? `<ul class="dossier-endorsements">${ends
        .map((x) => {
          const src = x.source_url
            ? `<a href="${esc(x.source_url)}" rel="noopener noreferrer" target="_blank">${esc(x.source)}</a>`
            : esc(x.source || "");
          return `<li><span class="endorsement-stance endorsement-${esc(x.stance)}">${esc(x.stance)}</span> ${esc(x.org)} <span class="muted">· ${src}</span></li>`;
        })
        .join("")}</ul>`
    : `<p class="muted empty">None loaded yet — see the <strong>Scrutiny</strong> tab (Ballotpedia, campaign pages, FEC IE).</p>`;

  const holds = d.holdings || [];
  const holdTeaser = holds.length
    ? `<p class="muted">${holds.length} holding${holds.length === 1 ? "" : "s"} on file — see the <strong>Personal finance</strong> tab.</p>`
    : `<p class="muted empty">Personal holdings (Senate eFD / House Clerk FD) live on the <strong>Personal finance</strong> tab.${
        e.holdings_skip
          ? ` <span class="mono">(${esc(String(e.holdings_skip))})</span>`
          : ""
      }</p>`;

  const cit = d.citizenship || {};
  const citHtml = cit.disclosed
    ? `<p>${esc((cit.countries || []).join(", ") || "—")}${
        cit.source_url
          ? ` <span class="muted">(<a href="${esc(cit.source_url)}" rel="noopener noreferrer" target="_blank">${esc(cit.source || "source")}</a>)</span>`
          : cit.source
            ? ` <span class="muted">(${esc(cit.source)})</span>`
            : ""
      }</p>${cit.note ? `<p class="muted">${esc(cit.note)}</p>` : ""}`
    : `<p class="muted">${esc(
        sourcesChecked.length
          ? notFound
          : cit.note || "Not disclosed in the public sources we check."
      )}</p>`;

  const notes = [...(career.notes || []), ...(d.empty_notes || [])]
    .filter(Boolean)
    .map((n) => `<li>${esc(n)}</li>`)
    .join("");

  const hasTimeline = !!(career.spans && career.spans.length);

  return `
    <div class="dossier-head">
      ${photo}
      <div class="dossier-head-main">
        ${birthLine}
        ${blurbHtml}
      </div>
    </div>

    <section class="dossier-section dossier-snapshot" aria-labelledby="dossier-snap-h">
      <h3 id="dossier-snap-h">At a glance</h3>
      <p class="meta muted dossier-lead">Public facts only — every line is cited. Missing fields mean we did not find them, not that they are secret.</p>
      <div class="dossier-snap-grid dossier-glance-grid">
        ${snapBlock("Education", education, notFound, { omitLabel: "Education" })}
        ${snapHtmlBlock("Family", familyGlanceHtml, notFound)}
        ${snapBlock("Work / profession", workSorted, notFound)}
        ${snapHtmlBlock("Personal", personalHtml, notFound)}
        <div class="dossier-snap-block">
          <h4>Citizenship</h4>
          ${citHtml}
        </div>
      </div>
    </section>

    ${
      family.length
        ? `<section class="dossier-section dossier-detail-facts" aria-labelledby="dossier-detail-h">
      <h3 id="dossier-detail-h">Detail</h3>
      <div class="dossier-snap-grid">
        ${snapBlock("Family", family, "Not disclosed in sources we check.")}
      </div>
    </section>`
        : ""
    }

    <section class="dossier-section dossier-analytics" aria-labelledby="dossier-life-h">
      <h3 id="dossier-life-h">Adult life (approx.)</h3>
      ${banner}
      <p class="meta muted dossier-lead">Year-level math from dated roles. Political includes elected office, political jobs, and bench time. Education years come from graduation / attendance dates when known.</p>
      <div class="dossier-table-wrap">
        <table class="dossier-frac-table">
          <thead><tr><th>Category</th><th>Share of adult life</th><th class="life-frac-cell"></th></tr></thead>
          <tbody>${fracRows || `<tr><td colspan="3" class="muted">No dated spans yet</td></tr>`}</tbody>
        </table>
      </div>
      ${
        hasTimeline
          ? `<h4 class="dossier-subh">Cited timeline</h4>
      <div class="dossier-table-wrap">
        <table class="dossier-timeline-table">
          <thead><tr><th>Cat</th><th>Role</th><th>Years</th><th>Source</th></tr></thead>
          <tbody>${spanRows}</tbody>
        </table>
      </div>`
          : `<p class="muted empty">No dated career spans yet.</p>`
      }
    </section>

    <section class="dossier-section" aria-labelledby="dossier-end-h">
      <h3 id="dossier-end-h">Endorsements</h3>
      ${endHtml}
    </section>

    <section class="dossier-section" aria-labelledby="dossier-econ-h">
      <h3 id="dossier-econ-h">Personal economic ties</h3>
      ${holdTeaser}
    </section>

    ${
      notes
        ? `<section class="dossier-section dossier-notes-sec" aria-labelledby="dossier-notes-h">
      <h3 id="dossier-notes-h">Notes</h3>
      <ul class="dossier-notes muted">${notes}</ul>
    </section>`
        : ""
    }
  `;
}

/** Kind label for personal holdings table. */
function holdingKindLabel(kind) {
  const k = String(kind || "").toLowerCase();
  if (k === "stock" || k === "security") return "Security";
  if (k === "property" || k === "real_estate" || k === "real estate")
    return "Property";
  if (k === "business") return "Business";
  if (k === "bank" || k === "cash") return "Bank / cash";
  if (k === "retirement") return "Retirement";
  if (k === "other" || !k) return "Other";
  return kind;
}

/**
 * Personal holdings (Senate eFD Part 3 / House Clerk FD Schedule A) — not campaign $.
 */
function renderPersonalFinanceBody(c, e) {
  const d = e.dossier || {};
  const holds = d.holdings || [];
  const portals = d.disclosure_portals || e.disclosure_portals || [];
  const reportUrl = e.efd_report_url || e.house_clerk_pdf_url || null;
  const reportDate = e.efd_report_date || e.house_clerk_filing_year || null;
  const stage = e.holdings_stage || null;

  const sourceBlurb =
    stage === "house_clerk_fd" || e.house_clerk_pdf_url
      ? "House Clerk financial disclosure (Schedule A assets as filed — ranges/text, not market marks)."
      : stage === "senate_efd" || e.efd_report_url
        ? "Senate eFD annual report Part 3 (assets as filed — ranges, not market marks)."
        : "Senate eFD annual Part 3 or House Clerk FD Schedule A when the stage succeeds.";

  const reportLink = reportUrl
    ? `<p class="meta">Source report${
        reportDate ? ` · ${esc(String(reportDate))}` : ""
      }: <a href="${esc(reportUrl)}" rel="noopener noreferrer" target="_blank">${esc(
        reportUrl.length > 72 ? reportUrl.slice(0, 69) + "…" : reportUrl
      )}</a></p>`
    : "";

  const portalHtml = portals.length
    ? `<section class="personal-finance-portals" aria-labelledby="pf-portals-h">
        <h3 id="pf-portals-h">Disclosure portals</h3>
        <ul class="dossier-disclosure-portals">${portals
          .map(
            (p) =>
              `<li><a href="${esc(p.url)}" rel="noopener noreferrer" target="_blank">${esc(p.label)}</a>
              <span class="muted"> — ${esc(p.note || "")}</span></li>`
          )
          .join("")}</ul>
      </section>`
    : "";

  if (!holds.length) {
    const skip = e.holdings_skip
      ? ` <span class="mono">(${esc(String(e.holdings_skip))})</span>`
      : "";
    const isFederal =
      c &&
      (c.chamber === "senate" ||
        c.chamber === "house" ||
        /senate|house|congress/i.test(c.office || ""));
    return `<h2>Personal finance</h2>
      <p class="meta muted">Personal assets from public financial disclosure — <em>not</em> campaign FEC/state committee receipts (see the Finance tab).</p>
      <p class="empty muted">No personal holdings loaded this session.${skip}
      ${
        isFederal
          ? " Open // Detail sources // for the eFD / House Clerk stage. Needs Wisp in Settings when the host blocks browser CORS."
          : " Federal Senate/House members load eFD or House Clerk FD here; state candidates usually have no parseable personal-holdings feed."
      }</p>
      ${reportLink}
      ${portalHtml}`;
  }

  const kindCounts = {};
  for (const h of holds) {
    const lab = holdingKindLabel(h.kind);
    kindCounts[lab] = (kindCounts[lab] || 0) + 1;
  }
  const stats = Object.entries(kindCounts)
    .sort((a, b) => b[1] - a[1])
    .map(
      ([lab, n]) =>
        `<div class="finance-stat"><span class="finance-label">${esc(lab)}</span><span class="finance-value">${n}</span></div>`
    )
    .join("");

  const rows = holds
    .map((h) => {
      const src = h.source_url
        ? `<a href="${esc(h.source_url)}" rel="noopener noreferrer" target="_blank">${esc(h.source || "source")}</a>`
        : esc(h.source || "—");
      return `<tr>
        <td class="nowrap muted">${esc(holdingKindLabel(h.kind))}</td>
        <td>${esc(h.description || "—")}</td>
        <td class="nowrap">${esc(h.amount_display || "—")}</td>
        <td class="dossier-src">${src}</td>
      </tr>`;
    })
    .join("");

  return `<h2>Personal finance</h2>
    <p class="meta muted">${esc(sourceBlurb)} Not campaign committee money.</p>
    <div class="finance-grid personal-finance-stats">
      <div class="finance-stat"><span class="finance-label">Holdings</span><span class="finance-value">${holds.length}</span></div>
      ${stats}
    </div>
    ${reportLink}
    <div class="dossier-table-wrap personal-finance-table-wrap">
      <table class="personal-finance-table">
        <thead><tr><th>Type</th><th>Asset</th><th>Value (as filed)</th><th>Source</th></tr></thead>
        <tbody>${rows}</tbody>
      </table>
    </div>
    <p class="meta muted" style="margin-top:0.75rem">Values are disclosure ranges or filer text — not live market prices. EIGA limits apply (voter-info OK; no bulk resale/solicitation).</p>
    ${portalHtml}`;
}

function trustChip(t) {
  const v = String(t || "").trim();
  if (!v) return "";
  return `<span class="muted endorsement-trust">${esc(v)}</span>`;
}

export function renderScrutinyBody(c, e) {
  const sc = e.scrutiny || {};
  const money = sc.money || e.money_signals || {};
  const signals = money.signals || [];
  const news = sc.news || e.news_hits || [];
  const ends = (e.dossier && e.dossier.endorsements) || sc.endorsements || [];
  const portals = sc.portals || [];

  const moneyHtml = signals.length
    ? `<ul class="scrutiny-signals">${signals
        .map(
          (s) => `<li>
          <strong>${esc(s.label)}</strong>
          <span class="finance-value">${esc(s.value_display)}</span>
          <p class="meta muted">${esc(s.note || "")} ${trustChip(s.trust)}</p>
        </li>`
        )
        .join("")}</ul>`
    : `<p class="muted empty">${esc(
        money.empty_note ||
          "Money signals appear after campaign-finance stages load itemized rows."
      )}</p>`;

  const endHtml = ends.length
    ? `<ul class="dossier-endorsements">${ends
        .map((x) => {
          const src = x.source_url
            ? `<a href="${esc(x.source_url)}" rel="noopener noreferrer" target="_blank">${esc(x.source)}</a>`
            : esc(x.source || "");
          return `<li><span class="endorsement-stance endorsement-${esc(x.stance)}">${esc(x.stance)}</span> ${esc(x.org)} <span class="muted">· ${src}${
            x.kind ? ` · ${esc(x.kind)}` : ""
          }</span> ${trustChip(x.trust)}</li>`;
        })
        .join("")}</ul>`
    : `<p class="muted empty">No org endorsements loaded yet (Ballotpedia, campaign /endorsements, FEC independent expenditures). Empty is common for local challengers.</p>`;

  const newsHtml = news.length
    ? `<ul class="scrutiny-news">${news
        .map((n) => {
          const when = n.date ? `<span class="nowrap muted">${esc(n.date)}</span> · ` : "";
          const href = n.url
            ? `<a href="${esc(n.url)}" rel="noopener noreferrer" target="_blank">${esc(n.title)}</a>`
            : esc(n.title);
          return `<li>${when}${href} <span class="muted">· ${esc(n.outlet || "")}</span> ${trustChip(n.trust)}</li>`;
        })
        .join("")}</ul>
      <p class="meta muted">Headlines are allegations or coverage — not findings. Read the source.</p>`
    : `<p class="muted empty">No news hits in the last ~90 days from GDELT / Google News for this name, or the name was too ambiguous to search.</p>`;

  const portalHtml = portals.length
    ? `<p class="meta">${portals
        .map(
          (p) =>
            `<a href="${esc(p.url)}" rel="noopener noreferrer" target="_blank">${esc(p.label)}</a>`
        )
        .join(" · ")}</p>`
    : "";

  return `
    <h2>Scrutiny</h2>
    <p class="meta muted">Cited signals only — not a verdict of “bought,” “scammer,” or “liar.” Keyword overlap is not a contradiction finding.</p>
    <h3>Money signals</h3>
    ${moneyHtml}
    <h3>Endorsements &amp; opposition</h3>
    ${endHtml}
    <h3>On the record</h3>
    ${newsHtml}
    <h3>License &amp; ethics</h3>
    ${renderRecordHits(sc)}
    <h3>Stated positions &amp; contrasts</h3>
    ${renderClaimContrasts(sc, e)}
    <h3>Portals</h3>
    ${portalHtml || `<p class="muted empty">Portals load with this panel.</p>`}`;
}

function renderRecordHits(sc) {
  const rows = sc.records || [];
  if (!rows.length) {
    return `<p class="muted empty">Checked Florida Bar / Ethics / JQC when this is a Florida candidate — no unique public record matched this name. Ambiguous matches are skipped. Empty is common.</p>`;
  }
  return `<ul class="scrutiny-records">${rows
    .map((r) => {
      const when = r.date ? `<span class="nowrap muted">${esc(r.date)}</span> · ` : "";
      const href = r.url
        ? `<a href="${esc(r.url)}" rel="noopener noreferrer" target="_blank">${esc(r.title)}</a>`
        : esc(r.title);
      const st = r.status ? ` <span class="muted">· ${esc(r.status)}</span>` : "";
      return `<li>${when}${href}${st}
        <p class="meta muted">${esc(r.detail || "")} · ${esc(r.source || "")} ${trustChip(r.trust || "official")}</p>
      </li>`;
    })
    .join("")}</ul>
    <p class="meta muted">Public filings and notices only — not a verdict. Pending complaints are often confidential.</p>`;
}

function renderClaimContrasts(sc, e) {
  const cards = sc.contrasts || [];
  const claims = sc.claims || [];
  const rows = cards.length
    ? cards
    : claims.map((c) => ({
        claim_text: c.text,
        topic: c.topic,
        claim_source: c.source,
        claim_url: c.source_url,
        claim_trust: c.trust,
        claim_kind: c.kind,
        matches: [],
        note: "",
      }));
  if (!rows.length) {
    return `<p class="muted empty">No campaign / Ballotpedia issue statements extracted. Empty is common when the page has no Issues or Campaign themes section.</p>`;
  }
  const votesLoaded = !!(e.votes && e.votes.length);
  return `<ul class="scrutiny-claims">${rows
    .map((card) => {
      const src = card.claim_url
        ? `<a href="${esc(card.claim_url)}" rel="noopener noreferrer" target="_blank">${esc(card.claim_source || "")}</a>`
        : esc(card.claim_source || "");
      const topic = card.topic
        ? `<span class="scrutiny-claim-topic">${esc(card.topic)}</span> `
        : "";
      const matches = card.matches || [];
      let related;
      if (matches.length) {
        related = `<ul class="scrutiny-related">${matches
          .map((m) => {
            const href = m.url
              ? `<a href="${esc(m.url)}" rel="noopener noreferrer" target="_blank">${esc(m.question)}</a>`
              : esc(m.question);
            const when = m.date ? `<span class="nowrap muted">${esc(m.date)}</span> · ` : "";
            const ov = (m.overlap || []).length
              ? ` <span class="muted">· ${esc(m.overlap.join(", "))}</span>`
              : "";
            return `<li>${when}${href} <span class="muted">· ${esc(m.position || "")}</span>${ov}</li>`;
          })
          .join("")}</ul>
          <p class="meta muted">${esc(card.note || "Keyword overlap only — not a finding.")}</p>`;
        if (card.llm_note) {
          const who = card.llm_model ? ` · ${esc(card.llm_model)}` : "";
          related += `<p class="scrutiny-llm">${esc(card.llm_note)} ${trustChip(card.llm_trust || "inference")}<span class="muted">${who}</span></p>
            <p class="meta muted">Model comparison — not a finding.</p>`;
        }
      } else if (votesLoaded) {
        const rec = String(e.votes_source || "").toLowerCase().includes("courtlistener")
          ? "opinions"
          : "votes/opinions";
        related = `<p class="meta muted">No keyword overlap with loaded ${rec}.</p>`;
      } else {
        related = `<p class="meta muted">No roll-calls or opinions loaded to pair yet.</p>`;
      }
      return `<li class="scrutiny-claim">
        ${topic}<span class="muted">${esc(card.claim_kind || "position")}</span>
        <p>${esc(card.claim_text)}</p>
        <p class="meta muted">${src} ${trustChip(card.claim_trust)}</p>
        ${related}
      </li>`;
    })
    .join("")}</ul>`;
}

export function patchDetailSections(c, enrich, opts = {}) {
  const e = enrich || {};
  const final = !!opts.final;
  const stageId = opts.stageId || "";
  const only = opts.only || "";
  const votes = document.querySelector("#detail-sec-votes");
  const aff = document.querySelector("#detail-sec-aff");
  const fin = document.querySelector("#detail-sec-finance");
  const personal = document.querySelector("#detail-sec-personal");
  const timeline = document.querySelector("#detail-sec-timeline");
  const extra = document.querySelector("#detail-sec-extra");
  const dossier = document.querySelector("#detail-sec-dossier");
  const scrutiny = document.querySelector("#detail-sec-scrutiny");

  const financeReady =
    !only &&
    (final ||
      !!e.finance ||
      !!e.finance_error ||
      !!e.finance_unavailable ||
      stageId === "totals" ||
      stageId === "principal" ||
      stageId === "fl_trefin" ||
      stageId === "fl_name_search" ||
      stageId === "fl_soe_cf" ||
      stageId === "nc_cf" ||
      stageId === "az_cf" ||
      stageId === "md_cf" ||
      stageId === "ftm" ||
      stageId === "profile");
  const personalReady =
    !only &&
    (final ||
      stageId === "senate_efd" ||
      stageId === "house_clerk_fd" ||
      stageId === "profile" ||
      !!(e.dossier && e.dossier.holdings && e.dossier.holdings.length) ||
      e.holdings_skip != null ||
      !!e.efd_report_url ||
      !!e.house_clerk_pdf_url ||
      !!(e.disclosure_portals && e.disclosure_portals.length));
  const timelineReady =
    only === "timeline" ||
    (!only &&
      (final ||
        stageId === "votes" ||
        stageId === "os_votes" ||
        stageId === "indiv" ||
        stageId === "cmte" ||
        stageId === "outside" ||
        stageId === "totals" ||
        stageId === "fl_trefin" ||
        stageId === "fl_name_search" ||
        stageId === "fl_soe_cf" ||
        stageId === "nc_cf" ||
        stageId === "az_cf" ||
        stageId === "md_cf" ||
        stageId === "ftm" ||
        stageId === "senate_efd" ||
        stageId === "house_clerk_fd" ||
        stageId === "profile" ||
        !!(e.votes && e.votes.length) ||
        !!(e.timeline_receipts && e.timeline_receipts.length) ||
        !!(e.top_individuals && e.top_individuals.length) ||
        !!(e.top_committees && e.top_committees.length) ||
        !!e.efd_report_date ||
        !!e.house_clerk_filing_year));
  const votesReady =
    only === "votes" ||
    (!only &&
      (final ||
        (e.votes && e.votes.length) ||
        e.votes_rate_limited ||
        stageId === "votes" ||
        stageId === "os_votes" ||
        stageId === "profile"));
  const affReady =
    !only &&
    (final ||
      stageId === "member" ||
      stageId === "os_resolve" ||
      stageId === "principal" ||
      stageId === "fl_trefin" ||
      stageId === "fl_name_search" ||
      stageId === "profile" ||
      (e.affiliations && e.affiliations.length));
  const extraReady =
    only === "extra" ||
    (!only &&
      (final ||
        stageId === "size" ||
        stageId === "indiv" ||
        stageId === "cmte" ||
        stageId === "outside" ||
        stageId === "fl_trefin" ||
        stageId === "fl_name_search" ||
        stageId === "fl_soe_cf" ||
        stageId === "nc_cf" ||
        stageId === "az_cf" ||
        stageId === "md_cf" ||
        stageId === "ftm" ||
        (e.size_buckets && e.size_buckets.length) ||
        (e.top_individuals && e.top_individuals.length) ||
        (e.top_committees && e.top_committees.length) ||
        (e.outside_spending && e.outside_spending.length)));

  const scrutinyReady =
    !only &&
    (final ||
      stageId === "money_signals" ||
      stageId === "bp_endorsements" ||
      stageId === "campaign_endorsements" ||
      stageId === "gdelt_news" ||
      stageId === "news_rss" ||
      stageId === "bp_claims" ||
      stageId === "campaign_claims" ||
      stageId === "claim_contrasts" ||
      stageId === "llm_contrasts" ||
      stageId === "fl_bar" ||
      stageId === "fl_ethics" ||
      stageId === "fl_jqc" ||
      stageId === "ai_verdict" ||
      stageId === "outside" ||
      stageId === "size" ||
      stageId === "totals" ||
      stageId === "fl_trefin" ||
      stageId === "fl_soe_cf" ||
      stageId === "profile" ||
      !!(e.scrutiny && (e.scrutiny.money || e.scrutiny.news || e.scrutiny.portals || (e.scrutiny.claims && e.scrutiny.claims.length) || (e.scrutiny.contrasts && e.scrutiny.contrasts.length) || (e.scrutiny.records && e.scrutiny.records.length))) ||
      !!(e.dossier && e.dossier.endorsements && e.dossier.endorsements.length));

  const dossierReady =
    !only &&
    (final ||
      !!e.dossier ||
      stageId === "member" ||
      stageId === "os_resolve" ||
      stageId === "fl_chamber_bio" ||
      stageId === "official_about" ||
      stageId === "ballotpedia_bio" ||
      stageId === "campaign_about" ||
      stageId === "wiki_extract" ||
      stageId === "dbpedia" ||
      stageId === "grokipedia" ||
      stageId === "wikidata_bio" ||
      stageId === "wiki_photo" ||
      stageId === "senate_efd" ||
      stageId === "house_clerk_fd" ||
      stageId === "fec_occupation" ||
      stageId === "outside" ||
      stageId === "profile");

  if (votes && votesReady) {
    const active = document.activeElement;
    const prevFocus =
      active && votes.contains(active) ? active.getAttribute("name") : null;
    const prevSel =
      prevFocus && active && active.selectionStart != null
        ? { start: active.selectionStart, end: active.selectionEnd }
        : null;
    votes.innerHTML = renderVotesBody(c, e, opts.votesUi || {});
    if (prevFocus) {
      const el = votes.querySelector(`[name="${CSS.escape(prevFocus)}"]`);
      if (el) {
        el.focus();
        if (prevSel && typeof el.setSelectionRange === "function") {
          try {
            el.setSelectionRange(prevSel.start, prevSel.end);
          } catch (_) {
            /* ignore */
          }
        }
      }
    }
  }
  if (aff && affReady) aff.innerHTML = renderAffBody(e);
  if (fin && financeReady) fin.innerHTML = renderFinanceBody(c, e);
  if (personal && personalReady) personal.innerHTML = renderPersonalFinanceBody(c, e);
  if (timeline && timelineReady) {
    timeline.innerHTML = renderTimelineBody(c, e);
    try {
      mountTimeline(timeline);
    } catch (err) {
      console.warn("timeline mount", err);
    }
  }
  if (extra && extraReady) {
    const active = document.activeElement;
    const prevFocus =
      active && extra.contains(active) ? active.getAttribute("name") : null;
    const prevSel =
      prevFocus && active && active.selectionStart != null
        ? { start: active.selectionStart, end: active.selectionEnd }
        : null;
    extra.innerHTML = renderExtraFinance(e, opts.financeUi || {});
    if (prevFocus) {
      const el = extra.querySelector(`[name="${CSS.escape(prevFocus)}"]`);
      if (el) {
        el.focus();
        if (prevSel && typeof el.setSelectionRange === "function") {
          try {
            el.setSelectionRange(prevSel.start, prevSel.end);
          } catch (_) {
            /* ignore */
          }
        }
      }
    }
  }
  if (scrutiny && scrutinyReady) {
    scrutiny.innerHTML = renderScrutinyBody(c, e);
  }
  if (dossier && dossierReady) {
    dossier.innerHTML = renderDossierBody(e);
    const { photoHtml, teaserHtml } = renderDetailHeaderTeaser(
      e,
      c && c.name
    );
    const photoEl = document.querySelector("#detail-header-photo");
    const teaserEl = document.querySelector("#detail-header-teaser");
    if (photoEl) {
      if (photoHtml) {
        photoEl.innerHTML = photoHtml;
        photoEl.hidden = false;
      }
    }
    if (teaserEl) teaserEl.innerHTML = teaserHtml || "";
    const linksEl = document.querySelector(".detail-header-links");
    if (linksEl && e.profile_url && /^https?:\/\//i.test(e.profile_url)) {
      const has = linksEl.innerHTML.includes(e.profile_url);
      if (!has) {
        const a = `<a href="${esc(e.profile_url)}" rel="noopener noreferrer" target="_blank">Source profile</a>`;
        linksEl.innerHTML = linksEl.innerHTML
          ? `${a} · ${linksEl.innerHTML}`
          : a;
      }
    } else if (!linksEl && e.profile_url && /^https?:\/\//i.test(e.profile_url)) {
      const body = document.querySelector(".detail-header-body");
      if (body) {
        const p = document.createElement("p");
        p.className = "detail-header-links meta";
        p.innerHTML = `<a href="${esc(e.profile_url)}" rel="noopener noreferrer" target="_blank">Source profile</a>`;
        body.appendChild(p);
      }
    }
  }
}

export function renderCandidateArticle(c, enrich) {
  const e = enrich || {};
  let votesHtml;
  if (e.votes && e.votes.length) {
    const rows = e.votes
      .map(
        (v) => `<tr>
        <td class="nowrap">${esc(v.date)}</td>
        <td><a href="${esc(v.url)}" rel="noopener noreferrer" target="_blank">${esc(v.question)}</a></td>
        <td>${esc(v.position)}</td>
        <td>${esc(v.result || "—")}</td>
      </tr>`
      )
      .join("");
    votesHtml = `
      <p class="meta muted">Recent roll-call votes${
        e.votes_source ? ` (${esc(e.votes_source)})` : ""
      }${
        e.votes_url
          ? ` · <a href="${esc(e.votes_url)}" rel="noopener noreferrer" target="_blank">member profile →</a>`
          : ""
      }</p>
      <table>
        <thead><tr><th>Date</th><th>Question</th><th>Vote</th><th>Result</th></tr></thead>
        <tbody>${rows}</tbody>
      </table>`;
  } else if (c.chamber === "state_senate" || c.chamber === "state_house") {
    if (e.openstates_configured) {
      votesHtml = e.votes_rate_limited
        ? `<p class="empty">Open States rate limit hit (free tier is 10 requests/min). Try again in a minute.</p>`
        : `<p class="empty">No state roll-call votes yet.</p>`;
    } else {
      votesHtml = `<p class="empty">State legislature votes require an Open States API key in Settings.</p>`;
    }
  } else if (isFecCandidateId(c.external_id)) {
    votesHtml = `<p class="empty">No recent roll-call votes found (challengers and non-members usually have none).</p>`;
  } else if (c.is_judge || c.chamber === "judicial") {
    votesHtml = `<p class="empty">${
      e.courtlistener_checked
        ? "Checked CourtListener — no authored opinions found."
        : "No published opinions loaded for this judicial seat."
    }</p>`;
  } else {
    votesHtml = `<p class="empty">Roll-call history is available for sitting members of Congress (via FEC) and state legislators (via Open States).</p>`;
  }

  let affHtml;
  if (e.affiliations && e.affiliations.length) {
    const hasRowSrc = e.affiliations.some((a) => a.source);
    const rows = e.affiliations
      .map((a) => {
        const period = a.start
          ? `${esc(a.start)}${a.end ? ` → ${esc(a.end)}` : " → present"}`
          : "—";
        return `<tr>
          <td>${esc(a.party)}</td>
          <td>${esc(a.role)}</td>
          <td class="nowrap">${period}</td>
          <td>${affSourceCell(a)}</td>
        </tr>`;
      })
      .join("");
    affHtml = `
      ${
        e.affiliations_source && !hasRowSrc
          ? `<p class="meta muted">Source: ${esc(e.affiliations_source)}</p>`
          : ""
      }
      ${affIdLinks(e)}
      <table>
        <thead><tr><th>Party</th><th>Role</th><th>Period</th><th>Source</th></tr></thead>
        <tbody>${rows}</tbody>
      </table>
      <p class="meta muted" style="margin-top:0.75rem">Public citable signals only (ballot filings, legislative service, official records) — not voter-registration history.${
        (e.affiliations || []).some(
          (a) => a.party === "Committee" || (a.role && a.role.includes("≠ voter affiliation"))
        )
          ? " Rows labeled Committee are campaign-finance entities, not the candidate’s voter-party registration."
          : ""
      }</p>`;
  } else {
    affHtml = `${affIdLinks(e)}<p class="empty">No public affiliation signals on file. We show only citable public records (ballot filings, legislative service) — not voter-registration data.</p>`;
  }

  let financeHtml;
  if (e.finance) {
    const f = e.finance;
    financeHtml = `
      <p class="meta muted">FEC cycle ${esc(f.cycle)}${
        f.coverage_end_date ? ` · as of ${esc(f.coverage_end_date)}` : ""
      }</p>
      <div class="finance-grid">
        <div class="finance-stat"><span class="finance-label">Raised</span><span class="finance-value">${esc(f.receipts_display)}</span></div>
        <div class="finance-stat"><span class="finance-label">Spent</span><span class="finance-value">${esc(f.disbursements_display)}</span></div>
        <div class="finance-stat"><span class="finance-label">Cash on hand</span><span class="finance-value">${esc(f.cash_on_hand_display)}</span></div>
        ${
          f.debts_display
            ? `<div class="finance-stat"><span class="finance-label">Debts</span><span class="finance-value">${esc(f.debts_display)}</span></div>`
            : ""
        }
      </div>
      ${
        f.individual_display || f.pac_display || f.party_display
          ? `<p class="meta muted" style="margin-top:0.75rem">Receipts breakdown</p>
        <div class="finance-grid finance-breakdown">
          ${f.individual_display ? `<div class="finance-stat"><span class="finance-label">Individual</span><span class="finance-value">${esc(f.individual_display)}</span></div>` : ""}
          ${f.pac_display ? `<div class="finance-stat"><span class="finance-label">Other committees (PACs)</span><span class="finance-value">${esc(f.pac_display)}</span></div>` : ""}
          ${f.party_display ? `<div class="finance-stat"><span class="finance-label">Party</span><span class="finance-value">${esc(f.party_display)}</span></div>` : ""}
        </div>`
          : ""
      }
      ${
        e.principal_committee
          ? `<p class="meta" style="margin-top:0.85rem">${esc(e.principal_committee.designation)}:
          <a href="${esc(e.principal_committee.url)}" rel="noopener noreferrer" target="_blank">${esc(e.principal_committee.name)}</a>
          <span class="muted">(${esc(e.principal_committee.committee_id)})</span></p>`
          : ""
      }
      <p class="meta muted" style="margin-top:0.85rem">
        Source: ${esc(f.source_label)} ·
        <a href="${esc(f.profile_url)}" rel="noopener noreferrer" target="_blank">FEC profile →</a>
      </p>`;
  } else if (e.finance_error) {
    financeHtml = `
      <p class="error">${esc(e.finance_error)}</p>
      <p class="hint">Totals load live from OpenFEC. Check your API key in Settings if you see rate limits.</p>
      ${fecProfileLink(c.external_id)}`;
  } else if (e.finance_unavailable) {
    const why =
      c.chamber === "state_senate" || c.chamber === "state_house"
        ? "State legislature candidates do not report to the FEC. Roll-calls need an Open States key; finance is not available here."
        : c.is_judge
          ? "Judicial candidates typically have no FEC campaign-finance file."
          : "Campaign finance totals are only available for federal candidates with an FEC id.";
    financeHtml = `<p class="empty">${esc(why)}</p>${sourceLinkHtml(c, e)}`;
  } else {
    financeHtml = `<p class="empty">No finance data.</p>`;
  }

  let sizeHtml = "";
  if (e.size_buckets && e.size_buckets.length) {
    const rows = e.size_buckets
      .map(
        (row) => `<tr>
        <td>${esc(row.label)}</td>
        <td>${esc(row.total_display)}</td>
        <td>${esc(row.count_display)}</td>
      </tr>`
      )
      .join("");
    sizeHtml = `<section class="card">
      <h2>Where money came from (by size)</h2>
      <p class="meta muted">Itemized individual contribution sizes, FEC cycle ${esc(e.finance_cycle || "")}</p>
      <table>
        <thead><tr><th>Size</th><th>Total</th><th>Contributions</th></tr></thead>
        <tbody>${rows}</tbody>
      </table>
    </section>`;
  }

  let contribHtml = "";
  if (
    (e.top_individuals && e.top_individuals.length) ||
    (e.top_committees && e.top_committees.length)
  ) {
    const ind =
      e.top_individuals && e.top_individuals.length
        ? `<p class="meta muted" style="margin-top:0.75rem">Individuals</p>
        <table>
          <thead><tr><th>Contributor</th><th>Total</th><th>Last gift</th></tr></thead>
          <tbody>${e.top_individuals
            .map(
              (row) => `<tr>
            <td><a href="${esc(row.url)}" rel="noopener noreferrer" target="_blank">${esc(row.name)}</a>
              ${row.location ? `<div class="muted note">${esc(row.location)}</div>` : ""}${
                row.gift_count
                  ? `<div class="muted note">${esc(String(row.gift_count))} itemized gifts</div>`
                  : ""
              }</td>
            <td class="nowrap">${esc(row.amount_display)}</td>
            <td class="nowrap">${esc(row.date || "—")}</td>
          </tr>`
            )
            .join("")}</tbody>
        </table>`
        : "";
    const cmte =
      e.top_committees && e.top_committees.length
        ? `<p class="meta muted" style="margin-top:1rem">Committees / PACs</p>
        <table>
          <thead><tr><th>Committee</th><th>Total</th><th>Last gift</th></tr></thead>
          <tbody>${e.top_committees
            .map(
              (row) => `<tr>
            <td><a href="${esc(row.url)}" rel="noopener noreferrer" target="_blank">${esc(row.name)}</a>
              ${row.location ? `<div class="muted note">${esc(row.location)}</div>` : ""}${
                row.gift_count
                  ? `<div class="muted note">${esc(String(row.gift_count))} itemized gifts</div>`
                  : ""
              }</td>
            <td class="nowrap">${esc(row.amount_display)}</td>
            <td class="nowrap">${esc(row.date || "—")}</td>
          </tr>`
            )
            .join("")}</tbody>
        </table>`
        : "";
    contribHtml = `<section class="card">
      <h2>Top itemized contributors</h2>
      <p class="meta muted">Unique donors/committees by summed Schedule A (from latest itemized page), cycle ${esc(e.finance_cycle || "")}</p>
      ${ind}${cmte}
    </section>`;
  }

  let outsideHtml = "";
  if (e.outside_spending && e.outside_spending.length) {
    const rows = e.outside_spending
      .map(
        (row) => `<tr>
        <td><a href="${esc(row.url)}" rel="noopener noreferrer" target="_blank">${esc(row.committee)}</a></td>
        <td>${esc(row.amount_display)}</td>
        <td>${esc(row.support_oppose)}</td>
      </tr>`
      )
      .join("");
    outsideHtml = `<section class="card">
      <h2>Outside spending (top)</h2>
      <p class="meta muted">Independent expenditures reported to the FEC (Schedule E), cycle ${esc(e.finance_cycle || "")}</p>
      <table>
        <thead><tr><th>Committee</th><th>Amount</th><th>Support / Oppose</th></tr></thead>
        <tbody>${rows}</tbody>
      </table>
    </section>`;
  }

  return `<article class="detail">
    <p class="crumb"><a href="#" id="back-ballot-article">← Back to ballot</a></p>
    <h1>${esc(c.name)}</h1>
    <p class="meta">${partyChip(c.party, c.party_class)} · ${esc(c.office)} · ${esc(c.jurisdiction)}</p>
    <p class="meta muted">${esc(c.election_name || "")}${
      c.election_date ? ` · ${esc(c.election_date)}` : ""
    }${c.is_incumbent ? " · Incumbent" : ""}${c.is_judge ? " · Judicial" : ""}${
      externalIdMeta(c.external_id)
    }</p>
    <p class="section-label">// Profile //</p>
    <section class="card">
      <h2>Summary</h2>
      <p>${esc(c.summary || "No summary available yet.")}</p>
      ${
        c.source_url
          ? `<p class="meta"><a href="${esc(c.source_url)}" rel="noopener noreferrer" target="_blank">Source profile →</a></p>`
          : ""
      }
    </section>
    <section class="card">
      <h2>${c.is_judge || c.chamber === "judicial" ? "Decisions & opinions" : "Voting record"}</h2>
      ${votesHtml}
    </section>
    <section class="card">
      <h2>Party / affiliation</h2>
      ${affHtml}
    </section>
    <p class="section-label">// Finance //</p>
    <section class="card">
      <h2>Campaign finance</h2>
      ${financeHtml}
    </section>
    ${sizeHtml}
    ${contribHtml}
    ${outsideHtml}
  </article>`;
}

export function indexCandidates(report) {
  const map = new Map();
  const walk = (cands) => {
    for (const c of cands || []) map.set(String(c.id), c);
  };
  for (const sec of report.ballot_sections || []) {
    if (sec.kind === "judicial") {
      for (const g of sec.seats || []) walk(g.candidates);
    } else if (sec.group) {
      walk(sec.group.candidates);
    }
  }
  for (const g of report.office_groups || []) walk(g.candidates);
  return map;
}

export function indexMeasures(report) {
  const map = new Map();
  for (const m of report?.measures || []) {
    if (m && m.id != null) map.set(String(m.id), m);
  }
  return map;
}

function measureFinLinkLabel(f) {
  if (!f) return "source →";
  if (f.source === "ftm_measure") return "FTM →";
  if (f.source === "md_cf_measure") return "MDCRIS →";
  if (f.committee_url && /followthemoney/i.test(f.committee_url)) return "FTM →";
  if (f.committee_url && /campaignfinance\.maryland/i.test(f.committee_url))
    return "MDCRIS →";
  return "DOS →";
}

function measureFinanceTableCells(f) {
  if (!f) {
    return {
      sponsor: `<span class="muted">…</span>`,
      oppose: `<span class="muted">…</span>`,
    };
  }
  const lines = Number(f.line_count) || 0;
  const top = (f.top_contributors || [])
    .slice(0, 2)
    .map(
      (c) =>
        `${esc(c.name)} ${esc(c.amount_display)}${
          c.gift_count ? ` (${esc(String(c.gift_count))})` : ""
        }`
    )
    .join("; ");
  const finLinkLabel = measureFinLinkLabel(f);
  const finSub =
    f.source === "ftm_measure" || f.source === "md_cf_measure"
      ? " support $"
      : " itemized $";
  const sponsorName = (f.committee_name || "").trim();
  const sponsorLabel =
    sponsorName &&
    !/^support\b/i.test(sponsorName) &&
    !/^unknown$/i.test(sponsorName)
      ? f.committee_url
        ? `<div class="measure-backer"><span class="endorsement-stance endorsement-support">support</span> <a href="${esc(f.committee_url)}" target="_blank" rel="noopener">${esc(sponsorName.length > 42 ? sponsorName.slice(0, 39) + "…" : sponsorName)}</a></div>`
        : `<div class="measure-backer"><span class="endorsement-stance endorsement-support">support</span> ${esc(sponsorName.length > 42 ? sponsorName.slice(0, 39) + "…" : sponsorName)}</div>`
      : "";
  let sponsor;
  if (lines > 0 || sponsorLabel) {
    sponsor = `${sponsorLabel}
      <div><strong>${esc(f.contributions_sum_display || "—")}</strong>
      <span class="muted">${finSub}</span></div>
      ${top ? `<div class="muted note">Backers: ${top}</div>` : ""}
      ${
        f.committee_url && !sponsorLabel
          ? `<div class="meta"><a href="${esc(f.committee_url)}" target="_blank" rel="noopener">${finLinkLabel}</a></div>`
          : ""
      }`;
  } else {
    sponsor = `<div class="muted">${esc(
      (f.note && f.note.length > 100 ? f.note.slice(0, 97) + "…" : f.note) ||
        "No support committee / itemized $."
    )}</div>
      ${
        f.committee_url
          ? `<div class="meta"><a href="${esc(f.committee_url)}" target="_blank" rel="noopener">${finLinkLabel}</a></div>`
          : ""
      }`;
  }
  const opposeList = Array.isArray(f.oppose) ? f.oppose : [];
  let oppose;
  if (opposeList.length) {
    oppose = opposeList
      .map((o) => {
        const oname = o.committee_name || o.account || "PAC";
        const oSum = o.contributions_sum_display || "—";
        const oLines = Number(o.line_count) || 0;
        const link = o.committee_url
          ? `<a href="${esc(o.committee_url)}" target="_blank" rel="noopener">${esc(
              oname.length > 36 ? oname.slice(0, 33) + "…" : oname
            )}</a>`
          : esc(oname.length > 36 ? oname.slice(0, 33) + "…" : oname);
        return `<div class="measure-backer"><span class="endorsement-stance endorsement-oppose">oppose</span> ${link}${
          oLines > 0 || (oSum && oSum !== "—")
            ? ` · <strong>${esc(oSum)}</strong>`
            : ""
        }</div>`;
      })
      .join("");
  } else {
    oppose = `<span class="muted">—</span>`;
  }
  return { sponsor, oppose };
}

function contributorTable(rows, emptyMsg) {
  const list = Array.isArray(rows) ? rows : [];
  if (!list.length) {
    return `<p class="muted empty">${esc(emptyMsg || "No itemized donors listed.")}</p>`;
  }
  const body = list
    .map((c) => {
      const gifts = c.gift_count ? ` · ${esc(String(c.gift_count))} gifts` : "";
      return `<tr>
        <td>${esc(c.name || "—")}</td>
        <td class="num">${esc(c.amount_display || "—")}</td>
        <td class="muted">${esc(c.city || c.employer || "")}${gifts}</td>
      </tr>`;
    })
    .join("");
  return `<table class="detail-list-table measure-donors-table">
    <thead><tr><th>Donor / backer</th><th>Amount</th><th>Notes</th></tr></thead>
    <tbody>${body}</tbody>
  </table>`;
}

function measureSideCard(side, stance) {
  if (!side) return "";
  const name = side.committee_name || side.account || "Committee";
  const sum = side.contributions_sum_display || "—";
  const note = side.note ? `<p class="muted note">${esc(side.note)}</p>` : "";
  const url = side.committee_url || side.trefin_url || side.profile_url || "";
  const link = url
    ? `<p class="meta"><a href="${esc(url)}" target="_blank" rel="noopener">${measureFinLinkLabel(side)} ${esc(name)}</a></p>`
    : "";
  const top = contributorTable(
    side.top_contributors,
    "No top contributors listed for this committee."
  );
  return `<div class="measure-side-card">
    <p class="measure-side-head">
      <span class="endorsement-stance endorsement-${esc(stance)}">${esc(stance)}</span>
      <strong>${esc(name)}</strong>
      <span class="measure-side-sum">${esc(sum)}</span>
    </p>
    ${link}
    ${note}
    ${top}
  </div>`;
}

/**
 * Full measure detail page (ballot click-through).
 * @param {object} m measure row (may include .finance from progressive enrich)
 * @param {{ election_name?: string, election_date?: string, zip?: string }} [meta]
 */
export function renderMeasureDetail(m, meta = {}) {
  if (!m) {
    return `<article class="measure-detail"><p class="empty">Measure not found.</p></article>`;
  }
  const code = m.measure_code ? esc(m.measure_code) : "";
  const title = esc(m.title || "Ballot measure");
  const heading = code ? `${code}: ${title}` : title;
  const electionBits = [meta.election_name, meta.election_date, m.jurisdiction]
    .filter(Boolean)
    .map(esc)
    .join(" · ");
  const summary = m.summary
    ? `<p class="measure-summary">${esc(m.summary)}</p>`
    : `<p class="muted empty">No summary yet${
        m.finance ? "" : " — still loading sources…"
      }.</p>`;
  const srcLink = [
    m.source_url
      ? `<a href="${esc(m.source_url)}" target="_blank" rel="noopener">Official measure page →</a>`
      : "",
    m.ballotpedia_url
      ? `<a href="${esc(m.ballotpedia_url)}" target="_blank" rel="noopener">Ballotpedia →</a>`
      : "",
  ]
    .filter(Boolean)
    .join(" · ");
  const srcLine = srcLink ? `<p class="meta">${srcLink}</p>` : "";

  const f = m.finance;
  let financeHtml;
  if (!f) {
    financeHtml = `<p class="muted empty" id="measure-finance-status">Loading committee finance…</p>`;
  } else {
    const srcLabel =
      f.source_label ||
      (f.source === "ftm_measure"
        ? "FollowTheMoney"
        : f.source === "md_cf_measure"
          ? "MDCRIS"
          : f.source === "fl_trefin"
            ? "FL TreFin"
            : f.source || "public filings");
    const totals = [];
    if (f.contributions_sum_display) {
      totals.push(
        `<div class="measure-total"><span class="muted">Support $</span><strong>${esc(
          f.contributions_sum_display
        )}</strong></div>`
      );
    }
    const opposeSides = Array.isArray(f.oppose) ? f.oppose : [];
    if (opposeSides.length) {
      const sideSum = opposeSides.reduce(
        (acc, o) => acc + (Number(o.contributions_sum) || 0),
        0
      );
      const firstDisplay = opposeSides
        .map((o) => o.contributions_sum_display)
        .find(Boolean);
      const display =
        sideSum > 0 && opposeSides.length > 1
          ? firstDisplay || String(sideSum)
          : firstDisplay ||
            (f.oppose_total != null && Number(f.oppose_total) > 0
              ? String(f.oppose_total)
              : "");
      if (display) {
        totals.push(
          `<div class="measure-total"><span class="muted">Oppose $</span><strong>${esc(
            display
          )}</strong></div>`
        );
      }
    } else if (f.oppose_total != null && Number(f.oppose_total) > 0) {
      totals.push(
        `<div class="measure-total"><span class="muted">Oppose $</span><strong>${esc(
          String(f.oppose_total)
        )}</strong></div>`
      );
    }

    const supportCard = measureSideCard(
      {
        committee_name: f.committee_name || "Support committees",
        contributions_sum_display: f.contributions_sum_display,
        top_contributors: f.top_contributors,
        committee_url: f.committee_url || f.profile_url || f.show_me_url,
        trefin_url: f.trefin_url,
        note: f.note,
        source: f.source,
      },
      "support"
    );
    const opposeCards = opposeSides.length
      ? opposeSides.map((o) => measureSideCard(o, "oppose")).join("")
      : `<p class="muted empty">No oppose committee $ found in sources we check.</p>`;

    const extraLinks = [
      f.profile_url
        ? `<a href="${esc(f.profile_url)}" target="_blank" rel="noopener">Profile →</a>`
        : "",
      f.show_me_url
        ? `<a href="${esc(f.show_me_url)}" target="_blank" rel="noopener">Show Me →</a>`
        : "",
      f.trefin_url && f.trefin_url !== f.committee_url
        ? `<a href="${esc(f.trefin_url)}" target="_blank" rel="noopener">TreFin →</a>`
        : "",
    ]
      .filter(Boolean)
      .join(" · ");

    financeHtml = `
      <p class="meta muted">Source: ${esc(srcLabel)}${
        Number(f.line_count) ? ` · ${esc(String(f.line_count))} itemized line(s)` : ""
      }</p>
      ${totals.length ? `<div class="measure-totals">${totals.join("")}</div>` : ""}
      ${f.note ? `<p class="muted note">${esc(f.note)}</p>` : ""}
      <h3 class="measure-sec-h">Support</h3>
      ${supportCard}
      <h3 class="measure-sec-h">Oppose</h3>
      ${opposeCards}
      ${extraLinks ? `<p class="meta measure-extra-links">${extraLinks}</p>` : ""}
    `;
  }

  return `
    <article class="measure-detail" id="measure-detail" data-measure-id="${esc(String(m.id))}">
      <header class="detail-header measure-detail-header">
        <p class="crumb"><a href="#" id="back-ballot">← Back to ballot</a></p>
        <h1 class="detail-name">${heading}</h1>
        ${electionBits ? `<p class="meta muted detail-header-line">${electionBits}</p>` : ""}
        ${srcLine}
        <p class="verdict-actions">${reloadButtonsHtml("measure", m.id)}</p>
      </header>
      <section class="card measure-section" id="measure-sec-overview">
        <h2>Overview</h2>
        ${summary}
      </section>
      <section class="card measure-section" id="measure-sec-endorsements">
        <h2>Endorsements</h2>
        ${measureEndorsementsHtml(m)}
      </section>
      <section class="card measure-section" id="measure-sec-finance">
        <h2>Committee finance</h2>
        <div id="measure-finance-body">${financeHtml}</div>
      </section>
      <p class="meta muted">Cited Ballotpedia Support/Oppose lists and public committees — not a complete endorsement list. Cite or omit.</p>
    </article>`;
}

function measureEndorsementsHtml(m) {
  const ends = Array.isArray(m.endorsements) ? m.endorsements : null;
  if (!ends) {
    return `<p class="muted empty">Loading endorsements…</p>`;
  }
  if (!ends.length) {
    return `<p class="muted empty">No org or official endorsements loaded yet (Ballotpedia Support/Oppose, sponsor/oppose committees). Empty is common before campaigns publish lists.</p>`;
  }
  return `<ul class="dossier-endorsements">${ends
    .map((x) => {
      const src = x.source_url
        ? `<a href="${esc(x.source_url)}" rel="noopener noreferrer" target="_blank">${esc(x.source)}</a>`
        : esc(x.source || "");
      return `<li><span class="endorsement-stance endorsement-${esc(x.stance)}">${esc(x.stance)}</span> ${esc(x.org)} <span class="muted">· ${src}${
        x.kind ? ` · ${esc(x.kind)}` : ""
      }</span> ${trustChip(x.trust)}</li>`;
    })
    .join("")}</ul>`;
}
