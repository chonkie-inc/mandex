# Mandex Launch Posts

---

## Twitter / X

### Option A (anti-cloud MCP angle)

> Stop paying for cloud MCP documentation services.
>
> mandex gives your AI agent the same docs — but local, version-pinned, and 25ms instead of 800ms.
>
> One download. Works offline. No API keys. No rate limits. No monthly bill.
>
> 100+ packages. 1,500+ versions. Free forever.
>
> curl -fsSL https://mandex.dev/install.sh | sh
>
> https://mandex.dev

### Option B (comparison thread)

> Cloud MCP docs services:
> - $20/mo+
> - 800ms+ latency per lookup
> - Rate limited
> - Requires internet
> - Version? Whatever they have cached
>
> mandex:
> - Free
> - 25ms search (local SQLite)
> - No rate limits
> - Works offline
> - Exact version you're using
>
> One download per package. Local forever.
>
> 100+ packages, 1,500+ versions. Works with Claude Code, Cursor, Codex, Copilot, and 6 more.
>
> https://mandex.dev

### Option C (short and sharp)

> Your AI agent doesn't need a cloud service to read docs.
>
> mandex — pull once, search locally forever. 25ms. Offline. Version-pinned. Free.
>
> https://mandex.dev

---

## Reddit (r/programming, r/LocalLLaMA, r/ChatGPTPro)

### Option A (r/programming style — technical, understated)

**Title:** Mandex — local documentation packages for AI coding agents (open source, Rust CLI)

**Body:**

We've been building mandex, an open-source CLI tool that gives AI agents instant access to library documentation without network calls.

**The problem:** AI coding agents (Claude Code, Cursor, Codex, etc.) frequently hallucinate API signatures because they either rely on training data (often outdated) or make slow web searches. When you're using FastAPI 0.115 but the agent's training data has 0.95, you get wrong code.

**How it works:**

- `mx pull fastapi@0.115.0` — downloads a compressed SQLite index (~2MB) with 1,965 searchable entries
- `mx search fastapi "rate limiting"` — full-text search in ~25ms, no network needed
- `mx sync` — reads your package.json/requirements.txt/Cargo.toml and pulls docs for all dependencies
- `mx init` — auto-detects and configures Claude Code, Cursor, Codex, Copilot, Windsurf, Cline, and 4 more agents

Everything is local, version-pinned, and free. The registry has 100+ packages with 1,500+ version-matched doc sets (numpy alone has 38 versions from 1.23 to 2.4).

Built in Rust. Optional ONNX reranker for better search quality. MIT licensed.

- GitHub: https://github.com/chonkie-inc/mandex
- Website: https://mandex.dev
- Install: `curl -fsSL https://mandex.dev/install.sh | sh`

Would love feedback from anyone using AI coding tools day-to-day.

### Option B (r/LocalLLaMA style — more casual, emphasize local-first)

**Title:** Made a tool that gives AI agents offline, version-pinned library docs — no more hallucinated APIs

**Body:**

Got tired of Claude/Cursor making up function signatures that don't exist in the version I'm actually using. So I built mandex.

It's dead simple: documentation gets packaged as compressed SQLite databases. You pull them once, they live on disk, and any AI agent can search them locally in milliseconds.

```
mx pull numpy@2.4.3    # 607 entries, 0.6 MB
mx search numpy "array broadcasting"
# results in 25ms, no internet needed
```

The cool part: `mx sync` reads your project's dependency files and auto-pulls docs for everything. And `mx init` configures whichever AI tool you use (supports 10 agents).

Registry has numpy, pandas, scipy, fastapi, react, nextjs, prisma, shadcn, and more — all version-matched.

Rust CLI, MIT licensed, no API keys, no rate limits.

https://mandex.dev

---

## Bookface (YC Internal)

### Option A (founder-to-founder, concise)

**Title:** Mandex — docs for AI agents (from the Chonkie team)

Hey all — we're the team behind Chonkie (the chunking library). We just shipped mandex, a tool that solves a specific problem we kept hitting: AI coding agents hallucinate APIs because they don't have access to the right docs for the right version.

mandex packages documentation as local SQLite indexes. Agents pull them once, search locally in 25ms, and never need to hit the network again. It auto-detects your project dependencies and configures itself for Claude Code, Cursor, Codex, and 7 other agents.

100+ packages in the registry. 1,500+ versions. Free and open source.

Install: `curl -fsSL https://mandex.dev/install.sh | sh`
Site: https://mandex.dev
GitHub: https://github.com/chonkie-inc/mandex

If you're using AI coding tools and getting wrong API suggestions, this fixes that. Happy to answer questions.

### Option B (more context on traction/direction)

**Title:** Show BC: Mandex — local-first documentation for AI agents

From the Chonkie team — we built mandex to solve a problem every AI coding agent has: they guess APIs instead of looking them up.

**What it does:** Packages library documentation as compressed SQLite databases. Agents pull once, search locally forever. 25ms latency, works offline, version-pinned.

**Why it matters:** As AI agents become the default way to write code, they need reliable access to docs — not web scraping, not RAG pipelines, not hoping the training data is fresh. mandex is the package manager for that.

