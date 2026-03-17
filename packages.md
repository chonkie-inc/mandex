# Target Packages

Libraries to prioritize for the initial mandex registry. Ordered by likely demand — rapidly-changing APIs where stale training data causes the most pain.

## Frontend / JavaScript

| Package | Ecosystem | Docs URL | Notes |
|---------|-----------|----------|-------|
| nextjs | npm (`next`) | https://nextjs.org/docs | App Router API changes every major version |
| react | npm (`react`) | https://react.dev/reference | Hooks, Server Components |
| tailwindcss | npm (`tailwindcss`) | https://tailwindcss.com/docs | v4 broke a lot of v3 patterns |
| typescript | npm (`typescript`) | https://www.typescriptlang.org/docs | |
| astro | npm (`astro`) | https://docs.astro.build | |
| shadcn-ui | npm (`shadcn/ui`) | https://ui.shadcn.com/docs | Component library, frequently updated |
| vite | npm (`vite`) | https://vite.dev/guide | |

## Backend / JavaScript

| Package | Ecosystem | Docs URL | Notes |
|---------|-----------|----------|-------|
| hono | npm (`hono`) | https://hono.dev/docs | Popular for Cloudflare Workers |
| drizzle | npm (`drizzle-orm`) | https://orm.drizzle.team/docs | Newer ORM, lots of breaking changes |
| prisma | npm (`prisma`) | https://www.prisma.io/docs | |

## AI / ML (Python)

| Package | Ecosystem | Docs URL | Notes |
|---------|-----------|----------|-------|
| langchain | pip (`langchain`) | https://python.langchain.com/docs | Extremely fast-moving API |
| langgraph | pip (`langgraph`) | https://langchain-ai.github.io/langgraph | Newer, agents/graphs |
| openai | pip (`openai`) | https://platform.openai.com/docs | SDK changes frequently |
| anthropic | pip (`anthropic`) | https://docs.anthropic.com | |
| pytorch | pip (`torch`) | https://pytorch.org/docs | Large, deep API surface |
| transformers | pip (`transformers`) | https://huggingface.co/docs/transformers | |

## Python Backend

| Package | Ecosystem | Docs URL | Notes |
|---------|-----------|----------|-------|
| fastapi | pip (`fastapi`) | https://fastapi.tiangolo.com | |
| pydantic | pip (`pydantic`) | https://docs.pydantic.dev | v2 broke v1 heavily |
| sqlalchemy | pip (`sqlalchemy`) | https://docs.sqlalchemy.org | |

## Cloud / Infra

| Package | Ecosystem | Docs URL | Notes |
|---------|-----------|----------|-------|
| supabase | npm/pip (`supabase`) | https://supabase.com/docs | Auth, DB, Storage APIs |
| cloudflare-workers | npm (`@cloudflare/workers-types`) | https://developers.cloudflare.com/workers | |
| ai-sdk | npm (`ai`) | https://sdk.vercel.ai/docs | Vercel AI SDK, fast-moving |

## Rust

| Package | Ecosystem | Docs URL | Notes |
|---------|-----------|----------|-------|
| axum | cargo (`axum`) | https://docs.rs/axum | Most popular Rust web framework |
| tokio | cargo (`tokio`) | https://tokio.rs/tokio/tutorial | Async runtime |
| serde | cargo (`serde`) | https://serde.rs | |

---

## Priority Order for v0.1 Seed

These 10 should be built first — highest pain / most queries:

1. `nextjs` — changes every major version, huge user base
2. `react` — Server Components changed a lot
3. `tailwindcss` — v4 migration pain
4. `fastapi` — most popular Python API framework
5. `langchain` — fastest-moving API in AI ecosystem
6. `pydantic` — v2 broke everything
7. `drizzle` — newer, less training data
8. `hono` — popular for edge/workers
9. `ai-sdk` — Vercel AI SDK, agent-heavy usage
10. `supabase` — auth + db API used in most full-stack demos
