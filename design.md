# Mandex — Design Document

## What is Mandex?

Mandex (manual + index) is a package registry for documentation. Library authors build searchable documentation packages from their existing markdown docs. The packages are zstd-compressed SQLite databases with FTS5 full-text search, distributed via CDN. Developers download them once and query them locally — offline, with zero rate limits, in sub-millisecond time.

The CLI is called `mx`. It's written in Rust. Single static binary, no runtime dependencies.

## The Problem

AI coding agents need documentation to write correct code. Without it, they fall back to stale training data — deprecated APIs, outdated patterns, code that doesn't compile. The documentation exists and it's freely available. The problem is getting it into the agent's context at the right time, in the right version, efficiently.

### Current approaches and their failures

**Direct web fetching.** Agents search the web, fetch pages. 5-10 HTTP requests per lookup, 95% noise by token count (nav bars, sidebars, cookie banners). Bot detection blocks requests. The aggregate traffic from agents looks like a DDoS. A feedback loop: tighter bot detection → more failures → more retries.

**llms.txt.** Sites serve `/llms.txt` or `/llms-full.txt` with clean LLM-optimized content. Solves the HTML noise problem but: no search index (full dump, 428k tokens for Tailwind), no versioning (`/llms.txt` serves whatever version is deployed), still requires a network request per lookup.

**Cloud MCP servers (Context7, Docfork).** Pre-indexed docs served via MCP. Good experience when it works — structured, relevant results. But: per-query pricing on free open source docs, rate limits (Context7 cut free tier by 92% in Jan 2026), infrastructure cost scales linearly with queries, version routing across N copies of mostly-identical content fails on BM25 disambiguation.

**Local MCP servers (@neuledge/context).** Local SQLite + FTS5 indexed from git repos. Right architecture, but: requires Node.js, MCP-only interface, every user rebuilds the same index, no author control over how docs appear.

### The core insight

Documentation has the same access pattern as software packages: authored once per version, distributed widely, read many times, never modified in place. The distribution model should match. You don't query npm on every `import`. You install packages locally.

## Architecture

### Package format

A mandex package is a zstd-compressed SQLite database with FTS5 full-text search.

```
pytorch@2.3.0.mandex   (compressed, on CDN)
  └── pytorch@2.3.0.db  (SQLite database, local after download)
```

### Schema

The schema is deliberately minimal:

```sql
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

Two columns. `name` is the first `#` heading in the source markdown file (or the filename). `content` is the full markdown text.

No `params`, `signature`, `returns`, `tags`, or `kind` columns. Documentation already contains all of that in markdown that LLMs parse naturally. A structured schema would need to work across PyTorch API references, Next.js conceptual guides, Tailwind utility listings, and Django tutorials. No field set fits all of them. Markdown does.

The CLI controls display — search results show the entry name and first N characters of content. Full content is returned when a specific entry is requested. Truncation is a display concern, not a schema concern.

### Why SQLite?

The package format must serve three roles: storage container, search index, and query engine. SQLite does all three in a single file with no server process.

FTS5 provides BM25 relevance ranking, porter stemming, prefix queries, phrase matching, and boolean operators — built into SQLite. A `.db` file is queryable with any SQLite client in any language. If mandex ceased to exist, every published package would remain fully functional. SQLite has backwards compatibility commitments through 2050.

The alternative (compressed JSON + separate search index like Tantivy) would require two artifacts per package with format coupling, and every client would need a compatible search engine version.

### Compression

Zstd. Documentation is highly compressible — 5-10x ratios on structured markdown. A 20MB package ships as 2-4MB. Zstd decompresses at 1-2 GB/s. The CLI decompresses once on download; no per-query cost.

### CDN distribution

Packages are hosted on Cloudflare R2 (S3-compatible, zero egress fees).

```
cdn.mandex.dev/v1/registry.json                    # global package listing
cdn.mandex.dev/v1/{package}/meta.json               # package metadata + version list
cdn.mandex.dev/v1/{package}/{version}.mandex         # compressed SQLite db
cdn.mandex.dev/v1/{package}/{version}.sha256         # checksum
```

No application server, no per-request computation, no index lookup. Serving a package to one developer costs the same as serving it to a million. This is the architectural decision that avoids rate limits entirely. Mandex packages are immutable artifacts — the distribution model is identical to binary releases behind a CDN.

### Local storage

```
~/.mandex/
├── config.toml                  # user configuration
├── cache/                       # global package cache (shared across projects)
│   ├── pytorch/
│   │   └── 2.3.0.db
│   ├── nextjs/
│   │   └── 14.2.0.db
│   └── ...
└── registry.json                # cached registry index
```

Packages are stored in a global cache, shared across all projects. If two projects use `react@19.1.0`, the package is downloaded once. Same model as pnpm's content-addressable store or cargo's registry cache.

Per-project scoping comes from `mx sync`, which writes a project-local manifest:

```
your-project/
├── .mandex/
│   └── manifest.json            # packages + versions for this project
├── package.json
└── ...
```

