use anyhow::{Context, Result};
use rusqlite::Connection;
use std::path::Path;

/// Schema version for mandex databases
const SCHEMA_VERSION: i32 = 1;

/// Creates a new mandex database with the required schema
pub fn create_db(path: &Path) -> Result<Connection> {
    let conn = Connection::open(path)?;

    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS metadata (
            key   TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS entries (
            id      INTEGER PRIMARY KEY AUTOINCREMENT,
            name    TEXT NOT NULL,
            content TEXT NOT NULL
        );

        CREATE VIRTUAL TABLE IF NOT EXISTS entries_fts USING fts5(
            name,
            content,
            content=entries,
            content_rowid=id,
            tokenize='porter unicode61'
        );

        -- Triggers to keep FTS index in sync
        CREATE TRIGGER IF NOT EXISTS entries_ai AFTER INSERT ON entries BEGIN
            INSERT INTO entries_fts(rowid, name, content)
            VALUES (new.id, new.name, new.content);
        END;

        CREATE TRIGGER IF NOT EXISTS entries_ad AFTER DELETE ON entries BEGIN
            INSERT INTO entries_fts(entries_fts, rowid, name, content)
            VALUES ('delete', old.id, old.name, old.content);
        END;

        CREATE TRIGGER IF NOT EXISTS entries_au AFTER UPDATE ON entries BEGIN
            INSERT INTO entries_fts(entries_fts, rowid, name, content)
            VALUES ('delete', old.id, old.name, old.content);
            INSERT INTO entries_fts(rowid, name, content)
            VALUES (new.id, new.name, new.content);
        END;
        ",
    )?;

    conn.execute(
        "INSERT OR REPLACE INTO metadata (key, value) VALUES ('schema_version', ?1)",
        [SCHEMA_VERSION.to_string()],
    )?;

    Ok(conn)
}

/// Opens an existing mandex database
pub fn open_db(path: &Path) -> Result<Connection> {
    Connection::open(path).with_context(|| format!("Failed to open database: {}", path.display()))
}

/// Inserts a documentation entry
pub fn insert_entry(conn: &Connection, name: &str, content: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO entries (name, content) VALUES (?1, ?2)",
        [name, content],
    )?;
    Ok(())
}

/// Sets a metadata key-value pair
pub fn set_metadata(conn: &Connection, key: &str, value: &str) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO metadata (key, value) VALUES (?1, ?2)",
        [key, value],
    )?;
    Ok(())
}

/// Gets a metadata value by key
pub fn get_metadata(conn: &Connection, key: &str) -> Result<Option<String>> {
    let mut stmt = conn.prepare("SELECT value FROM metadata WHERE key = ?1")?;
    let result = stmt
        .query_row([key], |row| row.get(0))
        .ok();
    Ok(result)
}

/// Search result from FTS5 query
pub struct SearchResult {
    pub name: String,
    pub content: String,
    #[allow(dead_code)]
    pub rank: f64,
}

/// Searches the FTS5 index using BM25 ranking
pub fn search(conn: &Connection, query: &str) -> Result<Vec<SearchResult>> {
    let fts_query = query
        .split_whitespace()
        .map(|w| format!("\"{w}\""))
        .collect::<Vec<_>>()
        .join(" ");

    let mut stmt = conn.prepare(
        "SELECT e.name, e.content, rank
         FROM entries_fts
         JOIN entries e ON e.id = entries_fts.rowid
         WHERE entries_fts MATCH ?1
         ORDER BY rank
         LIMIT 20",
    )?;

    let results = stmt
        .query_map([&fts_query], |row| {
            Ok(SearchResult {
                name: row.get(0)?,
                content: row.get(1)?,
                rank: row.get(2)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(results)
}

/// Gets a specific entry by name
pub fn get_entry(conn: &Connection, name: &str) -> Result<Option<(String, String)>> {
    let mut stmt = conn.prepare("SELECT name, content FROM entries WHERE name = ?1")?;
    let result = stmt
        .query_row([name], |row| Ok((row.get(0)?, row.get(1)?)))
        .ok();
    Ok(result)
}

/// Gets the entry count in the database
pub fn entry_count(conn: &Connection) -> Result<usize> {
    let count: usize = conn.query_row("SELECT COUNT(*) FROM entries", [], |row| row.get(0))?;
    Ok(count)
}
