# electionizer MCP

Local stdio MCP server for agent sessions on this repo. Prefer these tools over
blind full-file reads and ad-hoc shell greps.

## Tools

| Tool | Purpose |
|------|---------|
| `electionizer_status` | Repo health, wasm pkg, git, tooling (rg, rust-analyzer, ts lsp) |
| `electionizer_next` | Priority work **from HANDOFF plan** (edit HANDOFF, not here) |
| `electionizer_handoff` | Read `HANDOFF.md` (`plan` \| `done` \| `rules` \| `gaps` \| …) |
| `electionizer_rules` | Continuity rules (WASM purity, Wisp, no VR warehouse, …) |
| `electionizer_where` | Topic → primary files (`affiliation`, `finance`, `maryland`, …) |
| `electionizer_stages` | Locked finance/detail/affiliation stage order (no file I/O) |
| `electionizer_body_keys` | `state_bodies_json` key map |
| `electionizer_rg` | Project-scoped **ripgrep** (skips target/node_modules/pkg) |
| `electionizer_def` | Definition hits for a symbol (fn/struct/export) |
| `electionizer_refs` | Reference hits for a symbol |
| `electionizer_outline` | File outline with line numbers (large files) |
| `electionizer_slice` | Read ±N lines around a line (avoid full Read) |
| `electionizer_exports` | wasm-bindgen / `web/pkg` export surface |
| `electionizer_check` | Fast `cargo check` diagnostics (core/wasm/native/workspace) |
| `electionizer_test` | `cargo test` core (or `workspace: true`) |
| `electionizer_build_wasm` | `./scripts/build-wasm.sh` |
| `electionizer_arch_check` | WASM purity + wiring invariants |
| `electionizer_serve_help` | How to serve static / Wisp / smoke ZIPs |

## Session workflow

```
electionizer_status
electionizer_next
electionizer_where topic=affiliation   # or finance / florida / …
electionizer_def name=ballot_affiliations
electionizer_outline path=web/enrich.js
electionizer_slice path=… line=N
# …edit…
electionizer_check          # or electionizer_test
electionizer_build_wasm     # if core/wasm changed
# update HANDOFF Done + Plan before commit
```

## Wire-up

Project root `opencode.json` (also enables **LSP**: rust-analyzer + typescript-language-server):

```json
"mcp": {
  "electionizer": {
    "type": "local",
    "command": ["node", "mcp/electionizer/server.js"],
    "enabled": true,
    "timeout": 600000
  }
}
```

```bash
cd mcp/electionizer && npm install   # MCP SDK once
cd ../.. && npm install              # root: typescript-language-server (dev only)
rustup component add rust-analyzer   # once; linked to ~/.cargo/bin
# restart opencode after MCP or lsp config changes
```

Optional: `ELECTIONIZER_ROOT=/path/to/repo` if cwd is not the monorepo root.

## Design notes (why these tools)

Agents burn tokens re-reading `enrich.js` / core crates. This MCP keeps:

- **product locks** (stage order, body keys, where-map) as structured answers
- **navigation** (def/refs/outline/slice/rg) so only relevant windows are loaded
- **verify** (check/test/build/arch) as one-shot tools with clipped output
