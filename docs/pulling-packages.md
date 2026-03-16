# Pulling packages

## Download a package

```bash
mx pull pytorch@2.3.0
```

This downloads the `pytorch` documentation package at version `2.3.0` from the Mandex CDN, decompresses it, and stores it locally.

## Latest version

If you omit the version, `mx pull` fetches the latest available version:

```bash
mx pull pytorch
```

## What happens during pull

1. The CLI fetches the compressed `.mandex` file from `cdn.mandex.dev`
2. The file is decompressed (zstd) into a SQLite database
3. The database is stored at `~/.mandex/cache/<name>/<version>.db`
4. The FTS5 search index is ready to query immediately

## Already installed

If the package is already installed, `mx pull` will tell you:

```bash
$ mx pull pytorch@2.3.0
pytorch@2.3.0 is already installed
```

## Multiple versions

You can have multiple versions of the same package installed simultaneously:

```bash
mx pull nextjs@14.0.0
mx pull nextjs@16.0.0
```

When searching, mandex uses the latest installed version by default.
