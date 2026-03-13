use anyhow::Result;

use crate::storage::paths;

pub fn run(package: &str, version: Option<&str>) -> Result<()> {
    match version {
        Some(ver) => {
            let db_path = paths::package_db_path(package, ver)?;
            if !db_path.exists() {
                anyhow::bail!("{package}@{ver} is not installed");
            }
            std::fs::remove_file(&db_path)?;
            println!("Removed {package}@{ver}");

            // Clean up empty package directory
            let pkg_dir = paths::package_dir(package)?;
            if pkg_dir.exists() && pkg_dir.read_dir()?.next().is_none() {
                std::fs::remove_dir(&pkg_dir)?;
            }
        }
        None => {
            let pkg_dir = paths::package_dir(package)?;
            if !pkg_dir.exists() {
                anyhow::bail!("Package '{package}' is not installed");
            }
            std::fs::remove_dir_all(&pkg_dir)?;
            println!("Removed all versions of {package}");
        }
    }

    Ok(())
}
