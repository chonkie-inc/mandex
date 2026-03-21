use anyhow::Result;

use crate::storage::{db, paths};

pub fn run(package: &str, entry: &str) -> Result<()> {
    let installed = paths::installed_packages()?;
    let versions = installed
        .iter()
        .find(|(n, _)| n == package)
        .map(|(_, v)| v);

    let versions = match versions {
        Some(v) => v,
        None => anyhow::bail!("Package '{package}' is not installed. Run: mx pull {package}"),
    };

    let version = versions.last().unwrap();
    let db_path = paths::package_db_path(package, version)?;
    let conn = db::open_db(&db_path)?;

    // Try exact match first
    if let Some((name, content)) = db::get_entry(&conn, entry)? {
        println!("\x1b[1m{name}\x1b[0m  \x1b[2m{package}@{version}\x1b[0m\n");
        println!("{content}");
        return Ok(());
    }

    // Fall back to FTS search with the entry name as query
    let results = db::search(&conn, entry, 1)?;
    if let Some(result) = results.first() {
        println!("\x1b[1m{}\x1b[0m  \x1b[2m{package}@{version}\x1b[0m\n", result.name);
        println!("{}", result.content);
    } else {
        println!("No entry '{entry}' found in {package}@{version}");
    }

    Ok(())
}
