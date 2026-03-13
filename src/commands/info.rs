use anyhow::Result;

use crate::storage::{db, paths};

pub fn run(package: &str) -> Result<()> {
    let installed = paths::installed_packages()?;
    let versions = installed
        .iter()
        .find(|(n, _)| n == package)
        .map(|(_, v)| v);

    let versions = match versions {
        Some(v) => v,
        None => anyhow::bail!("Package '{package}' is not installed. Run: mx pull {package}"),
    };

    for version in versions {
        let db_path = paths::package_db_path(package, version)?;
        let conn = db::open_db(&db_path)?;
        let count = db::entry_count(&conn)?;
        let size = std::fs::metadata(&db_path)
            .map(|m| m.len())
            .unwrap_or(0);

        let name = db::get_metadata(&conn, "name")?
            .unwrap_or_else(|| package.to_string());
        let ver = db::get_metadata(&conn, "version")?
            .unwrap_or_else(|| version.to_string());

        println!("\x1b[1m{name}\x1b[0m@{ver}");
        println!("  Entries:  {count}");
        println!("  Size:     {:.1} MB", size as f64 / 1_048_576.0);
        println!("  Path:     {}", db_path.display());
    }

    Ok(())
}
