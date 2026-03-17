use anyhow::Result;

use crate::storage::{db, paths};

pub fn run(package: Option<&str>, query: &str) -> Result<()> {
    let packages = match package {
        Some(name) => {
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
        let version = versions.last().unwrap();
        let db_path = paths::package_db_path(name, version)?;
        let conn = db::open_db(&db_path)?;

        let results = db::search(&conn, query)?;
        for result in &results {
            // Header: package@version — entry name
            println!(
                "\x1b[33m{name}@{version}\x1b[0m — \x1b[1m{}\x1b[0m",
                result.name
            );
            println!();
            // Full content of the chunk
            println!("{}", result.content);
            // Separator between results
            println!("\n{}\n", "─".repeat(60));
            total_results += 1;
        }
    }

    if total_results == 0 {
        println!("No results for '{query}'");
    } else {
        println!(
            "\x1b[2m{total_results} result{}\x1b[0m",
            if total_results == 1 { "" } else { "s" }
        );
    }

    Ok(())
}
