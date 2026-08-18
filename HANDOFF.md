# Handoff: electionizer

**When:** 2026-08-18  
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

**Browser pipeline (today):**  
ZIP → geo → FEC → state bodies (**FL: DOS + county SOE VF**) → WASM report → measure $ (FL TreFin → MD MDCRIS → FTM) → **ballot score queue** (preview → scrape → grounded) → click → **Verdict** (reuse in-flight enrich) → Details.

**Rule:** JS I/O only; WASM pure. Cite every fact. Never invent family, citizenship, assets, party, or orientation.

**Finance stage order (locked):** FL acct → name-search → SOE VF → NC → AZ → MD → FTM → profile.  
**Measure finance order (locked):** FL TreFin → MD MDCRIS → FTM measure (thru 2024).  
**Federal bio stage order (locked):**  
`official_about` → `ballotpedia_bio` → `campaign_about` → `wiki_extract` → `dbpedia` → `grokipedia` → `wikidata_bio` → `wiki_photo` → `senate_efd` (Senate) **or** `house_clerk_fd` (House) → `votes`.

**Non-federal bio (locked):** after CF (+ optional `fl_chamber_bio` / `os_resolve`):  
`ballotpedia_bio` → `campaign_about` → `wiki_extract` → `dbpedia` → `grokipedia` → `wikidata_bio` → `wiki_photo` → (`os_votes` if state leg + key).  
**Judges:** after CF: `fl_courts_bio?` → `cl_person` → `cl_positions` → dense bio → `cl_opinions` (no `os_votes`).

**Scrutiny stages (locked, after bio/votes):**  
`bp_endorsements` → `campaign_endorsements` → `gdelt_news` → `news_rss` → `money_signals` → `bp_claims` → `campaign_claims` → `claim_contrasts` → `llm_contrasts` → `fl_bar` → `fl_ethics` → `fl_jqc` → `ai_verdict`.

**Landing:** ballot click → **Verdict** page (`showView("verdict")`). **Details →** tabs. `?tab=` skips Verdict.  
**Ballot list:** Fit column; office/measures sorted by voter-fit (high at top). Progress tree. **Scorecard** tabulates fit by party.  
**Detail tabs:** `dossier` · `scrutiny` · `votes` / **decisions** (judges) · `finance` · `personal` · `timeline` · `party` · `more`.

**Dossier:** coalesced facts in `bio.rs`; glance = edu → family → work → personal/orientation → citizenship; holdings → Personal finance tab. Cite or omit; GP never sole family/citizenship/orientation. Source priority official ≥ BP ≥ campaign ≥ WP/WD ≥ DBpedia ≥ FEC ≥ GP.

**Timeline:** `web/timeline.js` — lanes votes/decisions · campaign $ · personal disclosure; σ default 4w; GovTrack cap **500**.

**Voter profile:** home Likert 1–5 (no R/D split). Catalog = `voter-profile-defaults.json` (repo root; copied to `web/` on wasm build). Empty localStorage seeds via `importVoterProfileCatalog`. Export/import is the axis list only (no ratings). Client remaps fit (dislike 80 → 20). Unset/neutral (3) skipped.

**Privacy:** FL smoke ZIPs = `33334` (Broward) and `32901` (Brevard). Precinct+party and Likert stay in the browser.

---

## Coverage

| Region | Ballot |
|--------|--------|
| Federal | House/Senate FEC; finance + GovTrack (votes cap 500); CL party timeline; dossier; eFD / House Clerk holdings |
| Florida | DOS + **county SOE VF locals** + chambers + amendments; TreFin / name-search / SOE CF; measure TreFin; judges via CourtListener |
| Arizona | OfficialList + leg roster incumbents + Clean Elections measures + FTM $; SOS challengers blocked; Civic when keyed |
| North Carolina | NCSBE CSV + referendum PDF measures + CF; FTM $ thru 2024 |
| Maryland | SBE CSV + ballot questions; MDCRIS candidate + measure $; FTM archive |
| Other | Civic VIP when keyed+live; else OS incumbents; FTM when keyed |

---

## Shipped (do not reopen A–O)

A finance · B affiliation · C FL/AZ/NC/MD ballots · D Civic VIP · E measure $ · F dossier/career · G Wikidata/FEC occ · H dense bio hosts · I coalesce/family/orientation · J universal dense bio · K judicial Decisions + practice (CL; circuit dirs 2/4/5/7/10/12/15/17/18/20) · L Scrutiny · M Verdict card · N ballot fit + scorecard · O FL SOE locals.

