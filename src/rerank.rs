use anyhow::{Context, Result};
use ort::session::Session;
use ort::value::Tensor;
use std::path::Path;
use tokenizers::Tokenizer;

use crate::storage::db::SearchResult;

/// Download the reranker model if it doesn't exist locally.
pub fn ensure_model(model_path: &Path, cdn_url: &str) -> Result<()> {
    if model_path.exists() {
        return Ok(());
    }

    if let Some(parent) = model_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let cdn_root = cdn_url.trim_end_matches("/v1");
    let url = format!("{cdn_root}/models/reranker.onnx");
    // Message handled by caller (init.rs)

    let response = reqwest::blocking::get(&url)
        .with_context(|| format!("Failed to download reranker model from {url}"))?;

    if !response.status().is_success() {
        anyhow::bail!("Failed to download reranker model (HTTP {})", response.status());
    }

    let bytes = response.bytes()?;
    std::fs::write(model_path, &bytes)?;

    Ok(())
}

/// Rerank search results using the ONNX cross-encoder model.
/// Returns results sorted by relevance score, truncated to `limit`.
pub fn rerank(
    model_path: &Path,
    query: &str,
    candidates: Vec<SearchResult>,
    limit: usize,
) -> Result<Vec<SearchResult>> {
    if candidates.is_empty() {
        return Ok(candidates);
    }

    let tokenizer = Tokenizer::from_pretrained("cross-encoder/ms-marco-MiniLM-L-4-v2", None)
        .map_err(|e| anyhow::anyhow!("Failed to load tokenizer: {e}"))?;

    let mut session = Session::builder()
        .map_err(|e| anyhow::anyhow!("ort session builder: {e}"))?
        .with_intra_threads(1)
        .map_err(|e| anyhow::anyhow!("ort intra_threads: {e}"))?
        .commit_from_file(model_path)
        .map_err(|e| anyhow::anyhow!("ort load model: {e}"))?;

    // Prepare document pairs: (name + first 300 chars of content)
    let docs: Vec<(String, String)> = candidates
        .iter()
        .map(|r| {
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

    // Rebuild results in reranked order
    let mut candidates = candidates;
    let reranked: Vec<SearchResult> = scored
        .into_iter()
        .map(|(_, idx)| {
            // Take ownership by replacing with a dummy — indices are unique so each is taken once
            std::mem::replace(
                &mut candidates[idx],
                SearchResult {
                    name: String::new(),
                    content: String::new(),
                    rank: 0.0,
                },
            )
        })
        .collect();

    Ok(reranked)
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
        let encoding = tokenizer
            .encode((query, doc_text.as_str()), true)
            .unwrap();

        all_ids.push(encoding.get_ids().iter().map(|&x| x as i64).collect());
        all_mask.push(
            encoding
                .get_attention_mask()
                .iter()
                .map(|&x| x as i64)
                .collect(),
        );
        all_types.push(encoding.get_type_ids().iter().map(|&x| x as i64).collect());
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
