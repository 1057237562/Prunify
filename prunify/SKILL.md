---
name: prunify
description: Proxy bash commands through prunify to prune verbose output using JSON schemes
triggers:
  - prunify
  - prunify
  - prune output
  - trim output
  - prune command
---

# Prunify

Proxy and prune bash command output using JSON schemes.

## Framework Compatibility

This skill works with any agentic coding framework — **OpenCode**, **Claude Code**,
**Cline**, **Aider**, **CodeBuff**, and others. The workflows reference agent types
and tools by their OpenCode names (`explore`, `librarian`, `deep`), but every
framework provides equivalent capabilities:

| Capability | OpenCode | Claude Code | Cline |
|---|---|---|---|
| File/code search | `explore` | `/Error` or `grep` | `search` |
| External reference lookup | `librarian` | web search via `/thinking` | `fetch` |
| Complex multi-step execution | `deep` | `/thinking` | `plan` |
| File I/O | direct tools | direct tools | direct tools |

Adapt the agent type names in each workflow to your framework's conventions.

## Installation

```bash
cargo build --release
cp target/release/prunify ~/.local/bin/prunify   # or any PATH directory
```

The binary is built as `prunify` from the Cargo package. Copy or rename it
to `prunify` for the intended command name. You can also use `prunify`
directly.

## Invocation

```
prunify <command> [args...]
prunify --scheme-dir ./custom-schemes --no-mark git status
```

### Flags

| Flag | Description |
|------|-------------|
| `--scheme-dir <path>` | Custom directory for scheme JSON files (default: `.prunify/schemes/`) |
| `--verbose` | Enable verbose logging |
| `--no-mark` | Disable `[PRUNED]` and `[UNKNOWN COMMAND]` marks (which prompt use of `prunify skill`) |
| `--strict` | Reject unknown commands with an error instead of passthrough |
| `--rebuild-trie` | Force rebuild of the command trie cache (ignores `.prunify/trie.json`) |

Arguments after all flags are treated as the command to proxy. The command is
executed as-is through the shell, its output captured and pruned.

### Example

```bash
# Run ls -la through prunify with the built-in scheme
prunify ls -la
```

With the built-in scheme for `ls -la`, the output drops the "total N" line and
parent directory entries (`..`), and keeps only the permissions column:

```
-rw-r--r--
-rw-r--r--
drwxr-xr-x
```

## Three Modes

Prunify uses a trie-based command matcher to determine how to handle each
command. It tries exact match first, then prefix match, then falls through to
passthrough.

### 1. Exact Match

The command matches a known scheme exactly (token-for-token through the trie).
Output is pruned silently. No marks are appended.

Example: `prunify ls -la` when `.prunify/schemes/ls-la.json` contains
`"command": "ls -la"`.

### 2. Prefix Match

The command shares a common prefix with a known scheme but has additional
tokens. For example, `prunify ls -la --color=auto` shares the prefix `ls -la`
with the known scheme.

Output is pruned using the matched scheme. A `[PRUNED]` mark is appended:

```
[PRUNED] (prefix match: 2 tokens -- use `prunify skill` to optimize scheme)
```

This mark signals that the scheme may not be optimal for the full command.
When you see this mark, delegate scheme creation to your agent:

- **OpenCode**: `explore` (pattern analysis), `librarian` (format research), `deep` (complex design)
- **Claude Code**: Use `/thinking` for analysis, web search for format research
- **Cline**: Use `search` for output patterns, `fetch` for command format docs
- **Other**: Use your framework's equivalent of code search, web lookup, and file creation

#### Workflow: Optimizing a Prefix Match

