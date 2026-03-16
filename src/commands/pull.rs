use anyhow::{Context, Result};
use std::io::Read;

use crate::storage::{db, paths};

const CDN_BASE: &str = "https://cdn.mandex.dev/v1";
const API_BASE: &str = "https://api.mandex.dev";

pub fn run(package: &str) -> Result<()> {
    let (name, version) = parse_package_spec(package);

    // Resolve version if not specified
    let version = match version {
        Some(v) => v.to_string(),
        None => resolve_latest(name)?,
    };

    // Check if already installed
    let db_path = paths::package_db_path(name, &version)?;
    if db_path.exists() {
        println!("{name}@{version} is already installed");
        return Ok(());
    }

    // Download
    let url = format!("{CDN_BASE}/{name}/{version}.mandex");
    println!("Downloading {name}@{version}...");

    let response = reqwest::blocking::get(&url)
        .with_context(|| format!("Failed to download {url}"))?;

    if !response.status().is_success() {
        anyhow::bail!(
            "Package {name}@{version} not found (HTTP {})",
            response.status()
        );
    }

    let compressed = response.bytes()?;
    println!(
        "  ↓ {name}@{version}  {:.1} MB",
        compressed.len() as f64 / 1_048_576.0
    );

    // Decompress
    let mut decoder = zstd::Decoder::new(compressed.as_ref())?;
    let mut db_bytes = Vec::new();
    decoder.read_to_end(&mut db_bytes)?;

    // Write the database file
    std::fs::write(&db_path, &db_bytes)?;

    // Verify it's a valid mandex db
    let conn = db::open_db(&db_path)?;
    let count = db::entry_count(&conn)?;

    println!("  Installed to {}", db_path.display());
    println!("  {count} entries indexed");

    Ok(())
}

fn parse_package_spec(spec: &str) -> (&str, Option<&str>) {
    match spec.split_once('@') {
        Some((name, version)) => (name, Some(version)),
        None => (spec, None),
    }
}

fn resolve_latest(name: &str) -> Result<String> {
    let url = format!("{API_BASE}/packages/{name}/latest");
    let response = reqwest::blocking::get(&url)
        .with_context(|| format!("Failed to resolve latest version for {name}"))?;

    if !response.status().is_success() {
        anyhow::bail!("Package '{name}' not found in registry");
    }

    let data: serde_json::Value = response.json()?;
    let version = data["version"]
        .as_str()
        .context("Invalid response from registry")?;

    Ok(version.to_string())
}
