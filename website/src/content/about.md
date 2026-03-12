# Documentation should be distributed like packages, not served like APIs

This post isn't about documentation for humans. Humans are resourceful — we google, we bookmark, we skim Stack Overflow, we read the source when we have to. This is about documentation for AI agents: the coding assistants that need to look up the same APIs we do, except they can't open a browser tab and squint at a page until they find the right paragraph.

Agents need documentation to write correct code. When they don't have it, they fall back to training data — which might be months or years stale. You get code that references deprecated APIs, uses outdated patterns, or just doesn't compile. The documentation exists and it's freely available. The problem is getting it into the agent's context at the right time, in the right version, without wasting half the context window on HTML boilerplate.

Several approaches exist today. Each addresses a real part of the problem. Each breaks in a different way.

## How agents get documentation today

### Direct web fetching

The simplest approach. The agent searches the web for documentation, gets URLs, and fetches pages directly. Most agents with web access do this as a fallback when they don't have documentation in context.

```
Agent: I need to check the API for FastAPI's Depends function.
> web_search("FastAPI Depends injection")   
> fetch_url("https://fastapi.tiangolo.com/tutorial/dependencies/")
  ⚠ Blocked by Cloudflare bot detection
> fetch_url("https://stackoverflow.com/questions/...")
  ✓ 200 OK — but this is a 2022 answer about an older API
> fetch_url("https://fastapi.tiangolo.com/reference/dependencies/")
  ✓ 200 OK — 14,200 tokens of HTML, ~300 tokens of relevant content
```

A single lookup can require 5-10 HTTP requests and thousands of tokens of irrelevant HTML to extract a few lines of useful content. The page containing a function signature is roughly 95% noise by token count — navigation bars, sidebars, footers, cookie banners — because it was authored and rendered for humans, not agents. This cost is paid per lookup, per session, per developer. At scale, the aggregate traffic pattern is indistinguishable from a distributed denial-of-service (DDoS) attack. Site operators respond by tightening bot detection, which increases agent failure rates, which increases retries. A feedback loop that degrades the experience for everyone.

### llms.txt

A convention where site owners place a `/llms.txt` or `/llms-full.txt` file at their domain root, providing a machine-readable summary of the site's content optimized for LLM consumption. Instead of agents parsing HTML pages designed for humans, give them a dedicated endpoint with just the content. Some sites also provide `/llms-full.txt` with complete documentation in a single file.

```
Agent: I need docs for Tailwind's grid utilities.
> fetch_url("https://tailwindcss.com/llms-full.txt")
  ✓ 200 OK — 428,000 tokens
  [Loading entire Tailwind documentation into context...]
```

This solves the HTML noise problem — agents get clean, structured content without the navigation chrome. But it introduces its own issues.

The format is a full dump. An `llms-full.txt` for a non-trivial library can be tens or hundreds of thousands of tokens. An agent looking up a single function signature has to either ingest the entire file or do string matching over it. There's no search index, no relevance ranking, no way to retrieve just the relevant section without processing the whole document. For small libraries this is fine. For something like PyTorch or React, loading the full documentation into an agent's context window is impractical — it would consume most or all of the available context.

There's also no versioning. `/llms.txt` serves whatever version the site currently has deployed. A developer pinned to Next.js 14 hits `nextjs.org/llms.txt` and gets the Next.js 16 docs. There's no mechanism to request documentation for a specific version. The site operator would need to host versioned paths (`/v14/llms.txt`, `/v16/llms.txt`), but there's no convention for this and most sites don't.

And it still requires a network request. Every agent session that needs documentation makes an HTTP call to the library's site. This is better than scraping HTML, but it still depends on the site being up, being fast, and not rate-limiting agent traffic. The access pattern hasn't changed — it's still a synchronous fetch per documentation need.

llms.txt is the right idea — give agents content in a format they can use — but it stops short of solving the retrieval, versioning, and offline access problems.

### Cloud MCP documentation servers

Services like Context7 and Docfork pre-index open source library documentation and serve it via MCP (Model Context Protocol). Instead of each agent independently scraping the web, agents query a centralized server that returns structured, relevant documentation snippets. Context7 alone has over 42,000 GitHub stars — pre-indexed documentation with structured search clearly resonated.

```
Agent: I need to look up Next.js middleware API.
> mcp__context7__search_docs("nextjs", "middleware")
  ✓ Returns 3 relevant documentation snippets (~2,000 tokens)
```

When it works, the experience is good — fast, relevant, structured results. But the model has two structural issues: economics and versioning.

**The economic problem.** These services index open source documentation — content that library authors wrote and published freely — and serve it behind per-query pricing and rate limits. Context7's free tier went from approximately 6,000 requests per month to 500 in January 2026. At the rate agents consume documentation — 50 to 100 lookups per coding session for non-trivial work — that's roughly 10 sessions per month on the free tier.

The rate limits aren't arbitrary. They're a direct consequence of the architecture. Each agent query hits a server that must parse the request, search an index, and return a response. Every concurrent agent session is a burst of rapid-fire requests. The infrastructure needs to handle the aggregate load from all users, with spiky traffic patterns that are expensive to provision for. Per-query pricing and rate limits are the only way to keep the service economically viable under this model. The servers would fall over otherwise.

