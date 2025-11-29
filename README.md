# crabdocs

A naive CLI tool for searching Rust crates on [crates.io](https://crates.io) and browsing documentation from [docs.rs](https://docs.rs).

## Installation

```console
$ cargo install --git https://github.com/0x6b/crabdocs
```

## Usage

```console
$ crabdocs --help
```

For detailed command documentation and examples, see [skills/browsing-rust-docs/SKILL.md](skills/browsing-rust-docs/SKILL.md).

## Claude Code Skill

This repository includes a [Claude Code skill](https://docs.anthropic.com/en/docs/agents-and-tools/claude-code/skills) for integration with Claude Code. You can install it with:

```console
$ ln -s /path/to/crabdocs/skills/browsing-rust-docs ~/.claude/skills/
```

See [Skills documentation](https://docs.anthropic.com/en/docs/agents-and-tools/claude-code/skills) for more details.

## Acknowledgments

[nuskey8/docs-rs-mcp](https://github.com/nuskey8/docs-rs-mcp) for inspiration.

## License

MIT. See [LICENSE](LICENSE) for details.
