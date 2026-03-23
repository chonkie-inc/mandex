use anyhow::{Context, Result};
use std::io::Read;

use crate::storage::{db, paths, project};

const CDN_BASE: &str = "https://cdn.mandex.dev/v1";
const API_BASE: &str = "https://api.mandex.dev";

pub fn run(package: &str) -> Result<()> {
    let (name, version) = parse_package_spec(package);

    let version = match version {
        Some(v) => v.to_string(),
        None => resolve_latest(name)?,
    };

    let db_path = paths::package_db_path(name, &version)?;
    if db_path.exists() {
        println!("{name}@{version} is already installed");
        return Ok(());
    }

    println!("Downloading {name}@{version}...");
    download_package(name, &version)?;

    // Update project manifest + index if in a project directory
    if let Some(project_root) = project::find_project_dir() {
        let mut manifest = project::load_manifest(&project_root)?;
        manifest.packages.insert(name.to_string(), version.clone());
        project::save_manifest(&project_root, &manifest)?;
        project::rebuild_index(&project_root, &manifest)?;
    }

    Ok(())
}

/// Parse "name@version" or just "name"
pub fn parse_package_spec(spec: &str) -> (&str, Option<&str>) {
    match spec.split_once('@') {
        Some((name, version)) => (name, Some(version)),
        None => (spec, None),
    }
}

/// Resolve latest version from registry. Returns Err if package not in registry.
pub fn resolve_latest(name: &str) -> Result<String> {
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

/// Download and install a package. Returns entry count on success.
pub fn download_package(name: &str, version: &str) -> Result<usize> {
    let db_path = paths::package_db_path(name, version)?;

    let url = format!("{CDN_BASE}/{name}/{version}.mandex");
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

    let mut decoder = zstd::Decoder::new(compressed.as_ref())?;
    let mut db_bytes = Vec::new();
    decoder.read_to_end(&mut db_bytes)?;

    std::fs::write(&db_path, &db_bytes)?;

    let conn = db::open_db(&db_path)?;
    let count = db::entry_count(&conn)?;

    println!("  Installed to {}", db_path.display());
    println!("  {count} entries indexed");

    Ok(count)
}
