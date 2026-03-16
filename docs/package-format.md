# Package format

## Overview

A mandex package (`.mandex` file) is a zstd-compressed SQLite database with an FTS5 full-text search index.

```
pytorch@2.3.0.mandex    ← compressed file (distributed via CDN)
  └── pytorch@2.3.0.db  ← SQLite database (stored locally after pull)
```

## Why SQLite?

The package format needs to be a storage container, a search index, and a query engine — all in one file, with no server process.

SQLite does all three. FTS5 provides BM25 relevance ranking, porter stemming, prefix queries, phrase matching, and boolean operators. A mandex `.db` file is queryable with any SQLite client in any programming language.

If mandex ceased to exist, every published package would remain fully functional with any standard SQLite client. SQLite has backwards compatibility commitments through 2050.

## Why zstd?

Documentation is highly compressible text. Zstd achieves 5-10x compression ratios on structured markdown. A 20MB database ships as 2-4MB over the wire.

Zstd decompresses at 1-2 GB/s on modern hardware. The CLI decompresses once on download and stores the uncompressed `.db` locally. No per-query decompression cost.

## Schema

```sql
CREATE TABLE metadata (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE TABLE entries (
    id      INTEGER PRIMARY KEY AUTOINCREMENT,
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
```

### metadata table

Stores package-level information:

| Key | Example value |
|-----|--------------|
| `name` | `pytorch` |
| `version` | `2.3.0` |
| `schema_version` | `1` |
| `entry_count` | `2847` |

### entries table

Each row is one documentation entry, typically one markdown file from the source docs:

- **name** — derived from the first `#` heading in the file, or the filename
- **content** — the full markdown text of the file

### entries_fts

FTS5 virtual table that indexes `name` and `content` for full-text search with BM25 ranking. Uses porter stemming and unicode61 tokenization.

## Querying a package directly

Since mandex packages are standard SQLite databases, you can query them directly:

```bash
sqlite3 ~/.mandex/cache/pytorch/2.3.0.db \
  "SELECT name FROM entries_fts WHERE entries_fts MATCH 'attention' ORDER BY rank LIMIT 5;"
```

Or from Python:

```python
import sqlite3

conn = sqlite3.connect("~/.mandex/cache/pytorch/2.3.0.db")
results = conn.execute(
    "SELECT name, content FROM entries_fts WHERE entries_fts MATCH ? ORDER BY rank LIMIT 5",
    ("attention",)
).fetchall()
```
