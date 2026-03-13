use anyhow::{Context, Result};
use std::path::Path;
use walkdir::WalkDir;

use crate::storage::db;

pub fn run(path: &str, name: &str, version: &str, output: Option<&str>) -> Result<()> {
    let doc_dir = Path::new(path);
    if !doc_dir.is_dir() {
        anyhow::bail!("'{}' is not a directory", path);
    }

    let output_path = match output {
        Some(p) => p.to_string(),
        None => format!("{name}@{version}.mandex"),
    };

    let db_path = format!("{name}@{version}.db");

    // Create the SQLite database
    println!("Building {name}@{version}...");
    let conn = db::create_db(Path::new(&db_path))?;

    // Set metadata
    db::set_metadata(&conn, "name", name)?;
    db::set_metadata(&conn, "version", version)?;

    // Walk the directory and index markdown files
    let mut count = 0;
    for entry in WalkDir::new(doc_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| {
            let path = e.path();
            matches!(
                path.extension().and_then(|s| s.to_str()),
                Some("md" | "mdx" | "markdown")
            )
        })
    {
        let content =
            std::fs::read_to_string(entry.path()).with_context(|| {
                format!("Failed to read {}", entry.path().display())
            })?;

        if content.trim().is_empty() {
            continue;
        }

        // Extract name from first heading or use filename
        let entry_name = extract_heading(&content).unwrap_or_else(|| {
            entry
                .path()
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "untitled".to_string())
        });

        db::insert_entry(&conn, &entry_name, &content)?;
        count += 1;
    }

    if count == 0 {
        // Clean up empty db
        drop(conn);
        let _ = std::fs::remove_file(&db_path);
        anyhow::bail!("No markdown files found in '{}'", path);
    }

    db::set_metadata(&conn, "entry_count", &count.to_string())?;
    drop(conn);

    // Compress with zstd
    println!("Compressing...");
    let db_bytes = std::fs::read(&db_path)?;
    let compressed = zstd::encode_all(db_bytes.as_slice(), 19)?;

    std::fs::write(&output_path, &compressed)?;
    let _ = std::fs::remove_file(&db_path);

    let ratio = db_bytes.len() as f64 / compressed.len() as f64;
    println!(
        "Built {output_path} ({count} entries, {:.1} MB → {:.1} MB, {ratio:.1}x compression)",
        db_bytes.len() as f64 / 1_048_576.0,
        compressed.len() as f64 / 1_048_576.0,
    );

    Ok(())
}

/// Extracts the first markdown heading from content
fn extract_heading(content: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(heading) = trimmed.strip_prefix('#') {
            let heading = heading.trim_start_matches('#').trim();
            if !heading.is_empty() {
                return Some(heading.to_string());
            }
        }
    }
    None
}
