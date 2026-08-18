#!/usr/bin/env node
/**
 * electionizer MCP — project tools for agent sessions.
 * Stdio transport. Repo root = parent of mcp/electionizer (or ELECTIONIZER_ROOT).
 *
 * Prefer these over blind Read/Grep sweeps:
 *   status → next → where/def/refs/rg/outline → edit → test/check → build_wasm
 */
import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import { z } from "zod";
import { spawn } from "node:child_process";
import fs from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = process.env.ELECTIONIZER_ROOT
  ? path.resolve(process.env.ELECTIONIZER_ROOT)
  : path.resolve(__dirname, "../..");

const CONTINUITY = [
  "Do not reintroduce global drilldown overlay.",
  "JS I/O only in browser; WASM pure functions (no Rust HttpClient/store for WASM).",
  "Never ship shared API secrets (user keys in localStorage only).",
  "Prefer rustup toolchain for wasm (./scripts/build-wasm.sh).",
  "State scrapes: Wisp/libcurl.js preferred over cleartext CORS proxy.",
  "FL DOS: live extract via Wisp first; TSV upload/ship is fallback only.",
  "Product path is static web/ + WASM; native Axum is optional/dev.",
  "MD SBE CSVs need Wisp (no CORS); prefer GG over GP; Baltimore City ≠ County in county_key.",
  "After core/wasm Rust changes: build wasm before browser smoke.",
  "Geo: state_house_label holds MD lettered districts (30A); numeric house still set from digits.",
  "electionizer_next reads HANDOFF plan — update HANDOFF.md, not a parallel queue.",
  "Finance stage order (locked): FL acct → name-search → SOE VF → NC → AZ → MD → FTM → profile.",
  "Affiliation: public citable signals only; no bulk voter-registration.",
];

const COVERAGE = {
  federal: "All ZIPs — House/Senate FEC; detail finance + GovTrack",
  florida:
    "Live DOS (leg/statewide/judicial/local) + chambers + amendments; candidate + measure TreFin (incl. No-on-N oppose) via Wisp",
  arizona:
    "Leg roster + statewide OfficialList incumbents + Clean Elections measures + FTM measure $ thru 2024 (SOS challengers blocked)",
  north_carolina:
    "NCSBE CSV filings + referendum measures + FTM measure $ thru 2024; federal via FEC; zero-pad OK",
  maryland:
    "SBE GG/GP CSVs + ballot questions + MDCRIS live measure $ (+ FTM archive) — statewide + leg A/B/C + judges + county local",
  other_with_os_key:
    "Federal + Open States people.geo legislature incumbents; FTM measure $ when measures present",
  other_without_key: "Federal only (+ Civic/FTM measure $ if keyed/measures)",
};

const TEST_ZIPS = {
  "33334": "FL full (live DOS)",
  "85004": "AZ roster incumbents",
  "85701": "AZ alternate",
  "27601": "NC NCSBE filings + referendums (Wake)",
  "21401": "MD SBE filings + ballot questions (Annapolis — sen 30, house 30A)",
  "90210": "CA federal + OS incumbents if key",
  "10001": "NY federal + OS incumbents if key",
};

/** Named state_bodies_json keys JS → WASM → core. */
const STATE_BODY_KEYS = {
  "fl:dos": "FL DOS TSV / live extract body",
  "fl:senate": "FL Senate roster HTML",
  "fl:house": "FL House roster HTML",
  "fl:measures": "FL constitutional initiatives HTML",
  "fl:soe": "FL county SOE VoterFocus candidate_pr.php HTML",
  "fl:sample_ballot": "FL official sample-ballot PDF text (optional precinct filter)",
  "az:senate": "AZ Senate roster HTML",
  "az:house": "AZ House roster HTML",
  "az:measures": "AZ Clean Elections BallotMeasures HTML fragment",
  "az:officials": "AZ Clean Elections OfficialList HTML (statewide + LD incumbents)",
  "nc:candidates": "NCSBE Candidate_Listing CSV",
  "nc:measures": "NCSBE referendum list plain text (PDF extract)",
  "nc:measures_url": "NCSBE referendum PDF URL (cites)",
  "md:statewide": "MD SBE statewide candidatelist CSV",
  "md:local": "MD SBE all_counties candidatelist CSV",
  "md:phase": "GG or GP (source URL labeling)",
  "md:measures": "MD SBE ballot_questions.html (statewide + county)",
  "civic:voterinfo": "Google Civic voterInfoQuery JSON (VIP multi-state ballot)",
  "os:people.geo": "Open States people.geo JSON",
};

const STATE_MODULES = ["florida", "arizona", "north_carolina", "maryland"];