**Where we are:**
- 100+ packages, 1,500+ version-matched doc sets
- Supports 10 AI agents (Claude Code, Cursor, Codex, Copilot, etc.)
- Rust CLI, MIT licensed
- Free registry on Cloudflare R2/D1

**What's next:** More packages (targeting top 100 libraries across pip/npm/cargo), publisher CLI for library authors, and MCP server integration.

https://mandex.dev

---

## Hacker News

### Option A (Show HN — technical, minimal)

**Title:** Show HN: Mandex — Local documentation packages for AI coding agents

**Body:**

AI coding agents frequently hallucinate API signatures because they rely on training data that may be outdated or incomplete. mandex fixes this by packaging documentation as local SQLite databases that agents can search in milliseconds.

How it works:

    mx pull fastapi@0.115.0   # downloads ~2MB compressed SQLite index
    mx search fastapi "dependency injection"   # 25ms, no network
    mx sync   # reads package.json/requirements.txt, pulls all deps

The registry currently has 100+ packages with 1,500+ version-matched doc sets. numpy alone has 38 versions from 1.23.2 to 2.4.3, each with ~600 searchable entries extracted from docstrings.

Technical details:
- Rust CLI, ~5MB binary
- Packages are zstd-compressed SQLite with FTS5 full-text indexes
- Optional ONNX cross-encoder reranker for semantic ranking
- Auto-detects and configures 10 AI agents (Claude Code, Cursor, Codex, Copilot, Windsurf, Cline, OpenClaw, Amp, Antigravity, Gemini)
- `mx sync` parses package.json, requirements.txt, pyproject.toml, Cargo.toml

MIT licensed. Free registry. No API keys.

Install: curl -fsSL https://mandex.dev/install.sh | sh
GitHub: https://github.com/chonkie-inc/mandex
Website: https://mandex.dev

### Option B (more narrative)

**Title:** Show HN: Mandex – Docs as packages for AI agents

**Body:**

I kept watching Claude Code generate plausible-looking but wrong FastAPI code because its training data was from an older version. Web search helps sometimes, but it's slow and unreliable. RAG pipelines are overkill for "what are the params to this function?"

So I built mandex: a package manager for documentation. Library docs get compiled into compressed SQLite databases with FTS5 indexes. You pull them once, they live on disk, and any AI agent can search them locally.

    $ mx pull numpy@2.4.3
    ↓ numpy@2.4.3  0.6 MB
    ✓ 607 entries indexed

    $ mx search numpy "array creation"
    numpy@2.4.3 — numpy.zeros
    ...
    3 results (25ms)

`mx sync` reads your project's dependency files and pulls docs for everything automatically. `mx init` configures whichever AI tool you use.

The registry has 100+ packages across pip, npm, and cargo ecosystems — numpy (38 versions), pytorch (22), scipy (24), pandas (13), fastapi, react, nextjs, prisma, storybook, puppeteer, and more.

Written in Rust. Packages are ~0.5-3MB each. Optional ONNX reranker. MIT licensed.

https://mandex.dev

---

## LinkedIn

### Option A (professional, product-focused)

Excited to share what we've been building at Chonkie: **mandex** — documentation packages for AI coding agents.

The problem: AI agents hallucinate APIs. They generate code using outdated or incorrect function signatures because they don't have access to the right docs for the right version of a library.

mandex solves this by packaging documentation as local, searchable indexes. One download gives you version-pinned search with zero network latency — 25ms average.

What makes it different:
→ Local-first: no API calls, works offline
→ Version-pinned: exact docs for the exact library version you use
→ Universal: works with Claude Code, Cursor, GitHub Copilot, Codex, Windsurf, and 5 more agents
→ Auto-syncs: reads your package.json/requirements.txt and pulls docs for all dependencies

100+ packages in the registry today, including numpy, pandas, scipy, FastAPI, React, Next.js, Prisma, and more — with 1,500+ version-matched documentation sets.

Open source. Free forever. Built in Rust.

Try it: `curl -fsSL https://mandex.dev/install.sh | sh`

https://mandex.dev

### Option B (thought-leadership angle)

As AI coding agents become the default interface for software development, we need better infrastructure for how they access knowledge.

Today's approach — training data + web search — is unreliable. Agents guess at APIs. They mix up versions. They hallucinate parameters that don't exist.

At Chonkie, we built **mandex** to fix this: a package manager for documentation that gives AI agents instant, version-pinned access to library docs.

Think of it as npm for documentation. Pull once, search locally forever. No rate limits, no latency, no API keys.

The registry already supports 100+ packages across Python, JavaScript, and Rust ecosystems — with 1,500+ version-matched doc sets. numpy alone has 38 versions of documentation, so your agent always gets the right API for the version you're using.

We support 10 AI agents out of the box: Claude Code, Cursor, GitHub Copilot, Codex, Windsurf, Cline, OpenClaw, Amp, Antigravity, and Gemini.

This is just the beginning. As the ecosystem of AI coding tools grows, the need for reliable, version-matched documentation access will only increase.

Open source and free: https://mandex.dev
