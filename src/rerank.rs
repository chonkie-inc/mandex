use anyhow::{Context, Result};
use memmap2::Mmap;
use ort::session::Session;
use ort::value::Tensor;
use std::path::Path;
use tokie::Tokenizer;

use crate::storage::db::SearchResult;

/// Download a file from CDN if it doesn't exist locally.
fn ensure_cdn_file(local_path: &Path, cdn_url: &str, filename: &str) -> Result<()> {
    if local_path.exists() {
        return Ok(());
    }

    if let Some(parent) = local_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let cdn_root = cdn_url.trim_end_matches("/v1");
    let url = format!("{cdn_root}/models/{filename}");

    let response = reqwest::blocking::get(&url)
        .with_context(|| format!("Failed to download {filename} from {url}"))?;

    if !response.status().is_success() {
        anyhow::bail!("Failed to download {filename} (HTTP {})", response.status());
    }

    let bytes = response.bytes()?;
    std::fs::write(local_path, &bytes)?;

    Ok(())
}

/// Download the reranker model if it doesn't exist locally.
/// Prefers the pre-optimized .ort format; falls back to .onnx.
pub fn ensure_model(model_path: &Path, cdn_url: &str) -> Result<()> {
    // Try .ort first (pre-optimized, flatbuffers, faster to load)
    let ort_path = model_path.with_extension("ort");
    if ort_path.exists() {
        return Ok(());
    }
    // Try downloading .ort from CDN
    if ensure_cdn_file(&ort_path, cdn_url, "reranker.ort").is_ok() {
        return Ok(());
    }
    // Fall back to .onnx
    ensure_cdn_file(model_path, cdn_url, "reranker.onnx")
}

/// Resolve the actual model file path (.ort preferred over .onnx).
pub fn resolve_model_file(model_path: &Path) -> std::path::PathBuf {
    let ort_path = model_path.with_extension("ort");
    if ort_path.exists() {
        ort_path
    } else {
        model_path.to_path_buf()
    }
}

/// Download the tokenizer if it doesn't exist locally.
pub fn ensure_tokenizer(tokenizer_path: &Path, cdn_url: &str) -> Result<()> {
    ensure_cdn_file(tokenizer_path, cdn_url, "tokenizer.tkz")
}

/// Choose intra-op thread count: use half the available cores, minimum 1, maximum 4.
fn intra_threads() -> usize {
    let cpus = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);
    (cpus / 2).clamp(1, 4)
}

