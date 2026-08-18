CREATE TABLE IF NOT EXISTS zips (
    zip TEXT PRIMARY KEY NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    last_built_at TEXT,
    error TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS build_jobs (
    id TEXT PRIMARY KEY NOT NULL,
    zip TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'queued',
    stage TEXT NOT NULL DEFAULT 'queued',
    progress_pct INTEGER NOT NULL DEFAULT 0,
    message TEXT NOT NULL DEFAULT '',
    error TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    finished_at TEXT
);

CREATE INDEX IF NOT EXISTS idx_build_jobs_status ON build_jobs(status);
CREATE INDEX IF NOT EXISTS idx_build_jobs_zip ON build_jobs(zip);

CREATE TABLE IF NOT EXISTS jurisdictions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    ocd_id TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    level TEXT NOT NULL,
    state TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS zip_jurisdictions (
    zip TEXT NOT NULL,
    jurisdiction_id INTEGER NOT NULL,
    PRIMARY KEY (zip, jurisdiction_id),
    FOREIGN KEY (zip) REFERENCES zips(zip),
    FOREIGN KEY (jurisdiction_id) REFERENCES jurisdictions(id)
);

CREATE TABLE IF NOT EXISTS elections (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    election_date TEXT NOT NULL,
    scope TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(name, election_date)
);

CREATE TABLE IF NOT EXISTS races (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    election_id INTEGER NOT NULL,
    jurisdiction_id INTEGER NOT NULL,
    office TEXT NOT NULL,
    chamber TEXT,
    is_judicial INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (election_id) REFERENCES elections(id),
    FOREIGN KEY (jurisdiction_id) REFERENCES jurisdictions(id),
    UNIQUE(election_id, jurisdiction_id, office)
);

CREATE TABLE IF NOT EXISTS candidates (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    race_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    party TEXT NOT NULL,
    is_incumbent INTEGER NOT NULL DEFAULT 0,
    is_judge INTEGER NOT NULL DEFAULT 0,
    summary TEXT,
    external_id TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (race_id) REFERENCES races(id),
    UNIQUE(race_id, name)
);

CREATE INDEX IF NOT EXISTS idx_candidates_party ON candidates(party);

CREATE TABLE IF NOT EXISTS ballot_measures (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    election_id INTEGER NOT NULL,
    jurisdiction_id INTEGER NOT NULL,
    title TEXT NOT NULL,
    summary TEXT,
    measure_code TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (election_id) REFERENCES elections(id),
    FOREIGN KEY (jurisdiction_id) REFERENCES jurisdictions(id),
    UNIQUE(election_id, jurisdiction_id, title)
);

CREATE TABLE IF NOT EXISTS sources (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    url TEXT NOT NULL,
    publisher TEXT,
    retrieved_at TEXT NOT NULL,
    content_hash TEXT,
    note TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(url, retrieved_at)
);

CREATE TABLE IF NOT EXISTS entity_sources (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    entity_type TEXT NOT NULL,
    entity_id INTEGER NOT NULL,
    source_id INTEGER NOT NULL,
    FOREIGN KEY (source_id) REFERENCES sources(id),
    UNIQUE(entity_type, entity_id, source_id)
);

CREATE INDEX IF NOT EXISTS idx_entity_sources_entity ON entity_sources(entity_type, entity_id);

CREATE TABLE IF NOT EXISTS scrape_runs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    provider TEXT NOT NULL,
    zip TEXT,
    started_at TEXT NOT NULL,
    finished_at TEXT,
    status TEXT NOT NULL DEFAULT 'running',
    stats_json TEXT,
    error TEXT
);
