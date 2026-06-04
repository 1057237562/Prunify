---
name: prunifier
description: Proxy bash commands through prunify to prune verbose output using JSON schemes
triggers:
  - prunify
  - prunifier
  - prune output
  - trim output
  - prune command
---

# Prunifier

Proxy and prune bash command output using JSON schemes.

## Installation

```bash
cargo build --release
cp target/release/prunifier ~/.local/bin/prunify   # or any PATH directory
```

The binary is built as `prunifier` from the Cargo package. Copy or rename it
to `prunify` for the intended command name. You can also use `prunifier`
directly.

## Invocation

```
prunify <command> [args...]
prunify --scheme-dir ./custom-schemes --no-mark git status
```

### Flags

| Flag | Description |
|------|-------------|
| `--scheme-dir <path>` | Custom directory for scheme JSON files (default: `.prunifier/schemes/`) |
| `--verbose` | Enable verbose logging |
| `--no-mark` | Disable `[PRUNED]` and `[UNKNOWN COMMAND]` marks in output |
| `--strict` | Reject unknown commands with an error instead of passthrough |

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

Example: `prunify ls -la` when `.prunifier/schemes/ls-la.json` contains
`"command": "ls -la"`.

### 2. Prefix Match

The command shares a common prefix with a known scheme but has additional
tokens. For example, `prunify ls -la --color=auto` shares the prefix `ls -la`
with the known scheme.

Output is pruned using the matched scheme. A `[PRUNED]` mark is appended:

```
[PRUNED] (prefix match: 2 tokens -- scheme may be suboptimal)
```

This mark signals that the scheme may not be optimal for the full command.
When the skill detects this mark, consider spawning a subagent to generate a
more specific scheme.

#### Workflow: Optimizing a Prefix Match

1. **Run raw**: Execute the command without prunify to see the full output
2. **Analyze**: Compare pruned vs raw output. What useful info was lost? What
   noise remained?
3. **Design**: Create an optimized scheme JSON for the specific flags
4. **Choose subagent**: Delegate scheme creation to a subagent:
   - `explore` -- good for analyzing output patterns and finding noise
   - `librarian` -- good when researching command output formats
   - `deep` -- good for complex scheme design with multiple rules
5. **Write**: Save the optimized scheme to
   `.prunifier/schemes/<command-slug>.json`
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
[PRUNED] (prefix match: 2 tokens -- scheme may be suboptimal)
```

The verbose scheme keeps only tab-indented lines (`^\t`), but `--short` output
uses spaces. Every line is discarded, producing empty output. The `[PRUNED]`
mark alerts you, and the workflow above guides creation of a scheme tailored
for `git status --short`.

### 3. Passthrough

No matching scheme exists for the command. Raw output is passed through
unmodified. A `[UNKNOWN COMMAND]` mark is appended:

```
[UNKNOWN COMMAND] (no scheme found -- output is raw)
```

When the skill detects this mark, consider spawning a subagent to analyze the
command output and generate a new scheme.

#### Workflow: Creating a Scheme for an Unknown Command

1. **Run raw**: Execute the command without prunify to capture the full output
2. **Identify noise**: Which lines or columns are irrelevant? Headers? Metadata?
   Status lines?
3. **Design rules**: Combine `discard` (remove known noise) and `keep`
   (retain only relevant lines) rules
4. **Choose subagent**: Same as prefix match workflow
5. **Write**: Save to `.prunifier/schemes/<command-slug>.json`
6. **Verify**: Run `prunify <command>` again. The `[UNKNOWN COMMAND]` mark
   should disappear (exact match now)

## Scheme Format

Schemes are JSON documents stored in `.prunifier/schemes/`. Each scheme
targets one command and contains an ordered list of rules.

```json
{
  "command": "ls -la",
  "version": 1,
  "rules": [
    {
      "action": "discard",
      "match_condition": {
        "type": "Regex",
        "pattern": "^total\\s"
      },
      "description": "Discard the 'total N' block count line"
    }
  ]
}
```

Three match condition types are available in v1:

- **Regex** -- match lines by regular expression
- **Column** -- match a whitespace-split column by index and pattern
- **LineNumber** -- match specific 1-based line numbers

Each rule has an `action` of `"keep"` (drop non-matching lines) or `"discard"`
(drop matching lines). Rules are applied sequentially.

See [SCHEMA.md](../../SCHEMA.md) for the full specification, validation
requirements, and worked examples (git status, ls -la, ps aux).

## Configuration

Create a `.prunifier.yaml` file in the project root to override defaults:

```yaml
# Custom scheme directory (default: .prunifier/schemes/)
scheme_dir: ./my-schemes

# Enable verbose logging (default: false)
verbose: true

# Disable colored output (default: false)
no_color: true

# Reject unknown commands with error instead of passthrough (default: false)
strict: true
```

All four fields are optional. Omitted fields use their default values.

## Standalone Note

The `prunify` binary works entirely standalone. You can use it without this
skill or OpenCode. The skill provides workflow guidance for scheme generation
via subagents (e.g., detecting `[PRUNED]` or `[UNKNOWN COMMAND]` marks and
spawning explore/librarian agents to create new schemes).

## Mode 3: New Command Workflow

When prunify appends `[UNKNOWN COMMAND]` to stdout, no scheme exists for this command. A subagent should analyze the output and decide whether a scheme would improve agent efficiency.

### Workflow
1. **Review**: Examine the raw output — is there noise? Repetitive metadata? Long headers?
2. **Decide**: Not all commands need pruning. Commands that produce minimal output (e.g., `echo`, `pwd`) don't need schemes. Focus on commands whose output changes frequently or is verbose.
3. **Analyze**: If pruning would help, choose a subagent to analyze patterns:
   - `explore` — good for identifying output sections and noise patterns
   - `librarian` — good when the command has documented output formats to reference
   - `deep` — good for complex multi-rule scheme design
4. **Design**: The subagent drafts a scheme JSON with rules to remove noise and keep signal
5. **Write**: Save to `.prunifier/schemes/<command-slug>.json` (replace spaces with dashes)
6. **Test**: Run `prunify <command>` — verify output is usefully pruned without losing important info
7. **Contribute** (optional): If the scheme is high-quality and the command is common, consider submitting it as a built-in scheme

### Example: `docker ps`
`docker ps` produces wide tabular output with many columns. Most of the time, only CONTAINER ID, IMAGE, and NAMES are relevant.

**Workflow**:
1. Run `docker ps` raw → see 8+ columns, many irrelevant (CREATED, STATUS, PORTS for quick checks)
2. Decide: YES, this would benefit from pruning
3. Subagent analyzes column structure
4. Draft scheme: `discard` header-less columns, `keep` only columns 0, 1, and 6 (CONTAINER ID, IMAGE, NAMES)
5. Save to `.prunifier/schemes/docker-ps.json`
6. Verify: `prunify docker ps` → compact 3-column output

### Note
Not all commands need pruning. Skip scheme creation for commands that:
- Always produce minimal output (pwd, whoami, date)
- Produce critical output that should never be filtered (kill, rm without -f)
- Have output that varies too much to capture with a fixed scheme
