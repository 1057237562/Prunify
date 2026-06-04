# Prunify

**Proxy and prune shell command output.** Prunify sits between you and your shell commands — it executes the command, captures the output, and applies a configurable **scheme** to keep only what matters.

```
$ cargo test
test result: ok. 42 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

$ prunify cargo test
test result: ok. 42 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

No more `grep -v` pipelines. No more squinting at `ps aux` firehoses. Define a scheme once, prune forever.

---

## Quick Start

```bash
# Run a command normally (passthrough)
prunify ls -la

# Run with a known scheme
prunify git status         # → only changed files
prunify ps aux             # → only PID + COMMAND columns
prunify cargo test         # → only failures + summary

# Interactive mode (no command → bash-like REPL)
prunify
prunify $ ls -la
prunify $ cargo test
prunify $ exit
```

When no scheme exists for a command, output passes through unchanged with an `[UNKNOWN COMMAND]` marker.

---

## Installation

```bash
cargo install prunify
```

Or build from source:

```bash
git clone https://github.com/your-org/prunify
cd prunify
cargo build --release
# Binary at ./target/release/prunify
```

---

## How It Works

Prunify executes your command, captures its output, and routes it through a **dispatcher**:

```
┌──────────┐    ┌──────────────┐    ┌───────────┐    ┌──────────┐
│  command  │ → │  CommandTrie  │ → │  Scheme   │ → │  Output   │
│  you type │    │  (exact /    │    │  (rules)  │   │  (pruned) │
│           │    │   prefix)    │    │           │   │           │
└──────────┘    └──────────────┘    └───────────┘    └──────────┘
```

1. **Lookup** — the command is looked up in a **trie** (prefix tree) for exact or prefix match.
2. **Scheme** — if found, the associated **scheme** (JSON rules file) is applied.
3. **Prune** — rules filter the output line-by-line and/or column-by-column.
4. **Output** — the pruned result is printed with an optional marker.

### Dispatch Modes

| Mode | Description | Marker |
|---|---|---|
| `ExactMatch` | Command matched exactly in the trie | None |
| `PrefixMatch` | Command matched a prefix (e.g. `git status --short`) | `[PRUNED] (prefix match: N tokens)` |
| `Passthrough` | No scheme found — raw output | `[UNKNOWN COMMAND]` |

---

## Schemes

A **scheme** is a JSON document describing how to prune a command's output. Schemes live in `~/.prunify/schemes/` or a project-local `.prunify/schemes/`.

### Structure

```json
{
  "command": "git status",
  "version": 1,
  "rules": [
    {
      "action": "keep",
      "match_condition": {
        "type": "Regex",
        "pattern": "^\t"
      },
      "description": "Keep only tab-indented file entries"
    }
  ]
}
```

### Match Conditions

| Type | Description | Fields |
|---|---|---|
| `Regex` | Match/reject lines by regex | `pattern` |
| `Column` | Match/reject by column value (whitespace-split) | `index`, `pattern` |
| `LineNumber` | Match/reject specific line numbers | `lines` |

### Actions

| Action | Effect |
|---|---|
| `keep` | Drop lines that do **not** match |
| `discard` | Drop lines that **do** match |

Rules are applied sequentially. Multiple `keep` rules for different columns (e.g. `Column` keep index 1 + index 10) merge to keep both columns.

### Example Schemes

**`git status`** — show only changed file paths:
```json
{"action": "keep", "match_condition": {"type": "Regex", "pattern": "^\t"}}
```

**`ps aux`** — show only PID and COMMAND columns:
```json
{"action": "keep", "match_condition": {"type": "Column", "index": 1, "pattern": ".*"}},
{"action": "keep", "match_condition": {"type": "Column", "index": 10, "pattern": ".*"}}
```

**`cargo test`** — show only test failures + summary:
```json
{"action": "discard", "match_condition": {"type": "Regex", "pattern": "\\.\\.\\. ok$"}}
```

See [SCHEMA.md](./SCHEMA.md) for the full specification, and [.prunify/schemes/](./.prunify/schemes/) for bundled examples.

---

## Configuration

Create `.prunify.yaml` in your project root:

```yaml
scheme_dir: ./my-schemes     # Custom scheme directory
verbose: true                # Enable verbose logging
no_color: false              # Disable colored output
strict: true                 # Reject unknown commands (error instead of passthrough)
```

CLI flags override config file values:

| Flag | Shorthand | Description |
|---|---|---|
| `--scheme-dir <DIR>` | | Custom scheme directory |
| `--verbose` | `-v` | Verbose logging |
| `--no-mark` | | Suppress `[PRUNED]` / `[UNKNOWN COMMAND]` marks |
| `--strict` | | Reject unknown commands with error |
| `--rebuild-trie` | | Force rebuild of command trie cache |
| `--help` | `-h` | Print help |

---

## Safety Features

- **Recursion guard** — cannot proxy itself (`prunify prunify ls` is rejected)
- **TTY passthrough** — interactive programs (`vim`, `less`, `python`, etc.) bypass the proxy
- **Binary detection** — output with null bytes or >30% non-printable chars is left untouched
- **Signal forwarding** — `Ctrl+C` and `SIGTERM` are forwarded to child processes
- **ANSI stripping** — color codes are removed before rule matching (output is still raw)

---

## Project Layout

```
src/
├── main.rs          # Entry point, interactive REPL, command pipeline
├── lib.rs           # Library root
├── cli.rs           # CLAP CLI argument parser
├── error.rs         # Error types (PrunifyError, PrunifyResult)
├── config/
│   ├── mod.rs
│   ├── types.rs     # PrunifyConfig struct
│   └── loader.rs    # YAML config loader
├── scheme/
│   ├── mod.rs
│   ├── types.rs     # Scheme, Rule, Action, MatchCondition types
│   ├── loader.rs    # Scheme merging (default + project overrides)
│   ├── storage.rs   # JSON file I/O
│   └── schema.json  # JSON Schema for validation
├── engine/
│   ├── mod.rs
│   ├── trie.rs      # CommandTrie (prefix tree for command lookup)
│   ├── line_parser.rs    # Line-level rule application
│   ├── column_selector.rs # Column-level rule application
│   └── ansi_stripper.rs  # ANSI escape sequence removal
└── proxy/
    ├── mod.rs
    ├── dispatcher.rs    # Dispatch logic (exact/prefix/passthrough)
    ├── executor.rs      # Command execution + output capture
    ├── marking.rs       # [PRUNED] / [UNKNOWN COMMAND] markers
    ├── recursion_guard.rs # Self-invocation detection
    ├── tty.rs           # TTY detection + interactive command passthrough
    ├── binary_detector.rs # Binary output detection
    └── signal_handler.rs  # Ctrl+C / SIGTERM forwarding
```

---

## License

MIT
