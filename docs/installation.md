# Installation

## Quick install

```bash
curl -fsSL https://mandex.dev/install.sh | sh
```

This downloads the latest `mx` binary and places it in `/usr/local/bin/`.

## From crates.io

If you have Rust installed:

```bash
cargo install mandex
```

This installs the binary as `mx`.

## From source

```bash
git clone https://github.com/bhavnicksm/mandex.git
cd mandex
cargo build --release
cp target/release/mx /usr/local/bin/
```

## Verify installation

```bash
mx --version
mx --help
```

## Where packages are stored

Mandex stores downloaded documentation packages in `~/.mandex/cache/`. Each package is a SQLite database:

```
~/.mandex/
└── cache/
    ├── pytorch/
    │   └── 2.3.0.db
    ├── nextjs/
    │   └── 14.2.0.db
    └── ...
```

Packages are shared across all your projects. If two projects use the same library version, the package is downloaded once.