/// Rerank search results using the ONNX cross-encoder model.
/// Returns results sorted by relevance score, truncated to `limit`.
///
/// Each candidate is a tagged tuple `(tag, SearchResult)` where the tag is
/// preserved through reranking (e.g. package name + version).
pub fn rerank_tagged<T>(
    model_path: &Path,
    tokenizer_path: &Path,
    query: &str,
    candidates: Vec<(T, SearchResult)>,
    limit: usize,
) -> Result<Vec<(T, SearchResult)>> {
    if candidates.is_empty() {
        return Ok(candidates);
    }

    let tokenizer = Tokenizer::from_file(tokenizer_path)
        .map_err(|e| anyhow::anyhow!("Failed to load tokenizer: {e}"))?;

    let threads = intra_threads();
    let actual_model_path = resolve_model_file(model_path);
    let is_ort_format = actual_model_path.extension().is_some_and(|e| e == "ort");

    // mmap the model file for zero-copy loading
    let file = std::fs::File::open(&actual_model_path)
        .map_err(|e| anyhow::anyhow!("Failed to open model: {e}"))?;
    let mmap = unsafe { Mmap::map(&file) }
        .map_err(|e| anyhow::anyhow!("Failed to mmap model: {e}"))?;

    let mut builder = Session::builder()
        .map_err(|e| anyhow::anyhow!("ort session builder: {e}"))?;
    builder = builder.with_intra_threads(threads)
        .map_err(|e| anyhow::anyhow!("ort intra_threads: {e}"))?;

    // Skip graph optimizations for pre-optimized .ort models
    if is_ort_format {
        builder = builder.with_optimization_level(ort::session::builder::GraphOptimizationLevel::Disable)
            .map_err(|e| anyhow::anyhow!("ort optimization level: {e}"))?;
    }

    // Use zero-copy mmap loading (.ort gets direct byte access, .onnx gets regular memory load)
    let mut session = if is_ort_format {
        builder.commit_from_memory_directly(&mmap)
            .map_err(|e| anyhow::anyhow!("ort load model: {e}"))?
    } else {
        builder.commit_from_memory_directly(&mmap)
            .map_err(|e| anyhow::anyhow!("ort load model: {e}"))?
    };

    // Prepare document pairs: (name + first 300 chars of content)
    let docs: Vec<(String, String)> = candidates
        .iter()
        .map(|(_, r)| {
            let snippet: String = r.content.chars().take(300).collect();
            (r.name.clone(), snippet)
        })
        .collect();

    let (ids, mask, types, batch, seq_len) = tokenize_pairs(&tokenizer, query, &docs);

    let shape = vec![batch as i64, seq_len as i64];
    let ids_tensor = Tensor::from_array((shape.clone(), ids.into_boxed_slice()))
        .map_err(|e| anyhow::anyhow!("ort tensor: {e}"))?;
    let mask_tensor = Tensor::from_array((shape.clone(), mask.into_boxed_slice()))
        .map_err(|e| anyhow::anyhow!("ort tensor: {e}"))?;
    let types_tensor = Tensor::from_array((shape, types.into_boxed_slice()))
        .map_err(|e| anyhow::anyhow!("ort tensor: {e}"))?;

    let outputs = session
        .run(ort::inputs![ids_tensor, mask_tensor, types_tensor])
        .map_err(|e| anyhow::anyhow!("ort inference: {e}"))?;
    let (_logits_shape, logits_data) = outputs[0]
        .try_extract_tensor::<f32>()
        .map_err(|e| anyhow::anyhow!("ort extract: {e}"))?;

    // Pair scores with candidate indices and sort by score descending
    let mut scored: Vec<(f32, usize)> = logits_data
        .iter()
        .enumerate()
        .map(|(i, &score)| (score, i))
        .collect();
    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(limit);

    // Rebuild results in reranked order — wrap in Option so we can take ownership
    let mut candidates: Vec<Option<(T, SearchResult)>> = candidates.into_iter().map(Some).collect();
    let reranked: Vec<(T, SearchResult)> = scored
        .into_iter()
        .filter_map(|(_, idx)| candidates[idx].take())
        .collect();

    Ok(reranked)
}

/// Convenience wrapper for untagged results (single-package use).
pub fn rerank(
    model_path: &Path,
    tokenizer_path: &Path,
    query: &str,
    candidates: Vec<SearchResult>,
    limit: usize,
) -> Result<Vec<SearchResult>> {
    let tagged: Vec<((), SearchResult)> = candidates.into_iter().map(|r| ((), r)).collect();
    let reranked = rerank_tagged(model_path, tokenizer_path, query, tagged, limit)?;
    Ok(reranked.into_iter().map(|(_, r)| r).collect())
}

fn tokenize_pairs(
    tokenizer: &Tokenizer,
    query: &str,
    docs: &[(String, String)],
) -> (Vec<i64>, Vec<i64>, Vec<i64>, usize, usize) {
    let mut all_ids: Vec<Vec<i64>> = Vec::new();
    let mut all_mask: Vec<Vec<i64>> = Vec::new();
    let mut all_types: Vec<Vec<i64>> = Vec::new();

    for (name, content) in docs {
        let doc_text = format!("{} {}", name, content);
        let pair = tokenizer.encode_pair(query, &doc_text, true);

        all_ids.push(pair.ids.iter().map(|&x| x as i64).collect());
        all_mask.push(pair.attention_mask.iter().map(|&x| x as i64).collect());
        all_types.push(pair.type_ids.iter().map(|&x| x as i64).collect());
    }

    let batch = all_ids.len();
    let max_len = all_ids.iter().map(|s| s.len()).max().unwrap_or(0);

    let mut ids_flat = vec![0i64; batch * max_len];
    let mut mask_flat = vec![0i64; batch * max_len];
    let mut types_flat = vec![0i64; batch * max_len];

    for i in 0..batch {
        for j in 0..all_ids[i].len() {
            ids_flat[i * max_len + j] = all_ids[i][j];
            mask_flat[i * max_len + j] = all_mask[i][j];
            types_flat[i * max_len + j] = all_types[i][j];
        }
    }

    (ids_flat, mask_flat, types_flat, batch, max_len)
}
