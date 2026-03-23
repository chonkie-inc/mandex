# What is mandex?

Mandex is a package registry and CLI for library documentation, built for AI coding agents. It lets agents search version-pinned documentation locally instead of fetching docs from the web on every query.

```bash
mx pull fastapi@0.135.0
mx search fastapi "dependency injection"
```

After the initial download, all queries are local — no network call, no API key, no rate limit. The same query can run a thousand times in a session at zero cost.

## The problem

AI coding agents need library documentation to write correct code. Without it, they fall back to training data — which may be months or years stale. The result is code that references deprecated APIs, uses outdated patterns, or doesn't compile.

The documentation exists and it's freely available. The problem is getting it into the agent's context at the right time, in the right version, without wasting the context window on HTML boilerplate.

### How agents get docs today

**Direct web fetching** — the agent searches the web, fetches HTML pages, and parses out content. A single lookup can require 5-10 HTTP requests and thousands of tokens of irrelevant HTML (navigation, sidebars, cookie banners) to extract a few lines of useful content. At scale, the traffic pattern looks like a DDoS attack, and site operators respond by tightening bot detection.

**llms.txt** — site owners place a machine-readable text file at their domain root with clean documentation content. This solves the HTML noise problem, but the format is a full dump. An `llms-full.txt` for a non-trivial library can be hundreds of thousands of tokens. There's no search index, no versioning, and every session still requires a network request.

**Cloud MCP servers** — services like Context7 pre-index open source documentation and serve it via MCP. When it works, the experience is good. But the architecture forces per-query pricing and rate limits (Context7's free tier dropped from ~6,000 to 500 requests/month), and most services only index the latest version — not the version your project actually uses.

### The core insight

Documentation isn't dynamic content. It's written once per release and read by thousands of developers between releases. This is the same access pattern as software packages: authored once, distributed widely, read many times.

Software packages aren't served through request-response APIs. You don't query npm on every `import` statement. You install packages locally and they're available immediately. Documentation should work the same way.

## Architecture

### Package format

A mandex package is a zstd-compressed SQLite database with an FTS5 full-text search index.

```sql
CREATE TABLE entries (
    id      INTEGER PRIMARY KEY AUTOINCREMENT,
    name    TEXT NOT NULL,
    content TEXT NOT NULL
);

CREATE VIRTUAL TABLE entries_fts USING fts5(
    name, content,
    content=entries, content_rowid=id,
    tokenize='porter unicode61'
);
```

Two columns. `name` is derived from the first heading in the source markdown file. `content` is the full markdown text. There's no structured schema on top — documentation already contains function signatures, parameter lists, type annotations, and examples in markdown that LLMs parse naturally.

### Why SQLite?

The package format needs to serve three roles: storage container, search index, and query engine. SQLite does all three in a single file with no server process.

FTS5 provides full-text search with BM25 relevance ranking, porter stemming, prefix queries, phrase matching, and boolean operators — all built into SQLite. A mandex `.db` file is queryable with any SQLite client in any language.

If mandex as a project ceased to exist, every published package would remain fully functional. The files are standard SQLite databases with no proprietary encoding. The format is as stable as SQLite itself, which has a published commitment to backwards compatibility through 2050.

### Neural reranking

Search results from FTS5 are reranked by a local neural cross-encoder model for semantic relevance. The reranker is a 19MB ONNX model (ms-marco-MiniLM) that runs entirely on-device — no cloud API, no GPU required.

The reranker uses [tokie](https://github.com/chonkie-inc/tokie) for tokenization (a fast Rust tokenizer with pre-built binary `.tkz` format) and ONNX Runtime for inference with multi-threaded execution and memory-mapped model loading.

Typical search latency is **40ms** including FTS5 search and neural reranking.

### Per-project index

When you run `mx sync`, mandex reads your project's dependency files (`package.json`, `requirements.txt`, `Cargo.toml`, `pyproject.toml`) and builds a merged FTS5 index at `.mandex/index.db`. This means search queries one database instead of N separate package databases — cutting latency and ensuring only project-relevant docs are in scope.

```
my-project/
├── .mandex/
│   ├── manifest.json    # {"packages": {"fastapi": "0.135.1", "numpy": "2.4.3"}}
│   └── index.db         # merged FTS5 index
├── package.json
└── ...
```

### Download and sync

Packages are compressed with zstd before upload — documentation is highly compressible text, so a package with 2,000 entries at 20MB uncompressed ships as 2-4MB. Each download is an HTTP GET to a CDN edge node — no authentication, no API server.

Packages are stored in a global cache (`~/.mandex/cache/`), shared across projects. If two projects both use `react@19.2.0`, the package is downloaded once.

```bash
$ mx sync
  Reading package.json...
  ✓ react 19.2.0 (already installed)
  ✓ nextjs 16.1.7 (already installed)
  ↓ drizzle-orm@0.45.1 (new)
  Built search index (14505 entries from 3 packages)
```

## Building packages

Any directory of markdown or MDX files can be turned into a mandex package:

```bash
mx build ./docs --name my-lib --version 1.0.0
```

The command walks the directory, finds every `.md` and `.mdx` file, extracts the first `#` heading as the entry name, and stores the full content. Large files (>16KB) are split by `##` headings into sections. The output is a zstd-compressed SQLite database.

This is compatible with Docusaurus, MkDocs, Mintlify, Starlight, Sphinx (with markdown), and plain markdown collections. No format migration required — `mx build` operates on the common denominator.

The cost of publishing is running one command against a directory that already exists. Publishing can be added to CI alongside the `npm publish` or `pip upload` authors already run.

## Agent integrations

Mandex works with any AI coding assistant that can run shell commands.

```bash
mx init
```

This sets up integrations for detected AI tools:
- **Claude Code** — installs a [skill](https://docs.anthropic.com/en/docs/claude-code) that tells Claude to search mandex before generating code
- **Cursor** — appends rules to `.cursor/rules`
- **Windsurf** — appends rules to `.windsurfrules`
- **Codex** — adds instructions to `~/.codex/AGENTS.md`

The practical pattern: when an agent needs to generate code using a library, it runs `mx search` to find the relevant documentation, reads the entries, and writes code grounded in the exact API for the installed version.

## Commands

| Command | Description |
|---------|-------------|
| `mx pull <package>[@version]` | Download documentation for a library |
| `mx search <package> "<query>"` | Search within a package |
| `mx search "<query>"` | Search across all installed packages |
| `mx show <package> "<entry>"` | Show the full content of a specific entry |
| `mx sync` | Auto-detect project dependencies and download docs |
| `mx list` | Show installed packages with sizes |
| `mx info <package>` | Show package details and versions |
| `mx remove <package>` | Remove an installed package |
| `mx init` | Set up AI assistant integrations |
| `mx build <dir> --name <n> --version <v>` | Build a package from markdown |

## Getting started

```bash
# Install
curl -fsSL https://mandex.dev/install.sh | sh

# Set up your AI coding assistant
mx init

# Sync docs for your current project
cd your-project
mx sync

# Search
mx search nextjs "server actions"
```

Everything works offline after the initial download. The format is SQLite. The packages are portable. The source is [open](https://github.com/chonkie-inc/mandex).
