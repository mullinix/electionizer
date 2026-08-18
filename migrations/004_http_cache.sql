CREATE TABLE IF NOT EXISTS http_cache (
    cache_key TEXT PRIMARY KEY NOT NULL,
    body TEXT NOT NULL,
    content_type TEXT,
    source_url TEXT,
    fetched_at TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_http_cache_fetched ON http_cache(fetched_at);
