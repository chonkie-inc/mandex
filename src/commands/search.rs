use anyhow::Result;

#[cfg(feature = "reranker")]
use crate::config::{self, ConfigFile};
#[cfg(not(feature = "reranker"))]
use crate::config::ConfigFile;
#[cfg(feature = "reranker")]
use crate::rerank;
use crate::storage::{db, paths, project};

pub fn run(
    package: Option<&str>,
    query: &str,
    results_limit: usize,
    use_rerank: bool,
    rerank_candidates: usize,
    config: &ConfigFile,
) -> Result<()> {
    // If reranking, fetch more candidates from FTS5; otherwise just fetch the limit
    let fetch_limit = if use_rerank {
        rerank_candidates.max(results_limit)
    } else {
        results_limit
    };

    // Try project index first, fall back to global cache
    let final_results = if let Some(results) = try_project_search(package, query, fetch_limit)? {
        results
    } else {
        global_search(package, query, fetch_limit)?
    };

    // Convert to tagged format for reranking
    let mut all_results: Vec<((String, String), db::SearchResult)> = final_results
        .into_iter()
        .map(|r| {
            (
                (r.package, r.version),
                db::SearchResult {
                    name: r.name,
                    content: r.content,
                    rank: r.rank,
                },
            )
        })
        .collect();

    // Rerank or truncate
    #[cfg(feature = "reranker")]
    let final_results = if use_rerank && !all_results.is_empty() {
        let model_path = config::resolve_model_path(&config.search.rerank_model)?;
        rerank::ensure_model(&model_path, &config.network.cdn_url)?;
        let tokenizer_path = model_path.with_file_name("tokenizer.tkz");
        rerank::ensure_tokenizer(&tokenizer_path, &config.network.cdn_url)?;
        rerank::rerank_tagged(&model_path, &tokenizer_path, query, all_results, results_limit)?
    } else {
        all_results.truncate(results_limit);
        all_results
    };

    #[cfg(not(feature = "reranker"))]
    let final_results = {
        let _ = (use_rerank, config);
        all_results.truncate(results_limit);
        all_results
    };

    // Print results
    for ((name, version), result) in &final_results {
        println!(
            "\x1b[33m{name}@{version}\x1b[0m — \x1b[1m{}\x1b[0m",
            result.name
        );
        println!();
        println!("{}", result.content);
        println!("\n{}\n", "─".repeat(60));
    }

    let total_results = final_results.len();
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

/// Search using the project's merged index.db if available.
fn try_project_search(
    package: Option<&str>,
    query: &str,
    limit: usize,
) -> Result<Option<Vec<db::IndexSearchResult>>> {
    let project_root = match project::find_project_dir() {
        Some(root) => root,
        None => return Ok(None),
    };

    let idx_path = project::index_path(&project_root);
    if !idx_path.exists() {
        return Ok(None);
    }

    let conn = db::open_db(&idx_path)?;
    let results = db::search_index(&conn, query, limit, package)?;
    Ok(Some(results))
}

/// Fallback: search across global cache, iterating per-package.
fn global_search(
    package: Option<&str>,
    query: &str,
    limit: usize,
) -> Result<Vec<db::IndexSearchResult>> {
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
        return Ok(vec![]);
    }

    let mut all_results: Vec<db::IndexSearchResult> = Vec::new();

    for (name, versions) in &packages {
        let version = versions.last().unwrap();
        let db_path = paths::package_db_path(name, version)?;
        let conn = db::open_db(&db_path)?;

        let results = db::search(&conn, query, limit)?;
        for r in results {
            all_results.push(db::IndexSearchResult {
                package: name.clone(),
                version: version.clone(),
                name: r.name,
                content: r.content,
                rank: r.rank,
            });
        }
    }

    Ok(all_results)
}
