# Prunifier Scheme Specification (v1)

## Overview

A **scheme** is a JSON document that describes how to prune (filter) the output of a shell command into a concise, relevant subset. Schemes are line-based: they operate on whole lines or whitespace-split columns, **not** on syntax trees (AST-based selectors are reserved for v2+).

Each scheme contains a **command** identifier, a **version** number, and an ordered list of **rules**. The rules are applied sequentially; a line is included in the final output if no rule discards it.

---

## Format

```json
{
  "command": "<shell-command>",
  "version": 1,
  "rules": [
    { "<rule>" },
    { "<rule>" }
  ]
}
```

### Top-level fields

| Field     | Type      | Required | Description |
|-----------|-----------|----------|-------------|
| `command` | `string`  | yes      | The shell command whose output this scheme prunes (e.g. `"ls -la"`, `"git status"`, `"ps aux"`). |
| `version` | `integer` | yes      | Schema version. **Must be `1`** for the line-based format. |
| `rules`   | `array`   | yes      | Ordered list of pruning rule objects. At least one rule is expected. |

---

## Rules

Each rule combines an **action** with a **match condition**.

```json
{
  "action": "keep" | "discard",
  "match_condition": { "<condition>" },
  "description": "<optional note>"
}
```

### Action

| Action    | Meaning |
|-----------|---------|
| `"keep"`  | **Drop** lines that do **not** match the condition. Only matching lines survive. |
| `"discard"` | **Drop** lines that **do** match the condition. Everything else survives. |

`"keep"` is an **exclusionary filter** — it discards everything except what matches.
`"discard"` is an **inclusionary filter** — it removes only the lines that match.

### MatchCondition types

Three condition types are supported in v1:

---

#### `Regex` — match lines by regular expression

```json
{
  "type": "Regex",
  "pattern": "^total\\s"
}
```

| Field     | Type     | Required | Description |
|-----------|----------|----------|-------------|
| `type`    | `string` | yes      | Must be `"Regex"`. |
| `pattern` | `string` | yes      | A regular expression matched against the full text of each line. |

The pattern is tested against each line independently. If it matches anywhere on the line (partial match), the condition is satisfied.

---

#### `Column` — match by column value in tabular output

```json
{
  "type": "Column",
  "index": 0,
  "pattern": "^d"
}
```

| Field     | Type      | Required | Description |
|-----------|-----------|----------|-------------|
| `type`    | `string`  | yes      | Must be `"Column"`. |
| `index`   | `integer` | yes      | Zero-based column index (whitespace-split). |
| `pattern` | `string`  | yes      | A regular expression matched against the extracted column value. |

Lines are split on whitespace. The column at `index` is extracted and tested against `pattern`. Lines with fewer columns than `index` are treated as **not matching**.

---

#### `LineNumber` — match specific line numbers (1-based)

```json
{
  "type": "LineNumber",
  "lines": [1]
}
```

| Field   | Type        | Required | Description |
|---------|-------------|----------|-------------|
| `type`  | `string`    | yes      | Must be `"LineNumber"`. |
| `lines` | `integer[]` | yes      | One or more 1-based line numbers to match. |

---

### Optional fields on rules

| Field         | Type     | Description |
|---------------|----------|-------------|
| `description` | `string` | A human-readable note explaining the rule's purpose. Must be a non-empty string if present. |

---

## Examples

### Example 1: `git status` — show only staged and unstaged file paths

`git status` output includes header lines, section headings, and file entries:

```
On branch main
Your branch is up to date with 'origin/main'.

Changes not staged for commit:
  (use "git add <file>..." to update what will be committed)
  (use "git restore <file>..." to discard changes in working directory)
        modified:   src/main.rs
        modified:   src/lib.rs

Untracked files:
  (use "git add <file>..." to include in what will be committed)
        src/new_feature.rs
```

Scheme to strip headers and keep only file paths:

```json
{
  "command": "git status",
  "version": 1,
  "rules": [
    {
      "action": "discard",
      "match_condition": {
        "type": "Regex",
        "pattern": "^\t"
      },
      "description": "Discard lines that are NOT tab-indented (headers, hints)"
    },
    {
      "action": "keep",
      "match_condition": {
        "type": "Column",
        "index": 0,
        "pattern": "(modified|new file|deleted|renamed):"
      }
    },
    {
      "action": "keep",
      "match_condition": {
        "type": "Column",
        "index": 1,
        "pattern": ".*"
      },
      "description": "Keep only the file path column"
    }
  ]
}
```

