# 🧠 Cortex

> The universal AI memory layer. Your codebase's long-term memory.

Cortex runs on your machine, reads your code, and builds a **living knowledge graph** of your entire project. Then it feeds that context into Claude, ChatGPT, Cursor, Copilot — any AI tool you use.

**No cloud. No API keys. No lock-in.** Your data never leaves your laptop.

---

## What It Does

| Feature | Command | What Happens |
|---------|---------|--------------|
| **Ingest** | `cortex ingest --path .` | Reads Git history, parses code, extracts decisions |
| **Watch** | `cortex watch --path .` | Monitors files live, updates the graph in real-time |
| **Ask** | `cortex ask "why did we use redis?"` | Queries the knowledge graph with natural language |
| **Search** | `cortex search "payment retry logic"` | Semantic vector search via local embeddings |
| **Recall** | `cortex recall "Tuesday 3pm"` | Reconstructs your mental state at any point in time |
| **Focus** | `cortex focus` | Shows your current working context |
| **Debt** | `cortex debt` | Surfaces abandoned work and stale decisions |
| **Inject** | `Ctrl+Shift+Space` in Claude/ChatGPT | Auto-injects project context into AI prompts |

---

## Demo

```bash
# 1. Point at any project
$ cortex ingest --path ./my-app
→ Found 2,473 entities, 8,921 relations

# 2. Ask questions
$ cortex ask "what structs are in the database module?"
→ Found 14 Structs:
→   User        src/db/models.rs
→   Session     src/db/models.rs
→   ...

# 3. Search semantically
$ cortex search "authentication retry logic"
→ Score  Kind       Name
→ 0.847  Function   handle_auth_failure
→ 0.812  Function   refresh_token_silently

# 4. Recall your mental state
$ cortex recall "yesterday 3pm"
→ 🧠 MENTAL STATE RECONSTRUCTION
→ 📍 Wednesday, July 22 at 3:00 PM
→ 🎯 Active file: src/payment/gateway.rs
→ 💡 Decisions nearby: "We chose Stripe over PayPal..."
→ ❓ Questions you asked: "why does webhook timeout?"

# 5. Check cognitive debt
$ cortex debt
→ 🔥 Open refactor: src/payment/gateway.rs (14 edits today, no commit)
→ 💡 Stale decision: "Migrate to async/await" (12 days old)
```

---

## Installation

### macOS / Linux

```bash
curl -sSL https://raw.githubusercontent.com/yourusername/cortex/main/install.sh | bash
```

### From Source

```bash
git clone https://github.com/yourusername/cortex.git
cd cortex
cargo build --release
# Binary: ./target/release/cortex
```

### VS Code Extension

```bash
cd cortex-vscode
npm install
npx tsc
# Then in VS Code: Run & Debug → "Run Extension"
```

### Browser Extension

1. Open Chrome → `chrome://extensions/`
2. Enable **Developer mode**
3. **Load unpacked** → select `cortex-browser/`
4. Go to [claude.ai](https://claude.ai) or [chatgpt.com](https://chatgpt.com)
5. Press `Ctrl+Shift+Space` to inject context

---

## Requirements

- **Rust** (1.75+) for building
- **SQLite** (bundled, no setup)
- **Ollama** (optional, for semantic search)
  ```bash
  ollama pull nomic-embed-text
  ollama serve
  ```

---

## Architecture

```
┌─────────────────────────────────────────┐
│         Your Code (Git + Filesystem)    │
└─────────────────┬───────────────────────┘
                  │
┌─────────────────▼───────────────────────┐
│           Cortex Core (Rust)            │
│  • Git Parser      • Tree-sitter Parser │
│  • Decision NLP    • Event Logger       │
│  • Temporal Engine • Working Memory     │
└─────────────────┬───────────────────────┘
                  │
    ┌─────────────┼─────────────┐
    ▼             ▼             ▼
┌───────┐    ┌───────┐    ┌─────────┐
│ SQLite │    │ Ollama │    │  API    │
│ Graph  │    │Embeds  │    │ :8787   │
└────────┘    └────────┘    └─────────┘
                                │
                    ┌───────────┼───────────┐
                    ▼           ▼           ▼
                VS Code     Browser      Terminal
                Extension   Extension    (cortex CLI)
```

---

## Commands

| Command | Description |
|---------|-------------|
| `cortex ingest --path <dir>` | One-time Git history ingestion |
| `cortex watch --path <dir>` | Live file watcher + API server |
| `cortex serve` | API server only |
| `cortex ask "<query>"` | Natural language graph query |
| `cortex search "<query>"` | Semantic vector search |
| `cortex index` | Build embedding index |
| `cortex timeline [when]` | Work sessions (today, yesterday, last week) |
| `cortex recall "<when>"` | Mental state reconstruction |
| `cortex focus` | Current working context |
| `cortex stack` | Working memory stack |
| `cortex back` | Pop to previous context |
| `cortex where` | "Where was I?" |
| `cortex debt` | Cognitive debt dashboard |
| `cortex weekly` | Monday morning briefing |
| `cortex status` | Stats |
| `cortex version` | Version and system environment information |

---

## API Endpoints (`http://127.0.0.1:8787`)

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/health` | GET | Server health check |
| `/status` | GET | Database stats and entity counts |
| `/context` | GET | Assemble project context (`file_path`, `query`) |
| `/active-context` | GET | Assemble context for active editor file |
| `/focus` | GET | Retrieve formatted current focus summary |
| `/entities` | GET | List recent graph entities (`limit`, `kind`) |
| `/relations` | GET | List recent graph relations (`limit`) |

### Query & Export Capabilities

- **Query Limits & Parser**: Supports natural language limits (`top 10 functions`, `first 5 commits`, `limit 20`).
- **Working Memory Stack**: Configurable depth limit (`set_max_stack_depth`) with context tracking.
- **Cognitive Debt Exports**: Terminal dashboard reports and structured JSON exports (`format_debt_json`).

---

## Why I Built This

Every AI tool starts each conversation with amnesia. You paste the same context, explain the same codebase, rebuild the same mental model — over and over.

Cortex is the **missing memory layer**. It remembers your code, your decisions, your workflow. Every AI tool you use becomes 10× more useful because it actually knows you.

---

## License

MIT

---

## Contributing

Pull requests are welcome. For major changes, please open an issue first to discuss what you would like to change.
