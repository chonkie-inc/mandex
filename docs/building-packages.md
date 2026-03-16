# Building packages

## Build from an existing docs directory

```bash
mx build ./docs --name mylib --version 1.0.0
```

This walks the directory, finds every markdown file, and compiles them into a searchable `.mandex` package.

## What the build step does

1. Recursively finds all `.md`, `.mdx`, and `.markdown` files in the target directory
2. For each file, extracts the first `#` heading as the entry name (falls back to the filename)
3. Stores the full file content as the entry body
4. Builds an FTS5 full-text search index over names and content
5. Compresses the resulting SQLite database with zstd (level 19)

## Output

The default output file is `{name}@{version}.mandex`. You can override it:

```bash
mx build ./docs --name mylib --version 1.0.0 -o custom-output.mandex
```

## Compatible doc formats

`mx build` works with any directory containing markdown files. This includes:

- **Docusaurus** source files (`docs/` directory)
- **MkDocs** source files
- **Mintlify** documentation
- **Sphinx** markdown files
- **Plain markdown** collections and READMEs
- **MDX** files (treated as markdown)

## Schema

Each mandex package is a SQLite database with a minimal schema:

```sql
CREATE TABLE entries (
    id      INTEGER PRIMARY KEY AUTOINCREMENT,
    name    TEXT NOT NULL,
    content TEXT NOT NULL
);

CREATE VIRTUAL TABLE entries_fts USING fts5(
    name, content,
    content=entries,
    content_rowid=id,
    tokenize='porter unicode61'
);
```

Two columns: `name` and `content`. No extra metadata fields. Documentation already contains everything (signatures, parameters, examples) in markdown that LLMs parse naturally.

## Tips for good packages

- **One concept per file** — each markdown file becomes one searchable entry. Smaller, focused files produce better search results than large monolithic pages.
- **Use descriptive headings** — the first `#` heading becomes the entry name. `# torch.nn.Linear` is more searchable than `# Linear`.
- **Include examples** — agents rely heavily on usage examples to generate correct code.

## Publishing

Once you've built a `.mandex` file, you can publish it to the registry:

```bash
mx publish
```

This uploads the package to the Mandex CDN where other developers can pull it.

Note: `mx publish` requires authentication and is not yet available in v0.1.