This creates a misalignment between the access pattern and the serving model. Documentation is static content. It changes when a library publishes a new version — monthly, quarterly, sometimes less frequently. Between releases, the content is identical for every query. Serving it through a request-response API that does real-time index lookups is equivalent to running a database query to serve a paragraph that hasn't changed in three months. The compute cost per query is small, but multiplied across millions of agent invocations, it forces a pricing model that constrains usage at exactly the point where agents need to be unconstrained.

**The versioning problem.** Consider Next.js. The App Router API surface changed meaningfully between versions 13, 14, 15, and 16. A developer on version 14 needs different documentation than one on version 16 — different function signatures, different patterns, different recommended approaches. The content overlap between adjacent major versions might be 80%, with the remaining 20% containing breaking changes.

A centralized server that wants to support multiple versions has two options, both bad.

The first is to index every version. The search index now contains N copies of mostly-identical content, with subtle differences distributed across them. When an agent queries "App Router dynamic routes," the index returns results from multiple versions with similar BM25 scores — the content overlap makes them nearly indistinguishable by relevance ranking alone. The query needs to carry version context to disambiguate, which means the agent needs to extract the correct dependency version from the project and pass it with every request. This adds per-query complexity and depends on the agent reliably determining which version the project uses — a step that agents handle inconsistently.

The second is to index only the latest version. This is simpler and what most MCP servers do. But most developers aren't on the latest version of every dependency. The agent returns documentation for v16 when the project is pinned to v14. The code looks plausible but uses APIs that don't exist in the installed version. The failure mode — code that almost works but doesn't quite compile — is worse than an obviously wrong answer, because the developer spends time debugging subtle version mismatches instead of recognizing an obvious error.

## Mandex: documentation as packages

All three approaches treat documentation as something to be fetched on demand — from a website, a text file, or an API. The architectures differ, but the access pattern is the same: the agent needs information, so it makes a network request and waits for a response. The problems (rate limits, version mismatches, context bloat) are consequences of this model.

But documentation isn't dynamic content. It doesn't change based on who's reading it or when they ask. A library's docs are written once per release and read by thousands of developers between releases. This is the same access pattern as software packages: authored once per version, distributed widely, read many times, never modified in place.

Software packages aren't served through request-response APIs. You don't query npm on every `import` statement. You install packages locally and they're available immediately — fast, offline, versioned. Documentation should work the same way.

Mandex is a package registry for documentation. Library authors build searchable documentation packages from their existing docs. The packages are compressed and distributed through a CDN. Developers download them once and query them locally.

```bash
mx pull pytorch@2.3.0
mx pull nextjs@14.0.0
mx search pytorch "attention mechanism"
```

After the initial download, all queries are local. There's no network call, no server process, no rate limit, no API key. The same query can run a thousand times in a session at the same cost: zero.

Versioning is solved by the package model itself. `mx pull nextjs@14.0.0` downloads a package containing only the Next.js 14 documentation. The search index has no v16 content to confuse results. There's no version routing, no query-time disambiguation, no dependence on the agent correctly passing version context. The right version was selected at download time.

The CLI outputs to stdout — it can be piped, redirected, or read by an agent through tool invocation. Mandex works with any agent that can execute shell commands — Claude Code, Cursor, Copilot, or a custom agent framework. `mx serve` starts an MCP server for environments where that protocol is preferred, but MCP is a transport layer on top, not a requirement.

## For library authors

The build step works on documentation as it already exists. There's no custom format, no required frontmatter schema, no migration from current tooling.

```bash
mx build ./docs --name pytorch --version 2.3.0
```

The command walks the target directory, finds every markdown and MDX file, and creates one database entry per file. The first `#` heading becomes the entry name; the full content becomes the entry body. The FTS5 search index is built over both columns. This is compatible with documentation source formats already in widespread use — Docusaurus, MkDocs, Mintlify, plain README collections. `mx build` operates on the common denominator.

The cost of publishing is running one command against a directory that already exists. There's no docs rewrite, no format adoption, no workflow change. Publishing can be added to the release CI pipeline alongside the npm publish or PyPI upload authors already run. In return, every developer using mandex gets the author's documentation — at the correct version, searchable, offline — without the author operating any infrastructure. The CDN handles distribution. The CLI handles search. The author's only job is the one they already have: writing good docs.

With scraping-based tools, the library author has no say in how their documentation is indexed, chunked, or presented to agents. Poorly chunked docs produce poor search results, and the author can't fix it. With mandex, the author controls the source material and can structure it for the best possible agent experience — or just ship their existing docs and let the format do its job.

## Architecture

### Package format

A Mandex package is a zstd-compressed SQLite database with an FTS5 full-text search index.

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

Two columns. `name` is derived from the first heading in the source markdown file, or the filename if no heading exists. `content` is the full markdown text.

