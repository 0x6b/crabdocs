# docs-rs-cli

A naive CLI tool for searching Rust crates on [crates.io](https://crates.io) and browsing documentation from [docs.rs](https://docs.rs).

## Installation

```console
$ cargo install --git https://github.com/0x6b/docs-rs-cli
```

## Usage

```
Usage: docs-rs-cli <COMMAND>

Commands:
  search              Search for Rust crates by keywords on crates.io
  search-items-in     Search for items within a crate's documentation
  show-readme         Show README/overview content of the specified crate
  show-items-summary  Show summary of item types in a crate
  show-item-doc       Show documentation of a specific item
  help                Print this message or the help of the given subcommand(s)
```

### Examples

```console
# Search for crates
$ docs-rs-cli search "async runtime" --sort downloads -p 5

# Show crate README
$ docs-rs-cli show-readme tokio

# List item types in a crate
$ docs-rs-cli show-items-summary tokio

# Search for items in a crate
$ docs-rs-cli search-items-in tokio spawn

# Show documentation for a specific item
$ docs-rs-cli show-item-doc tokio struct tokio::sync::Mutex
```

## Claude Code Skill

This repository includes a [Claude Code skill](https://docs.anthropic.com/en/docs/agents-and-tools/claude-code/skills) for integration with Claude Code. You can install it with:

```console
$ ln -s /path/to/docs-rs-cli/skills/browsing-rust-docs ~/.claude/skills/
```

See [Skills documentation](https://docs.anthropic.com/en/docs/agents-and-tools/claude-code/skills) for more details.

## Acknowledgments

[nuskey8/docs-rs-mcp](https://github.com/nuskey8/docs-rs-mcp) for inspiration.

## License

MIT. See [LICENSE](LICENSE) for details.
