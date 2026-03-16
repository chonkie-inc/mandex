# Searching documentation

## Search across all packages

```bash
mx search "attention mechanism"
```

This searches the FTS5 index across all installed documentation packages and returns ranked results.

## Search within a specific package

```bash
mx search pytorch "linear layer"
```

The first argument is the package name, the second is the query. This only searches within the specified package.

## How search works

Mandex uses SQLite FTS5 with BM25 relevance ranking:

- **Porter stemming** — "tokenizer" matches "tokenize", "routing" matches "route"
- **Prefix queries** — partial word matching
- **Phrase matching** — multi-word queries are matched as phrases
- **Ranked results** — most relevant entries appear first

## Search output

Results show the package name, version, entry name, and a preview of the content:

```
  pytorch@2.3.0 — MultiheadAttention
  Allows the model to jointly attend to information
  from different representation subspaces...

  pytorch@2.3.0 — ScaledDotProductAttention
  Computes scaled dot-product attention on query,
  key, and value tensors...
```

## Show a specific entry

To see the full content of a specific documentation entry:

```bash
mx show pytorch MultiheadAttention
```

This first tries an exact name match. If no exact match is found, it falls back to an FTS5 search and returns the top result.
