---
name: browsing-rust-docs
description: Searches crates.io for Rust crates and retrieves documentation from docs.rs. Use when looking up Rust crate documentation, finding structs/traits/functions/enums in a crate, or exploring what a crate provides.
---

# Rust Documentation Browser

CLI tool for searching crates.io and browsing docs.rs documentation.

## Prerequisites

Requires `docs-rs-cli` binary in PATH. If you don't have it, run

```bash
cargo install --git https://github.com/0x6b/docs-rs-cli
```

## Commands

### search - Find crates on crates.io

```bash
docs-rs-cli search <QUERY> [-p PER_PAGE] [-s SORT] [--page N]
```

- `-p, --per-page`: Results per page (default: 10, max: 100)
- `-s, --sort`: relevance | downloads | recent-downloads | recent-updates | new
- `--page`: Page number (1-indexed)

```bash
docs-rs-cli search "async runtime" --sort downloads -p 5
```

### show-readme - Get crate overview

```bash
docs-rs-cli show-readme <CRATE> [-v VERSION]
```

```bash
docs-rs-cli show-readme tokio
docs-rs-cli show-readme serde -v 1.0.100
```

### show-items-summary - List item type counts

```bash
docs-rs-cli show-items-summary <CRATE> [-v VERSION]
```

```bash
docs-rs-cli show-items-summary tokio
# Output: struct: 163, fn: 70, trait: 17, enum: 16, ...
```

### search-items-in - Find items in a crate

```bash
docs-rs-cli search-items-in <CRATE> <QUERY> [-v VERSION] [-t TYPE]
```

- `-t, --item-type`: struct | trait | fn | enum | type | const | static | macro | union | module

Returns item names with ready-to-use `show-item-doc` commands:

```bash
docs-rs-cli search-items-in tokio spawn
# - task::spawn (fn)
#   `docs-rs-cli show-item-doc tokio fn tokio::task::spawn`
# - task::spawn_blocking (fn)
#   `docs-rs-cli show-item-doc tokio fn tokio::task::spawn_blocking`

docs-rs-cli search-items-in serde "" -t trait  # List all traits
```

### show-item-doc - Get item documentation

```bash
docs-rs-cli show-item-doc <CRATE> <TYPE> <PATH> [-v VERSION]
```

Types: module, struct, enum, trait, fn, type, const, static, macro, union

```bash
docs-rs-cli show-item-doc tokio struct tokio::sync::Mutex
docs-rs-cli show-item-doc tokio fn tokio::task::spawn
docs-rs-cli show-item-doc serde trait serde::Serialize
```

## Typical Workflow

1. Search for crates: `docs-rs-cli search "json parser"`
2. Read crate overview: `docs-rs-cli show-readme serde_json`
3. See available items: `docs-rs-cli show-items-summary serde_json`
4. Find specific items: `docs-rs-cli search-items-in serde_json value`
5. Read item docs: `docs-rs-cli show-item-doc serde_json enum serde_json::Value`