There's no `params` column, no `signature` column, no `returns` or `tags` or `kind`. This is intentional. Documentation already contains all of that information — function signatures, parameter lists, type annotations, examples, usage notes — in markdown that LLMs parse without difficulty. Imposing a structured schema on top would require that schema to work across every kind of library documentation: PyTorch's API references, Next.js's conceptual guides, Tailwind's utility class listings, Django's tutorial-style docs. No field set fits all of them. Markdown handles this naturally.

The CLI controls display. When listing search results, it shows the entry name and the first N characters of content — enough for the agent to decide which entry to read in full. When an agent requests a specific entry, the CLI returns the complete content. The truncation is a display concern handled at query time, not a schema concern baked into the data.

### SQLite as the package format

The package format needs to serve three roles: storage container, search index, and query engine. SQLite does all three in a single file with no server process.

FTS5 provides full-text search with BM25 relevance ranking, porter stemming ("tokenizer" matches "tokenize"), prefix queries, phrase matching, and boolean operators. This is built into SQLite, not a separate library. A mandex `.db` file is queryable with any SQLite client in any programming language — Python's `sqlite3`, Rust's `rusqlite`, the `sqlite3` CLI, any of them.

This has a practical consequence for the format's longevity. If Mandex as a project ceased to exist, every published package would remain fully functional. The files are standard SQLite databases. There's no proprietary encoding, no format-specific decoder required, no version compatibility to manage. The format is as stable as SQLite itself, which has a published commitment to backwards compatibility through 2050.

The alternative — compressed JSON with a separate search index (Tantivy, Meilisearch, etc.) — would require shipping two artifacts per package with format coupling between them, and every consuming client would need to embed a compatible search engine version. SQLite avoids this entirely.

### Download and sync

Packages are compressed with zstd before upload — documentation is highly compressible text, so a package with 2,000 entries at 20MB uncompressed ships as 2-4MB over the wire. Each download is an HTTP GET to a CDN edge node — no authentication, no API server, no query processing.

Packages are stored in a global cache (`~/.mandex/cache/`), shared across all projects. If two projects both use `react@19.1.0`, the package is downloaded once. This is the same model as pnpm's content-addressable store or cargo's global registry cache.

The per-project behavior comes from `mx sync`. It reads the project's dependency files — `package.json`, `requirements.txt`, `Cargo.toml`, `pyproject.toml`, `go.mod` — resolves each dependency to a mandex package via registry metadata, downloads any missing packages into the global cache, and writes a project-local manifest (`.mandex/manifest.json`) listing which packages and versions this project uses.

```bash
$ mx sync
  Reading package.json...
  Resolved 14 dependencies to mandex packages
  ↓ react@19.1.0          2.1 MB  [===========] done
  ↓ next@14.2.0           4.7 MB  [===========] done
  ↓ tailwindcss@4.1.0     1.8 MB  [===========] done
  ↓ @tanstack/query@5.0   1.2 MB  [===========] done
  ... (10 more)
  Synced 14 packages in 1.4s
```

When `mx search` runs inside a project directory, it reads the manifest and queries only the packages relevant to that project. You might have 50 packages in your global cache across all your projects, but a search in your Next.js project only hits the 14 databases listed in that project's manifest. The dependency file is the source of truth for which documentation is in scope.


## Open questions

### Bootstrapping the registry

Package registries have a cold start problem. Developers won't adopt the tool without packages available. Authors won't publish without a user base.

The initial approach is to seed the registry with 20-30 packages for widely-used libraries across ecosystems — React, Next.js, PyTorch, FastAPI, Tailwind, Django, etc. — built from their existing public documentation. This provides enough coverage to be immediately useful and demonstrates the format. From there, the path is making `mx build && mx publish` low-friction enough that library maintainers add it to their release process.

### Package ownership and trust

Who can publish the `pytorch` package? The current plan is first-come-first-served namespace registration with a verification path for official maintainers. This mirrors how npm and crates.io handle it, with the addition of verified publisher badges for packages maintained by the library's own team.

### Large libraries

A library like PyTorch has thousands of API entries. The resulting `.db` file might be 10-50MB uncompressed. This is likely acceptable — it's a one-time download, and 50MB is small by modern standards. If it becomes a problem, the format supports splitting into sub-packages (`pytorch-core`, `pytorch-nn`, `pytorch-optim`), but this adds complexity to the dependency resolution and should only be done if there's a demonstrated need.

### Documentation freshness

Mandex doesn't solve the problem of documentation going stale if authors don't publish updated packages. It moves the responsibility to the library maintainer, which is where it belongs — they're already responsible for keeping their docs current. The build-and-publish step can be integrated into CI/CD, so publishing a new mandex package is part of the release process rather than a separate manual step.

## Getting started

Mandex is written in Rust. The CLI is a single static binary called `mx`.

```bash
curl -fsSL https://mandex.dev/install.sh | sh

cd your-project
mx sync
mx search nextjs "middleware"
```

The format is SQLite. The packages are portable. The source is open. Everything works offline after the initial download.

---

Agents need documentation to write correct code. The documentation exists. The missing piece was never the content — it was the distribution model. Package it, version it, and distribute it through infrastructure that scales to zero marginal cost. Then let agents query it locally, as many times as they need, without asking anyone's permission.
