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

## Build Strategy

**Clone & build:** Most packages have docs in a GitHub repo we can clone and run `mx build ./docs` against.

**llms.txt available:** nextjs, langchain, langgraph, pydantic, ai-sdk — can use as a quick check/fallback.

**Needs custom work:**
- `openai` / `anthropic` — HTML only, need to scrape or wait for official markdown
- `pytorch` — RST source, would need conversion
- `sqlalchemy` — RST source, would need conversion
- `serde` — minimal docs, low priority