1. **Run raw**: Execute the command without prunify to see the full output
2. **Audit**: Classify every line using [Pruning Strategy](#pruning-strategy) below
3. **Design**: Create an optimized scheme JSON for the specific flags
4. **Choose agent approach**: Delegate scheme creation to your agent using
   its native tools for pattern analysis, format research, and file writing
5. **Write**: Save the optimized scheme to
   `.prunify/schemes/<command-slug>.json`
6. **Verify**: Run `prunify <command>` again. The `[PRUNED]` mark should
   disappear (exact match now)

#### Example: `git status --short`

If a scheme exists for `git status` (verbose) but you run `git status --short`,
the prefix match fires because `--short` is an extra token beyond the known
scheme. The verbose scheme's rules often don't fit the short format:

**Before (raw `git status --short`):**
```
 M src/main.rs
 M src/lib.rs
?? new_file.rs
```

**After (pruned through the verbose `git status` scheme — all lines dropped):**
```
[PRUNED] (prefix match: 2 tokens -- use `prunify skill` to optimize scheme)
```

The verbose scheme keeps only tab-indented lines (`^\t`), but `--short` output
uses spaces. Every line is discarded, producing empty output. The `[PRUNED]`
mark alerts you, and the workflow above guides creation of a scheme tailored
for `git status --short`.

### 3. Passthrough / Unknown Command

No matching scheme exists for the command. Raw output is passed through
unmodified. A `[UNKNOWN COMMAND]` mark is appended:

```
[UNKNOWN COMMAND] (no scheme found -- use `prunify skill` to create scheme)
```

When you see this mark, delegate scheme creation to your agent (see
agent mapping in the [Prefix Match](#2-prefix-match) section).

#### Workflow: Creating a Scheme for an Unknown Command

1. **Run raw**: Execute the command without prunify to capture the full output
2. **Audit**: Classify every line using [Pruning Strategy](#pruning-strategy) below
3. **Design rules**: Combine `discard` (remove known noise) and optionally
   `keep` (retain only relevant lines) rules
4. **Choose agent approach**: Same as prefix match workflow — use your
   framework's native tools for analysis, research, and file creation
5. **Write**: Save to `.prunify/schemes/<command-slug>.json`
6. **Verify**: Run `prunify <command>` again. The `[UNKNOWN COMMAND]` mark
   should disappear (exact match now)

## Pruning Strategy

A good scheme removes 80-95% of output while keeping every line that matters.
Achieve this by systematically classifying each line type and discarding
aggressively.

### Output Auditing Method

Before writing rules, run the raw command and answer for every line:

| Category | Example | Verdict |
|----------|---------|---------|
| **Structural whitespace** | Blank lines, separator lines | Discard |
| **Redundant metadata** | `running N tests` (count is in summary), header banners | Discard |
| **Green/passing status** | `test ... ok`, `PASS`, `SUCCESS` — lines that just say "everything worked" | Discard |
| **Repetitive per-item lines** | File lists in `git status`, process list in `ps aux` | Evaluate — maybe discard if summary exists |
| **Build/progress lines** | `Finished ...`, `Compiling ...`, progress bars | Discard |
| **Compiler warnings** | `warning: ...` and associated source markup (`-->`, `\|`, `= note:`) | Discard (go to stderr, prune for safety) |
| **Result summaries** | `test result: ok. N passed...`, `N files changed` | **Keep** |
| **Failure/error lines** | `FAILED`, `error:`, `Error:`, exception traces | **Keep** |
| **Headers with useful context** | `Running unittests src/lib.rs` | **Keep** (but discard if redundant) |
| **Data rows** | File entries in `ls -la`, process rows in `ps aux` | **Keep** (unless column-pruned) |

### Three Noise Levels

Start with Level 3 and work backward until the output is useful:

1. **Conservative** — discard only obvious noise (warnings, blank lines, build progress). ~20-40% reduction.
2. **Moderate** — also discard redundant headers and per-item status lines that duplicate a summary. ~50-70% reduction.
3. **Aggressive** — discard passing/OK status lines entirely, keep only failures and the final rollup. ~80-95% reduction.

**Default recommendation**: Level 3 (Aggressive). The `[UNKNOWN COMMAND]` or `[PRUNED]` mark already tells the user pruning happened. If they need more detail, they can run raw.

### Rule Design Principles

1. **Prefer `discard` over `keep`**: Discard removes specific noise while keeping everything else. Keep drops everything that doesn't match — one missed pattern means data loss. Only use `keep` when you are certain about exactly which lines are signal (e.g., column selection after discard).

2. **Discard the most specific patterns first**: Order rules from most specific to most general. Specific patterns (like `^warning:`) won't accidentally catch signal. General patterns (like `\s*$`) go later.

3. **Anchor patterns**: Use `^` and `$` anchors to avoid matching inside unexpected lines. `^warning:` is safe; `warning` alone might match a test name.

4. **Cover all noise types in one pass**: Run the raw output through `| grep -c` to count lines per pattern. If 90% of lines are `... ok`, and the summary already has the count, discard the `... ok` lines.

5. **Preserve failure context**: Never discard lines containing `FAILED`, `Error`, `error:`, `panic`, `traceback`, or `exception`. When tests fail, those lines are the entire point of the output.

### Counting Lines to Set Targets

Use shell commands to measure noise before writing rules:

```bash
# Count total lines
cargo test 2>/dev/null | wc -l

# Count lines of each type
echo "blank:   $(cargo test 2>/dev/null | grep -c '^\s*$')"
echo "running: $(cargo test 2>/dev/null | grep -c '^running [0-9]')"
echo "... ok:  $(cargo test 2>/dev/null | grep -c '\.\.\. ok$')"
echo "FAILED:  $(cargo test 2>/dev/null | grep -c 'FAILED')"
echo "summary: $(cargo test 2>/dev/null | grep -c '^test result:')"
```

When the sum of noise lines > 80% of total, aggressive pruning is justified.

## Scheme Format

Schemes are JSON documents stored in `.prunify/schemes/`. Each scheme
targets one command and contains an ordered list of rules.

```json
{
  "command": "cargo test",
  "version": 1,
  "rules": [
    {
      "action": "discard",
      "match_condition": {
        "type": "Regex",
        "pattern": "^\\s*$"
      },
      "description": "Discard blank lines between test sections"
    },
    {
      "action": "discard",
      "match_condition": {
        "type": "Regex",
        "pattern": "^\\.\\.\\. ok$"
      },
      "description": "Discard passing test lines — only FAILED tests matter"
    },
    {
      "action": "discard",
      "match_condition": {
        "type": "Regex",
        "pattern": "^running \\d+ tests?$"
      },
      "description": "Discard running-N-tests headers — count is in the summary"
    }
  ]
}
```

**File naming**: `.prunify/schemes/<command-slug>.json` — replace spaces with dashes
(e.g., `cargo test` → `cargo-test.json`, `git status --short` → `git-status-short.json`).

**Field reference**:

| Field | Required | Description |
|-------|----------|-------------|
| `command` | Yes | Exact command string (e.g., `"cargo test"`) |
| `version` | Yes | Must be `1` (only v1 exists) |
| `rules` | Yes | Array of rule objects, applied in order |
| `rules[].action` | Yes | `"discard"` (drop matching) or `"keep"` (drop non-matching) |
| `rules[].match_condition` | Yes | Match specification (see below) |
| `rules[].description` | No | Human-readable explanation of what this rule does |

**Match condition types**:

| Type | Fields | Description |
|------|--------|-------------|
| `Regex` | `pattern` | Match lines by regular expression |
| `Column` | `index`, `pattern` | Match a whitespace-split column by index and pattern |
| `LineNumber` | `lines` | Match specific 1-based line numbers |

**Rule application**: Rules fire sequentially. Each rule's output is the next
rule's input. `discard` removes matching lines; `keep` removes NON-matching
lines.

See [SCHEMA.md](../../SCHEMA.md) for the full specification, validation
requirements, and worked column-selector examples.

## Worked Examples

### `cargo test` — Aggressive Pruning

**Raw output**: 200 lines. All tests pass.

**Audit**:

| Line type | Count | Verdict |
|-----------|-------|---------|
| Blank lines | 60 | Discard (structural whitespace) |
| `running N tests` | 19 | Discard (redundant — count is in summary) |
| `test ... ok` | 100 | Discard (passing status — every test passed, individual lines are noise) |
| `test result:` | 20 | **Keep** (the essential summary) |
| `FAILED` | 0 | **Keep** (critical when present; lines without `... ok` suffix survive) |

**Target**: 200 lines → ~20 lines (90% reduction).

**Scheme** (at `.prunify/schemes/cargo-test.json`):

```json
{
  "command": "cargo test",
  "version": 1,
  "rules": [
    {
      "action": "discard",
      "match_condition": {
        "type": "Regex",
        "pattern": "^\\s*$"
      },
      "description": "Discard blank and whitespace-only lines between test sections"
    },
    {
      "action": "discard",
      "match_condition": {
        "type": "Regex",
        "pattern": "^running \\d+ tests?$"
      },
      "description": "Discard 'running N tests' headers — counts are already in the test result summary"
    },
    {
      "action": "discard",
      "match_condition": {
        "type": "Regex",
        "pattern": "\\.\\.\\. ok$"
      },
      "description": "Discard lines for passing tests — only FAILED tests and the summary are relevant"
    }
  ]
}
```

**Result** (all tests pass):
```
test result: ok. 25 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 4 passed; 0 failed; ...
test result: ok. 8 passed; 0 failed; ...
...
```

**When tests fail**, lines containing `FAILED`, the `failures:` header, failure
details, and compilation errors survive — none match the discard patterns.

## Configuration

Create a `.prunify.yaml` file in the project root to override defaults:

```yaml
# Custom scheme directory (default: .prunify/schemes/)
scheme_dir: ./my-schemes

# Enable verbose logging (default: false)
verbose: true

# Disable colored output (default: false)
no_color: true

# Reject unknown commands with error instead of passthrough (default: false)
strict: true
```

All four fields are optional. Omitted fields use their default values.

## Command Trie Cache

Prunify builds a **command trie** from all loaded schemes to efficiently
match commands. To avoid rebuilding this trie on every invocation, it is
cached to `.prunify/trie.json` after the first run.

### Auto-Invalidation

The cache is **automatically invalidated** when any scheme file changes.
Prunify compares the modification timestamps of all `.json` files in
`.prunify/schemes/` against the cached trie. If any scheme is newer, the
trie is rebuilt.

### Forced Rebuild

If you need to force a rebuild (e.g., after manually editing a scheme file
without changing its timestamp, or after switching branches):

```bash
prunify --rebuild-trie <command>
```

This ignores the cached trie and rebuilds from the current scheme files.

### How It Works

1. On startup, prunify checks if `.prunify/trie.json` exists and is
   newer than all scheme files.
2. If fresh → load from cache (fast, no computation).
3. If stale or missing → rebuild the trie from schemes and save it to
   `.prunify/trie.json`.

The trie file is JSON and can be inspected manually (`cat .prunify/trie.json`).

## Standalone Note

The `prunify` binary works entirely standalone — no agent framework required.
This skill provides workflow guidance for agent-driven scheme generation
(e.g., detecting `[PRUNED]` or `[UNKNOWN COMMAND]` marks and having your agent
create or refine schemes). It is compatible with OpenCode, Claude Code, Cline,
Aider, and any other agentic coding tool.

## Decision Guide: When to Prune

Not all commands need pruning. Apply the audit method and check:

| Criterion | Prune? |
|-----------|--------|
| Output > 50 lines with > 60% noise | ✅ Yes, aggressive |
| Output 10-50 lines with obvious noise (warnings, headers) | ✅ Yes, moderate |
| Output < 10 lines and always consistent | ❌ No (e.g., `pwd`, `whoami`, `date`) |
| Output is critical and should never be filtered | ❌ No (e.g., `kill`, `rm`, `mv`) |
| Output varies too much for fixed patterns | ❌ No (e.g., `find` with different predicates) |

## Workflows by Mode

### Mode 2 (Prefix Match) — Optimizing a Partial Match

When `[PRUNED]` appears, the existing scheme is close but not exact:

1. **Run raw**: Execute the command without prunify to see the full output
2. **Audit**: Use the [Pruning Strategy](#pruning-strategy) to classify lines
3. **Diff**: Compare raw vs pruned — what was lost? What noise survived?
4. **Design**: Create a new scheme for the specific command (with all flags)
5. **Choose agent approach**: Delegate to your framework's analysis tools
   (code search for patterns, web search for format docs, etc.)
6. **Write**: Save to `.prunify/schemes/<command-slug>.json`
7. **Verify**: `[PRUNED]` mark should disappear; target ≥80% reduction

### Mode 3 (Passthrough) — Creating a New Scheme

When `[UNKNOWN COMMAND]` appears, no scheme exists:

1. **Run raw**: Capture the full output
2. **Decide**: Apply the [Decision Guide](#decision-guide-when-to-prune) —
   not every command needs a scheme
3. **Audit**: Classify every line type with the [Pruning Strategy](#pruning-strategy)
4. **Count**: Measure noise percentage. If > 60%, pruning is justified
5. **Design**: Write discard rules for each noise category, starting with
   the most aggressive level and relaxing only if useful info is lost
6. **Write**: Save to `.prunify/schemes/<command-slug>.json`
7. **Verify**: `[UNKNOWN COMMAND]` mark disappears. Run with `--no-mark` to
   check the clean output, then without to confirm the signal is complete
8. **Contribute** (optional): High-quality schemes for common commands can
   be promoted to built-in schemes
