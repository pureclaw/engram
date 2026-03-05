# engram

> Give your agents a brain.

Semantic memory for AI agents. Index your knowledge base, search by meaning — not keywords. Single binary, no server required.

AI agents are only as good as what they can recall. **engram** gives agents persistent, searchable memory over any collection of plain text and markdown files — without a database server, cloud service, or complex infrastructure.

```
$ engram search "how do I handle feed loss in the market maker"
 1. notes/trading/feed-loss-handling.md (score: 0.923)
    Cancel all open orders immediately when the price feed drops. Stale quotes...

 2. notes/trading/neverlose-design.md (score: 0.871)
    Time-based stale detection: >10s no midpoint update triggers feed lost state...
```

Give an agent access to `engram search` and it can retrieve the right context from thousands of documents in milliseconds — without stuffing everything into the prompt.

## Why engram for agents?

| Problem | engram's answer |
|---|---|
| Context windows are finite | Retrieve only what's relevant, not everything |
| Agents forget between sessions | Persistent index survives restarts |
| RAG needs a database server | sqlite-vec runs as a single file, zero infra |
| Cloud embedding APIs leak data | Local Ollama embedding, nothing leaves the machine |
| Binary format lock-in | Plain files stay plain — any tool can still read them |

## Design principles

- **Your files are never modified.** engram reads them; it never writes to them.
- **The index is a sidecar.** `~/.engram/index.db` is a derived artifact — delete it and rebuild anytime. Your markdown is always the source of truth.
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
# Index your knowledge base — index is created automatically on first run
engram add ~/notes --recursive

# Search by meaning
engram search "machine learning approaches for time series regime detection"

# Check status
engram status
```

## Using engram with AI agents

engram is designed to be called as a tool from agent systems (OpenClaw, LangChain, custom loops, etc.):

```bash
# Agent calls this to retrieve context before answering
engram search "$(QUERY)" --limit 5

# Agent indexes new knowledge after a session
engram add ~/notes/new-discovery.md
```

The output is plain text — easy to parse, pipe, or inject directly into a prompt. No SDK required.

## Commands

| Command | Description |
|---|---|
| `engram add <paths...>` | Index files or directories (creates index on first run) |
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

**Default:** Ollama (local) when available, otherwise OpenAI-compatible.

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