When `mx search` runs inside a project, it reads the manifest and queries only the relevant packages. 50 packages in global cache, but a search in your Next.js project only hits the 14 listed in that project's manifest.

### Package name mapping

The `mx sync` command maps ecosystem dependency names to mandex package names:

1. Check registry metadata — each mandex package declares which ecosystem packages it documents (e.g., `torch` in pip → `pytorch` in mandex)
2. Try exact name match — `requests` in pip → `requests` in mandex
3. Fail gracefully — skip packages with no matching docs

## CLI interface

```bash
# Install
curl -fsSL https://mandex.dev/install.sh | sh

# Package management
mx pull pytorch@2.3.0             # download specific version
mx pull pytorch                    # download latest
mx sync                            # read project deps, download matching docs
mx list                            # list installed packages
mx remove pytorch                  # remove a package
mx update                          # update all to latest

# Search & query
mx search "linear layer"           # search across all installed packages
mx search pytorch "nn.Linear"      # search within a specific package
mx show nextjs useRouter           # display a specific entry

# Publishing
mx build ./docs --name pytorch --version 2.3.0   # build from existing docs
mx publish                         # upload to registry

# MCP server mode
mx serve                           # start MCP server for compatible clients

# Info
mx info pytorch                    # show package metadata
```

## Building packages

The build step works on documentation as it already exists. No custom format, no frontmatter, no migration.

```bash
mx build ./docs --name pytorch --version 2.3.0
```

Walks the directory, finds every markdown/MDX file, creates one entry per file. First `#` heading → entry name. Full content → entry body. FTS5 index built over both columns.

Compatible with Docusaurus, MkDocs, Mintlify, Sphinx markdown, plain READMEs. `mx build` operates on the common denominator: markdown files in a directory.

Publishing can be added to release CI alongside `npm publish` or `twine upload`.

## Rust CLI

Single static binary via Rust. No Node.js, Python, or JVM dependency.

Startup latency matters — agents invoke the CLI repeatedly. Node.js CLI: ~200ms V8 overhead per invocation. Rust binary: under 5ms. Over 50 queries: 10 seconds vs 250 milliseconds.

`rusqlite` with compile-time bundled SQLite + FTS5. No dependency on system SQLite version.

## Open questions

### Bootstrapping the registry

Cold start problem. Plan: seed with 20-30 packages for popular libraries (React, Next.js, PyTorch, FastAPI, Tailwind, Django, etc.) built from existing public docs. Make `mx build && mx publish` low-friction enough that maintainers add it to their release process.

### Package ownership and trust

First-come-first-served namespace registration with verification path for official maintainers. Verified publisher badges for packages maintained by the library's own team. Mirrors npm/crates.io approach.

### Large libraries

PyTorch: thousands of entries, 10-50MB uncompressed. Likely acceptable as a one-time download. Sub-packages (`pytorch-core`, `pytorch-nn`) are possible but add dependency resolution complexity — only if demonstrated need.

### Documentation freshness

Mandex doesn't solve stale docs if authors don't publish updates. Moves responsibility to where it belongs — the library maintainer. Build-and-publish integrates into CI/CD alongside existing release workflows.

### Versioning granularity

Mirror library major.minor versions. Independent patch versions for doc-only fixes (e.g., `pytorch@2.3.0` docs might get a `2.3.0-1` doc patch).

## Why this might not work

1. **Cold start** — registries are hard to bootstrap without packages
2. **Maintenance burden** — docs go stale if authors don't update
3. **Competitors** — @neuledge/context has 100+ packages, Context7/Docfork have massive user bases
4. **Context windows growing** — the problem may shrink as models get larger context and fresher training data
5. **Format adoption** — yet another thing for authors to publish

## Why this might work

1. **Pain is real and growing** — Context7's rate limit cuts prove per-query pricing is broken for agent workflows
2. **Local-first is correct** — docs are read-heavy, write-rare. CDN distribution matches the access pattern.
3. **Zero-config for authors** — `mx build ./docs` works on existing markdown. No format adoption cost.
4. **Rust binary** — installs in seconds, no runtime conflicts, fast startup
5. **Registry flywheel** — seed packages → developers install → authors publish → more packages
6. **Ecosystem-agnostic** — Python, JS, Rust, Go. One tool for all docs.

## v0.1 scope

MVP:
- [ ] `mx pull <package>` — download from CDN
- [ ] `mx search <query>` — FTS5 search across installed packages
- [ ] `mx show <package> <entry>` — display full entry
- [ ] `mx list` — list installed packages
- [ ] `mx build <dir>` — build a .mandex from markdown source
- [ ] 5-10 hand-curated seed packages
- [ ] CDN setup on Cloudflare R2
- [ ] Basic registry metadata (registry.json, meta.json)

NOT in v0.1:
- `mx sync` (project auto-detection)
- `mx publish` (authenticated publishing)
- `mx serve` (MCP server mode)
- Web registry browser (beyond the static site)
- User accounts / auth
