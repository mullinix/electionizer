# Optional static state extracts

## Florida DOS candidate list

The static app fetches DOS Candidate Tracking **live** through Wisp + libcurl.js
(POST `extractCanList.asp` for STA+MUL+LOC), matching the native server path.
Results cache in IndexedDB until you refresh data.

Optional fallbacks if live fetch fails:

1. **Upload** a merged TSV in Settings (localStorage).
2. **Ship** `fl-dos-2026.tsv` (or `fl-cts-2026.tsv`) in this directory.
3. **CORS proxy** POST (advanced; less reliable than Wisp).

### Refresh script (offline / CDN ship)

```bash
./scripts/refresh-fl-dos.sh
# or: ./scripts/refresh-fl-dos.sh 2026 20261103-GEN
```

Place `fl-dos-2026.tsv` here and redeploy `web/` if you want a no-network fallback.
