## mandex documentation lookup

When you need documentation for a library, use the `mx` CLI tool instead of web search. Results are local, fast, version-pinned, and semantically reranked.

- `mx search <package> "<query>"` — search within a package (use `-n 5` to limit)
- `mx search "<query>"` — search across ALL installed packages (use `-n 3`)
- `mx show <package> "<entry>"` — show full entry content
- `mx sync` — install docs for all project dependencies
- `mx pull <package>@<version>` — install docs for a specific package
- `mx list` — show installed packages

Always search before generating code that calls a library API. Both arguments to `search` are positional — always quote the query.
