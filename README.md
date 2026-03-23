<p align="center">
  <img src="website/public/logo-black.png" height="60" alt="mandex">
</p>

<p align="center">
  <b>Documentation packages for AI agents.</b><br>
  <i>The fastest way for agents to read the docs.</i>
</p>

<p align="center">
  <a href="https://crates.io/crates/mandex"><img src="https://img.shields.io/crates/v/mandex.svg" alt="crates.io"></a>
  <a href="https://github.com/chonkie-inc/mandex/releases"><img src="https://img.shields.io/github/v/release/chonkie-inc/mandex" alt="release"></a>
  <a href="https://mandex.dev"><img src="https://img.shields.io/badge/registry-mandex.dev-059669" alt="registry"></a>
  <a href="https://github.com/chonkie-inc/mandex/blob/main/LICENSE"><img src="https://img.shields.io/github/license/chonkie-inc/mandex" alt="license"></a>
</p>

---

mandex distributes library documentation like packages — versioned, cached locally, searchable offline in **40ms**. No API keys, no rate limits, no network dependency.

Your AI agent gets the right docs for the right version, instantly.

## Install

```bash
curl -fsSL https://mandex.dev/install.sh | sh
```

Or with Cargo:

```bash
cargo install mandex
```

## Quick start

```bash
# Pull docs for a library
mx pull fastapi

# Search across installed docs
mx search fastapi "rate limiting"

# Show a specific entry
mx show fastapi "Rate Limiting"

# Auto-sync docs for all project dependencies
mx sync
```

## Why mandex?

| | Cloud docs (MCP/API) | mandex |
|---|---|---|
| **Latency** | 300–500ms per query | **40ms** (local FTS5 + neural reranking) |
| **Offline** | ❌ | ✅ |
| **Rate limits** | Per-provider quotas | Unlimited |
| **Versioning** | Latest only or manual | Pinned to exact version |
| **Setup** | API keys, tokens, config | `mx pull <package>` |

## How it works

1. **Pull** — download a compressed documentation index (SQLite + FTS5) from the registry
2. **Search** — full-text search with BM25 ranking, automatically reranked by a local neural cross-encoder for semantic relevance
3. **Show** — retrieve the full documentation entry for an exact match

Documentation is stored as local SQLite databases in `~/.mandex/cache/`. A per-project `.mandex/index.db` merges all project dependencies into a single index for fastest search.

## Agent integrations

mandex works with any AI coding assistant that can run shell commands:

```bash
# Set up integrations (Claude Code, Cursor, Windsurf, Codex)
mx init

# Or just use mx directly in your agent's tool calls
mx search nextjs "server actions"
```

`mx init` installs a [skill](https://docs.anthropic.com/en/docs/claude-code) for Claude Code, cursor rules for Cursor, and agent instructions for Codex — so your agent automatically searches mandex before writing code.

## Commands

| Command | Description |
|---------|-------------|
| `mx search <package> "<query>"` | Search within a package |
| `mx search "<query>"` | Search across all installed packages |
| `mx show <package> "<entry>"` | Show full entry content |
| `mx pull <package>[@version]` | Install docs for a package |
| `mx sync` | Auto-detect and install docs for project dependencies |
| `mx list` | Show installed packages |
| `mx info <package>` | Show package details |
| `mx remove <package>` | Remove a package |
| `mx init` | Set up AI assistant integrations |
| `mx build <dir> --name <n> --version <v>` | Build a `.mandex` package from markdown |

## Registry

**55 packages** across npm, pip, and cargo — [browse the full registry](https://mandex.dev/registry).

<details>
<summary>View all packages</summary>

**npm:** ai-sdk, angular, astro, better-auth, claude-code, clerk, drizzle-orm, express, fumadocs, hono, langchain-js, mongodb, nest, nextjs, nuxt, openclaw, opencode, playwright, prisma, react, shadcn-ui, supabase, svelte, tailwindcss, tanstack-query, trpc, turborepo, vite, vitest, vue, zod, zustand

**pip:** celery, django, fastapi, flask, httpx, instructor, langchain, langgraph, llama-index, numpy, pandas, pydantic, pytest, requests, scipy, sqlalchemy, streamlit, transformers, uvicorn

**cargo:** axum, mandex, serde, tokio

</details>

## Performance

- **40ms** average search latency (local FTS5 + reranking)
- **70ms** project-wide search across all dependencies (merged index)
- **5ms** tokenizer load (tokie `.tkz` binary format)
- Pre-optimized ONNX reranker with mmap + multi-threaded inference

## Building packages

Have documentation for a library? Build and publish it:

```bash
# Build from a directory of markdown files
mx build ./docs --name my-lib --version 1.0.0

# Output: my-lib@1.0.0.mandex (compressed SQLite + FTS5 index)
```

mandex accepts any directory of `.md` / `.mdx` files — Docusaurus, MkDocs, Mintlify, Starlight, or plain markdown all work.

## Contributing

Contributions welcome. See the [issues](https://github.com/chonkie-inc/mandex/issues) for open tasks.

## License

MIT
