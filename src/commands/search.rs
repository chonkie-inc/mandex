use anyhow::Result;

use crate::storage::{db, paths};

const PREVIEW_CHARS: usize = 200;

pub fn run(package: Option<&str>, query: &str) -> Result<()> {
    let packages = match package {
        Some(name) => {
            // Search within a specific package — find installed versions
            let pkg_dir = paths::package_dir(name)?;
            if !pkg_dir.exists() {
                anyhow::bail!("Package '{name}' is not installed. Run: mx pull {name}");
            }
            let installed = paths::installed_packages()?;
            let versions: Vec<_> = installed
                .into_iter()
                .filter(|(n, _)| n == name)
                .collect();
            if versions.is_empty() {
                anyhow::bail!("Package '{name}' is not installed. Run: mx pull {name}");
            }
            versions
        }
        None => paths::installed_packages()?,
    };

    if packages.is_empty() {
        println!("No packages installed. Run: mx pull <package>");
        return Ok(());
    }

    let mut total_results = 0;

    for (name, versions) in &packages {
        // Search the latest installed version
        let version = versions.last().unwrap();
        let db_path = paths::package_db_path(name, version)?;
        let conn = db::open_db(&db_path)?;

        let results = db::search(&conn, query)?;
        for result in &results {
            let preview = truncate_preview(&result.content, PREVIEW_CHARS);
            println!("  \x1b[33m{name}@{version}\x1b[0m — {}", result.name);
            println!("  {preview}");
            println!();
            total_results += 1;
        }
    }

    if total_results == 0 {
        println!("No results for '{query}'");
    }

    Ok(())
}

fn truncate_preview(content: &str, max_chars: usize) -> String {
    // Skip the first heading line if present, show the next meaningful content
    let meaningful: String = content
        .lines()
        .skip_while(|l| l.trim().is_empty())
        .skip_while(|l| l.trim().starts_with('#'))
        .skip_while(|l| l.trim().is_empty())
        .take(3)
        .collect::<Vec<_>>()
        .join(" ");

    let trimmed = meaningful.trim();
    if trimmed.len() > max_chars {
        format!("{}...", &trimmed[..max_chars])
    } else {
        trimmed.to_string()
    }
}
