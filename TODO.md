# TODO

## Search Quality Improvements

Ranked by impact vs effort. Context7 comparison showed three failure modes:
wrong granularity, vocabulary mismatch, and no title-match signal.

### 1. Snippet-level chunking at build time
Split on `##` headings during `mx build` instead of one entry per file.
Each entry should cover one concept. This is the root cause of most bad
results — BM25 works well when the indexed unit is focused. Zero runtime cost.

### 2. Strip boilerplate at build time
Remove Apache license headers, MDX directives (`[[open-in-colab]]`,
`⚠️ Note that this file...`), and HTML comments before indexing. Reduces
noise tokens that dilute BM25 scores.

### 3. Re-rank on title match + code presence
After FTS5 returns top-N, apply a lightweight re-scorer:
- boost if query terms appear in the entry name/title
- boost if entry contains a code block
- boost for shorter/more focused entries

Pure arithmetic on already-fetched rows. Sub-millisecond.

### 4. Query expansion with a static synonym map
Fix zero-result cases caused by vocabulary mismatch (e.g. "fine-tune bert"
returns nothing). Rewrite queries before hitting FTS5:

```
fine-tune  → finetune, finetuning, training
load       → from_pretrained, import
pipeline   → pipe, inference
```

No network, no model, microseconds.

### 5. Local embedding model (offline)
Quantized `all-MiniLM-L6` (~25 MB ONNX) bundled in the `mx` binary.
Pre-compute entry embeddings at `mx build` time, store in the `.mandex` file.
At search time, encode the query and do cosine similarity re-ranking.
Adds ~10–30 ms per query, stays fully offline. Right ceiling before needing
an API.

### 6. API-based reranking (last resort)
Send top-20 FTS5 results to a reranker API (Cohere, Jina, etc.).
Adds ~200–400 ms and an external dependency. Expose as an opt-in config flag.

---

## Website

- Add total entry count to the package detail page header card
- Consider adding a "copy MCP config" button (for agent integration)
