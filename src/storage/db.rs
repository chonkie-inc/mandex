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

/// Boost multiplier applied to AND results when merging with OR results.
/// FTS5 BM25 ranks are negative (more negative = better). We multiply AND ranks
/// by this factor to make them more negative (better) relative to OR-only results.
const AND_BOOST: f64 = 2.0;

/// Searches the FTS5 index using BM25 ranking.
/// Runs both AND and OR queries, merges results with AND results boosted.
pub fn search(conn: &Connection, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
    let stop_words: std::collections::HashSet<String> = stop_words::get(stop_words::LANGUAGE::English)
        .iter()
        .map(|s| s.to_string())
        .collect();

    // Strip punctuation, then filter stop words
    let cleaned: Vec<String> = query
        .split_whitespace()
        .map(|w| w.chars().filter(|c| c.is_alphanumeric()).collect::<String>())
        .filter(|w| !w.is_empty() && !stop_words.contains(&w.to_lowercase()))
        .collect();
    let words: Vec<&str> = cleaned.iter().map(|s| s.as_str()).collect();

    // Single word — no need for AND/OR merging
    if words.len() <= 1 {
        let fts_query = if words.is_empty() { query.to_string() } else { words[0].to_string() };
        return run_fts_query(conn, &fts_query, 1.0, limit);
    }

    let and_query = words.join(" ");
    let or_query = words.join(" OR ");

    // Run AND query (boosted)
    let and_results = run_fts_query(conn, &and_query, AND_BOOST, limit).unwrap_or_default();

    // Run OR query (unboosted)
    let or_results = run_fts_query(conn, &or_query, 1.0, limit).unwrap_or_default();

    // Merge: AND results first, then OR-only results, deduped by name
    let mut seen = std::collections::HashSet::new();
    let mut merged: Vec<SearchResult> = Vec::new();

    for r in and_results.into_iter().chain(or_results) {
        if seen.insert(r.name.clone()) {
            merged.push(r);
        }
    }

    // Sort by boosted rank (FTS5 ranks are negative — more negative is better)
    merged.sort_by(|a, b| a.rank.partial_cmp(&b.rank).unwrap_or(std::cmp::Ordering::Equal));
    merged.truncate(limit);

    Ok(merged)
}

fn run_fts_query(conn: &Connection, fts_query: &str, boost: f64, limit: usize) -> Result<Vec<SearchResult>> {
    let mut stmt = conn.prepare(
        "SELECT e.name, e.content, rank
         FROM entries_fts
         JOIN entries e ON e.id = entries_fts.rowid
         WHERE entries_fts MATCH ?1
         ORDER BY rank
         LIMIT ?2",
    )?;

    let results = stmt
        .query_map(rusqlite::params![fts_query, limit], |row| {
            Ok(SearchResult {
                name: row.get(0)?,
                content: row.get(1)?,
                rank: row.get::<_, f64>(2)? * boost,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(results)
}

/// Search result from the merged project index (includes package info).
pub struct IndexSearchResult {
    pub package: String,
    pub version: String,
    pub name: String,
    pub content: String,
    pub rank: f64,
}

/// Searches the merged project index.
/// If `package_filter` is Some, only returns results from that package.
pub fn search_index(
    conn: &Connection,
    query: &str,
    limit: usize,
    package_filter: Option<&str>,
) -> Result<Vec<IndexSearchResult>> {
    let stop_words: std::collections::HashSet<String> = stop_words::get(stop_words::LANGUAGE::English)
        .iter()
        .map(|s| s.to_string())
        .collect();

    let cleaned: Vec<String> = query
        .split_whitespace()
        .map(|w| w.chars().filter(|c| c.is_alphanumeric()).collect::<String>())
        .filter(|w| !w.is_empty() && !stop_words.contains(&w.to_lowercase()))
        .collect();
    let words: Vec<&str> = cleaned.iter().map(|s| s.as_str()).collect();

    if words.len() <= 1 {
        let fts_query = if words.is_empty() { query.to_string() } else { words[0].to_string() };
        return run_index_fts_query(conn, &fts_query, 1.0, limit, package_filter);
    }

    let and_query = words.join(" ");
    let or_query = words.join(" OR ");

    let and_results = run_index_fts_query(conn, &and_query, AND_BOOST, limit, package_filter).unwrap_or_default();
    let or_results = run_index_fts_query(conn, &or_query, 1.0, limit, package_filter).unwrap_or_default();

    let mut seen = std::collections::HashSet::new();
    let mut merged: Vec<IndexSearchResult> = Vec::new();

    for r in and_results.into_iter().chain(or_results) {
        // Dedup by (package, name) to avoid cross-package collisions
        let key = format!("{}:{}", r.package, r.name);
        if seen.insert(key) {
            merged.push(r);
        }
    }

    merged.sort_by(|a, b| a.rank.partial_cmp(&b.rank).unwrap_or(std::cmp::Ordering::Equal));
    merged.truncate(limit);

    Ok(merged)
}

fn run_index_fts_query(
    conn: &Connection,
    fts_query: &str,
    boost: f64,
    limit: usize,
    package_filter: Option<&str>,
) -> Result<Vec<IndexSearchResult>> {
    let (sql, params_vec): (String, Vec<Box<dyn rusqlite::types::ToSql>>) = if let Some(pkg) = package_filter {
        (
            "SELECT e.package, e.version, e.name, e.content, rank
             FROM entries_fts
             JOIN entries e ON e.id = entries_fts.rowid
             WHERE entries_fts MATCH ?1 AND e.package = ?2
             ORDER BY rank
             LIMIT ?3".to_string(),
            vec![
                Box::new(fts_query.to_string()) as Box<dyn rusqlite::types::ToSql>,
                Box::new(pkg.to_string()),
                Box::new(limit as i64),
            ],
        )
    } else {
        (
            "SELECT e.package, e.version, e.name, e.content, rank
             FROM entries_fts
             JOIN entries e ON e.id = entries_fts.rowid
             WHERE entries_fts MATCH ?1
             ORDER BY rank
             LIMIT ?2".to_string(),
            vec![
                Box::new(fts_query.to_string()) as Box<dyn rusqlite::types::ToSql>,
                Box::new(limit as i64),
            ],
        )
    };

    let mut stmt = conn.prepare(&sql)?;
    let params_refs: Vec<&dyn rusqlite::types::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();
    let results = stmt
        .query_map(params_refs.as_slice(), |row| {
            Ok(IndexSearchResult {
                package: row.get(0)?,
                version: row.get(1)?,
                name: row.get(2)?,
                content: row.get(3)?,
                rank: row.get::<_, f64>(4)? * boost,
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