/** Locked product knowledge so agents don't re-derive from source every session. */
const STAGES = {
  finance_order: [
    "fl_trefin (FL acct)",
    "fl_name_search",
    "fl_soe_cf (VoterFocus)",
    "nc_cf",
    "az_cf",
    "md_cf",
    "ftm (keyed)",
    "profile",
  ],
  federal_detail: [
    "totals",
    "principal",
    "indiv",
    "fec_occupation (Sched A self-contributor occ/employer)",
    "cmte",
    "size",
    "outside",
    "member (congress-legislators → affiliations + bioguide photo)",
    "official_about (CL term url → house.gov/senate.gov About via Wisp)",
    "ballotpedia_bio (CL id or H6 name+state+office title guess → education/profession/family)",
    "campaign_about (BP campaign link or campaign source_url → About; fill gaps)",
    "wiki_extract (plain extract → fill gaps only)",
    "dbpedia (N-Triples infobox → fill gaps only)",
    "grokipedia (typeahead + page HTML via Wisp; fill gaps; no family/citizenship)",
    "wikidata_bio (CL wikidata id → facts/spans)",
    "wiki_photo (Wikipedia REST thumb if no photo)",
    "senate_efd (Senate only — eFD annual Part 3 assets → holdings; Wisp session)",
    "house_clerk_fd (House only — Clerk FD Schedule A PDF → holdings; Wisp session)",
    "votes (GovTrack)",
    "bp_endorsements → campaign_endorsements → gdelt_news → news_rss → money_signals → bp_claims → campaign_claims → claim_contrasts → llm_contrasts → fl_bar → fl_ethics → fl_jqc → ai_verdict (Track L + M; llm optional user key; verdict Responses + live search; home voter profile remaps fit)",
  ],
  /** J1 dense bio + Track K judicial CourtListener */
  non_federal_detail: [
    "CF first (finance_order — FL/NC/AZ/MD/FTM as applicable)",
    "fl_chamber_bio (FL Senate/House profile URL only)",
    "fl_courts_bio (FL judges — SC/DCA + circuits 2/4/5/7/10/12/15/17/18/20; official photo+edu+career when pages exist)",
    "cl_person (judges — CourtListener people match; optional token)",
    "cl_positions (judges — practice + bench spans → Legal/Political)",
    "os_resolve (state_senate/state_house + Open States key only)",
    "ballotpedia_bio (name+state+office title candidates → person match)",
    "campaign_about",
    "wiki_extract (CL/OS title or J3 name-title guess via summary match)",
    "dbpedia / grokipedia / wikidata_bio / wiki_photo",
    "cl_opinions (judges — authored opinions → Decisions tab / enrich.votes; cap 100)",
    "os_votes (state leg + key only)",
    "bp_endorsements → campaign_endorsements → gdelt_news → news_rss → money_signals → bp_claims → campaign_claims → claim_contrasts → llm_contrasts → fl_bar → fl_ethics → fl_jqc → ai_verdict (Track L + M; llm optional user key; verdict Responses + live search; home voter profile remaps fit)",
  ],
  state_leg_detail_extra: [
    "os_resolve (match + detail fetch + extras facts) — leg only",
    "dense bio hosts (same as all non-federal)",
    "os_votes — leg only",
  ],
  judicial_detail_extra: [
    "fl_courts_bio (FL)",
    "cl_person → cl_positions → dense bio → cl_opinions",
    "no os_votes; Votes tab label = Decisions",
  ],
  affiliation: {
    init: "ballot_affiliations_js(party, office, is_incumbent, is_judge, source_publisher, source_url)",
    federal_overwrite: "member stage → match_legislator_by_fec_js → CL term spans with cites",
    os_merge:
      "os_resolve → pick_openstates_person_js.affiliations → merge_affiliation_spans_js (ballot kept)",
    judicial:
      "is_judge → merit retention / nonpartisan / party designation; Group N (seat) in role",
    b6_committee:
      "campaign_committee_affiliation_js — party=Committee, role ≠ voter affiliation (FEC principal + FL TreFin + NC/AZ/MD CF + FTM)",
    next: "Coverage/smoke or new state filings",
    rules: [
      "Cite or omit — no span without source name; URL when known",
      "No silent merge of conflicting parties",
      "Ambiguous person match → skip span",
      "No bulk voter-registration",
    ],
  },
  browser_pipeline:
    "ZIP → Zippo → TIGERweb+FCC → FEC → state bodies → WASM build_live_ballot_report → progressive measure finance (FL TreFin → FTM measures) → measure endorsements (BP Support/Oppose) → background ballot score queue (M-early per item) → click → Verdict / Details enrich",
};