**Verdict (M+P, current):** click → Verdict; Details → tabs. Queue: preview (identity + profile meaning, no tools) → shared enrich (`skipAi`) → grounded (packed scrapes + profile meaning + `web_search`/`x_search`). `ai_verdict` stage stays click-only. xAI Responses + Agent Tools (do **not** send `search_parameters` — HTTP 410). Chat fallback packed-only. Default model `grok-4.6`. AI-found orgs → `Endorsement` `trust: news`. Axis scores stay alignment; client remaps fit.

**List scoring (N+P, current):** Preview chip (muted · hourglass · “still analyzing”) then grounded replaces it. Queue 1–8 parallel (default 3) covers scrape+score. Click reuses `startSharedEnrich`. Cache until refresh data/score, a race, or a name (Civic VIP 12h). Profile change remaps cached cards (no new LLM).

**Rubrics (locked in `verdict.rs`):**

| Pack | Axes |
|------|------|
| Republican | maga · america_first · trump · neocon · zionist · bush_era · reagan_era · nixon_era · tea_party · + issues |
| Democrat | communism · socialism · green_new_deal · aoc · classical_liberal · clinton_era · jfk_era · lbj_era · obama_era · fdr_era · occupy · + issues |
| Independent | left_lean · right_lean · libertarian · populist · ron_paul · + issues |
| Judge | left_lean · right_lean · originalism · living_constitution · party_line · tds · constitutional_applicability · gun_rights · adl_aligned |
| Measure | tax_direction (−100..+100) · restriction_direction · constitutional_tension · incumbent_class_benefit |
| Issues | lgbtq · cannabis · voter_id · medical_freedom · remigration · abortion (−100 reduce / +100 access) · health_insurance (−100 fraud / +100 single-payer) · h1b (−100 reduce / +100 increase) · border (−100 open / +100 closed) |

`zionist` = pro-Israel *policy*, not ethnicity. `communism` = explicit program, not a tax vote. `tds` = Trump treated unlike similarly situated parties (compared case/quote).

**Packed today (grounded/refine):** endorsements, 12 news, 16 claims, 40 votes, 32 dossier facts, career blurb+flag+~8 spans, ~12 holdings, affiliations, cited family_summary, contrasts, Bar/Ethics/JQC records, finance totals, 15 donors. Preview packs identity + every `VoterPref` (label, definition, low/high poles). Likert 1 = low pole, 5 = high pole. Builtin + custom extras. No photos/portals/undisclosed orientation/full opinions.

---

## Federal holdings (keep — burned time)

**Senate eFD** (`efdsearch.senate.gov`, Wisp session): GET home → CSRF → POST agree → POST `/search/` → POST `/search/report/data/` → GET `/search/view/annual/{uuid}/` → Part 3. Live row = `[first, last, display, report-link, date]`. Accept Annual / New Filer / candidate / `/view/annual/`; not PTR. Agree is 302 empty body — follow redirect. Cookie-jar race: `settleSessionCookies` after every session body; `curlPostForm` `redirect:"manual"` + explicit GET; report GET + one re-agree retry.

**House Clerk FD** (`disclosures-clerk.house.gov`, Wisp): ViewSearch → antiforgery → ViewMemberSearchResult → text PDF Schedule A. Prefer latest FD Original. Image-only PDF → honest skip. Honorific skip in name match (`Hon`/`Mr`).

**`to_js`:** `Serializer::new().serialize_maps_as_objects(true)` or `obj.last` is always undefined.

EIGA §105(c): voter-info OK; no bulk resale/solicitation/credit.

---

## Plan: Track P — grounded scores + parallel scrape (active)

**Why:** List fit is search + Likert numbers. Click refine packs only a slice of enrich. Builtin “what it means” / poles never reach the model. User waits 1–2hr anyway and wants filings in the prompt — preview chip OK, then a grounded score.

**Do not reopen A–O.** Do not change locked finance / bio / scrutiny *merge* order. Two full live-search passes per name would blow the 1–2hr window — preview stays cheap (no tools).

### Product

```
ballot ready
  → pool (1–8, existing slider)
       per name:
         1. Preview score (identity + profile meaning, no tools)
         2. Enrich that person (parallelize independent fetches)
         3. Grounded score (packed scrapes + profile meaning + web/X search, equal weight)
  → chip: preview (hourglass · “still analyzing”) → final
```

- Preview: muted chip + tooltip **Preview · still analyzing**. List may sort; not done.
- Grounded: replaces card, drop preview.
- Click mid-flight: reuse `startSharedEnrich`; Verdict banner if still preview.
- Refresh data/score: unchanged (bust cache, rerun).
- Public Mercury Wisp throttles; local `./scripts/run-static.sh` handles higher parallelism.

