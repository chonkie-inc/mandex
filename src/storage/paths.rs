use anyhow::{Context, Result};
use std::path::PathBuf;

/// Returns the root mandex directory (~/.mandex/)
pub fn mandex_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().context("Could not determine home directory")?;
    let dir = home.join(".mandex");
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// Returns the global package cache directory (~/.mandex/cache/)
pub fn cache_dir() -> Result<PathBuf> {
    let dir = mandex_dir()?.join("cache");
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// Returns the path for a specific package version's db file
/// e.g. ~/.mandex/cache/pytorch/2.3.0.db
pub fn package_db_path(name: &str, version: &str) -> Result<PathBuf> {
    let dir = cache_dir()?.join(name);
    std::fs::create_dir_all(&dir)?;
    Ok(dir.join(format!("{version}.db")))
}

/// Returns the package directory in cache (e.g. ~/.mandex/cache/pytorch/)
pub fn package_dir(name: &str) -> Result<PathBuf> {
    let dir = cache_dir()?.join(name);
    Ok(dir)
}

/// Lists all installed packages and their versions
pub fn installed_packages() -> Result<Vec<(String, Vec<String>)>> {
    let cache = cache_dir()?;
    let mut packages = Vec::new();

    if !cache.exists() {
        return Ok(packages);
    }

    let mut entries: Vec<_> = std::fs::read_dir(&cache)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let name = entry.file_name().to_string_lossy().to_string();
        let mut versions: Vec<String> = std::fs::read_dir(entry.path())?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "db"))
            .filter_map(|e| {
                e.path()
                    .file_stem()
                    .map(|s| s.to_string_lossy().to_string())
            })
            .collect();
        versions.sort();
        if !versions.is_empty() {
            packages.push((name, versions));
        }
    }

    Ok(packages)
}
