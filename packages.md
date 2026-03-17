# Target Packages

Libraries to prioritize for the initial mandex registry. Ordered by likely demand.

## Documentation Availability

| Package | Ecosystem | Markdown docs | Source | llms.txt | Notes |
|---------|-----------|--------------|--------|----------|-------|
| nextjs | npm | ✅ | `/docs` in main repo (MDX) | ✅ nextjs.org/docs/llms-full.txt | canary branch |
| react | npm | ✅ | separate repo: reactjs/react.dev | ❌ | |
| tailwindcss | npm | ✅ | tailwindlabs/tailwindcss.com (MDX) | ❌ | explicitly rejected llms.txt |
| typescript | npm | ✅ | microsoft/TypeScript-Website `/packages` | ❌ | complex monorepo |
| astro | npm | ✅ | separate repo: withastro/docs `/src/content/docs` | ❌ | Starlight |
| shadcn-ui | npm | ✅ | monorepo: `/apps/www/content/docs` (MDX) | ❌ | |
| vite | npm | ✅ | separate repo: vitejs/vite `/docs` | ❌ | |
| hono | npm | ✅ | honojs/hono `/docs` | ❌ | |
| drizzle | npm | ✅ | separate repo: drizzle-team/drizzle-orm-docs (MDX) | ❌ | |
| prisma | npm | ✅ | separate repo: prisma/web `/apps/docs` (MDX) | ❌ | |
| langchain | pip | ✅ | separate repo: langchain-ai/docs (MDX) | ✅ docs.langchain.com/llms.txt | |
| langgraph | pip | ✅ | in langchain-ai/langgraph `/docs` | ✅ | |
| openai | pip | ❌ | no markdown — docs on platform.openai.com | ❌ | HTML only |
| anthropic | pip | ❌ | no markdown — docs on docs.anthropic.com | ❌ | HTML only |
| pytorch | pip | ⚠️ | pytorch/docs — auto-generated HTML | ❌ | RST source, no clean markdown |
| transformers | pip | ✅ | in huggingface/transformers `/docs/source/en` | ❌ | large, build system |
| fastapi | pip | ✅ | in tiangolo/fastapi `/docs/en` | ❌ | pending PR |
| pydantic | pip | ✅ | separate website repo | ✅ docs.pydantic.dev/latest/llms.txt | |
| sqlalchemy | pip | ⚠️ | `/doc/build` — reStructuredText, not Markdown | ❌ | Sphinx/RST |
| supabase | npm/pip | ✅ | monorepo: supabase/supabase `/apps/docs` | ❌ | |
| cloudflare-workers | npm | ✅ | cloudflare/cloudflare-docs `/src/content/docs` (MDX) | ❌ | |
| ai-sdk | npm | ✅ | in vercel/ai `/docs` (MDX) | ✅ ai-sdk.dev/llms.txt | |
| axum | cargo | ✅ | inline + guides in tokio-rs/axum | ❌ | sparse guides |
| tokio | cargo | ✅ | separate website: tokio.rs | ❌ | |
| serde | cargo | ⚠️ | minimal — mostly API docs | ❌ | |

**Summary:** 20/25 have markdown available. 2 are HTML-only (openai, anthropic). 3 are partial/RST (pytorch, sqlalchemy, serde).

---

## Priority Order for v0.1 Seed

1. `nextjs` — MDX in main repo, llms.txt available
2. `fastapi` — clean markdown, popular Python API framework
3. `langchain` — markdown available, llms.txt, fast-moving
4. `pydantic` — separate docs repo, llms.txt available
5. `tailwindcss` — markdown available, v4 migration pain
6. `drizzle` — separate docs repo, newer ORM
7. `ai-sdk` — MDX in main repo, llms.txt available
8. `hono` — markdown in repo, popular for edge
9. `transformers` — large markdown docs in repo
10. `react` — separate react.dev repo

---

## Version Coverage

Which versions to index per package, prioritized by adoption.

| Package | Versions to index | Currently indexed | Notes |
|---------|------------------|-------------------|-------|
| nextjs | 13.x, 14.x, 15.x | 15.0.0 ✅ | 13→14 App Router introduction, 14→15 major changes |
| react | 18.x, 19.x | 19.0.0 ✅ | v18 hooks/Suspense, v19 Server Components |
| tailwindcss | 3.x, 4.x | 4.0.0 ✅ | v4 is a near-complete rewrite of v3 |
| pydantic | v1 (1.10.x), v2 (2.x) | 2.0.0 ✅ | v1→v2 was a complete rewrite, many projects still on v1 |
| langchain | 0.1.x, 0.2.x, 0.3.x | 0.3.0 ✅ | API changed significantly at each minor |
| fastapi | 0.100.x, 0.115.x | 0.115.0 ✅ | Relatively stable, latest is sufficient for now |
| pytorch | 2.0.x, 2.1.x, 2.3.x | 2.3.0 ✅ | 2.0 was torch.compile, 2.1+ widely in prod |
| transformers | 4.36.x, 4.40.x | 4.40.0 ✅ | Pipelines API stable, point releases fine |
| drizzle | 0.30.x, 0.36.x | 0.36.0 ✅ | Fast-moving, query API changed between versions |
| ai-sdk | 3.x, 4.x | 4.0.0 ✅ | v4 restructured the API significantly |
| hono | 4.x | 4.0.0 ✅ | Stable, latest sufficient |
| langgraph | 0.2.x, 0.3.x | 0.3.0 ✅ | Fast-moving agent API |

**Total target:** ~20 additional versions on top of the 12 currently indexed.

---

## Build Strategy

**Clone & build:** Most packages have docs in a GitHub repo we can clone and run `mx build ./docs` against.

**llms.txt available:** nextjs, langchain, langgraph, pydantic, ai-sdk — can use as a quick check/fallback.

**Needs custom work:**
- `openai` / `anthropic` — HTML only, need to scrape or wait for official markdown
- `pytorch` — RST source, would need conversion
- `sqlalchemy` — RST source, would need conversion
- `serde` — minimal docs, low priority
