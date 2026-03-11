# Mandex — Design Document

## What is Mandex?

Mandex (manual + index) is a local-first documentation index registry and CLI (`mx`). It lets library authors publish compressed, searchable documentation packages to a CDN, and lets developers (and their AI agents) download and query them locally — offline, with zero rate limits, in sub-millisecond time.

## The Problem

AI coding agents hallucinate when they don't have up-to-date documentation. The current solutions all have problems:

1. **Training data** — stale by months/years, agents confidently suggest deprecated APIs
2. **Cloud MCP servers (Context7, Docfork)** — rate-limited, require network calls for every query, single point of failure mid-session
3. **Local MCP servers (@neuledge/context)** — better, but still MCP overhead per query, requires Node.js runtime, builds docs from git repos at install time
4. **Dash/Zeal docsets** — human-focused (full HTML), not optimized for agent consumption, no community registry model

### Why not just use @neuledge/context?

It's the closest thing to what we want, but:

- **TypeScript/Node dependency** — heavy runtime requirement for what should be a simple file download + search
- **MCP-only interface** — docs are only accessible through MCP, not as a standalone CLI or library
- **Build-from-source model** — you point it at a git repo and it builds the index locally. This means every user repeats the same work, and quality depends on the repo's markdown structure
- **No author-curated packages** — library authors don't control how their docs appear. The indexer scrapes markdown and hopes for the best
- **59 stars, 2 contributors** — early stage, unclear if it will gain traction

### Why not Docfork or Context7?

- Both are **cloud-first** — they own the infra, you pay per query
- Context7 cut their free tier by 92% (6,000 → 500 req/month) in Jan 2026, breaking workflows overnight
- Docfork is essentially a Context7 clone with "Cabinets" (curated stacks)
- Fundamental model flaw: charging per-query for documentation access doesn't scale with how agents work (10s-100s of lookups per session)

## What Makes Mandex Different

### 1. Author-first registry

Library authors publish their own doc packages, controlling quality, structure, and versioning. Think crates.io or npm, but for documentation. This is the key differentiator — no scraping, no hoping the markdown is well-structured. Authors know their API surface best.

### 2. Rust CLI, no runtime dependency

Single static binary. No Node.js, no Python, no JVM. Download and run. Fast startup, minimal resource usage.

### 3. CDN-distributed, not server-distributed

Doc packages are pre-built, compressed, and hosted on Cloudflare R2. Downloading a package is a single HTTP GET — same as downloading a binary release. No build step, no git clone, no indexing. The CDN scales infinitely at near-zero marginal cost.

### 4. Not MCP-locked

`mx search` works from any terminal. It can be piped, scripted, embedded. MCP integration is a layer on top, not the only interface. This means:
- Humans can use it directly
- Agents can use it via tool definitions, MCP, or just shell commands
- CI/CD can query docs programmatically

### 5. Project-aware auto-sync

`mx sync` reads your project's dependency files (package.json, requirements.txt, Cargo.toml, pyproject.toml, go.mod, etc.) and downloads matching doc packages. Zero-config for the common case.

## Architecture

### Doc Package Format

A mandex package is a **zstd-compressed SQLite database** with FTS5 full-text search.

```
pytorch@2.3.0.mandex   (the compressed file on CDN)
  └── pytorch@2.3.0.db  (SQLite database after decompression)
```

#### SQLite Schema

```sql
-- Package metadata
CREATE TABLE metadata (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
-- Populated with: name, version, description, homepage, repository, license, authors

-- Documentation entries
CREATE TABLE entries (
    id        INTEGER PRIMARY KEY AUTOINCREMENT,
    name      TEXT NOT NULL,          -- e.g. "torch.nn.Linear"
    kind      TEXT NOT NULL,          -- "class", "function", "method", "module", "guide", "example"
    signature TEXT,                   -- e.g. "torch.nn.Linear(in_features, out_features, bias=True)"
    brief     TEXT NOT NULL,          -- one-line description
    content   TEXT NOT NULL,          -- full documentation (markdown)
    params    TEXT,                   -- JSON array of {name, type, desc}
    returns   TEXT,                   -- return type/description
    examples  TEXT,                   -- code examples (markdown)
    see_also  TEXT,                   -- JSON array of related entry names
    tags      TEXT                    -- comma-separated tags for filtering
);

-- Full-text search index
CREATE VIRTUAL TABLE entries_fts USING fts5(
    name,
    brief,
    content,
    tags,
    content=entries,
    content_rowid=id,
    tokenize='porter unicode61'
);
```

#### Why this schema?

