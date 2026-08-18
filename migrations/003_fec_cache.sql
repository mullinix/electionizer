CREATE TABLE IF NOT EXISTS fec_cache (
    cache_key TEXT PRIMARY KEY NOT NULL,
    state TEXT NOT NULL,
    office TEXT NOT NULL,
    district TEXT,
    cycle INTEGER NOT NULL,
    body_json TEXT NOT NULL,
    fetched_at TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_fec_cache_fetched ON fec_cache(fetched_at);