Result:

```
src/main.rs
src/lib.rs
src/new_feature.rs
```

---

### Example 2: `ls -la` — show only file permissions and names

`ls -la` output:

```
total 64
drwxr-xr-x  12 user  staff   384 Jan 15 10:00 .
drwxr-xr-x   5 user  staff   160 Jan 14 09:00 ..
-rw-r--r--   1 user  staff  1024 Jan 15 10:00 README.md
-rw-r--r--   1 user  staff 24576 Jan 15 10:00 main.py
drwxr-xr-x   3 user  staff    96 Jan 15 10:00 src/
```

Scheme to remove the total line, parent directory entries, and keep only the permissions column:

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
    },
    {
      "action": "discard",
      "match_condition": {
        "type": "Regex",
        "pattern": "\\.\\.$"
      },
      "description": "Discard parent directory entry (..)"
    },
    {
      "action": "keep",
      "match_condition": {
        "type": "Column",
        "index": 0,
        "pattern": "."
      },
      "description": "Keep only the permissions column (first whitespace-delimited field)"
    }
  ]
}
```

Result:

```
-rw-r--r--
-rw-r--r--
drwxr-xr-x
```

---

### Example 3: `ps aux` — show only PID and COMMAND columns

`ps aux` output (simplified):

```
USER         PID %CPU %MEM    VSZ   RSS TTY      STAT START   TIME COMMAND
root           1  0.0  0.3 167956 12084 ?        Ss   Jan01   0:03 /sbin/init
www-data    1024  2.5  0.5 452168 20456 ?        S    10:15   1:23 /usr/bin/nginx
ubuntu      2048 12.3  1.2 897432 49152 ?        Sl   10:20   5:45 /usr/bin/python3
```

Scheme to discard the header row and keep only PID (col 1) and COMMAND (col 10):

```json
{
  "command": "ps aux",
  "version": 1,
  "rules": [
    {
      "action": "discard",
      "match_condition": {
        "type": "LineNumber",
        "lines": [1]
      },
      "description": "Discard the header row"
    },
    {
      "action": "keep",
      "match_condition": {
        "type": "Column",
        "index": 1,
        "pattern": ".*"
      },
      "description": "Keep PID column (column 1, zero-indexed)"
    },
    {
      "action": "keep",
      "match_condition": {
        "type": "Column",
        "index": 10,
        "pattern": ".*"
      },
      "description": "Keep COMMAND column (column 10, zero-indexed)"
    }
  ]
}
```

Result:

```
1 /sbin/init
1024 /usr/bin/nginx
2048 /usr/bin/python3
```

---

## Validation

Schemes can be validated against the JSON Schema at [`src/scheme/schema.json`](src/scheme/schema.json).

```bash
# Using Python + jsonschema
python3 -c "
import json, jsonschema
with open('src/scheme/schema.json') as f:
    schema = json.load(f)
with open('path/to/your-scheme.json') as f:
    instance = json.load(f)
jsonschema.validate(instance, schema)
"
```

### Validation guarantees

| Check | Description |
|-------|-------------|
| `command` | Must be a non-empty string. |
| `version` | Must be exactly `1`. |
| `rules` | Must be a non-empty array. |
| `action` | Must be `"keep"` or `"discard"`. No other values accepted. |
| `match_condition.type` | Must be one of `"Regex"`, `"Column"`, `"LineNumber"`. |
| `pattern` (Regex) | Required when type is `"Regex"`. |
| `index` (Column) | Required when type is `"Column"`. Must be >= 0. |
| `pattern` (Column) | Required when type is `"Column"`. |
| `lines` (LineNumber) | Required when type is `"LineNumber"`. Must be an array of integers with at least one element. |
| `additionalProperties` | Not allowed at any level. |

---

## Versioning

| Version | Description |
|---------|-------------|
| 1       | Line-based selectors (Regex, Column, LineNumber). Current. |
| 2+      | *(future)* AST-based selectors for structured output. |

