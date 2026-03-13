use anyhow::Result;

use crate::storage::{db, paths};

pub fn run() -> Result<()> {
    let packages = paths::installed_packages()?;

    if packages.is_empty() {
        println!("No packages installed. Run: mx pull <package>");
        return Ok(());
    }

    for (name, versions) in &packages {
        for version in versions {
            let db_path = paths::package_db_path(name, version)?;
            let conn = db::open_db(&db_path)?;
            let count = db::entry_count(&conn).unwrap_or(0);
            let size = std::fs::metadata(&db_path)
                .map(|m| m.len())
                .unwrap_or(0);

            println!(
                "  {name}@{version}  ({count} entries, {:.1} MB)",
                size as f64 / 1_048_576.0,
            );
        }
    }

    Ok(())
}
