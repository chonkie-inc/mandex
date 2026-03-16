CREATE TABLE IF NOT EXISTS packages (
    name        TEXT PRIMARY KEY,
    description TEXT NOT NULL DEFAULT '',
    ecosystem   TEXT NOT NULL DEFAULT ''
);

CREATE TABLE IF NOT EXISTS versions (
    package      TEXT NOT NULL REFERENCES packages(name),
    version      TEXT NOT NULL,
    entry_count  INTEGER NOT NULL DEFAULT 0,
    size_bytes   INTEGER NOT NULL DEFAULT 0,
    published_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (package, version)
);