### Phases

| ID | Work | Status |
|----|------|--------|
| **P0** | Pack `resolveProfileAxes()` (label, definition, low/high poles) on every `VoterPref`. Prompt: 1 = low pole, 5 = high pole | **Done** |
| **P1** | Pack missing scrape on grounded/refine: career blurb+flag+~8 spans; ~12 holdings; affiliations; cited family_summary; contrasts; Bar/Ethics/JQC. Facts 16→32. No photos/portals/undisclosed orientation/full opinions | **Done** |
| **P2** | Queue worker = preview → `enrichCandidate(..., { skipAi: true })` → grounded. One enrich per person (share with click). `ai_verdict` stage stays click-only | **Done** |
| **P3** | Within-person: fetch independent hosts in parallel; **apply in locked order**. Serial: finance fallbacks, eFD/House cookie jar, CL person→positions→opinions | **Done** (news pair + Bar/Ethics/JQC; finance/bio/CL still serial) |
| **P4** | UI: preview chip + spinner; progress tree shows scrape stages; Verdict preview banner; Settings copy that slider covers scrape+score | **Done** |
| **P5** | Prompt: packed filings and live search are both evidence; cite either; don’t invent; conflict → say so. Tests + wasm + SW bump + this plan → Done | **Done** |

**Parallel fetch / serial apply:**

| Parallel fetch | Apply in locked order |
|----------------|------------------------|
| FEC totals / principal / indiv / cmte / size / outside | as now |
| BP / campaign / wiki / DBpedia / GP / Wikidata | official ≥ BP ≥ campaign ≥ wiki… |
| GDELT + Google News | after endorsements slot |
| FL Bar + Ethics + JQC | after contrasts |
| Money signals | after finance exists |
| Contrasts | after claims + votes |

### Files

`web/scoreboard.js` · `web/enrich.js` · `web/verdict.js` · `web/render.js` · `web/app.js` · `web/settings.js` · `web/index.html` · `crates/electionizer-core/src/verdict.rs` · wasm if packer shape changes · `web/sw.js`

### Acceptance

- Preview body has no `web_search`; grounded body has tools + packed facts.
- Builtin abortion poles + a custom extra appear in the request body.
- Packer includes career / holdings / affiliations when present.
- Click reuses in-flight enrich; no second full scrape for the same person.
- List ~1–2hr at default 3 (local Wisp); public Wisp may need lower parallelism.

### Out of scope

OCR/FD, PACER, changing locked merge priority, scoring before identity exists.

---

## Backlog (not active)

- ZIP ≠ precinct (centroid) for **state house/senate/CD**; county locals from SOE + optional sample-ballot filter. Street still helps Civic only when VIP live.
- AZ SOS challengers bot-blocked; SeeTheMoney thin outside browser
- Civic VIP seasonal
- FTM archive thru 2024 only
- Bioguide / congress.gov / Vote Smart bot-blocked
- Grokipedia: no public article API; Wisp; AI content — keep low trust
- Dual citizenship / orientation almost never in filings (empty copy is correct)
- OCR / scanned House FD PDFs
- Fuller Schedule A / more states
- Timeline: dated outside $ (IE); deeper votes beyond 500; disbursement dates when available
- **K leftovers:** Ballotpedia case blurbs; full opinion scrapes beyond CL; more circuit hosts (1/3/6/8/9/11/13/14/16/19 when verified); dedicated `JudicialDecision` if filters outgrow `VoteRecord`; Bar profile **parse** if Cloudflare wall drops
- Judicial financial disclosures (state)
- PACER / bulk RECAP — out of product scope

---

## Continuity rules

- Do **not** reintroduce global drilldown overlay.
- JS I/O only; WASM pure.
- Never ship shared API secrets.
- Prefer Wisp for state hosts and non-CORS bio hosts (House.gov, Grokipedia, **efdsearch.senate.gov**, **disclosures-clerk.house.gov**).
- After core/wasm Rust: `./scripts/build-wasm.sh` before browser smoke.
- Prefer MCP nav (`where` / `def` / `refs` / `rg` / `outline` / `slice` / `stages`).
- Candidate finance order locked; measure finance order locked; federal bio stage order locked; scrutiny order locked.
- Dossier: cite or omit; coalesce before display; source priority official ≥ BP ≥ campaign ≥ WP/WD ≥ DBpedia ≥ FEC ≥ GP.
- Update this HANDOFF Plan + Backlog before commit.

---

**Next:** Track P shipped (SW v51). Civic VIP still 12h. Optional P leftover: parallel bio-host fetch / FEC finance stages. K leftovers + OCR/FD stay backlog.
