use anyhow::{Context, Result};
use chunk::chunk;
use std::path::Path;
use walkdir::WalkDir;

use crate::storage::db;

/// Max chunk size in bytes (~16KB keeps most sections intact)
const MAX_CHUNK_SIZE: usize = 16384;

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
        let content = std::fs::read_to_string(entry.path())
            .with_context(|| format!("Failed to read {}", entry.path().display()))?;

        if content.trim().is_empty() {
            continue;
        }

        // Get the page-level title
        let page_title = extract_heading(&content).unwrap_or_else(|| {
            entry
                .path()
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "untitled".to_string())
        });

        // Split by markdown headings into sections
        let sections = split_by_headings(&content);

        if sections.len() <= 1 {
            // Small file or no subheadings — chunk by size using the chunk crate
            let chunks: Vec<&[u8]> = chunk(content.as_bytes())
                .size(MAX_CHUNK_SIZE)
                .delimiters(b"\n")
                .collect();

            if chunks.len() <= 1 {
                // Single chunk — store as-is
                db::insert_entry(&conn, &page_title, content.trim())?;
                count += 1;
            } else {
                for (i, c) in chunks.iter().enumerate() {
                    let text = String::from_utf8_lossy(c);
                    let trimmed = text.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    let chunk_name = if chunks.len() > 1 {
                        format!("{page_title} (part {})", i + 1)
                    } else {
                        page_title.clone()
                    };
                    db::insert_entry(&conn, &chunk_name, trimmed)?;
                    count += 1;
                }
            }
        } else {
            // Multiple sections — each heading section is an entry
            for (section_title, section_content) in &sections {
                let trimmed = section_content.trim();
                if trimmed.is_empty() {
                    continue;
                }

                let entry_name = if section_title == &page_title {
                    page_title.clone()
                } else {
                    format!("{page_title} > {section_title}")
                };

                // If a single section is still too large, chunk it further
                if trimmed.len() > MAX_CHUNK_SIZE {
                    let chunks: Vec<&[u8]> = chunk(trimmed.as_bytes())
                        .size(MAX_CHUNK_SIZE)
                        .delimiters(b"\n")
                        .collect();

                    for (i, c) in chunks.iter().enumerate() {
                        let text = String::from_utf8_lossy(c);
                        let t = text.trim();
                        if t.is_empty() {
                            continue;
                        }
                        let name = if chunks.len() > 1 {
                            format!("{entry_name} (part {})", i + 1)
                        } else {
                            entry_name.clone()
                        };
                        db::insert_entry(&conn, &name, t)?;
                        count += 1;
                    }
                } else {
                    db::insert_entry(&conn, &entry_name, trimmed)?;
                    count += 1;
                }
            }
        }
    }

    if count == 0 {
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

/// Splits markdown content by ## headings into (title, content) sections.
/// The content before the first ## is grouped under the page's # title.
fn split_by_headings(content: &str) -> Vec<(String, String)> {
    let mut sections: Vec<(String, String)> = Vec::new();
    let mut current_title = String::new();
    let mut current_content = String::new();

    for line in content.lines() {
        let trimmed = line.trim();

        // Detect heading level
        if trimmed.starts_with("## ") || trimmed.starts_with("### ") {
            // Save previous section
            if !current_title.is_empty() || !current_content.trim().is_empty() {
                if current_title.is_empty() {
                    current_title = "Introduction".to_string();
                }
                sections.push((current_title.clone(), current_content.clone()));
            }

            // Start new section — include the heading line in the content
            current_title = trimmed
                .trim_start_matches('#')
                .trim()
                .to_string();
            current_content = format!("{line}\n");
        } else if trimmed.starts_with("# ") && sections.is_empty() && current_content.trim().is_empty() {
            // Top-level heading at the start — use as page title but don't start a section
            current_title = trimmed.trim_start_matches('#').trim().to_string();
            current_content = format!("{line}\n");
        } else {
            current_content.push_str(line);
            current_content.push('\n');
        }
    }

    // Don't forget the last section
    if !current_title.is_empty() || !current_content.trim().is_empty() {
        if current_title.is_empty() {
            current_title = "Introduction".to_string();
        }
        sections.push((current_title, current_content));
    }

    sections
}
