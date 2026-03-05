# engram

> Fast, portable knowledge base CLI with semantic search.

engram indexes your plain text and markdown files into a local [sqlite-vec](https://github.com/asg017/sqlite-vec) database and lets you search them **by meaning**, not just keywords.

```
$ engram search "how do I handle feed loss in the market maker"
 1. notes/trading/feed-loss-handling.md (score: 0.923)
    Cancel all open orders immediately when the price feed drops. Stale quotes...

 2. notes/trading/neverlose-design.md (score: 0.871)
    Time-based stale detection: >10s no midpoint update triggers feed lost state...
```

## Design principles

- **Your files are never modified.** engram reads them; it never writes to them.
- **The index is a sidecar.** `~/.engram/index.db` is a derived artifact — delete it and rebuild anytime.
- **Local-first, private by default.** Works with a local [Ollama](https://ollama.com) instance (`nomic-embed-text`) — no API key, no data leaving your machine. Falls back to OpenAI-compatible APIs when Ollama isn't available.
- **One binary, no runtime deps.** SQLite is bundled. Ships as a single static binary for all major platforms.
- **Plain files stay plain.** Works alongside `grep`, `ripgrep`, `fzf`, git, Obsidian, or anything else that reads markdown.

## Installation

```bash
# From source (requires Rust 1.75+)
cargo install --git https://github.com/pureclaw/engram

# Pre-built binaries (coming soon)
# https://github.com/pureclaw/engram/releases
```

## Quick start

```bash
# 1. Initialize
engram init

# 2. Index your notes
engram add ~/notes --recursive

# 3. Search
engram search "machine learning approaches for time series regime detection"

# 4. Check status
engram status
```

## Commands

| Command | Description |
|---|---|
| `engram init` | Initialize the index |
| `engram add <paths...>` | Index files or directories |
| `engram search <query>` | Search by meaning |
| `engram remove <paths...>` | Remove files from index |
| `engram rebuild` | Rebuild index from scratch |
| `engram status` | Show index stats |

## Embedding providers

engram auto-detects the best available provider at `init` time and records the choice in the index. To switch providers, run `engram rebuild`.

| Provider | How to use | Privacy |
|---|---|---|
| `ollama/nomic-embed-text` | Run `ollama pull nomic-embed-text` | ✅ Fully local |
| `openai/text-embedding-3-small` | Set `OPENAI_API_KEY` | Cloud API |
| OpenRouter | Set `OPENROUTER_API_KEY` | Cloud API |

**Default:** Ollama (local) if available, otherwise OpenAI-compatible.

## Supported file types

`.md`, `.txt`, `.rst`, `.org`, `.adoc`

## Platforms

| Platform | Status |
|---|---|
| Linux x86_64 | ✅ |
| Linux aarch64 | ✅ |
| macOS aarch64 (Apple Silicon) | ✅ |
| macOS x86_64 | ✅ |
| Windows x86_64 | planned |

## Contributing

PRs welcome. See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

MIT