- **`name` + `kind` + `signature`** — enough for an agent to identify and use an API without reading the full content
- **`brief`** — one-line summary, ideal for search result listings (low token cost)
- **`content`** — full docs when the agent needs depth
- **`params`/`returns`** — structured data for function/method entries, agents can use these directly
- **`examples`** — separated from content so agents can request "just show me an example"
- **FTS5 with porter stemming** — handles "tokenizer" matching "tokenize", "linear" matching "linearity", etc.
- **BM25 ranking** — built into FTS5, gives relevance-ranked results for free

### CDN Layout

```
cdn.mandex.dev/v1/registry.json                    # global package listing
cdn.mandex.dev/v1/{package}/meta.json               # package metadata + version list
cdn.mandex.dev/v1/{package}/{version}.mandex         # compressed SQLite db
cdn.mandex.dev/v1/{package}/{version}.sha256          # checksum
```

Hosted on Cloudflare R2 (S3-compatible, zero egress fees).

`registry.json`:
```json
{
  "packages": {
    "pytorch": {
      "latest": "2.3.0",
      "description": "PyTorch deep learning framework",
      "ecosystem": "pip"
    },
    "nextjs": {
      "latest": "16.0.0",
      "description": "The React framework",
      "ecosystem": "npm"
    }
  }
}
```

`meta.json`:
```json
{
  "name": "pytorch",
  "description": "PyTorch deep learning framework",
  "ecosystem": "pip",
  "package_name": "torch",
  "homepage": "https://pytorch.org",
  "repository": "https://github.com/pytorch/pytorch",
  "versions": [
    {
      "version": "2.3.0",
      "published": "2026-03-01T00:00:00Z",
      "size_bytes": 524288,
      "entry_count": 1847,
      "sha256": "abc123..."
    }
  ]
}
```

### Local Storage

```
~/.mandex/
├── config.toml                  # user configuration
├── registry.json                # cached registry index
├── packages/
│   ├── pytorch/
│   │   └── 2.3.0.db            # decompressed, ready to query
│   ├── nextjs/
│   │   └── 16.0.0.db
│   └── ...
└── mappings.toml                # ecosystem package name → mandex package name
```

`mappings.toml` solves the naming problem:
```toml
[pip]
torch = "pytorch"
beautifulsoup4 = "beautifulsoup"
scikit-learn = "sklearn"

[npm]
next = "nextjs"
"@anthropic-ai/sdk" = "anthropic-sdk"
```

### CLI Interface

```bash
# Package management
mx pull pytorch                    # download latest pytorch docs
mx pull pytorch@2.3.0             # download specific version
mx sync                            # auto-detect project deps, download matching docs
mx list                            # list installed packages
mx remove pytorch                  # remove a package
mx update                          # update all packages to latest

# Search & query
mx search "linear layer"           # search across all installed packages
mx search pytorch "nn.Linear"      # search within a specific package
mx show pytorch torch.nn.Linear    # show full docs for a specific entry
mx show --brief pytorch torch.nn.Linear  # just the signature + one-liner

# Publishing (for authors)
mx login                           # authenticate with mandex registry
mx publish my-package.mandex       # publish a package
mx build ./docs-dir                # build a .mandex package from source docs

# Info
mx info pytorch                    # show package metadata
mx doctor                          # check local state, connectivity
```

### Package Name Mapping Strategy

The `mx sync` command needs to map dependency names to mandex package names:

1. **Check local `mappings.toml`** cache first
2. **Check `meta.json` `package_name` field** — each mandex package declares what ecosystem package(s) it documents
3. **Try exact name match** — `requests` in pip → `requests` in mandex
4. **Fail gracefully** — skip packages with no matching docs, don't error

### Publishing Flow (for library authors)

```
1. Author creates documentation entries (JSON, TOML, or Markdown with frontmatter)
2. `mx build ./docs/` compiles them into a .mandex SQLite package
3. `mx publish` uploads to R2 via authenticated API
4. CDN serves it globally within seconds
```

#### Source format for authoring (one file per entry):

```markdown
---
name: torch.nn.Linear
kind: class
signature: "torch.nn.Linear(in_features, out_features, bias=True, device=None, dtype=None)"
brief: "Applies an affine linear transformation to the input."
tags: neural-network, layer, linear
see_also: ["torch.nn.Bilinear", "torch.nn.LazyLinear"]
---

## Parameters

- **in_features** (`int`): size of each input sample
- **out_features** (`int`): size of each output sample
- **bias** (`bool`): If `True`, adds a learnable bias. Default: `True`

## Shape

- Input: (*, H_in) where * means any number of dimensions and H_in = in_features
- Output: (*, H_out) where H_out = out_features

## Examples

```python
m = nn.Linear(20, 30)
input = torch.randn(128, 20)
output = m(input)
print(output.size())  # torch.Size([128, 30])
`` `
```

