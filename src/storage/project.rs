use anyhow::Result;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use super::{db, paths};

/// Project manifest tracking which packages belong to this project.
#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct Manifest {
    pub packages: BTreeMap<String, String>,
}

/// Walk CWD upwards looking for a project root.
/// A project root is a directory containing package.json, Cargo.toml,
/// requirements.txt, pyproject.toml, or an existing .mandex/ directory.
pub fn find_project_dir() -> Option<PathBuf> {
    let cwd = std::env::current_dir().ok()?;
    let markers = [
        ".mandex",
        "package.json",
        "Cargo.toml",
        "requirements.txt",
        "pyproject.toml",
    ];

    let mut dir = cwd.as_path();
    loop {
        for marker in &markers {
            if dir.join(marker).exists() {
                return Some(dir.to_path_buf());
            }
        }
        dir = dir.parent()?;
    }
}

/// Returns the .mandex/ directory for a project root.
pub fn mandex_dir(project_root: &Path) -> PathBuf {
    project_root.join(".mandex")
}

/// Returns the path to manifest.json.
fn manifest_path(project_root: &Path) -> PathBuf {
    mandex_dir(project_root).join("manifest.json")
}

/// Returns the path to index.db.
pub fn index_path(project_root: &Path) -> PathBuf {
    mandex_dir(project_root).join("index.db")
}

/// Load the project manifest. Returns default if it doesn't exist.
pub fn load_manifest(project_root: &Path) -> Result<Manifest> {
    let path = manifest_path(project_root);
    if !path.exists() {
        return Ok(Manifest::default());
    }
    let content = fs::read_to_string(&path)?;
    Ok(serde_json::from_str(&content)?)
}

/// Save the project manifest.
pub fn save_manifest(project_root: &Path, manifest: &Manifest) -> Result<()> {
    let dir = mandex_dir(project_root);
    fs::create_dir_all(&dir)?;
    let content = serde_json::to_string_pretty(manifest)?;
    fs::write(manifest_path(project_root), content)?;
    Ok(())
}

/// Rebuild the merged FTS5 index from all packages in the manifest.
/// Returns the total number of entries indexed.
pub fn rebuild_index(project_root: &Path, manifest: &Manifest) -> Result<usize> {
    let dir = mandex_dir(project_root);
    fs::create_dir_all(&dir)?;

    let idx_path = index_path(project_root);

    // Remove old index if it exists
    if idx_path.exists() {
        fs::remove_file(&idx_path)?;
    }

    let conn = rusqlite::Connection::open(&idx_path)?;
    conn.execute_batch(
        "
        CREATE TABLE entries (
            id      INTEGER PRIMARY KEY AUTOINCREMENT,
            package TEXT NOT NULL,
            version TEXT NOT NULL,
            name    TEXT NOT NULL,
            content TEXT NOT NULL
        );

        CREATE VIRTUAL TABLE entries_fts USING fts5(
            name,
            content,
            content=entries,
            content_rowid=id,
            tokenize='porter unicode61'
        );

        CREATE TRIGGER entries_ai AFTER INSERT ON entries BEGIN
            INSERT INTO entries_fts(rowid, name, content) VALUES (new.id, new.name, new.content);
        END;
        ",
    )?;

    let mut total = 0usize;

    for (package, version) in &manifest.packages {
        let db_path = paths::package_db_path(package, version)?;
        if !db_path.exists() {
            continue;
        }

        let pkg_conn = db::open_db(&db_path)?;
        let mut stmt = pkg_conn.prepare("SELECT name, content FROM entries")?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;

        for row in rows {
            let (name, content) = row?;
            conn.execute(
                "INSERT INTO entries (package, version, name, content) VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![package, version, name, content],
            )?;
            total += 1;
        }
    }

    Ok(total)
}
