# engram

> Give your agents a brain.

Semantic memory for AI agents. Index your knowledge base, search by meaning — not keywords. Single binary, local models, no server required.

AI agents are only as good as what they can recall. **engram** gives agents persistent, searchable memory over any collection of plain text and markdown files — without a database server, cloud service, or complex infrastructure.

```
$ engram search "why does my container keep restarting"
 1. docs/ops/pod-oom-crash-loop.md        (dist: 15.2)
    ...set resource requests and limits; the kernel OOMKills the process when...

 2. docs/ops/k8s-node-pressure.md         (dist: 17.1)
    ...node memory pressure causes the kubelet to evict pods without limits set...
```

Give an agent access to `engram search` and it can retrieve the right context from thousands of documents in milliseconds — without stuffing everything into the prompt.

## Keyword search vs. semantic search

Say you have a runbook note about a Kubernetes issue:

```markdown
# pod-oom-crash-loop.md

The container is being OOMKilled by the kubelet. Resource limits are not
set, so the pod gets scheduled on nodes without enough headroom and the
kernel terminates the process when memory pressure spikes.

Fix: add resource requests and limits to the deployment manifest. Set
requests.memory to the p50 observed usage and limits.memory to the p99.
Use VPA in recommendation mode to calibrate initial values before
committing to hard limits.
```

Grep works if you remember the right words:

```
$ grep -rl "OOMKilled\|resource limits\|kubelet" docs/
docs/ops/pod-oom-crash-loop.md
```

But natural language draws a blank:

```
$ grep -rl "container keeps dying" docs/
(no matches)
```

engram finds it — the meaning matches even though none of the words do:

```
$ engram search "container keeps dying after a few minutes" --limit 5
 1. docs/ops/pod-oom-crash-loop.md         (dist: 15.2)
    ...the kernel terminates the process when memory pressure spikes...

 2. docs/ops/k8s-node-pressure.md          (dist: 16.8)
    ...kubelet evicts pods that exceed their memory footprint...

 3. docs/ops/docker-healthcheck-tuning.md  (dist: 18.1)
    ...increase the start_period before the health check begins firing...

 4. docs/ops/cgroup-limits.md              (dist: 18.4)
    ...cgroup v2 memory.max enforcement kills processes that exceed the limit...

 5. docs/architecture/service-sizing.md    (dist: 19.2)
    ...right-sizing services reduces both cost and crash-loop frequency...
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
# Index your notes — index is created automatically on first run
engram add ~/notes

# Search by meaning
engram search "how do I configure connection pooling for high throughput"

# Check what's indexed
engram status
```

## Using engram with AI agents

engram is designed to be called as a tool from agent systems (OpenClaw, LangChain, custom loops, etc.):

```bash
# Agent retrieves relevant context before answering
engram search "$QUERY" --limit 5

# Agent indexes a new document after capturing knowledge
engram add ~/notes/new-runbook.md
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

**Options for `add`:**

| Flag | Description |
|---|---|
| `--no-progress` | Plain text output instead of progress bar (useful in scripts/CI) |
| `--recursive` / `-r` | Recursively index directories (default: on) |

## Embedding providers

engram auto-detects the best available provider at init time and records the choice in the index. To switch providers, run `engram rebuild`.

| Provider | How to enable | Privacy |
|---|---|---|
| `ollama/nomic-embed-text` | `ollama pull nomic-embed-text` | ✅ Fully local |
| `openai/text-embedding-3-small` | Set `OPENAI_API_KEY` | Cloud API |
| OpenRouter | Set `OPENROUTER_API_KEY` | Cloud API |

**Default:** Ollama (local) when available, otherwise OpenAI-compatible.

## Supported file types

`.md` `.txt` `.rst` `.org` `.adoc`

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
