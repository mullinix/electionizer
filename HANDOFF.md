# Handoff: electionizer

**When:** 2026-08-16  
**Repo:** [github.com/mullinix/electionizer](https://github.com/mullinix/electionizer)  
**Product path:** Client-only static app (`web/` + WASM). Native Axum optional/dev only.

---

## What it is

ZIP → ballot report (live data, sources cited).

- **JS owns I/O** (fetch, IndexedDB, libcurl.js/Wisp)
- **Rust WASM is pure** parse/map/build (`electionizer-core`)
- No backend required; no shared API secrets (user keys in localStorage)

---

## How to run

```bash
./scripts/build-wasm.sh
python3 -m http.server 8080 --directory web
# open http://127.0.0.1:8080/
```

| Settings | Notes |
|----------|--------|
| Mode | Live (default) or Fixture |
| FEC key | `DEMO_KEY` if unset — rate-limited |
| Open States key | Optional — incumbents + detail votes + person image/roles |
| Google Civic key | Optional — multi-state VIP full contests (seasonal) |
| Wisp URL | Prefer `./scripts/run-static.sh` → Settings → **This origin** |
| CORS proxy | Fallback only |
| FTM key | Optional — candidate CF + measure $ thru 2024 |
| xAI / OpenAI key | Optional — Verdict card + L6 contrast notes. Needs Wisp. |

**Smoke ZIPs:** `33334` (FL — primary) · `10001` / `90210` (federal) · `85004` (AZ) · `27601` (NC) · `21401` (MD) · `32901` (FL Brevard / Melbourne)

**Native:** `cargo test --workspace` · `cargo run` → `:3000`  
**Agent MCP:** `opencode.json` → `mcp/electionizer`

---

## Architecture

```
crates/electionizer-core/   pure domain (+ bio dossier)
crates/electionizer-wasm/   thin wasm-bindgen exports
web/                        static client (product)
src/                        native Axum + SQLite (dev only)
```

**Browser pipeline:**  
ZIP → geo → FEC → state bodies (**FL: DOS + county SOE VF**) → WASM report → measure $ (FL TreFin → MD MDCRIS → FTM) → **ballot score queue** (Track N, M-early per item) → click → **Verdict** (Track M) → Details → staged enrich (finance + votes + affiliation + dossier/bio).

**Rule:** JS I/O only; WASM pure. Cite every fact. Never invent family, citizenship, assets, party, or orientation.

**Finance stage order (locked):** FL acct → name-search → SOE VF → NC → AZ → MD → FTM → profile.  
**Measure finance order (locked):** FL TreFin → MD MDCRIS → FTM measure (thru 2024).  
**Federal bio stage order (locked):**  
`official_about` → `ballotpedia_bio` → `campaign_about` → `wiki_extract` → `dbpedia` → `grokipedia` → `wikidata_bio` → `wiki_photo` → `senate_efd` (Senate) **or** `house_clerk_fd` (House) → `votes`.

**Non-federal bio stage order (Track J):** after CF (+ optional `fl_chamber_bio` / `os_resolve`):  
`ballotpedia_bio` → `campaign_about` → `wiki_extract` → `dbpedia` → `grokipedia` → `wikidata_bio` → `wiki_photo` → (`os_votes` if state leg + key).  
**Judges (Track K):** after CF: `fl_courts_bio?` → `cl_person` → `cl_positions` → dense bio J → `cl_opinions` (no `os_votes`).

**Landing:** ballot click → **Verdict** card (Track M). **Details →** existing tabs. `?tab=` skips Verdict.  
**Ballot list:** Fit column; each office/measures sorted by voter-fit (high at top). Progress tree while the score queue runs. **Scorecard** view tabulates fit by party for each role.

**Detail tabs:** `dossier` · **`scrutiny`** (Track L — money signals, endorsements, news) · `votes` (legislators) / **decisions** (judges — Track K) · `finance` (campaign $) · `personal` (eFD/House FD holdings) · `timeline` (correlation) · `party` · `more`. Deep links `?tab=`.

---

## Coverage (shipped)

| Region | Ballot |
|--------|--------|
| Federal | House/Senate FEC; finance + GovTrack (votes cap 500); CL party timeline; multi-source dossier; eFD / House Clerk holdings; correlation timeline |
| Florida | DOS filings (federal/state/multi-county) + **county SOE VF locals** + chambers + amendments; TreFin / name-search / SOE CF; measure TreFin; judges Decisions+practice via CourtListener |
| Arizona | OfficialList + leg roster incumbents + Clean Elections measures + FTM $; SOS challengers blocked; Civic when keyed |
| North Carolina | NCSBE CSV + referendum PDF measures + CF; FTM $ thru 2024 |
| Maryland | SBE CSV + ballot questions; MDCRIS candidate + measure $; FTM archive |
| Other | Civic VIP when keyed+live; else OS incumbents; FTM when keyed |

### Shipped product (do not reopen)

Finance (A), affiliation (B), state ballots FL/AZ/NC/MD (C), Civic VIP (D), measure $ (E), dossier shell + career fractions (F), Wikidata/FEC occ (G), dense bio hosts BP/WP/official/DBpedia/GP/campaign (H), dossier coalesce + multi-cite + family summary + orientation (I), detail tabs, UX polish P0–P7, `?tab=` deep links, measure detail click-through, **Senate eFD holdings**, **House Clerk FD holdings** (text PDFs), **eFD session/cookie-jar fix**, **Personal finance tab**, **Correlation timeline** (vertical + cursor/zoom; SW v12), **Track J universal dense bio** (all non-federal offices; SW v13), **Track K judicial Decisions + law practice** (CourtListener; SW v17), **K follow-on** circuit directories 2/4/5/7/10/12/15/17/18/20 + Bar/opinion portals (SW v18), **Track L Scrutiny** (SW v23), **Track M AI Verdict card** (SW v33), **Track N ballot fit + party scorecard** (SW v39).

**Dossier (current):** Coalesced facts in core (`bio.rs`); multi-cite chips; glance = edu → family → work → personal/orientation → citizenship; holdings teaser → **Personal finance** tab (table + portals). Cite or omit; GP never sole family/citizenship/orientation.

**Timeline (current):** `web/timeline.js` — vertical lanes (votes/decisions · campaign $ · personal disclosure); Gaussian σ default 4w (slider); cursor scrub + pin; wheel/± zoom + shift-drag pan; itemized Sched A lines + dated tops + eFD/House filing dates. GovTrack list cap **500**. Judge/CL: votes lane labeled **Decisions**.

**Privacy:** FL smoke ZIPs = `33334` (Broward) and `32901` (Brevard / Melbourne). Precinct+party and voter-profile Likert stay in the user’s browser.

---

## Shipped: Track M — AI Verdict card

**Shipped.** M0–M8. Do not reopen A–L. Last stage: `ai_verdict`.

Ballot click lands on a **Verdict** page (not a tab, not an overlay). **Details** opens today’s tabbed drilldown. Grok always live-searches. Verdict + numeric score + linked evidence.

### Product

```
ballot click
  → Verdict page
       [Details →]   → existing tabs / measure page
       [← Ballot]
```

- No overlay. `showView("verdict")` + `#verdict-view`.
- `?tab=` still opens Details (existing deep links).
- No key: Verdict shows Settings CTA; Details still works.
- Measures: same landing + Details → current measure page.

### Card

Headline verdict · overall 0–100 · cited summary · locked rubric axis bars · live-search finds · chips into Details tabs · model/tools line.

Home **voter profile** (all rubric axes, no R/D split, 1–5 Likert, localStorage) remaps those numbers into personal fit: dislike an axis at 80 → display 20. Weighted aggregate = overall fit (lower = worse). Unset/neutral (3) skipped. Heatmap is cividis-like blue→gold (colorblind-safe). Axis scores sent to the model stay alignment; client remaps display.

Unsourced axis → `—`, not 0. Bio facts still cite-or-omit (no invented family/citizenship/orientation/assets).

### Rubrics (locked in `verdict.rs`)

| Pack | Axes |
|------|------|
| Republican | maga · america_first · trump · neocon · zionist · bush_era · reagan_era · nixon_era · tea_party · + issues |
| Democrat | communism · socialism · green_new_deal · aoc · classical_liberal · clinton_era · jfk_era · lbj_era · obama_era · fdr_era · occupy · + issues |
| Independent | left_lean · right_lean · libertarian · populist · ron_paul · + issues |
| Judge | left_lean · right_lean · originalism · living_constitution · party_line · tds · constitutional_applicability · gun_rights · adl_aligned |
| Measure | tax_direction (−100..+100) · restriction_direction · constitutional_tension · incumbent_class_benefit |
| Issues (all candidates) | lgbtq · cannabis · voter_id · medical_freedom · remigration · abortion (−100 reduce / +100 access) · health_insurance (−100 fraud / +100 single-payer) · h1b (−100 reduce / +100 increase) · border (−100 open / +100 closed) |

`zionist` = pro-Israel *policy*, not ethnicity. `communism` = explicit program, not a tax vote. `tds` = Trump treated unlike similarly situated parties (compared case/quote).

### Pipeline

A–L unchanged. Last stage: `ai_verdict`.

Dual-pass (cap 2 / subject): **M-early** on click (identity + search) → **M-refine** when enrich finishes (`ai_verdict` stage). Cache IndexedDB until Scrape/AI reload.

xAI **Responses** `POST /v1/responses` + Agent Tools `web_search` + `x_search`. Do **not** send `search_parameters` (Live Search is deprecated, HTTP 410). Chat Completions fallback is packed-only (no live search). OpenAI: Responses + `web_search`. User key + Wisp. L6 Chat Completions contrast notes stay under Scrutiny. Default Grok model: `grok-4.6`.

AI-found orgs → `Endorsement` `trust: news` (never silent filing).

### Phases

| ID | Work | Status |
|----|------|--------|
| **M0** | Verdict landing + Details/Ballot crumbs. No-key CTA. `?tab=` unchanged | done |
| **M1** | Core `verdict.rs`: rubrics, packer, prompt, parse/validate, wasm | done |
| **M2** | `runVerdictPass` + packed render + persistent cache | done |
| **M3** | Always `web_search` + `x_search` | done |
| **M4** | Merge AI-found endorsements into Scrutiny (`news`) | done |
| **M5** | Judge + measure packs + measure landing | done |
| **M6** | Dual-pass (early + refine) | done |
| **M7** | BP endorsement parser (Donalds h5/H350 + widget-key; skip Notable outgoing; campaign heading cards) | done |
| **M8** | Tests, wasm, SW v33, MCP, HANDOFF, smoke | done |

### Files

`crates/electionizer-core/src/verdict.rs` · wasm exports · `web/verdict.js` · `web/enrich.js` · `web/app.js` · `web/index.html` · `web/settings.js` · `web/app.css` · SW v38

### Acceptance

- Click Donalds → Verdict with headline, overall score, R-axis bars, cites. Endorsements can appear even if Scrutiny scrape is empty (`news`).
- **Details** → current tabs; enrich already running/done.
- Judge (Canady) → judge axes. FL amendment → tax/restriction/constitution.
- No key → CTA, Details works. No Wisp → honest skip.
- `?tab=scrutiny` skips Verdict.

---

## Shipped: Track N — ballot fit + party scorecard

**Shipped.** Do not reopen A–M. List scoring is **M-early only** (identity + live search). Full enrich still runs on click.

### Product

```
ballot ready
  → score queue (1–8 parallel, cache until reload)
       role → candidate → scrape (pack + search) → AI score
  → each office/measures sorted by voter-fit (high at top)
  → Scorecard view: party columns + avg per role
       click name → Verdict
```

- Fit column on ballot tables. Unscored stay at the bottom (`…` / `—`).
- Progress tree on ballot + scorecard (`#score-progress`). Pause while a Verdict is open; resume on back.
- No key / no Wisp → CTA; lists stay original order.
- Profile change remaps cached cards (no new LLM calls). Cache key ignores profile.
- Score queue: 1–8 parallel M-early jobs (Settings slider, default 3). Click enrich stays serial.
- Cache lasts until **refresh data** / **refresh score** on the ballot, a race, or a name (Civic VIP 12h).

### Files

`web/scoreboard.js` · `web/render.js` · `web/app.js` · `web/index.html` · `web/app.css` · SW v47

---

## Shipped: Track L — voter scrutiny (Brevard-first)

**Shipped.** L0–L9. Do not reopen A–K.

Brevard official sample ballot is a VoterFocus **PDF** (precinct + party in Settings).

### Why

Shipped product cites filings/bios. Endorsements were FEC IE only. No news, no capture ratios, no campaign-vs-record. Track L adds a **Scrutiny** tab: signals, not verdicts. Never auto-label bought/scammer/liar.

### Product rules

- JS I/O; WASM pure; no shared secrets.
- Every card cited + trust tag (`filing` / `official` / `reference` / `campaign` / `news` / `opinion` / `inference`).
- News = “reported by X”, not a finding.
- Ambiguous name match → skip.

### Stages (append after existing bio/votes — do not reorder A–K)

`bp_endorsements` → `campaign_endorsements` → `gdelt_news` → `news_rss` → `money_signals` → `bp_claims` → `campaign_claims` → `claim_contrasts` → `llm_contrasts` → `fl_bar` → `fl_ethics` → `fl_jqc`

### Phases

| ID | Work | Status |
|----|------|--------|
| **L0** | Precinct+party Settings/home; VoteBrevard sample-ballot PDF banner; Scrutiny tab shell + portals | done |
| **L1** | `money_signals` from loaded FEC/TreFin/SOE rows (large-donor, PAC, top-5, out-of-state/county) | done |
| **L2** | Ballotpedia + campaign `/endorsements` harvest → `Endorsement` + Scrutiny | done |
| **L3** | GDELT (CORS*) + Google News RSS; last-name filter | done |
| **L4** | Extract `PublicClaim`s from campaign/BP | done |
| **L5** | Keyword-pair claims vs votes/opinions | done |
| **L6** | Optional LLM contrast cards (user key; xAI/OpenAI; Wisp; verdict words dropped) | done |
| **L7** | Bar / Ethics / JQC parse | done |
| **L8** | Measure endorsements + Nov amendments | done |
| **L9** | Tests, wasm, SW, MCP, smoke 32901 | done (214 lib; live VF PDF + BP A3 6/6 + Ethics Form 6 Canady + GDELT; in-app click-through still useful) |

### Sources (session 1)

Use: GDELT (free, CORS), Google News RSS (Wisp), Ballotpedia HTML, campaign pages, existing finance, VoteBrevard PDF link.  
Skip: NewsAPI ($449/mo), OpenSecrets API (dead Apr 2025; OS Pro is $4/mo web-only).  
Signup if wanted: FEC, Open States, CL token, Congress.gov (L5).

### Files

`crates/electionizer-core/src/scrutiny.rs` · `web/enrich.js` · `web/state.js` · `web/render.js` · `web/settings.js` · `web/app.js` · wasm exports · SW v23

### Acceptance (L0–L8)

- Home/Settings store precinct + party → ballot banner links official sample PDF.
- Scrutiny tab: money signals and/or honest empty; endorsements when BP/campaign/IE have them; news headlines or honest empty; portal chips.
- Stated positions from BP Campaign themes / campaign `/issues` when present; Contrasts = keyword overlap with loaded votes/opinions (or honest empty). Overlap is not a lie/bought verdict.
- Optional LLM notes (Settings xAI/OpenAI key + Wisp) sit under overlap cards; trust `inference`; verdict words dropped. Skip if no key.
- No verdict scores. Hard-refresh SW v23.
- FL Scrutiny **License & ethics**: unique Bar standing when directory matches; Ethics Form 1/6 filings + named final orders; JQC notices for judges (first+last). Ambiguous / CF / too-many → skip. Not a verdict.
- Measure detail **Endorsements**: Ballotpedia Support/Oppose lists when the FL (or other keyed state) index uniquely matches the DOS/SBE title; plus already-loaded sponsor/oppose committees (trust `filing`). Nov 2026 FL: Amendment 3 homestead / budget stabilization / ag TPP. Ambiguous match → skip. Arguments quotes are not endorsements. Hard-refresh SW v23.

---

## Shipped: Track K — judicial decisions + law practice

**Shipped:** CourtListener decisions + practice for judges. K follow-on: expanded FL circuit directories + Decisions portals (Bar, opinions). Hard-refresh SW v18+.

**Was**

1. **Votes tab** judges got a hard stub — no opinions.
2. **Law practice** thin unless courts bio / dense hosts listed firms.
3. No structured decisions pipeline for `is_judge`.

**Goal**

| Surface | Legislator today | Judge after K |
|---------|------------------|---------------|
| **Votes tab** (`?tab=votes`) | GovTrack / Open States roll-calls | **Decisions / opinions** list (same table chrome: date · matter · role · disposition · link) |
| **Dossier Work + career** | OS/CL/BP spans | **Historical law practice** (firms, roles, years) as cited `Legal` spans + work facts |
| **Timeline** | votes lane = roll-calls | Reuse votes lane for dated opinions when present (no new lane required in v1) |

**Non-goals**

- Do not invent holdings, case outcomes, or firm tenure.
- No PACER bulk / paid RECAP fetch queues; no OCR of opinion PDFs.
- Do not reopen A–J, eFD, House FD, Track J bio order, measure $.
- No full text of opinions in-app (link out).
- No bulk Florida Bar scrape if Cloudflare walls; portal + honest skip OK.
- Federal legislators unchanged (still GovTrack `votes`).

### Data model (reuse first)

`VoteRecord` already fits a decision row:

| Field | Legislator | Judge mapping |
|-------|------------|---------------|
| `date` | vote date | opinion / order date |
| `question` | bill/question | case name + short cite (or docket) |
| `position` | Yea/Nay/… | **Author** · **Join** · **Concur** · **Dissent** · **Panel** · **Order** (trial) |
| `result` | Passed/Failed | disposition if known (Affirmed/Reversed/…) else `—` |
| `url` | GovTrack | CourtListener / court host opinion URL |

Keep one list (`enrich.votes`) + `votes_source` / `votes_url` / caps. UI copy only branches on `c.is_judge` (heading **Decisions** vs **Voting record**; filters: role instead of Yea/Nay when judicial).

Optional later (not v1): dedicated `JudicialDecision` type if filters need more fields (court_id, cluster_id). Prefer not.

**Practice / career:** existing `CareerSpan` + `LifeCategory::Legal` + dossier facts (`work` / prefer kind `legal` when firm/practice). Coalesce + cite-or-omit unchanged.

### Primary host: CourtListener (Free Law Project)

API root: `https://www.courtlistener.com/api/rest/v4/`  
Useful resources: `people`, `positions`, `opinions`, `clusters`, `search`, `courts`.

| Stage id | I/O (JS) | Pure (core/wasm) |
|----------|----------|------------------|
| `cl_person` | GET people search (name + court filter when mappable) | match person; store `enrich.courtlistener_id` |
| `cl_positions` | GET positions for person | → `CareerSpan` Legal/Political + work facts (employer, job_title, dates) |
| `cl_opinions` | GET opinions/clusters by author_id (cap **≤100** v1; link “more on CourtListener”) | map → `VoteRecord[]`; set `votes_source = "CourtListener"` |

**Auth / Settings:** optional user **CourtListener token** in localStorage (same pattern as FTM/OS) — never ship a shared key. Anonymous may work at low QPS; token preferred. **Wisp** if browser CORS blocks (same as eFD/courts).

**Court id map (core):** ballot office → CL `court` id where stable, e.g. FL Supreme → `fla`; DCA districts → CL DCA ids; Circuit N → circuit court id when CL has it. Unknown court → name-only people search + soft office/state match; ambiguous → skip (cite-or-omit).

**Coverage reality**

| Bench | Opinions in CL | Practice in CL positions |
|-------|----------------|--------------------------|
| FL Supreme / DCA | Often good | Often good for sitting justices |
| Circuit / county | Sparse published opinions | Spotty; empty OK |
| Challengers (never on bench) | None | Prior firm only if in CL/BP/courts bio |
| MD / NC judges | Variable | Same pipeline when person matches |
| AZ judicial | Thin on ballot today | Same if candidate appears |

### Secondary hosts (practice + portals)

| Source | Role | Notes |
|--------|------|--------|
| `fl_courts_bio` (shipped) | Already Legal spans from officesPositions + prose | **K strengthen:** firm/partner/associate/private practice sentences → `legal` facts + dated spans; keep Bar/DOS portals |
| Dense bio (J) | BP Profession / prior offices | Ensure firm lines classify `legal` not generic work when obvious |
| Florida Bar | Portal link only unless Wisp HTML profile is parseable without bot wall | Do not block stage pipeline on Bar |
| State opinion portals | Fallback links on empty decisions | flcourts opinions search, etc. — portal, not full scrape in v1 |
| Ballotpedia cases | Out of scope v1 | Optional later if CL miss |

### Locked judge detail stage order (after CF)

Non-federal judge path (insert; do not reorder finance or dense bio hosts):

1. CF (finance_order) — unchanged  
2. `fl_courts_bio` — when FL + judicial (unchanged slot; strengthen parse)  
3. `cl_person` → `cl_positions` → (dense bio J order) → `cl_opinions`  
   - Or: `cl_person` early; `cl_positions` before or after `fl_courts_bio` (merge spans); `cl_opinions` **last** among judge extras (fills votes tab), parallel to federal `votes` / leg `os_votes`  
4. Dense bio: `ballotpedia_bio` → … → `wiki_photo` (J locked)  
5. **No** `os_votes` for judges  

Suggested concrete append for `is_judge`:

```
… CF …
fl_courts_bio? 
cl_person
cl_positions          # practice + prior bench → dossier/career
ballotpedia_bio … wiki_photo
cl_opinions           # → enrich.votes (decisions tab)
```

### UX

- Tab label: keep `data-tab="votes"` deep link; **visible label** = “Decisions” when `c.is_judge`, else “Votes”.
- Heading inside panel: “Decisions & opinions” vs “Voting record”.
- Empty (after stages): “Checked CourtListener — no authored opinions found.” + portal chips (CL search URL, flcourts, Bar). Never claim legislative votes.
- Filters: reuse year; map position filter to Author/Join/Concur/Dissent when `votes_source` is CourtListener.
- Meta line: `N opinions · CourtListener` (not “roll-call votes”).
- Dossier glance Work: firm/practice lines first when present; career Legal fraction should move once positions/courts spans load.

### Phases

| ID | Work | Status |
|----|------|--------|
| **K0** | UX shell only: judge tab label + empty copy + portals; still no fetch | done |
| **K1** | Core: office → CL court id helpers; people match (name + court/state); wasm exports; fixtures | done (`courtlistener.rs`) |
| **K2** | `cl_person` + `cl_positions` stage in `enrich.js`; merge Legal/Political spans + work facts into dossier; Settings optional CL token | done |
| **K3** | `cl_opinions`: clusters/opinions → `VoteRecord[]`; cap; `votes_source`/`votes_url`; render Decisions filters | done |
| **K4** | Strengthen `fl_courts` (+ BP if needed) private-practice / firm sentence → `legal` spans/facts | done |
| **K5** | Timeline: dated opinions already in `enrich.votes` appear on votes lane — verify; no new lane | done (lane label Decisions when judge/CL) |
| **K6** | Tests (match, map opinion row, practice span); `cargo test -p electionizer-core --lib`; `./scripts/build-wasm.sh`; SW bump; MCP `stages` | done (183 lib tests; SW v17; live Canady CL node smoke) |
| **K7** | Smoke: `33334` FL SC or DCA justice with CL hit → Decisions rows + Legal career; circuit/county judge → honest empty OK; federal House votes regression; MD/NC judge if on ballot | done (node CL: Canady 100/195 + practice; Labarga 16; John Smith skip; browser hard-refresh SW v17) |

### Acceptance

- FL appellate/SC judge on `33334` (when CL has person): **Decisions** tab lists authored/joined opinions with cites + links; source CourtListener.  
- Same person: dossier/career shows **prior law practice** spans when CL positions or courts bio list firms — cited.  
- Circuit/county / challenger: empty decisions + “checked …” OK; no fake rows.  
- Non-judges: Votes tab + GovTrack/OS unchanged.  
- Cite or omit; no shared CL API secret.  
- Wisp path documented if CORS blocks.

### Files (expected)

- `web/enrich.js` — `planStages` judge extras; CL stage runners  
- `web/render.js` — Decisions copy/filters; tab label  
- `web/app.js` / settings — optional CourtListener token  
- `crates/electionizer-core/src/` — new `courtlistener.rs` (or `judicial.rs`) parse/match/map; `bio.rs` practice classify tweaks  
- `crates/electionizer-wasm/src/lib.rs` — thin exports  
- `web/sw.js` — CACHE bump  
- `mcp/electionizer/server.js` — stages text  
- `HANDOFF.md` — this plan → Done when shipped  

### Risks / research before K1

1. Browser CORS on `courtlistener.com/api` — probe; else Wisp-only.  
2. People search ambiguity (common names) — require court or middle initial; else skip.  
3. Trial-judge opinion scarcity — product expectation = empty, not failure.  
4. Rate limits — cache IndexedDB until reload; cap opinions.  
5. CL ToS / attribution — show CourtListener as source name + link out.

---

## Shipped: Track J — universal dense bio (all offices)

Dense bio hosts run for every non-federal candidate after CF. Judicial empty copy waits for hosts. SW v13+. See Done table.

**Locked non-federal bio order** (still in force):  
`ballotpedia_bio` → `campaign_about` → `wiki_extract` → `dbpedia` → `grokipedia` → `wikidata_bio` → `wiki_photo`  
(+ `fl_chamber_bio` / `fl_courts_bio` / `os_resolve` / `os_votes` per existing scope).

---

## Federal personal holdings (shipped reference)

### Senate eFD — `senate_efd`

| Item | Detail |
|------|--------|
| Host | `efdsearch.senate.gov` (Wisp session) |
| Flow | GET home → CSRF → POST agree → POST `/search/` → POST `/search/report/data/` → GET `/search/view/annual/{uuid}/` → Part 3 `#grid_items` |
| Live row shape | `[first, last, "Last, First (Senator)", "<a>…report…</a>", date]` (not legacy name-link/office/state) |
| Holdings types | **Annual**, **New Filer**, candidate reports, `/view/annual/` paths — **not** PTR |
| Law | EIGA §105(c): voter-info OK; no bulk resale/solicitation/credit |

### House Clerk FD — `house_clerk_fd`

| Item | Detail |
|------|--------|
| Host | `disclosures-clerk.house.gov` (Wisp session) |
| Flow | GET `ViewSearch` → antiforgery → POST `ViewMemberSearchResult` (empty year OK) → GET `public_disc/financial-pdfs/{year}/{docId}.pdf` → `pdf-extract` → Schedule A |
| Prefer | Latest **FD Original** (not PTR); name + state + district |
| Bulk ZIP | `{year}FD.zip` is **index only** (TXT/XML DocIDs) — no assets |
| PDFs | Text OK (e.g. Pelosi); image-only (e.g. Bilirakis) → honest skip |
| Law | Same EIGA §105(c) limits |

### eFD / holdings live-debug notes (2026-08-08)

Keep these — they burned real time:

1. **`serde_wasm_bindgen` Maps** — default `to_js(json!({…}))` emits ES6 `Map`, so `obj.last` is always `undefined` → fake “no last name”. Fix: `Serializer::new().serialize_maps_as_objects(true)` in `electionizer-wasm` `to_js`. Affects all `json!` object returns.

2. **eFD agree is 302 empty body** — POST `prohibition_agreement=1` → `302 Location: /search/` + `sessionid`, body length 0. `curlPostForm` must follow one redirect (or GET `/search/` after). Else “eFD agreement failed”.

3. **Live DataTables columns ≠ fixture** — live = first/last/display/report-link/date; old fixture was name-link/office/state/type/date. Parser supports both.

4. **New Filer ≠ title “Annual”** — e.g. Ashley Moody only has “New Filer Report for …”; path still `/search/view/annual/{uuid}/`. eFD UI groups New Filer under Annual. `efd_is_holdings_report` accepts annual + new filer + candidate + `/view/annual/` path; excludes PTR.

5. **House Clerk names** — `Pelosi, Hon.. Nancy` needs honorific skip in name match (`Hon`/`Mr`/…).

6. **Dossier Education squish** — glance Education used 2-col `.dossier-kv` with sr-only `dt` → value stuck in ~32% first column. Use `.dossier-fact-line` / bare list for unlabeled facts.

7. **Wisp required** for eFD + House Clerk (no CORS). Stage skip reason surfaces on holdings empty copy + // Detail sources //.

8. **Service worker** — bump `web/sw.js` `CACHE` when shipping web/js/css/wasm so browsers pick up fixes.

9. **libcurl.js cookie-jar race → “session lost on report view”** — eFD search JSON works *without* `sessionid` (only CSRF); report view 302s to `/search/home/` without it. `HTTPSession` writes `CURLOPT_COOKIEJAR` only on easy cleanup, deferred via `setTimeout(1)`. Starting the next hop in the same turn drops the agree `sessionid`. Fix: `settleSessionCookies` after every session body read; `curlPostForm` uses `redirect:"manual"` + explicit GET follow; eFD report GET manual + one re-agree retry. (`web/curl-transport.js`, `web/enrich.js`)

**Smoke:** Senate Moody/Gillibrand → **Personal finance** + **Timeline** (Wisp). House text-PDF member → holdings; image PDF → portal only. Hard-refresh after SW bump (v12).

---

## Plan: FL county SOE on the ballot (active)

**Why missing (Brevard primary vs app):**

DOS Candidate Tracking is **federal / state / multi-county only**. County commission, school board, Canaveral Port Authority, and county judge qualify with the **Supervisor of Elections**. VoteBrevard lists them as local anchors (`#cc1`, `#schbd1`, `#cpa3`, `#ccj3`), not DOS CanList links. The VF page itself says county/local live there; DOS has the rest.

`map_filings_for_geo` only emits locals that appear in the DOS TSV. Live `LOC` extract does not carry these SOE races. VoterFocus `candidate_pr.php` already has every qualified name — used only for **finance match after click**, never for ballot construction. Precinct is stored for the sample-ballot PDF banner only; ZIP centroid cannot know BCC D1 vs D2, school board D1, or port D3/D5.

Live VF 2026 (Qualified / Unopposed):

| Office | On a Brevard VF sample | In DOS extract |
|--------|------------------------|----------------|
| County Commissioner, District 1 | yes | no |
| Canaveral Port Authority, Districts 3 & 5 | yes | no |
| County Court Judge, Group 3 | yes (countywide) | often dropped (empty County col or not in TSV) |
| School Board, District 1 | yes | no |

**How they stay on the ballot**

1. Fetch VF `candidate_pr.php` at **ballot-build** (body `fl:soe`). Same parser as finance.
2. Map Qualified / Qualified Write-In / Unopposed → `SnapshotCandidate`. Skip DNQ / Withdrawn / Redesignated.
3. Skip federal / state / circuit (DOS + FEC already cover). Dedup by name + office family.
4. Default include (no precinct needed): county commission, school board, county judge, port authority, soil & water, countywide constitutional officers.
5. Municipal / CDD / rec districts: only if official sample-ballot text names them (avoid dumping every city/CDD).
6. When precinct + party set (Brevard VF): fetch sample-ballot PDF, extract text (`fl:sample_ballot`), keep districted locals whose contest appears. Countywide always kept. Unusable extract → fall back to default include (never drop the four races).
7. Cite VF / VoteBrevard. No shared secrets. Smoke ZIP `32901`.

**Phases**

| ID | Work | Status |
|----|------|--------|
| **O0** | Core: `map_soe_hits_for_geo` + status/office filters + sample-text filter + tests | done |
| **O1** | `fl:soe` + optional `fl:sample_ballot` bodies; `fetchStateBodies` VF list (+ PDF when precinct set) | done |
| **O2** | Wasm / SW bump / MCP body keys / this plan → Done | done |

Shipped. SW v45. Hard-refresh, then Brevard (`32901`): BCC, port, county judge, school board via SOE. Native Axum may pass empty SOE (product path is static). Do not reopen A–N.

---

## Backlog (not active)

- ZIP ≠ precinct (centroid) for **state house/senate/CD**; county locals now from SOE + optional sample-ballot filter. Street still helps Civic only when VIP live.  
- AZ SOS challengers bot-blocked; SeeTheMoney thin outside browser  
- Civic VIP seasonal  
- FTM archive thru 2024 only  
- Bioguide / congress.gov / Vote Smart bot-blocked  
- Grokipedia: no public article API; Wisp; AI content — keep low trust  
- Dual citizenship / orientation almost never in filings (empty copy is correct)  
- OCR / scanned House FD PDFs  
- Fuller Schedule A / more states  
- Timeline: dated outside $ (IE); deeper votes beyond 500; disbursement dates when available  
- **Track K follow-ons (remaining):** Ballotpedia case blurbs; full opinion scrapes beyond CL (not just portals); more circuit hosts (1/3/6/8/9/11/13/14/16/19 when bio pages verified); dedicated `JudicialDecision` type if filters outgrow `VoteRecord`; Bar profile **parse** if Cloudflare wall drops (portal shipped)  

- Judicial financial disclosures (state) — separate from K  
- PACER / bulk RECAP — out of product scope  

---

## Continuity rules

- Do **not** reintroduce global drilldown overlay.  
- JS I/O only; WASM pure.  
- Never ship shared API secrets.  
- Prefer Wisp for state hosts and non-CORS bio hosts (House.gov, Grokipedia, **efdsearch.senate.gov**, **disclosures-clerk.house.gov**).  
- After core/wasm Rust: `./scripts/build-wasm.sh` before browser smoke.  
- Prefer MCP nav (`where` / `def` / `refs` / `rg` / `outline` / `slice` / `stages`).  
- Candidate finance order locked; measure finance order locked; federal bio stage order locked.  
- Dossier: cite or omit; coalesce before display; source priority official ≥ BP ≥ campaign ≥ WP/WD ≥ DBpedia ≥ FEC ≥ GP.  
- Update this HANDOFF Plan + Backlog before commit.  

---

## Done recently (compact)

| When | Note |
|------|------|
| Tracks A–I + tabs + UX P0–P7 | Shipped prior |
| `?tab=` + measure detail | Shipped |
| **Senate eFD** | Live rows + New Filer; agree 302 follow; Map→object `to_js`; Moody/Gillibrand fixtures |
| **House Clerk FD** | Search + Schedule A PDF holdings; text only; image skip |
| **Dossier UX** | Education full-width fact lines; holdings empty shows stage skip |
| **eFD session lost** | libcurl jar race: settle cookies after each hop; manual 302 follow; report re-agree retry |
| **Personal finance tab** | Detail tab for eFD / House Clerk holdings (table + portals); Finance tab = campaign $ only |
| **Correlation timeline** | Vertical Timeline tab + cursor/pin + zoom/pan; GovTrack votes cap 500; Gaussian σ default 4w; SW v12 |
| **Track J** | Universal dense bio for all non-federal (planStages + BP office titles + wiki name guess); SW v13; planStages smoke + live BP DeSantis |
| **FL courts bio** | `fl_courts_bio` stage — SC/DCA Next.js + circuit directories; accent fold Muniz; Bar/DOS portals |
| **Track J** | Universal dense bio (all non-federal); planStages + BP titles + wiki guess; SW v13 |
| **Track K** | CourtListener Decisions + practice spans; judge tab label; optional CL token; SW v17 |
    | **K follow-on** | Circuit dirs 2/4/5/7/10/12/15/17/18/20; C7/C12/C10 link+bio parsers; Decisions portals (Bar + SC/DCA opinions + records); roster-only honest skip; SW v18 |
    | **Track L L0–L5** | Scrutiny tab; precinct+party sample-ballot banner; money signals; BP/campaign endorsements; GDELT + Google News; PublicClaim extract + keyword contrasts; SW v20 |
    | **Track L L6** | Optional LLM contrast notes (xAI/OpenAI user key; Wisp; no verdicts); SW v21 |
    | **Track L L7** | FL Bar standing + Ethics filings/orders + JQC notices; unique name only; SW v22 |
    | **Track L L8** | Measure Endorsements (BP Support/Oppose + committee sides); Nov 2026 FL amendments match; SW v23 |
    | **Track L L9** | 214 lib tests; live smoke: VF sample PDF; BP A3 Support/Oppose (DeSantis… / Berman…; arguments skipped); Ethics EFDMS Form 6 Canady; GDELT 200. In-app `32901` click-through still useful. |
    | **Track M M0–M8** | Verdict landing + dual-pass Responses (`web_search`/`x_search` Agent Tools; chat fallback packed-only); R/D/I/judge/measure rubrics; AI-found endorsements `news`; Donalds BP h5/H350 parser (live ~95); SW v36; 227 lib tests. Hard-refresh then `33334` Donalds. |
    | **Verdict parse** | Ignore search-result `headline`s; only assistant message text; JS salvage + prose fallback. SW v37. |
    | **Voter profile fit** | Home Likert 1–5 on all axes (no R/D split); localStorage; invert disliked (80→20); weighted fit + cividis heatmap; profile in AI prompt. SW v38. |
    | **Track N** | Ballot score queue (M-early); fit column + sort; party scorecard view; progress tree. SW v39. |
    | **Fit chip contrast** | Heatmap scores use dark/light ink (WCAG) instead of same-as-fill. SW v40. |
    | **Score progress fold** | Completed roles autocollapse; whole tree folds when scoring finishes. SW v41. |
    | **Voter profile issues** | Obama/FDR eras, Ron Paul, Tea Party, Occupy, LGBTQ, cannabis, voter ID, medical freedom, remigration; signed abortion / insurance / H-1B / border. SW v42. |
    | **Verdict subject lock** | Enrich/verdict paints are session+gen gated; cards stamped with subject key so another person cannot overwrite the open page. SW v43. |
    | **Score pool** | M-early queue runs 1–8 parallel (default 3); 429 backoff; profile no longer busts verdict cache. SW v44. |
    | **FL SOE locals** | VF `candidate_pr.php` at ballot-build (`fl:soe`); precinct sample-ballot PDF filter (`fl:sample_ballot`). BCC / port / county judge / school board. SW v45. |
    | **Forever cache + reload** | IndexedDB persists until **refresh data** / **refresh score** on the ballot, a race, or a name (Civic VIP still 12h). SW v47. |

**Next:** Hard-refresh SW v47. Refresh data/score on the ballot header, a race, or a name. Civic VIP still 12h. K leftovers + OCR/FD stay backlog.
