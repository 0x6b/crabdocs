---
name: browsing-rust-docs
description: Searches crates.io for Rust crates and retrieves documentation from docs.rs. Use when looking up Rust crate documentation, finding structs/traits/functions/enums in a crate, or exploring what a crate provides.
---

# Rust Documentation Browser

CLI tool for searching crates.io and browsing docs.rs documentation.

## Prerequisites

Requires `crabdocs` binary in PATH. If you don't have it, run

```bash
cargo install --git https://github.com/0x6b/crabdocs
```

## Commands

### search - Find crates on crates.io

```bash
crabdocs search <QUERY> [-p PER_PAGE] [-s SORT] [--page N]
```

- `-p, --per-page`: Results per page (default: 10, max: 100)
- `-s, --sort`: relevance | downloads | recent-downloads | recent-updates | new
- `--page`: Page number (1-indexed)

```bash
crabdocs search "async runtime" --sort downloads -p 5
```

### show-readme - Get crate overview

```bash
crabdocs show-readme <CRATE> [-v VERSION]
```

```bash
crabdocs show-readme tokio
crabdocs show-readme serde -v 1.0.100
```

### show-items-summary - List item type counts

```bash
crabdocs show-items-summary <CRATE> [-v VERSION]
```

```bash
crabdocs show-items-summary tokio
# Output: struct: 163, fn: 70, trait: 17, enum: 16, ...
```

### search-items-in - Find items in a crate

```bash
crabdocs search-items-in <CRATE> <QUERY> [-v VERSION] [-t TYPE]
```

- `-t, --item-type`: struct | trait | fn | enum | type | const | static | macro | union | module

Returns item names with ready-to-use `show-item-doc` commands:

```bash
crabdocs search-items-in tokio spawn
# - task::spawn (fn)
#   `crabdocs show-item-doc tokio fn tokio::task::spawn`
# - task::spawn_blocking (fn)
#   `crabdocs show-item-doc tokio fn tokio::task::spawn_blocking`

crabdocs search-items-in serde "" -t trait  # List all traits
```

### show-item-doc - Get item documentation

```bash
crabdocs show-item-doc <CRATE> <TYPE> <PATH> [-v VERSION]
```

Types: module, struct, enum, trait, fn, type, const, static, macro, union
Path: should be fully-qualified name i.e. not `task::spawn` but `tokio::task::spawn`

```bash
crabdocs show-item-doc tokio struct tokio::sync::Mutex
crabdocs show-item-doc tokio fn tokio::task::spawn
crabdocs show-item-doc serde trait serde::Serialize
```

## Typical Workflow

1. Search for crates: `crabdocs search "json parser"`
2. Read crate overview: `crabdocs show-readme serde_json`
3. See available items: `crabdocs show-items-summary serde_json`
4. Find specific items: `crabdocs search-items-in serde_json value`
5. Read item docs: `crabdocs show-item-doc serde_json enum serde_json::Value`
