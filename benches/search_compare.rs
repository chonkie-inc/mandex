use anyhow::Result;
use rusqlite::Connection;
use std::collections::HashSet;
use std::time::Instant;
use tantivy::collector::TopDocs;
use tantivy::query::{BooleanQuery, BoostQuery, FuzzyTermQuery, Occur, QueryParser};
use tantivy::schema::*;
use tantivy::{doc, Index, IndexWriter, ReloadPolicy, Term};

// ─── Test queries by category ───────────────────────────
const BASIC_QUERIES: &[&str] = &[
    "install",
    "how to install",
    "sqlite schema",
    "building packages",
    "search documentation",
];

const TYPO_QUERIES: &[(&str, &str)] = &[
    ("instal", "install"),
    ("sqllite", "sqlite"),
    ("packges", "packages"),
    ("documentaton", "documentation"),
    ("comprssion", "compression"),
];

const PREFIX_QUERIES: &[(&str, &str)] = &[
    ("auth", "should match authentication/authorize"),
    ("comp", "should match compression/compatible"),
    ("mark", "should match markdown"),
    ("ver", "should match version/verify"),
    ("inst", "should match install/installed"),
];

const AMBIGUOUS_QUERIES: &[&str] = &[
    "file",
    "package",
    "search",
    "command",
    "version",
];

const PHRASE_QUERIES: &[&str] = &[
    "full text search",
    "documentation packages",
    "custom loss function",
    "installed packages",
    "search index",
];

