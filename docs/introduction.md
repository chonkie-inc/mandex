# Introduction

Mandex is a package registry for documentation. Library authors build searchable documentation packages from their existing markdown docs. Developers download them once and query them locally — offline, with zero rate limits, in sub-millisecond time.

## How it works

1. **Authors** run `mx build ./docs` against their existing documentation directory. Each markdown file becomes a searchable entry in a SQLite database with FTS5 full-text search.
2. **The registry** hosts compressed packages on a CDN. Downloading a package is a single HTTP GET — no API server, no authentication required.
3. **Developers** run `mx pull <package>` to download documentation packages. All queries after that are local — no network, no rate limits.

## Why Mandex?

AI agents make hundreds of documentation lookups per coding session. The existing approaches all have problems:

- **Web fetching** gets blocked by bot detection and wastes tokens on HTML noise
- **llms.txt** is a full dump with no search — hundreds of thousands of tokens for a single lookup
- **Cloud MCP servers** charge per-query for free open source docs and impose rate limits that break agent workflows

Mandex treats documentation like software packages: versioned, compressed, distributed via CDN, queried locally. The same model that makes npm and cargo work.

## Quick start

```bash
# Install
curl -fsSL https://mandex.dev/install.sh | sh

# Pull documentation for a library
mx pull pytorch@2.3.0

# Search locally
mx search pytorch "attention mechanism"

# Show a specific entry
mx show pytorch MultiheadAttention
```

## Key features

- **Local-first** — all queries run against local SQLite databases, no network needed after download
- **Version-pinned** — `mx pull nextjs@14.0.0` gives you exactly the v14 docs, no version confusion
- **Zero-config for authors** — `mx build ./docs` works on any existing markdown directory
- **CLI-first** — output goes to stdout, works with any agent that can run shell commands
- **Single binary** — Rust CLI with no runtime dependencies, starts in under 5ms