const WHERE = {
  ballot: [
    "web/live.js — browser pipeline",
    "web/state.js — FL/AZ/NC/MD fetch bodies",
    "crates/electionizer-core/src/federal.rs — ballot_report_from_live_with_state",
    "crates/electionizer-core/src/state_ballot.rs — extras_from_state_bodies + apply",
    "crates/electionizer-wasm/src/lib.rs — build_live_ballot_report",
    "web/render.js — ballot + progressive detail DOM",
    "web/scoreboard.js — ballot fit queue + party scorecard",
  ],
  florida: [
    "web/state.js — fetchFlDosLive, SOE VF list, sample ballot, measures, chambers, TreFin finance",
    "crates/electionizer-core/src/states/florida.rs — TSV/roster/measures/TreFin parsers",
    "crates/electionizer-core/src/states/florida_soe.rs — VoterFocus SOE ballot locals + finance",
    "crates/electionizer-core/src/state_ballot.rs — fl_extras_from_bodies",
    "scripts/refresh-fl-dos.sh — offline TSV fallback only",
  ],
  arizona: [
    "web/state.js + web/state-urls.js — roster fetch",
    "crates/electionizer-core/src/states/arizona.rs — parse_roster + measures + SeeTheMoney CF",
    "web/state.js — fetchAzMeasuresHtml (Clean Elections)",
    "crates/electionizer-core/src/state_ballot.rs — az_extras_from_rosters",
  ],
  north_carolina: [
    "web/state.js — NCSBE CSV + referendum PDF fetch",
    "crates/electionizer-core/src/states/north_carolina.rs — parse/map + measures + NCSBE CF",
    "crates/electionizer-core/src/state_ballot.rs — nc_extras_from_csv",
  ],
  maryland: [
    "web/state.js — MD SBE GG/GP CSV fetch (Wisp required)",
    "crates/electionizer-core/src/states/maryland.rs — parse/map + MDCRIS CF",
    "crates/electionizer-core/src/state_ballot.rs — md_extras_from_csv",
    "crates/electionizer-core/src/federal.rs — state_house_label (30A)",
    "Smoke: 21401 Annapolis",
  ],
  geo: [
    "crates/electionizer-core/src/federal.rs — parse_tigerweb_identify_json, CensusGeo",
    "crates/electionizer-core/src/models.rs — GeoResolution.state_house_label",
    "web/live.js — tigerwebIdentifyUrl, FCC area",
  ],
  wisp: [
    "web/curl-transport.js — libcurl.js load + fetch/post",
    "web/settings.js — DEFAULT_WISP_URL, get/set Wisp",
    "web/app.js — Settings UI Default / This origin",
    "scripts/run-static.sh — local wisp-python + static",
  ],
  openstates: [
    "web/live.js — people.geo fetch when key set",
    "crates/electionizer-core/src/state_ballot.rs — openstates_extras_from_people_geo",
    "crates/electionizer-core/src/openstates.rs — pick_person, votes helpers",
    "web/enrich.js — os_resolve / os_votes stages",
  ],
  detail: [
    "web/enrich.js — staged FEC/GovTrack/OS/state CF",
    "web/render.js — patchDetailSections, renderAffBody, renderFinanceBody",
    "web/app.js — openDetail",
    "crates/electionizer-core/src/{fec,govtrack,openstates,ftm}.rs",
    "src/detail.rs — native SSE enrich (dev)",
  ],
  affiliation: [
    "crates/electionizer-core/src/models.rs — AffiliationSpan (source, source_url)",
    "crates/electionizer-core/src/govtrack.rs — ballot_affiliations, build_fec_index CL spans",
    "crates/electionizer-wasm/src/lib.rs — ballot_affiliations_js",
    "web/enrich.js — init affiliations + member CL overwrite",
    "web/render.js — renderAffBody + Source column",
    "crates/electionizer-core/src/openstates.rs — affiliations_from_openstates_person, merge_affiliation_spans",
    "HANDOFF.md — B roadmap (B1–B5 done; B4/B6 optional)",
  ],
  finance: [
    "web/enrich.js — planStages + fl_trefin/name_search/soe/nc/az/md/ftm",
    "crates/electionizer-core/src/fec.rs — OpenFEC totals/sched A",
    "crates/electionizer-core/src/states/florida.rs — TreFin + name-search",
    "crates/electionizer-core/src/states/florida_soe.rs — VoterFocus",
    "crates/electionizer-core/src/states/{north_carolina,arizona,maryland}.rs — state CF",
    "crates/electionizer-core/src/ftm.rs — FollowTheMoney",
    "web/render.js — renderFinanceBody",
  ],
  measures: [
    "web/state.js — enrichFlMeasureSummaries + enrichFlMeasureFinance + enrichMdMeasureFinance + enrichFtmMeasureFinance + enrichMeasureEndorsements",
    "crates/electionizer-core/src/scrutiny.rs — BP measure index match + Support/Oppose lists",
    "crates/electionizer-core/src/states/florida.rs — measures, TreFin, oppose PAC",
    "crates/electionizer-core/src/states/maryland.rs — MDCRIS ballot-issue measure $ (E2)",
    "crates/electionizer-core/src/ftm.rs — FTM ballot measure list/overview/donors (E1)",
    "web/render.js — measures Sponsor $ / Oppose $ + detail Endorsements",
    "web/app.js — progressiveMeasureEnrich (FL TreFin → MD MDCRIS → FTM → BP endorsements)",
  ],
  wasm: [
    "scripts/build-wasm.sh",
    "crates/electionizer-wasm/src/lib.rs — all #[wasm_bindgen] exports",
    "web/pkg/ (gitignored output)",
    "Use electionizer_exports to list bindgen surface",
  ],
  native: [
    "src/main.rs, src/web.rs, src/providers/, src/detail.rs",
    "templates/, migrations/, electionizer.example.toml",
  ],
  deploy: [
    "netlify.toml",
    "scripts/build-wasm.sh (CI bootstrap)",
    "README.md Deploy section",
  ],
  settings: ["web/settings.js", "web/index.html Settings view", "web/app.js loadSettingsForm"],
  mcp: [
    "mcp/electionizer/server.js — tools",
    "mcp/electionizer/README.md",
    "opencode.json — mcp + lsp wire-up",
    "HANDOFF.md — plan source of truth",
  ],
  models: [
    "crates/electionizer-core/src/models.rs — CandidateSummary, BallotCandidateRow, AffiliationSpan, GeoResolution, SnapshotCandidate",
    "Ballot path: SnapshotCandidate → BallotCandidateRow → CandidateSummary (source_url + source_publisher)",
  ],
  fec: [
    "crates/electionizer-core/src/fec.rs — pure parsers",
    "crates/electionizer-core/src/federal.rs — map FEC → ballot",
    "web/enrich.js — totals/indiv/cmte/outside stages",
  ],
  govtrack: [
    "crates/electionizer-core/src/govtrack.rs — CL index, affiliations, votes assemble",
    "web/enrich.js — member + votes stages",
  ],
};

const TOPIC_KEYS = Object.keys(WHERE);

function text(s) {
  return { content: [{ type: "text", text: String(s) }] };
}

function errText(s) {
  return { content: [{ type: "text", text: String(s) }], isError: true };
}

async function exists(p) {
  try {
    await fs.access(p);
    return true;
  } catch {
    return false;
  }
}

function run(cmd, args, { cwd = ROOT, timeoutMs = 300_000, env } = {}) {
  return new Promise((resolve) => {
    const child = spawn(cmd, args, {
      cwd,
      env: env ? { ...process.env, ...env } : process.env,
      shell: false,
    });
    let out = "";
    let err = "";
    const t = setTimeout(() => {
      child.kill("SIGTERM");
      resolve({
        code: -1,
        stdout: out,
        stderr: err + `\n[timeout after ${timeoutMs}ms]`,
      });
    }, timeoutMs);
    child.stdout?.on("data", (d) => {
      out += d.toString();
    });
    child.stderr?.on("data", (d) => {
      err += d.toString();
    });
    child.on("close", (code) => {
      clearTimeout(t);
      resolve({ code: code ?? 1, stdout: out, stderr: err });
    });
    child.on("error", (e) => {
      clearTimeout(t);
      resolve({ code: 1, stdout: out, stderr: String(e) });
    });
  });
}