fn main() -> Result<()> {
    let db_path = std::env::args()
        .nth(1)
        .filter(|a| !a.starts_with('-') && a.ends_with(".db"))
        .unwrap_or_else(|| {
            let home = dirs::home_dir().unwrap();
            let p = home.join(".mandex/cache/mandex/0.1.0.db");
            if p.exists() {
                return p.to_string_lossy().to_string();
            }
            panic!("No mandex package found. Run: mx pull mandex@0.1.0");
        });

    println!("Database: {db_path}\n");

    // Load entries from SQLite
    let conn = Connection::open(&db_path)?;
    let mut stmt = conn.prepare("SELECT name, content FROM entries")?;
    let entries: Vec<(String, String)> = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
        .filter_map(|r| r.ok())
        .collect();

    println!("Loaded {} entries\n", entries.len());

    // Load stop words
    let stop_words: HashSet<String> = stop_words::get(stop_words::LANGUAGE::English)
        .iter()
        .map(|s| s.to_string())
        .collect();

    // ─── Build Tantivy index ──────────────────────────────
    let tmp_dir = tempfile::tempdir()?;
    let tmp_path = tmp_dir.path();

    let mut schema_builder = Schema::builder();
    let name_field = schema_builder.add_text_field("name", TEXT | STORED);
    let content_field = schema_builder.add_text_field("content", TEXT | STORED);
    let schema = schema_builder.build();

    let index = Index::create_in_dir(tmp_path, schema.clone())?;
    let mut index_writer: IndexWriter = index.writer(50_000_000)?;

    for (name, content) in &entries {
        index_writer.add_document(doc!(
            name_field => name.as_str(),
            content_field => content.as_str(),
        ))?;
    }
    index_writer.commit()?;

    let reader = index
        .reader_builder()
        .reload_policy(ReloadPolicy::Manual)
        .try_into()?;
    let searcher = reader.searcher();
    let query_parser = QueryParser::for_index(&index, vec![name_field, content_field]);

    // Size comparison
    let fts5_size = std::fs::metadata(&db_path)?.len();
    let tantivy_size: u64 = walkdir::WalkDir::new(tmp_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter_map(|e| e.metadata().ok())
        .map(|m| m.len())
        .sum();

    println!("══════════════════════════════════════════════════════════");
    println!("  SIZE COMPARISON");
    println!("══════════════════════════════════════════════════════════");
    println!("  FTS5 (data + index):    {:.1} KB", fts5_size as f64 / 1024.0);
    println!("  Tantivy index only:     {:.1} KB", tantivy_size as f64 / 1024.0);
    println!("  Tantivy total (db+idx): {:.1} KB", (fts5_size + tantivy_size) as f64 / 1024.0);
    println!();

    // ═════════════════════════════════════════════════════════
    // TEST 1: BASIC QUERIES
    // ═════════════════════════════════════════════════════════
    println!("══════════════════════════════════════════════════════════");
    println!("  TEST 1: BASIC QUERIES");
    println!("══════════════════════════════════════════════════════════\n");

    for query in BASIC_QUERIES {
        let (fts_results, fts_time) = fts5_search(&conn, query, &stop_words)?;
        let (tv_results, tv_time) = tantivy_search(&searcher, &query_parser, query)?;

        println!("  Q: \"{query}\"");
        println!("  FTS5 ({:.0}μs): {}", fts_time, format_results(&fts_results));
        println!("  Tantivy ({:.0}μs): {}", tv_time, format_results_f32(&tv_results));
        println!();
    }

    // ═════════════════════════════════════════════════════════
    // TEST 2: TYPO TOLERANCE / FUZZY MATCHING
    // ═════════════════════════════════════════════════════════
    println!("══════════════════════════════════════════════════════════");
    println!("  TEST 2: TYPO TOLERANCE (fuzzy matching)");
    println!("══════════════════════════════════════════════════════════\n");

    for (typo, correct) in TYPO_QUERIES {
        let (fts_results, fts_time) = fts5_search(&conn, typo, &stop_words)?;

        // Tantivy fuzzy search (Levenshtein distance 1)
        let (tv_results, tv_time) = {
            let tv_start = Instant::now();
            let mut fuzzy_queries: Vec<(Occur, Box<dyn tantivy::query::Query>)> = Vec::new();
            for word in typo.split_whitespace() {
                let term_name = Term::from_field_text(name_field, word);
                let term_content = Term::from_field_text(content_field, word);
                let fq_name = FuzzyTermQuery::new(term_name, 1, true);
                let fq_content = FuzzyTermQuery::new(term_content, 1, true);
                fuzzy_queries.push((Occur::Should, Box::new(fq_name)));
                fuzzy_queries.push((Occur::Should, Box::new(fq_content)));
            }
            let bool_query = BooleanQuery::new(fuzzy_queries);
            let top_docs = searcher.search(&bool_query, &TopDocs::with_limit(5))?;

            let results: Vec<(String, f32)> = top_docs
                .iter()
                .map(|(score, addr)| {
                    let doc = searcher.doc::<TantivyDocument>(*addr).unwrap();
                    let name = doc.get_first(name_field).and_then(|v| v.as_str()).unwrap_or("?").to_string();
                    (name, *score)
                })
                .collect();
            (results, tv_start.elapsed().as_micros())
        };

        println!("  Q: \"{typo}\" (intended: \"{correct}\")");
        println!("  FTS5 ({:.0}μs): {}", fts_time, if fts_results.is_empty() { "NO RESULTS".to_string() } else { format_results(&fts_results) });
        println!("  Tantivy fuzzy ({:.0}μs): {}", tv_time, if tv_results.is_empty() { "NO RESULTS".to_string() } else { format_results_f32(&tv_results) });
        println!();
    }

    // ═════════════════════════════════════════════════════════
    // TEST 3: PREFIX / PARTIAL MATCHING
    // ═════════════════════════════════════════════════════════
    println!("══════════════════════════════════════════════════════════");
    println!("  TEST 3: PREFIX / PARTIAL MATCHING");
    println!("══════════════════════════════════════════════════════════\n");

    for (prefix, desc) in PREFIX_QUERIES {
        // FTS5 prefix query uses *
        let fts_query = format!("{prefix}*");
        let (fts_results, fts_time) = fts5_search_raw(&conn, &fts_query)?;

        // Tantivy prefix query
        let tv_query_str = format!("{prefix}*");
        let (tv_results, tv_time) = {
            let tv_start = Instant::now();
            let parsed = query_parser.parse_query(&tv_query_str);
            let results = if let Ok(q) = &parsed {
                let top_docs = searcher.search(q.as_ref(), &TopDocs::with_limit(5))?;
                top_docs
                    .iter()
                    .map(|(score, addr)| {
                        let doc = searcher.doc::<TantivyDocument>(*addr).unwrap();
                        let name = doc.get_first(name_field).and_then(|v| v.as_str()).unwrap_or("?").to_string();
                        (name, *score)
                    })
                    .collect::<Vec<_>>()
            } else {
                Vec::new()
            };
            (results, tv_start.elapsed().as_micros())
        };

        println!("  Q: \"{prefix}\" ({desc})");
        println!("  FTS5 \"{fts_query}\" ({:.0}μs): {}", fts_time, if fts_results.is_empty() { "NO RESULTS".to_string() } else { format_results(&fts_results) });
        println!("  Tantivy \"{tv_query_str}\" ({:.0}μs): {}", tv_time, if tv_results.is_empty() { "NO RESULTS".to_string() } else { format_results_f32(&tv_results) });
        println!();
    }

    // ═════════════════════════════════════════════════════════
    // TEST 4: PHRASE QUERIES
    // ═════════════════════════════════════════════════════════
    println!("══════════════════════════════════════════════════════════");
    println!("  TEST 4: PHRASE QUERIES");
    println!("══════════════════════════════════════════════════════════\n");

    for query in PHRASE_QUERIES {
        // FTS5 phrase query
        let fts_phrase = format!("\"{}\"", query);
        let (fts_results, fts_time) = fts5_search_raw(&conn, &fts_phrase)?;

        // Tantivy phrase query
        let tv_phrase = format!("\"{}\"", query);
        let (tv_results, tv_time) = {
            let tv_start = Instant::now();
            let parsed = query_parser.parse_query(&tv_phrase)?;
            let top_docs = searcher.search(&parsed, &TopDocs::with_limit(5))?;
            let results: Vec<(String, f32)> = top_docs
                .iter()
                .map(|(score, addr)| {
                    let doc = searcher.doc::<TantivyDocument>(*addr).unwrap();
                    let name = doc.get_first(name_field).and_then(|v| v.as_str()).unwrap_or("?").to_string();
                    (name, *score)
                })
                .collect();
            (results, tv_start.elapsed().as_micros())
        };

        println!("  Q: \"{query}\" (exact phrase)");
        println!("  FTS5 ({:.0}μs): {}", fts_time, if fts_results.is_empty() { "NO RESULTS".to_string() } else { format_results(&fts_results) });
        println!("  Tantivy ({:.0}μs): {}", tv_time, if tv_results.is_empty() { "NO RESULTS".to_string() } else { format_results_f32(&tv_results) });
        println!();
    }

    // ═════════════════════════════════════════════════════════
    // TEST 5: AMBIGUOUS QUERIES (many matches, test ranking)
    // ═════════════════════════════════════════════════════════
    println!("══════════════════════════════════════════════════════════");
    println!("  TEST 5: AMBIGUOUS QUERIES (ranking quality)");
    println!("══════════════════════════════════════════════════════════\n");

    for query in AMBIGUOUS_QUERIES {
        let (fts_results, fts_time) = fts5_search(&conn, query, &stop_words)?;
        let (tv_results, tv_time) = tantivy_search(&searcher, &query_parser, query)?;

        println!("  Q: \"{query}\"");
        println!("  FTS5 ({:.0}μs, {} results): {}", fts_time, fts_results.len(), format_results(&fts_results));
        println!("  Tantivy ({:.0}μs, {} results): {}", tv_time, tv_results.len(), format_results_f32(&tv_results));
        println!();
    }

    // ═════════════════════════════════════════════════════════
    // TEST 6: FIELD BOOSTING (name match vs content match)
    // ═════════════════════════════════════════════════════════
    println!("══════════════════════════════════════════════════════════");
    println!("  TEST 6: FIELD BOOSTING (name vs content)");
    println!("══════════════════════════════════════════════════════════\n");

    let boosted_queries = &["schema", "install", "compression", "overview"];
    for query in boosted_queries {
        // FTS5: no field boosting possible
        let (fts_results, fts_time) = fts5_search(&conn, query, &stop_words)?;

        // Tantivy: boost name field 3x
        let (tv_results, tv_time) = {
            let tv_start = Instant::now();
            let term_name = Term::from_field_text(name_field, query);
            let term_content = Term::from_field_text(content_field, query);
            let name_query = tantivy::query::TermQuery::new(term_name, IndexRecordOption::WithFreqsAndPositions);
            let content_query = tantivy::query::TermQuery::new(term_content, IndexRecordOption::WithFreqsAndPositions);
            let boosted_name = BoostQuery::new(Box::new(name_query), 3.0);
            let bool_query = BooleanQuery::new(vec![
                (Occur::Should, Box::new(boosted_name)),
                (Occur::Should, Box::new(content_query)),
            ]);
            let top_docs = searcher.search(&bool_query, &TopDocs::with_limit(5))?;
            let results: Vec<(String, f32)> = top_docs
                .iter()
                .map(|(score, addr)| {
                    let doc = searcher.doc::<TantivyDocument>(*addr).unwrap();
                    let name = doc.get_first(name_field).and_then(|v| v.as_str()).unwrap_or("?").to_string();
                    (name, *score)
                })
                .collect();
            (results, tv_start.elapsed().as_micros())
        };

        // Tantivy without boosting for comparison
        let (tv_unboosted, _) = tantivy_search(&searcher, &query_parser, query)?;

        println!("  Q: \"{query}\"");
        println!("  FTS5 (no boost, {:.0}μs):        {}", fts_time, format_results(&fts_results));
        println!("  Tantivy (no boost):             {}", format_results_f32(&tv_unboosted));
        println!("  Tantivy (name 3x boost, {:.0}μs): {}", tv_time, format_results_f32(&tv_results));
        println!();
    }

    // ═════════════════════════════════════════════════════════
    // TEST 7: LATENCY UNDER LOAD (many sequential queries)
    // ═════════════════════════════════════════════════════════
    println!("══════════════════════════════════════════════════════════");
    println!("  TEST 7: LATENCY — 100 sequential queries");
    println!("══════════════════════════════════════════════════════════\n");

    let all_queries: Vec<&str> = BASIC_QUERIES.iter()
        .chain(AMBIGUOUS_QUERIES.iter())
        .copied()
        .collect();

    // FTS5
    let fts_start = Instant::now();
    for _ in 0..10 {
        for q in &all_queries {
            let _ = fts5_search(&conn, q, &stop_words)?;
        }
    }
    let fts_total = fts_start.elapsed();

    // Tantivy
    let tv_start = Instant::now();
    for _ in 0..10 {
        for q in &all_queries {
            let _ = tantivy_search(&searcher, &query_parser, q)?;
        }
    }
    let tv_total = tv_start.elapsed();

    println!("  FTS5:    {:.1}ms total ({:.0}μs/query avg)", fts_total.as_secs_f64() * 1000.0, fts_total.as_micros() as f64 / 100.0);
    println!("  Tantivy: {:.1}ms total ({:.0}μs/query avg)", tv_total.as_secs_f64() * 1000.0, tv_total.as_micros() as f64 / 100.0);
    println!();

    // ═════════════════════════════════════════════════════════
    // SUMMARY
    // ═════════════════════════════════════════════════════════
    println!("══════════════════════════════════════════════════════════");
    println!("  SUMMARY");
    println!("══════════════════════════════════════════════════════════\n");
    println!("  Package size:   FTS5 {:.1}KB vs Tantivy {:.1}KB (+{:.0}%)",
        fts5_size as f64 / 1024.0,
        (fts5_size + tantivy_size) as f64 / 1024.0,
        (tantivy_size as f64 / fts5_size as f64) * 100.0
    );
    println!("  Avg latency:    FTS5 {:.0}μs vs Tantivy {:.0}μs",
        fts_total.as_micros() as f64 / 100.0,
        tv_total.as_micros() as f64 / 100.0,
    );
    println!("  Fuzzy matching: FTS5 ✗  vs Tantivy ✓");
    println!("  Field boosting: FTS5 ✗  vs Tantivy ✓");
    println!("  Prefix queries: FTS5 ✓ (manual *) vs Tantivy ✓");
    println!("  Stop words:     FTS5 ✗ (needs crate) vs Tantivy ✗ (needs config)");
    println!("  Portability:    FTS5 ✓ (any SQLite client) vs Tantivy ✗ (Tantivy only)");
    println!("  Dependencies:   FTS5 +1 (stop-words) vs Tantivy +99 crates");

    Ok(())
}

// ─── Helper functions ───────────────────────────────────

fn fts5_search(conn: &Connection, query: &str, stop_words: &HashSet<String>) -> Result<(Vec<(String, f64)>, u128)> {
    let words: Vec<&str> = query
        .split_whitespace()
        .filter(|w| !stop_words.contains(&w.to_lowercase()))
        .collect();

    let fts_query = if words.is_empty() {
        query.to_string()
    } else if words.len() == 1 {
        words[0].to_string()
    } else {
        words.join(" OR ")
    };

    fts5_search_raw(conn, &fts_query)
}

fn fts5_search_raw(conn: &Connection, fts_query: &str) -> Result<(Vec<(String, f64)>, u128)> {
    let start = Instant::now();

    let mut stmt = conn.prepare(
        "SELECT e.name, rank
         FROM entries_fts
         JOIN entries e ON e.id = entries_fts.rowid
         WHERE entries_fts MATCH ?1
         ORDER BY rank
         LIMIT 5",
    )?;

    let results: Vec<(String, f64)> = stmt
        .query_map([fts_query], |row| Ok((row.get(0)?, row.get(1)?)))?
        .filter_map(|r| r.ok())
        .collect();

    let elapsed = start.elapsed().as_micros();
    Ok((results, elapsed))
}

fn tantivy_search(
    searcher: &tantivy::Searcher,
    query_parser: &QueryParser,
    query: &str,
) -> Result<(Vec<(String, f32)>, u128)> {
    let schema = searcher.schema();
    let name_field = schema.get_field("name").unwrap();

    let start = Instant::now();
    let parsed = query_parser.parse_query(query)?;
    let top_docs = searcher.search(&parsed, &TopDocs::with_limit(5))?;

    let results: Vec<(String, f32)> = top_docs
        .iter()
        .map(|(score, addr)| {
            let doc = searcher.doc::<TantivyDocument>(*addr).unwrap();
            let name = doc.get_first(name_field).and_then(|v| v.as_str()).unwrap_or("?").to_string();
            (name, *score)
        })
        .collect();

    let elapsed = start.elapsed().as_micros();
    Ok((results, elapsed))
}

fn format_results(results: &[(String, f64)]) -> String {
    if results.is_empty() {
        return "NO RESULTS".to_string();
    }
    results
        .iter()
        .map(|(name, _)| name.as_str())
        .collect::<Vec<_>>()
        .join(" | ")
}

fn format_results_f32(results: &[(String, f32)]) -> String {
    if results.is_empty() {
        return "NO RESULTS".to_string();
    }
    results
        .iter()
        .map(|(name, _)| name.as_str())
        .collect::<Vec<_>>()
        .join(" | ")
}
