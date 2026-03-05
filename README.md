# engram

> Give your agents a brain.

Semantic memory for AI agents. Index your knowledge base, search by meaning — not keywords. Single binary, no server required.

AI agents are only as good as what they can recall. **engram** gives agents persistent, searchable memory over any collection of plain text and markdown files — without a database server, cloud service, or complex infrastructure.

```
$ engram search "what should the system do when exchange connectivity drops"
 1. notes/trading/feed-loss-handling.md    (dist: 17.4)
    ...cancel all open orders immediately when feed loss is detected...

 2. notes/trading/go-live-readiness.md     (dist: 18.4)
    ...live feed loss handling must be implemented before any live trading...
```

Give an agent access to `engram search` and it can retrieve the right context from thousands of documents in milliseconds — without stuffing everything into the prompt.

## Keyword search vs. semantic search

Say you have a note about your trading system's market data handling:

```markdown
# feed-loss-handling.md

The current stale-check only fires **before any data has ever arrived**.
If the Coinbase WebSocket feed was delivering data and then disconnects,
the market maker **keeps placing orders against a stale price**.

This is a livelock: open limit orders sit on the exchange while the MM
loops, generates quotes from the last known price (which could be 60s or
more stale), and continues placing. If BTC moves 0.5% during an outage,
we're quoting the wrong price into a gap.

Fix: replace the point-in-time check with a time-since-last-update check.
When feed loss is detected, cancel all open orders immediately.
```

Grep requires you to know which words are in the document:

```
$ grep -rl "WebSocket\|stale price\|livelock" notes/
notes/feed-loss-handling.md
```

Miss the exact terms? Miss the document:

```
$ grep -rl "pulling quotes when exchange connectivity drops" notes/
(no matches)
```

engram finds it anyway — the meaning matches even though none of the words do:

```
$ engram search "what should the system do when exchange connectivity drops" --limit 5
 1. notes/feed-loss-handling.md          (dist: 17.4)
    ...cancel all open orders immediately when feed loss is detected...

 2. notes/go-live-readiness.md           (dist: 18.4)
    ...live feed loss handling must be implemented before any live trading...

 3. notes/pre-live-implementation-plan.md (dist: 18.9)
    ...two items block live trading: position limits and feed loss handling...

 4. notes/testing-methodologies.md       (dist: 19.1)
    ...simulate exchange disconnection during active order management...

 5. notes/market-making-design.md        (dist: 19.8)
    ...order cancellation logic on connectivity events...
```

Lower distance = stronger match. Results are ranked by semantic similarity across your entire knowledge base — without a server, without a cloud API, and without knowing ahead of time which words your documents use.

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
