use anyhow::Result;
use std::collections::{BTreeMap, HashSet};
use std::path::Path;

use crate::commands::pull;
use crate::storage::{paths, project};

struct DetectedDep {
    name: String,
    #[allow(dead_code)]
    source: String,
}

pub fn run() -> Result<()> {
    println!();
    println!("  \x1b[1mScanning project dependencies...\x1b[0m");

    let mut deps: Vec<DetectedDep> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();

    // Detect and parse each dependency file
    if Path::new("package.json").exists() {
        let found = parse_package_json("package.json")?;
        println!(
            "  \x1b[2mFound package.json ({} deps)\x1b[0m",
            found.len()
        );
        for name in found {
            if seen.insert(name.clone()) {
                deps.push(DetectedDep {
                    name,
                    source: "npm".to_string(),
                });
            }
        }
    }

    if Path::new("requirements.txt").exists() {
        let found = parse_requirements_txt("requirements.txt")?;
        println!(
            "  \x1b[2mFound requirements.txt ({} deps)\x1b[0m",
            found.len()
        );
        for name in found {
            if seen.insert(name.clone()) {
                deps.push(DetectedDep {
                    name,
                    source: "pip".to_string(),
                });
            }
        }
    }

    if Path::new("pyproject.toml").exists() {
        let found = parse_pyproject_toml("pyproject.toml")?;
        println!(
            "  \x1b[2mFound pyproject.toml ({} deps)\x1b[0m",
            found.len()
        );
        for name in found {
            if seen.insert(name.clone()) {
                deps.push(DetectedDep {
                    name,
                    source: "pip".to_string(),
                });
            }
        }
    }

    if Path::new("Cargo.toml").exists() {
        let found = parse_cargo_toml("Cargo.toml")?;
        println!(
            "  \x1b[2mFound Cargo.toml ({} deps)\x1b[0m",
            found.len()
        );
        for name in found {
            if seen.insert(name.clone()) {
                deps.push(DetectedDep {
                    name,
                    source: "cargo".to_string(),
                });
            }
        }
    }

    if deps.is_empty() {
        println!();
        println!("  No dependency files found in current directory.");
        println!("  \x1b[2mLooking for: package.json, requirements.txt, pyproject.toml, Cargo.toml\x1b[0m");
        println!();
        return Ok(());
    }

    println!();

    // Get installed packages
    let installed = paths::installed_packages()?;
    let installed_names: HashSet<String> = installed.iter().map(|(n, _)| n.clone()).collect();

    let mut new_count = 0;
    let mut existing_count = 0;
    let mut not_found_count = 0;

    // Track resolved packages for manifest
    let mut resolved: BTreeMap<String, String> = BTreeMap::new();

    for dep in &deps {
        // Already installed?
        if installed_names.contains(&dep.name) {
            let version = installed
                .iter()
                .find(|(n, _)| n == &dep.name)
                .map(|(_, v)| v.last().unwrap().clone())
                .unwrap_or_else(|| "?".to_string());
            println!(
                "  \x1b[32m✓\x1b[0m {} \x1b[2m{} (already installed)\x1b[0m",
                dep.name, version
            );
            resolved.insert(dep.name.clone(), version);
            existing_count += 1;
            continue;
        }

        // Try to resolve from registry
        match pull::resolve_latest(&dep.name) {
            Ok(version) => {
                // Pull it
                match pull::download_package(&dep.name, &version) {
                    Ok(_) => {
                        println!(
                            "  \x1b[32m↓\x1b[0m {}@{} \x1b[2m(new)\x1b[0m",
                            dep.name, version
                        );
                        resolved.insert(dep.name.clone(), version);
                        new_count += 1;
                    }
                    Err(e) => {
                        println!(
                            "  \x1b[31m✗\x1b[0m {} \x1b[2m— download failed: {}\x1b[0m",
                            dep.name, e
                        );
                        not_found_count += 1;
                    }
                }
            }
            Err(_) => {
                println!(
                    "  \x1b[2m· {} — not in registry\x1b[0m",
                    dep.name
                );
                not_found_count += 1;
            }
        }
    }

    println!();
    println!(
        "  \x1b[1mSynced {} package{}\x1b[0m \x1b[2m({} new, {} up to date{})\x1b[0m",
        new_count + existing_count,
        if new_count + existing_count == 1 { "" } else { "s" },
        new_count,
        existing_count,
        if not_found_count > 0 {
            format!(", {} not in registry", not_found_count)
        } else {
            String::new()
        }
    );

    // Build project manifest + merged index
    if !resolved.is_empty() {
        let project_root = std::env::current_dir()?;
        let manifest = project::Manifest { packages: resolved };
        project::save_manifest(&project_root, &manifest)?;

        let total_entries = project::rebuild_index(&project_root, &manifest)?;
        println!(
            "  \x1b[2mBuilt search index ({} entries from {} packages)\x1b[0m",
            total_entries,
            manifest.packages.len()
        );
    }

    println!();

    Ok(())
}

fn parse_package_json(path: &str) -> Result<Vec<String>> {
    let content = std::fs::read_to_string(path)?;
    let json: serde_json::Value = serde_json::from_str(&content)?;

    let mut names = Vec::new();

    for key in &["dependencies", "devDependencies"] {
        if let Some(deps) = json.get(key).and_then(|v| v.as_object()) {
            for name in deps.keys() {
                names.push(name.clone());
            }
        }
    }

    Ok(names)
}

fn parse_requirements_txt(path: &str) -> Result<Vec<String>> {
    let content = std::fs::read_to_string(path)?;
    let mut names = Vec::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with('-') {
            continue;
        }
        // Strip version specifiers: ==, >=, <=, ~=, !=, <, >
        let name = line
            .split(&['=', '>', '<', '~', '!', '[', ';'][..])
            .next()
            .unwrap_or("")
            .trim();
        if !name.is_empty() {
            // Normalize: PEP 503 says underscores and hyphens are equivalent
            names.push(name.to_lowercase().replace('_', "-"));
        }
    }

    Ok(names)
}

fn parse_pyproject_toml(path: &str) -> Result<Vec<String>> {
    let content = std::fs::read_to_string(path)?;
    let toml: toml::Value = toml::from_str(&content)?;

    let mut names = Vec::new();

    // PEP 621: project.dependencies = ["fastapi>=0.115", "pydantic"]
    if let Some(deps) = toml
        .get("project")
        .and_then(|p| p.get("dependencies"))
        .and_then(|d| d.as_array())
    {
        for dep in deps {
            if let Some(s) = dep.as_str() {
                let name = s
                    .split(&['=', '>', '<', '~', '!', '[', ';', ' '][..])
                    .next()
                    .unwrap_or("")
                    .trim();
                if !name.is_empty() {
                    names.push(name.to_lowercase().replace('_', "-"));
                }
            }
        }
    }

    // Poetry: tool.poetry.dependencies
    if let Some(deps) = toml
        .get("tool")
        .and_then(|t| t.get("poetry"))
        .and_then(|p| p.get("dependencies"))
        .and_then(|d| d.as_table())
    {
        for name in deps.keys() {
            if name != "python" {
                names.push(name.to_lowercase().replace('_', "-"));
            }
        }
    }

    Ok(names)
}

fn parse_cargo_toml(path: &str) -> Result<Vec<String>> {
    let content = std::fs::read_to_string(path)?;
    let toml: toml::Value = toml::from_str(&content)?;

    let mut names = Vec::new();

    if let Some(deps) = toml.get("dependencies").and_then(|d| d.as_table()) {
        for name in deps.keys() {
            names.push(name.clone());
        }
    }

    Ok(names)
}
