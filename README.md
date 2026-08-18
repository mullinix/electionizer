# electionizer

ZIP → ballot report (live data, sources cited). Drop a ZIP code, get federal (and FL/AZ state) candidates with finance and votes.

[github.com/mullinix/electionizer](https://github.com/mullinix/electionizer)

**Client-only static app** is the product path: JS owns I/O, Rust WASM is pure parse/map/build. No backend required.

## Quick start (static)

```bash
# needs rustup + wasm32-unknown-unknown + wasm-pack
cargo install wasm-pack   # once
./scripts/build-wasm.sh

# Plain static (default public Wisp handles AZ/FL chamber scrapes)
python3 -m http.server 8080 --directory web
# open http://127.0.0.1:8080/

# Optional: local Wisp instead of Mercury public instance
# ./scripts/run-static.sh   # then Settings → Wisp → “This origin”
```

| Settings | Notes |
|----------|--------|
| Mode | **Live** (default) or Fixture |
| FEC key | localStorage; DEMO_KEY is rate-limited — get one at [api.open.fec.gov](https://api.open.fec.gov/developers/) |
| Open States key | Optional; state legislature **incumbents on ballot** (`people.geo`) + detail votes |
| Google Civic key | Optional; multi-state VIP **full contests** when feed is live (seasonal; ZIP centroid) |
| Wisp URL | Default `wss://wisp.mercurywork.shop/` ([libcurl.js](https://github.com/ading2210/libcurl.js)); override or clear in Settings; local via `./scripts/run-static.sh` |
| CORS proxy | Fallback e.g. `https://corsproxy.io/?` |
| FL DOS | Live via Wisp/libcurl (native-equivalent); optional TSV upload/ship as fallback |

**Test ZIPs:** `33334` (FL) · `32901` (FL Brevard) · `85004`/`85701` (AZ) · `27601` (NC) · `21401` (MD) · `90210` (CA federal) · `10001` (NY federal)

### Browser pipeline

ZIP → Zippo → TIGERweb+FCC → FEC → optional FL/AZ bodies → WASM report → click candidate → staged enrich (FEC/GovTrack/OpenStates).

Responses cache in IndexedDB until you refresh data/score (Civic VIP 12h). Themes: 8 HUD swatches in Settings. Sage inspired by [SUB0PT1MAL](https://github.com/SUB0PT1MAL)’s Freenet archive ([Bluesky](https://bsky.app/profile/sub0pt1mal.bsky.social) · [ArtStation](https://www.artstation.com/sub0pt1mal) · [X](https://x.com/SUB0PT1MAL)).

## Project layout

```
crates/electionizer-core/   pure domain (no HTTP/SQLite/Axum)
crates/electionizer-wasm/   wasm-bindgen thin exports over core
web/                        static client (primary product)
  pkg/                      wasm-pack output (gitignored)
  data/                     optional FL DOS TSV + README
scripts/build-wasm.sh
src/                        native Axum + SQLite (optional/dev)
templates/ static/ migrations/ testdata/
```

```bash
cargo test --workspace
```

**Rule:** No Rust `HttpClient` / store trait for WASM. JS fetch + pure core.

## Coverage

| Region | What you get |
|--------|----------------|
| Federal (all ZIPs) | House/Senate via OpenFEC; finance + GovTrack votes on detail |
| Florida | Live DOS extract → legislature, statewide, judicial, county/local; chambers + amendments; candidate + measure TreFin via Wisp |
| Arizona | Leg roster + statewide OfficialList incumbents (Wisp); Clean Elections measures + FTM measure $ (thru 2024); SOS challengers blocked |
| North Carolina | NCSBE candidate listing CSV (direct CORS) — leg + county judicial/local; NCSBE referendum measures + FTM measure $ (thru 2024); federal via FEC |
| Maryland | SBE general/primary CSVs + ballot questions via Wisp; **MDCRIS live** measure $ (+ FTM archive); federal via FEC |
| Other states | Federal + **Google Civic full contests** when keyed and VIP live; else **Open States legislature incumbents** when keyed; FTM measure $ when measures present (thru 2024) |

ZIP ≠ precinct (centroid districts). Chamber HTML hosts need a CORS proxy in the browser.

## Native server (optional / dev)

Still supported for local SQLite caching, SSE candidate detail, and daemon refresh.

```bash
cp electionizer.example.toml electionizer.toml
# edit → set [fec].api_key

cargo run
# open http://127.0.0.1:3000
# --fresh · --provider fixture|live · --mode both|serve|daemon
```

| Source | Priority |
|--------|----------|
| CLI flags / env | highest |
| `electionizer.toml` | middle |
| built-in defaults | lowest (`DEMO_KEY`, live, …) |

Config keys: `--fec-api-key` / `ELECTIONIZER_FEC_API_KEY`, `--openstates-api-key`, `fec.cache_ttl_hours`, `--db`, `--bind`, `--cycle`. See `electionizer.example.toml`.

Native pipeline: Zippo → Census → FEC → FL DOS/AZ → SQLite; detail via SSE.

## Status

**Done (static + core)**

- Live federal ballot (TIGERweb/FCC + OpenFEC) and fixture mode
- Staged candidate detail (finance, votes, party timeline)
- FL DOS TSV path + AZ roster incumbents in browser
- Themes, voter portal links, IndexedDB cache
- Shared pure core used by WASM and native

**Done (native extras)**

- On-demand build + HTMX progress, job queue, daemon stale-ZIP refresh
- SQLite FEC cache, Settings UI writing `electionizer.toml`

**Planned**

- More states / full AZ challenger filings
- Fuller Schedule A / measure committee product
- Public Wisp endpoint for shared AZ/FL scrapes (or Freenet-side fetch)

See `HANDOFF.md` for backlog and continuity.

## Deploy

Ship `web/` including a built `web/pkg/` from `./scripts/build-wasm.sh`. No server secrets — users supply FEC/Open States/Wisp keys in Settings. Optional: include `web/data/fl-dos-{cycle}.tsv` for Florida without upload.

| Target | Notes |
|--------|--------|
| Pages / Netlify / CDN | Static only. `netlify.toml` runs `./scripts/build-wasm.sh` → publish `web/`. Federal + AZ/FL via default public Wisp. |
| VPS with Wisp | `./scripts/run-static.sh` or `wisp-python --static=web/`; users can switch Settings → This origin. |
| Freenet | Bundle static assets only; no API secrets. FEC keys client-side. Default Wisp is a public third-party endpoint (users may override/clear). |

`./scripts/run-static.sh` creates `.venv-wisp` (gitignored) and runs [wisp-python](https://pypi.org/project/wisp-python/) (AGPL server — separate from this app’s license).
