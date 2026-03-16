# Managing packages

## List installed packages

```bash
mx list
```

Shows all installed packages with their version, entry count, and size:

```
  pytorch@2.3.0  (2847 entries, 8.2 MB)
  nextjs@14.2.0  (891 entries, 4.7 MB)
  fastapi@0.115.0  (287 entries, 1.4 MB)
```

## Package info

```bash
mx info pytorch
```

Shows detailed information about an installed package:

```
pytorch@2.3.0
  Entries:  2847
  Size:     8.2 MB
  Path:     /Users/you/.mandex/cache/pytorch/2.3.0.db
```

## Remove a package

Remove all versions of a package:

```bash
mx remove pytorch
```

Remove a specific version:

```bash
mx remove pytorch --version 2.3.0
```

## Storage location

All packages are stored in `~/.mandex/cache/`. Each package gets its own directory, with one `.db` file per version:

```
~/.mandex/cache/
├── pytorch/
│   ├── 2.2.0.db
│   └── 2.3.0.db
├── nextjs/
│   └── 14.2.0.db
└── fastapi/
    └── 0.115.0.db
```

To see the total disk usage:

```bash
du -sh ~/.mandex/cache/
```
