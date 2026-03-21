use anyhow::Result;

#[cfg(feature = "reranker")]
use crate::config::{self, ConfigFile};
#[cfg(not(feature = "reranker"))]
use crate::config::ConfigFile;
#[cfg(feature = "reranker")]
use crate::rerank;
use crate::storage::{db, paths};

pub fn run(
    package: Option<&str>,
    query: &str,
    results_limit: usize,
    use_rerank: bool,
    rerank_candidates: usize,
    config: &ConfigFile,
) -> Result<()> {
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

    // If reranking, fetch more candidates from FTS5; otherwise just fetch the limit
    let fetch_limit = if use_rerank {
        rerank_candidates.max(results_limit)
    } else {
        results_limit
    };

    let mut total_results = 0;

    for (name, versions) in &packages {
        let version = versions.last().unwrap();
        let db_path = paths::package_db_path(name, version)?;
        let conn = db::open_db(&db_path)?;

        let mut results = db::search(&conn, query, fetch_limit)?;

        #[cfg(feature = "reranker")]
        if use_rerank && !results.is_empty() {
            let model_path = config::resolve_model_path(&config.search.rerank_model)?;
            rerank::ensure_model(&model_path, &config.network.cdn_url)?;
            results = rerank::rerank(&model_path, query, results, results_limit)?;
        } else {
            results.truncate(results_limit);
        }

        #[cfg(not(feature = "reranker"))]
        {
            let _ = (use_rerank, config);
            results.truncate(results_limit);
        }

        for result in &results {
            println!(
                "\x1b[33m{name}@{version}\x1b[0m — \x1b[1m{}\x1b[0m",
                result.name
            );
            println!();
            println!("{}", result.content);
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