function clip(s, max = 12_000) {
  if (!s) return "";
  if (s.length <= max) return s;
  return s.slice(0, max) + `\n… [truncated ${s.length - max} chars]`;
}

/** Resolve path under ROOT; reject escapes. */
function underRoot(rel) {
  if (!rel || rel === "." || rel === "./") return ROOT;
  const abs = path.resolve(ROOT, rel);
  const root = ROOT.endsWith(path.sep) ? ROOT : ROOT + path.sep;
  if (abs !== ROOT && !abs.startsWith(root)) {
    throw new Error(`path escapes repo root: ${rel}`);
  }
  return abs;
}

function relFromRoot(abs) {
  return path.relative(ROOT, abs) || ".";
}

async function readHandoff() {
  const p = path.join(ROOT, "HANDOFF.md");
  if (!(await exists(p))) return null;
  return fs.readFile(p, "utf8");
}

/** Extract a ## section from HANDOFF (through next ## or EOF). */
function handoffSection(body, headingPrefix) {
  const start = body.indexOf(headingPrefix);
  if (start < 0) return null;
  const rest = body.slice(start);
  const cut = rest.indexOf("\n## ", 1);
  return (cut > 0 ? rest.slice(0, cut) : rest).trim();
}

async function gitDirtySummary() {
  const st = await run("git", ["status", "-sb"], { timeoutMs: 15_000 });
  const log = await run("git", ["log", "--oneline", "-5"], { timeoutMs: 15_000 });
  return {
    status: st.stdout.trim() || st.stderr.trim(),
    recent: log.stdout.trim(),
  };
}

async function findRg() {
  const candidates = ["rg", "/opt/homebrew/bin/rg", "/usr/local/bin/rg"];
  for (const c of candidates) {
    const r = await run(c, ["--version"], { timeoutMs: 5_000 });
    if (r.code === 0) return c;
  }
  return null;
}

async function rgSearch({
  pattern,
  path: searchPath = ".",
  glob,
  type,
  context = 0,
  maxCount = 40,
  caseInsensitive = false,
  fixed = false,
  filesOnly = false,
}) {
  const rg = await findRg();
  if (!rg) return { code: 1, stdout: "", stderr: "rg (ripgrep) not found on PATH" };

  let abs;
  try {
    abs = underRoot(searchPath);
  } catch (e) {
    return { code: 1, stdout: "", stderr: String(e.message || e) };
  }

  const args = ["--no-heading", "--line-number", "--color", "never"];
  if (filesOnly) args.push("-l");
  else {
    args.push("--max-count", String(Math.max(1, Math.min(maxCount, 200))));
    if (context > 0) {
      args.push("-C", String(Math.min(context, 5)));
    }
  }
  if (caseInsensitive) args.push("-i");
  if (fixed) args.push("-F");
  if (glob) args.push("--glob", glob);
  if (type) args.push("--type", type);
  // Always skip heavy/generated dirs
  for (const g of [
    "!**/target/**",
    "!**/node_modules/**",
    "!**/web/pkg/**",
    "!**/.venv-wisp/**",
    "!**/*.db",
  ]) {
    args.push("--glob", g);
  }
  args.push("--", pattern, abs);

  const r = await run(rg, args, { timeoutMs: 30_000 });
  // rg exit 1 = no matches
  if (r.code === 1 && !r.stderr.trim()) {
    return { code: 0, stdout: "(no matches)", stderr: "" };
  }
  // Rewrite absolute paths to repo-relative for readability
  const root = ROOT.endsWith(path.sep) ? ROOT : ROOT + path.sep;
  const stdout = (r.stdout || "")
    .split("\n")
    .map((line) => (line.startsWith(root) ? line.slice(root.length) : line))
    .join("\n");
  return { ...r, stdout };
}