Or bulk JSON for programmatic generation:

```json
[
  {
    "name": "torch.nn.Linear",
    "kind": "class",
    "signature": "torch.nn.Linear(in_features, out_features, bias=True)",
    "brief": "Applies an affine linear transformation to the input.",
    "content": "Full markdown documentation here...",
    "params": [
      {"name": "in_features", "type": "int", "desc": "size of each input sample"},
      {"name": "out_features", "type": "int", "desc": "size of each output sample"}
    ],
    "examples": "```python\nm = nn.Linear(20, 30)\n```",
    "see_also": ["torch.nn.Bilinear"]
  }
]
```

## Open Questions

### 1. How to bootstrap the registry?

The chicken-and-egg problem: developers won't install mandex without packages, authors won't publish without users. Options:
- **Seed it ourselves** — manually curate 20-30 popular packages (React, Next.js, PyTorch, FastAPI, etc.) to prove the format works
- **Auto-generate from existing docs** — scrape + LLM-summarize popular library docs as a starting point, then let authors take over
- **Partner with library authors** — reach out to maintainers of popular libraries directly

### 2. Authentication & trust for publishing

- Who can publish `pytorch`? The PyTorch team, or anyone?
- Options: namespace ownership (like npm), verified publishers, or open-with-moderation
- Probably start simple: first-come-first-served with abuse reporting, add verification later

### 3. How to handle large libraries?

PyTorch has thousands of API entries. A single .db could be 10s of MB. Options:
- **Sub-packages**: `pytorch-core`, `pytorch-nn`, `pytorch-optim`, etc.
- **Lazy loading**: download a lightweight index first, fetch full entries on demand
- **Just let it be big**: 10MB compressed is fine for a one-time download

### 4. Versioning granularity

- One mandex package version per library version? (e.g., `pytorch@2.3.0` maps to PyTorch 2.3.0)
- Or independent versioning? (e.g., mandex package v3 covers PyTorch 2.3.x)
- Probably: mirror library major.minor, with independent patch for doc fixes

### 5. MCP integration

Should mandex ship an MCP server mode (`mx serve`) so it plugs into existing MCP clients? Probably yes, but as a secondary interface — the CLI-first approach is the core value.

### 6. How does this compare to just putting docs in CLAUDE.md / .cursorrules?

Some teams paste relevant docs into their project's AI instruction files. This works for small, stable APIs but:
- Doesn't scale (token limits, manual maintenance)
- Not searchable (full dump into context)
- Not versioned or shared
- Mandex is the infra that feeds those files, not a replacement for them

## Why This Might Not Work

1. **Adoption chicken-and-egg** — registries are hard to bootstrap. Without packages, no users; without users, no authors.
2. **Maintenance burden** — docs go stale. If authors don't update their mandex packages, the tool has the same staleness problem as training data.
3. **"Good enough" competitors** — @neuledge/context already exists with 100+ packages. Context7/Docfork have massive user bases despite rate limits.
4. **Agents might not need this** — as model context windows grow and training data freshness improves, the problem may shrink.
5. **Format fragmentation** — yet another doc format that authors need to support alongside their existing docs.

## Why This Might Work

1. **The pain is real and growing** — Context7's rate limit cuts prove cloud doc services have a fundamental business model problem. Agents make too many queries for per-request pricing.
2. **Local-first is the right architecture** — docs are read-heavy, write-rare. Download once, query forever. This is what CDNs are built for.
3. **Author-curated > auto-scraped** — the quality ceiling is much higher when library authors control their doc packages.
4. **Rust CLI is a distribution advantage** — single binary, no runtime. Works everywhere, installs in seconds.
5. **The registry flywheel** — once popular packages exist, developers install mandex. Once developers exist, authors are motivated to publish. The hard part is the first 20 packages.
6. **Ecosystem-agnostic** — works with Python, JS, Rust, Go, etc. One tool for all your docs.

## v0.1 Scope

Minimum viable product:
- [ ] `mx pull <package>` — download from CDN
- [ ] `mx search <query>` — FTS5 search across installed packages
- [ ] `mx show <package> <entry>` — display full entry
- [ ] `mx list` — list installed packages
- [ ] `mx build <dir>` — build a .mandex from markdown source
- [ ] 3-5 hand-curated seed packages (e.g., FastAPI, Next.js, Tailwind)
- [ ] Basic CDN setup on Cloudflare R2

NOT in v0.1:
- `mx sync` (project auto-detection)
- `mx publish` (authenticated publishing)
- MCP server mode
- Web UI / registry browser
- User accounts / auth