function escapeRegExp(s) {
  return s.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

const server = new McpServer({
  name: "electionizer",
  version: "0.3.0",
});

// —— status ——
server.tool(
  "electionizer_status",
  "Snapshot of electionizer repo health: paths, wasm build presence, git dirty, state modules, coverage. Call at session start.",
  {},
  async () => {
    const checks = {
      root: ROOT,
      handoff: await exists(path.join(ROOT, "HANDOFF.md")),
      readme: await exists(path.join(ROOT, "README.md")),
      wasm_js: await exists(path.join(ROOT, "web/pkg/electionizer_wasm.js")),
      wasm_bin: await exists(path.join(ROOT, "web/pkg/electionizer_wasm_bg.wasm")),
      build_script: await exists(path.join(ROOT, "scripts/build-wasm.sh")),
      run_static: await exists(path.join(ROOT, "scripts/run-static.sh")),
      curl_transport: await exists(path.join(ROOT, "web/curl-transport.js")),
      core: await exists(path.join(ROOT, "crates/electionizer-core/src/lib.rs")),
      native_src: await exists(path.join(ROOT, "src/main.rs")),
      netlify: await exists(path.join(ROOT, "netlify.toml")),
      venv_wisp: await exists(path.join(ROOT, ".venv-wisp")),
      rust_analyzer: await exists(
        path.join(process.env.HOME || "", ".cargo/bin/rust-analyzer")
      ),
      rg: !!(await findRg()),
      ts_lsp: await exists(
        path.join(ROOT, "node_modules/typescript-language-server/lib/cli.mjs")
      ),
    };
    const modules = {};
    for (const m of STATE_MODULES) {
      modules[m] = await exists(
        path.join(ROOT, `crates/electionizer-core/src/states/${m}.rs`)
      );
    }
    const git = await gitDirtySummary();
    const lines = [
      `# electionizer status`,
      `root: ${ROOT}`,
      ``,
      `## artifacts`,
      ...Object.entries(checks).map(([k, v]) =>
        `- ${k}: ${v === true || v === false ? (v ? "yes" : "NO") : v}`
      ),
      ``,
      `## state modules (core)`,
      ...Object.entries(modules).map(([k, v]) => `- ${k}: ${v ? "yes" : "NO"}`),
      ``,
      `## git`,
      "```",
      git.status,
      "```",
      ``,
      `recent:`,
      "```",
      git.recent || "(none)",
      "```",
      ``,
      `## coverage (quick)`,
      ...Object.entries(COVERAGE).map(([k, v]) => `- **${k}**: ${v}`),
      ``,
      `## test zips`,
      ...Object.entries(TEST_ZIPS).map(([z, n]) => `- \`${z}\` — ${n}`),
      ``,
      `## session tip`,
      `- electionizer_next → electionizer_where / def / refs / rg (avoid full-file reads)`,
      `- After Rust: electionizer_check or electionizer_test → electionizer_build_wasm if wasm path`,
      `- LSP: rust-analyzer + typescript-language-server (restart opencode after config change)`,
    ];
    return text(lines.join("\n"));
  }
);

// —— handoff / plan ——
server.tool(
  "electionizer_handoff",
  "Read HANDOFF.md (or a section). Use for continuity and next-session plan.",
  {
    section: z
      .enum(["all", "plan", "rules", "architecture", "run", "done", "gaps"])
      .optional()
      .describe("Subset to return; default all"),
  },
  async ({ section = "all" }) => {
    const body = await readHandoff();
    if (!body) return errText("HANDOFF.md missing");
    if (section === "all") return text(body);

    const markers = {
      plan: "## Plan for next session",
      rules: "## Continuity rules",
      architecture: "## Architecture",
      run: "## How to run",
      done: "## Done recently",
      gaps: "## Known gaps",
    };
    const slice = handoffSection(body, markers[section]);
    return text(slice || body);
  }
);

// —— rules ——
server.tool(
  "electionizer_rules",
  "Return continuity / architecture rules agents must not violate.",
  {},
  async () => {
    const body = await readHandoff();
    const fromFile = body && handoffSection(body, "## Continuity rules");
    if (fromFile) {
      return text(
        [
          fromFile,
          "",
          "## MCP baked-in extras",
          ...CONTINUITY.filter(
            (r) => !fromFile.toLowerCase().includes(r.slice(0, 40).toLowerCase())
          ).map((r) => `- ${r}`),
        ].join("\n")
      );
    }
    return text(
      ["# Continuity rules", ...CONTINUITY.map((r, i) => `${i + 1}. ${r}`)].join(
        "\n"
      )
    );
  }
);

// —— arch check ——
server.tool(
  "electionizer_arch_check",
  "Static checks for WASM purity and project invariants (HttpClient in wasm crate, secrets patterns, pkg presence, state modules).",
  {},
  async () => {
    const findings = [];

    const wasmToml = path.join(ROOT, "crates/electionizer-wasm/Cargo.toml");
    if (await exists(wasmToml)) {
      const t = await fs.readFile(wasmToml, "utf8");
      for (const bad of ["axum", "sqlx", "reqwest", "tokio"]) {
        if (new RegExp(`^\\s*${bad}\\s*=`, "m").test(t)) {
          findings.push(`FAIL: electionizer-wasm Cargo.toml depends on ${bad}`);
        }
      }
    }

    const coreToml = path.join(ROOT, "crates/electionizer-core/Cargo.toml");
    if (await exists(coreToml)) {
      const t = await fs.readFile(coreToml, "utf8");
      for (const bad of ["axum", "sqlx", "reqwest"]) {
        if (new RegExp(`^\\s*${bad}\\s*=`, "m").test(t)) {
          findings.push(`FAIL: electionizer-core Cargo.toml depends on ${bad}`);
        }
      }
    }

    const wasmSrc = path.join(ROOT, "crates/electionizer-wasm/src");
    const grepped = await rgSearch({
      pattern: "HttpClient|reqwest|sqlx",
      path: relFromRoot(wasmSrc),
      maxCount: 20,
    });
    if (grepped.stdout && grepped.stdout !== "(no matches)") {
      findings.push(`FAIL: wasm src mentions I/O stacks:\n${grepped.stdout.trim()}`);
    }

    for (const f of ["web/live.js", "web/state.js", "web/curl-transport.js"]) {
      if (!(await exists(path.join(ROOT, f)))) {
        findings.push(`WARN: missing ${f}`);
      }
    }

    for (const m of STATE_MODULES) {
      if (
        !(await exists(
          path.join(ROOT, `crates/electionizer-core/src/states/${m}.rs`)
        ))
      ) {
        findings.push(`WARN: missing states/${m}.rs`);
      }
    }

    if (await exists(path.join(ROOT, "crates/electionizer-core/src/states/maryland.rs"))) {
      const live = await fs.readFile(path.join(ROOT, "web/live.js"), "utf8");
      if (!live.includes("md:statewide") || !live.includes("md:local")) {
        findings.push("WARN: maryland.rs present but web/live.js missing md: body keys");
      }
      const sb = await fs.readFile(
        path.join(ROOT, "crates/electionizer-core/src/state_ballot.rs"),
        "utf8"
      );
      if (!sb.includes("md:statewide") && !sb.includes("BODY_MD_STATEWIDE")) {
        findings.push("WARN: maryland.rs present but state_ballot missing MD bodies");
      }
    }

    if (!(await exists(path.join(ROOT, "web/pkg/electionizer_wasm.js")))) {
      findings.push("WARN: web/pkg missing — run electionizer_build_wasm");
    }

    if (await exists(path.join(ROOT, ".env"))) {
      findings.push("NOTE: .env present locally (should stay gitignored)");
    }

    const ok = findings.filter((f) => f.startsWith("FAIL")).length === 0;
    const lines = [
      `# arch_check ${ok ? "PASS" : "FAIL"}`,
      ...(findings.length ? findings.map((f) => `- ${f}`) : ["- no issues"]),
      ``,
      `state_bodies keys:`,
      ...Object.entries(STATE_BODY_KEYS).map(([k, v]) => `- \`${k}\` — ${v}`),
      ``,
      `rules:`,
      ...CONTINUITY.map((r) => `- ${r}`),
    ];
    return text(lines.join("\n"));
  }
);

// —— build wasm ——
server.tool(
  "electionizer_build_wasm",
  "Run ./scripts/build-wasm.sh (rustup + wasm-pack → web/pkg/). Use after core/wasm Rust changes.",
  {},
  async () => {
    const script = path.join(ROOT, "scripts/build-wasm.sh");
    if (!(await exists(script))) return errText("scripts/build-wasm.sh missing");
    const r = await run("bash", [script], { timeoutMs: 600_000 });
    const body = [
      `# build-wasm exit ${r.code}`,
      clip(r.stdout),
      r.stderr ? `--- stderr ---\n${clip(r.stderr)}` : "",
    ]
      .filter(Boolean)
      .join("\n");
    return r.code === 0 ? text(body) : errText(body);
  }
);

// —— test ——
server.tool(
  "electionizer_test",
  "Run cargo tests. Default: electionizer-core lib tests (fast). workspace=true for full workspace.",
  {
    workspace: z
      .boolean()
      .optional()
      .describe("If true, cargo test --workspace; else -p electionizer-core --lib"),
  },
  async ({ workspace = false }) => {
    const args = workspace
      ? ["test", "--workspace"]
      : ["test", "-p", "electionizer-core", "--lib"];
    const r = await run("cargo", args, { timeoutMs: 600_000 });
    const body = [
      `# cargo ${args.join(" ")} → ${r.code}`,
      clip(r.stdout + "\n" + r.stderr, 16_000),
    ].join("\n");
    return r.code === 0 ? text(body) : errText(body);
  }
);

// —— cargo check (fast diagnostics without full test) ——
server.tool(
  "electionizer_check",
  "Fast cargo check diagnostics (core by default). Prefer over reading whole crates when verifying compiles.",
  {
    target: z
      .enum(["core", "wasm", "native", "workspace"])
      .optional()
      .describe("What to check; default core"),
  },
  async ({ target = "core" }) => {
    const args =
      target === "workspace"
        ? ["check", "--workspace", "--message-format=short"]
        : target === "wasm"
          ? ["check", "-p", "electionizer-wasm", "--message-format=short"]
          : target === "native"
            ? ["check", "-p", "electionizer", "--message-format=short"]
            : ["check", "-p", "electionizer-core", "--message-format=short"];
    const r = await run("cargo", args, { timeoutMs: 300_000 });
    const body = [`# cargo ${args.join(" ")} → ${r.code}`, clip(r.stdout + "\n" + r.stderr, 14_000)].join(
      "\n"
    );
    return r.code === 0 ? text(body) : errText(body);
  }
);

// —— serve hints ——
server.tool(
  "electionizer_serve_help",
  "How to serve the static app and optional Wisp; does not start long-running servers.",
  {},
  async () => {
    return text(`# Serve electionizer static

## Plain static (default public Wisp)
\`\`\`bash
cd ${ROOT}
./scripts/build-wasm.sh   # if web/pkg missing
python3 -m http.server 8080 --directory web
# http://127.0.0.1:8080/
\`\`\`

## Local static + Wisp (preferred for FL/AZ/MD)
\`\`\`bash
./scripts/run-static.sh
# Settings → Wisp → This origin
\`\`\`

## Smoke ZIPs
${Object.entries(TEST_ZIPS)
  .map(([z, n]) => `- ${z}: ${n}`)
  .join("\n")}

## After Rust changes
1. electionizer_check or electionizer_test
2. electionizer_build_wasm if wasm path touched
3. hard-refresh browser (SW may cache pkg)
`);
  }
);

// —— keys / payload map ——
server.tool(
  "electionizer_body_keys",
  "List state_bodies_json keys (JS → WASM → core) and which states use them.",
  {},
  async () => {
    return text(
      [
        `# state_bodies_json keys`,
        ``,
        ...Object.entries(STATE_BODY_KEYS).map(([k, v]) => `- \`${k}\` — ${v}`),
        ``,
        `## fetch owners`,
        `- FL/AZ/MD: web/state.js fetchStateBodies (Wisp preferred)`,
        `- NC: web/state.js direct S3 CSV (CORS-open)`,
        `- OS: web/live.js when key set and state not already rich`,
        `- apply: crates/electionizer-core/src/state_ballot.rs extras_from_state_bodies`,
      ].join("\n")
    );
  }
);

// —— stages (product knowledge, no file I/O) ——
server.tool(
  "electionizer_stages",
  "Locked detail/finance/affiliation stage order and browser pipeline. Use instead of re-reading enrich.js for order.",
  {},
  async () => {
    return text(
      [
        `# Stages / pipeline (locked product knowledge)`,
        ``,
        `## Browser pipeline`,
        STAGES.browser_pipeline,
        ``,
        `## Finance stage order (locked)`,
        ...STAGES.finance_order.map((s, i) => `${i + 1}. ${s}`),
        ``,
        `## Federal detail stages`,
        ...STAGES.federal_detail.map((s, i) => `${i + 1}. ${s}`),
        ``,
        `## Non-federal detail (after CF — statewide / judicial / local / leg)`,
        ...STAGES.non_federal_detail.map((s, i) => `${i + 1}. ${s}`),
        ``,
        `## State leg extras only`,
        ...STAGES.state_leg_detail_extra.map((s, i) => `${i + 1}. ${s}`),
        ``,
        `## Affiliation`,
        `- init: ${STAGES.affiliation.init}`,
        `- federal: ${STAGES.affiliation.federal_overwrite}`,
        `- next: ${STAGES.affiliation.next}`,
        `- rules:`,
        ...STAGES.affiliation.rules.map((r) => `  - ${r}`),
        ``,
        `Code: web/enrich.js planStages + stage runners; render: web/render.js; ballot scores: web/scoreboard.js`,
      ].join("\n")
    );
  }
);

// —— file map ——
server.tool(
  "electionizer_where",
  "Map a concern to primary files. Prefer before opening many files. Topics include affiliation, finance, models, fec, govtrack.",
  {
    topic: z.enum(TOPIC_KEYS).describe("Area of the codebase"),
  },
  async ({ topic }) => {
    return text(
      [`# where: ${topic}`, ...WHERE[topic].map((l) => `- ${l}`)].join("\n")
    );
  }
);

// —— next plan (from HANDOFF) ——
server.tool(
  "electionizer_next",
  "Next-work plan from HANDOFF.md (single source of truth). Update HANDOFF plan section, not this tool.",
  {},
  async () => {
    const body = await readHandoff();
    if (!body) {
      return errText("HANDOFF.md missing — cannot derive next plan");
    }
    const plan = handoffSection(body, "## Plan for next session");
    const done = handoffSection(body, "## Done recently");
    const lines = [
      `# Next work (from HANDOFF)`,
      ``,
      plan || "_(no Plan for next session section)_",
      ``,
      done ? `---\n\n${done}` : "",
      ``,
      `## Agent workflow reminder`,
      `1. electionizer_status at session start`,
      `2. electionizer_next (or handoff section=plan)`,
      `3. electionizer_where / def / refs / rg — not full-tree Read`,
      `4. electionizer_check or electionizer_test after core changes`,
      `5. electionizer_build_wasm if wasm path touched`,
      `6. Update HANDOFF Done + Plan before commit`,
    ]
      .filter(Boolean)
      .join("\n");
    return text(lines);
  }
);

// —— rg (project-scoped ripgrep) ——
server.tool(
  "electionizer_rg",
  "Project-scoped ripgrep. Prefer over Bash rg/grep. Globs skip target/node_modules/web/pkg.",
  {
    pattern: z.string().describe("Regex (or fixed string if fixed=true)"),
    path: z
      .string()
      .optional()
      .describe("Repo-relative path to search (default .)"),
    glob: z.string().optional().describe("rg --glob e.g. '*.rs' or 'web/**'"),
    type: z.string().optional().describe("rg --type e.g. rust, js"),
    context: z.number().optional().describe("Lines of context (-C), max 5"),
    maxCount: z.number().optional().describe("Max matches per file (default 40)"),
    caseInsensitive: z.boolean().optional(),
    fixed: z.boolean().optional().describe("Literal -F match"),
    filesOnly: z.boolean().optional().describe("List files only (-l)"),
  },
  async (args) => {
    const r = await rgSearch(args);
    if (r.code !== 0 && r.stderr && r.stdout !== "(no matches)") {
      return errText(clip((r.stderr || "") + "\n" + (r.stdout || ""), 10_000));
    }
    return text(clip(r.stdout || "(no matches)", 14_000));
  }
);

// —— def: jump to definition-ish ——
server.tool(
  "electionizer_def",
  "Find definition sites for a symbol (fn/struct/enum/type/const/export). Returns path:line hits — then Read only those lines.",
  {
    name: z.string().describe("Symbol name e.g. ballot_affiliations, AffiliationSpan"),
    lang: z
      .enum(["all", "rust", "js"])
      .optional()
      .describe("Restrict language; default all"),
  },
  async ({ name, lang = "all" }) => {
    const n = escapeRegExp(name.trim());
    if (!n) return errText("name required");

    const patterns = [];
    if (lang === "all" || lang === "rust") {
      patterns.push(
        `(?:pub\\s+)?(?:async\\s+)?fn\\s+${n}\\b`,
        `(?:pub\\s+)?struct\\s+${n}\\b`,
        `(?:pub\\s+)?enum\\s+${n}\\b`,
        `(?:pub\\s+)?type\\s+${n}\\b`,
        `(?:pub\\s+)?(?:const|static)\\s+${n}\\b`,
        `fn\\s+${n}_js\\b`
      );
    }
    if (lang === "all" || lang === "js") {
      patterns.push(
        `export\\s+(?:async\\s+)?function\\s+${n}\\b`,
        `export\\s+(?:const|let|class)\\s+${n}\\b`,
        `function\\s+${n}\\b`,
        `(?:const|let)\\s+${n}\\s*=`
      );
    }

    const chunks = [];
    for (const pattern of patterns) {
      const r = await rgSearch({
        pattern,
        glob: lang === "rust" ? "*.rs" : lang === "js" ? "*.{js,mjs}" : undefined,
        maxCount: 15,
      });
      if (r.stdout && r.stdout !== "(no matches)") {
        chunks.push(r.stdout.trim());
      }
    }

    if (!chunks.length) {
      // fallback: plain name with word boundary
      const r = await rgSearch({
        pattern: `\\b${n}\\b`,
        glob: lang === "rust" ? "*.rs" : lang === "js" ? "*.{js,mjs}" : undefined,
        maxCount: 20,
      });
      return text(
        r.stdout && r.stdout !== "(no matches)"
          ? `# def fallback (mentions) ${name}\n${clip(r.stdout, 10_000)}`
          : `# def ${name}\n(no matches)`
      );
    }

    // de-dupe lines
    const seen = new Set();
    const lines = [];
    for (const c of chunks.join("\n").split("\n")) {
      if (!c || seen.has(c)) continue;
      seen.add(c);
      lines.push(c);
    }
    return text(`# def ${name}\n${clip(lines.join("\n"), 12_000)}`);
  }
);

// —— refs: find references ——
server.tool(
  "electionizer_refs",
  "Find references to a symbol across the repo (word-boundary). Use before renames or call-site updates.",
  {
    name: z.string().describe("Symbol name"),
    path: z.string().optional().describe("Optional repo-relative subpath"),
    glob: z.string().optional().describe("Optional rg glob"),
    maxCount: z.number().optional().describe("Max per file (default 30)"),
  },
  async ({ name, path: searchPath, glob, maxCount = 30 }) => {
    const n = name.trim();
    if (!n) return errText("name required");
    const r = await rgSearch({
      pattern: `\\b${escapeRegExp(n)}\\b`,
      path: searchPath,
      glob,
      maxCount,
    });
    return text(`# refs ${n}\n${clip(r.stdout || "(no matches)", 14_000)}`);
  }
);

// —— outline: file structure without full read ——
server.tool(
  "electionizer_outline",
  "Outline a source file (pub items / exports / top-level fns) with line numbers. Prefer before Read of large files.",
  {
    path: z
      .string()
      .describe("Repo-relative file e.g. crates/electionizer-core/src/govtrack.rs"),
  },
  async ({ path: rel }) => {
    let abs;
    try {
      abs = underRoot(rel);
    } catch (e) {
      return errText(String(e.message || e));
    }
    if (!(await exists(abs))) return errText(`missing: ${rel}`);
    const body = await fs.readFile(abs, "utf8");
    const lines = body.split("\n");
    const out = [];
    const isRs = abs.endsWith(".rs");
    const isJs = /\.(js|mjs|cjs)$/.test(abs);

    for (let i = 0; i < lines.length; i++) {
      const line = lines[i];
      const n = i + 1;
      if (isRs) {
        if (
          /^\s*pub\s+(?:async\s+)?fn\s+\w+/.test(line) ||
          /^\s*pub\s+struct\s+\w+/.test(line) ||
          /^\s*pub\s+enum\s+\w+/.test(line) ||
          /^\s*pub\s+type\s+\w+/.test(line) ||
          /^\s*pub\s+(?:const|static)\s+\w+/.test(line) ||
          /^\s*pub\s+trait\s+\w+/.test(line) ||
          /^\s*pub\s+mod\s+\w+/.test(line) ||
          /^\s*fn\s+\w+/.test(line) && !/^\s*\/\/\//.test(line) ||
          /^\s*#\[wasm_bindgen\]/.test(line) ||
          /^\s*#\[cfg\(test\)\]/.test(line) ||
          /^\s*mod\s+tests\b/.test(line)
        ) {
          out.push(`${n}: ${line.trim()}`);
        }
      } else if (isJs) {
        if (
          /^export\s+/.test(line) ||
          /^(?:async\s+)?function\s+\w+/.test(line) ||
          /^(?:const|let|class)\s+\w+\s*=/.test(line) ||
          /^\s*\/\/\s*——/.test(line) ||
          /^\s*\/\/\s*#+\s/.test(line)
        ) {
          out.push(`${n}: ${line.trim().slice(0, 140)}`);
        }
      } else {
        if (/^#{1,3}\s/.test(line) || /^[A-Z][A-Z0-9_ ]{3,}$/.test(line.trim())) {
          out.push(`${n}: ${line.trim().slice(0, 120)}`);
        }
      }
    }

    if (!out.length) {
      return text(
        `# outline ${rel}\n(${lines.length} lines — no structured items matched; file may be data/config)`
      );
    }
    return text(
      `# outline ${rel} (${lines.length} lines)\n${clip(out.join("\n"), 14_000)}`
    );
  }
);

// —— wasm exports ——
server.tool(
  "electionizer_exports",
  "List #[wasm_bindgen] export fn names from electionizer-wasm (or from built web/pkg JS).",
  {
    source: z
      .enum(["rust", "pkg", "both"])
      .optional()
      .describe("Where to read; default both"),
  },
  async ({ source = "both" }) => {
    const parts = [];
    if (source === "rust" || source === "both") {
      const r = await rgSearch({
        pattern: "^pub fn \\w+",
        path: "crates/electionizer-wasm/src",
        glob: "*.rs",
        maxCount: 200,
      });
      parts.push(`## rust pub fn (wasm crate)\n${r.stdout || "(none)"}`);
    }
    if (source === "pkg" || source === "both") {
      const pkg = path.join(ROOT, "web/pkg/electionizer_wasm.js");
      if (await exists(pkg)) {
        const r = await rgSearch({
          pattern: "^export function \\w+",
          path: "web/pkg/electionizer_wasm.js",
          maxCount: 200,
        });
        parts.push(`## web/pkg export function\n${r.stdout || "(none)"}`);
      } else {
        parts.push("## web/pkg\n(missing — run electionizer_build_wasm)");
      }
    }
    return text(`# wasm exports\n\n${parts.join("\n\n")}`);
  }
);

// —— slice: read N lines around a hit without loading whole file ——
server.tool(
  "electionizer_slice",
  "Read a line window from a file (default ±25 around line). Prefer over full-file Read for large sources.",
  {
    path: z.string().describe("Repo-relative file"),
    line: z.number().describe("1-based center line"),
    before: z.number().optional().describe("Lines before (default 25)"),
    after: z.number().optional().describe("Lines after (default 40)"),
  },
  async ({ path: rel, line, before = 25, after = 40 }) => {
    let abs;
    try {
      abs = underRoot(rel);
    } catch (e) {
      return errText(String(e.message || e));
    }
    if (!(await exists(abs))) return errText(`missing: ${rel}`);
    const body = await fs.readFile(abs, "utf8");
    const lines = body.split("\n");
    const center = Math.max(1, Math.floor(line));
    const start = Math.max(1, center - Math.min(Math.max(before, 0), 80));
    const end = Math.min(lines.length, center + Math.min(Math.max(after, 0), 120));
    const out = [];
    for (let i = start; i <= end; i++) {
      out.push(`${i}: ${lines[i - 1]}`);
    }
    return text(
      `# ${rel}:${start}-${end} (center ${center}, total ${lines.length})\n${out.join("\n")}`
    );
  }
);

async function main() {
  const transport = new StdioServerTransport();
  await server.connect(transport);
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
