## Task: TTY Detector (completed 2026-06-04)

### Files created/changed
- `Cargo.toml` — added `libc = "0.2"` dependency
- `src/proxy/tty.rs` — `TtyDetector` struct with two methods:
  - `is_tty()` — calls `unsafe { libc::isatty(libc::STDOUT_FILENO) != 0 }` to detect if stdout is a TTY
  - `should_passthrough(command: &str)` — extracts first token via `split_whitespace().next()` and checks against a static list of known interactive binaries (`vim`, `nano`, `htop`, `top`, `less`, `more`, `emacs`, `screen`, `tmux`, `man`, `irb`, `python`, `node`)
- `src/proxy/mod.rs` — created with `pub mod tty; pub use tty::TtyDetector;`
- `src/lib.rs` — added `pub mod proxy;`
- `tests/tty_test.rs` — 3 integration tests:
  - `test_detect_tty_stdout` — asserts `is_tty()` returns `false` in piped (cargo test) environment
  - `test_detect_non_tty_stdout` — same assertion, duplicate for clarity
  - `test_tty_passthrough_skips_pruning` — verifies known interactive commands return true, non-interactive return false

### Key decisions
- `is_tty()` returns `false` on any failure (safe default — failing open means we prune by default rather than skipping)
- `is_tty()` is Linux/macOS only via `libc::isatty`; no Windows-specific code
- `should_passthrough` only checks the first command token (binary name), ignores arguments — simple and correct for interactive program detection
- Unit tests in `src/proxy/tty.rs` cover edge cases (empty/whitespace input, known interactive with arguments)
- Integration tests in `tests/tty_test.rs` cover the public API surface

## Task: Column Selector (completed 2026-06-04)

### Files created/changed
- `src/engine/column_selector.rs` — `ColumnSelector` struct with `apply_rules()`:
  - For each Column-type rule with `Action::Keep`: keep only the specified column index from each line
  - For each Column-type rule with `Action::Discard`: remove the specified column, rejoin remaining with single space
  - Non-Column rules (Regex, LineNumber) are skipped entirely
  - Lines with fewer columns than the index: skip the rule (line kept as-is, no panic)
  - Empty/whitespace-only lines preserved as-is
  - If discarding all columns leaves nothing, the line is omitted from output
- `src/engine/mod.rs` — added `pub mod column_selector;` and `pub use column_selector::ColumnSelector;`
- `tests/column_selector_test.rs` — 8 tests (5 required + 3 bonus):
  - `test_keep_specific_columns` — Keep col 0, verify only that column survives
  - `test_discard_specific_columns` — Discard col 1, verify remaining columns rejoined
  - `test_whitespace_separator` — Lines with tabs/multiple spaces, uses `split_whitespace()`
  - `test_variable_column_count` — Lines with different column counts handled gracefully
  - `test_column_index_out_of_bounds` — Index > column count: line preserved as-is
  - `test_discard_out_of_bounds` — Same for Discard action
  - `test_empty_output` — Empty string returns empty
  - `test_non_column_rules_skipped` — Regex and LineNumber rules ignored

### Key decisions
- `ColumnSelector` is separate from `LineParser` — ColumnSelector does column-level selection (which columns to output per line), while LineParser does line-level filtering (which lines to include). Both handle `MatchCondition::Column` rules but for different purposes.
- Uses `split_whitespace()` for robust whitespace handling (tabs, multiple spaces, leading/trailing)
- `pattern` field in `MatchCondition::Column` is ignored by ColumnSelector (used by LineParser for pattern matching)
- Results are rejoined with `" "` (single space) regardless of original whitespace

## Task 15: Config Loader (completed 2026-06-04)

- Created `src/config/loader.rs` with `ConfigLoader` struct and `load(path: Option<&Path>) -> PrunifyResult<PrunifyConfig>` method.
- `load()` returns `PrunifyConfig::default()` when path is `None` or file doesn't exist (not an error).
- Invalid YAML produces `PrunifyError::ConfigError` (via `serde_yaml::from_str` + `map_err`).
- After deserialization, `load()` merges defaults: fields absent from YAML (`None`) are filled from `PrunifyConfig::default()`.
- Updated `src/config/mod.rs` with `pub mod loader;` and `pub use loader::ConfigLoader;`.
- Updated `src/lib.rs` to re-export `ConfigLoader` via `pub use config::{ConfigLoader, PrunifyConfig};`.
- Created `tests/config_loader_test.rs` with 4 integration tests (all pass):
  - `test_load_yaml_config` — fully-specified YAML, all 4 fields verified
  - `test_missing_config_uses_defaults` — non-existent path returns default
  - `test_invalid_yaml_errors` — malformed YAML yields `ConfigError`
  - `test_partial_config_merges_defaults` — only `verbose: true` set, absent fields filled from defaults
- Test temp files created via `std::fs::File::create` + `write!` (no `tempfile` dependency needed).
- `serde_yaml::from_str::<PrunifyConfig>()` for YAML parsing; `deny_unknown_fields` on `PrunifyConfig` handles unknown field rejection.

- Created `src/error.rs` with `PrunifyError` enum (8 variants) and `PrunifyResult<T>` type alias using `thiserror`.
- `PrunifyError` uses `#[from]` for automatic conversion from `std::io::Error`, `serde_json::Error`, and `regex::Error`.
- Added `thiserror = "2"`, `serde_json = "1"`, `regex = "1"` to `Cargo.toml` dependencies.
- Re-exported `PrunifyError` and `PrunifyResult` from `src/lib.rs`.

- Created `src/scheme/schema.json` — JSON Schema (draft-07) for the v1 line-based scheme format.
  - Validates: `command` (string, required), `version` (const 1), `rules` (array of objects).
  - Supports 3 MatchCondition types: `Regex` (pattern), `Column` (index + pattern), `LineNumber` (lines[]).
  - Actions: `"keep"` (drop non-matching) and `"discard"` (drop matching).
  - `additionalProperties: false` enforced at all levels; `description` optional on rules.
- Created `SCHEMA.md` — full specification with prose, field reference tables, and 3 worked examples (git status, ls -la, ps aux).

- 2026-06-04: Test infrastructure set up (RED phase).
  - Binary name is `prunify` (package name), not `prunify`. References in test helpers and shell scripts use `./target/debug/prunify`.
  - `tests/common/mod.rs` (not `common.rs`) required for `mod common;` in integration tests.
  - `test_scheme_validator_rejects_invalid_json` panics because binary exits 0 ("Hello, world!") instead of rejecting invalid JSON — correct TDD RED behavior.

## Task 14: Config Types (completed 2026-06-04)

- Created `src/config/types.rs` with `PrunifyConfig` struct (4 fields: `scheme_dir`, `verbose`, `no_color`, `strict`), all `Option` typed, with `#[serde(deny_unknown_fields)]`.
- Implemented `Default` for `PrunifyConfig`: `scheme_dir: None`, verbose/no_color/strict: `Some(false)`.
- Added `serde_yaml = "0.9"` to Cargo.toml.
- Created `src/config/mod.rs` re-exporting `PrunifyConfig`.
- Added `pub mod config;` and `pub use config::PrunifyConfig;` to `lib.rs`.
- 4 test cases pass: basic YAML deserialization, empty YAML `{}` (all fields None), unknown field rejection via `deny_unknown_fields`, and scheme_dir PathBuf resolution.

## Task 1: Project Scaffolding (completed 2026-06-04)

### Cargo.toml quirks
- `cargo init --name prunify` generated `edition = "2024"` and an empty `[dependencies]`
- When editing Cargo.toml, be careful not to introduce duplicate keys — the file already had some pre-populated dependencies (thiserror, serde_json, regex) which caused duplicate key errors
- Overwriting via edit with oldString/newString for the full [dependencies] block resolved the duplicate issue

### Directory structure decisions
- Kept pre-existing files untouched: SCHEMA.md, src/error.rs, src/scheme/schema.json, tests/shell_tests.sh
- Created new dirs: src/scheme/, src/config/, src/engine/, src/proxy/, tests/, tests/common/, .prunify/schemes/, .omo/evidence/
- Binary name stays `prunify` for now (will add `[[bin]] name = "prunify"` later)
- Library is at src/lib.rs (currently just a placeholder comment)
- Pre-existing test helpers in tests/common/mod.rs reference `./target/debug/prunify` binary — that binary doesn't exist yet, so tests will fail at TDD RED phase

### Build verification
- `cargo build`: successful (32 crates compiled)
- `cargo run`: output "prunify v0.1.0" as expected
- `cargo test`: 1 test fails (expected RED — references prunify binary not yet built)

## Task 6: Scheme Storage module (completed 2026-06-04)

### Files created
- `src/scheme/types.rs` — `Scheme`, `Rule`, `MatchCondition` structs with `validate()` (version=1, rules non-empty)
- `src/scheme/storage.rs` — `SchemeStorage` with `load()`, `load_all()`, `validate_scheme_file()`
- `src/scheme/mod.rs` — module root exposing both submodules
- `tests/scheme_storage_test.rs` — 4 TDD tests (load valid, missing file, invalid JSON, load_all skips non-JSON)
- `tests/fixtures/valid-scheme.json` — valid git-status scheme fixture
- `tests/fixtures/invalid-schema.json` — deliberately malformed JSON fixture

### Key decisions
- `SchemeStorage` is a stateless struct (no fields) — all methods are associated functions
- `load_all` skips non-.json files gracefully and continues on parse errors (eprintln warning)
- `load_all` returns empty vec if directory doesn't exist (not an error)
- Scheme type uses `#[serde(tag = "type")]` for `MatchCondition` enum (internally tagged)
- `description` on Rule is optional (`#[serde(default)]`)
- `lib.rs` updated to include `pub mod scheme;`

## Task 2: Scheme Data Types (completed 2026-06-04)

### Implementation details
- `src/scheme/types.rs` — 4 types: `Scheme`, `Rule`, `Action`, `MatchCondition` using `#[serde(deny_unknown_fields)]` on all.
- `Scheme::validate()` checks: version == 1, non-empty rules, all regex patterns compile.
- `Action` uses `#[serde(rename_all = "lowercase")]` → JSON accepts `"keep"` / `"discard"`.
- `MatchCondition` uses `#[serde(tag = "type", rename_all = "snake_case")]` → internally tagged enum with tags `"regex"`, `"column"`, `"line_number"`.
- Re-exported all types from `src/scheme/mod.rs` and registered `pub mod scheme` in `src/lib.rs`.

### Serde findings
- `deny_unknown_fields` on internally tagged enums works per-variant (extra fields not in the matched variant are rejected).
- Internally tagged enums use `#[serde(tag = "type")]` without `content` — `content` is for adjacently tagged enums only.
- `#[serde(default)]` on `Option<T>` fields allows omission in JSON without error.
- Missing required fields in a matched internally-tagged variant (e.g. `index` in `Column`) produces a serde error, not a validation error.

### Tests
- 4 integration tests in `tests/scheme_types_test.rs` all pass.
- `test_deserialize_valid_scheme` — valid JSON deserializes and passes `validate()`.
- `test_reject_invalid_action` — `"delete"` action rejected by serde (unknown variant).
- `test_reject_missing_command` — missing `command` field rejected by serde (missing field).
- `test_column_rule_requires_index` — `Column` without `index` rejected by serde (missing field in internally tagged variant).

## Task 15: Trie Matcher (completed 2026-06-04)

### Files created/changed
- `src/engine/trie.rs` — `CommandTrie` struct with `insert()`, `search_exact()`, `search_prefix()` methods
- `src/engine/mod.rs` — `pub mod trie; pub use trie::CommandTrie;`
- `tests/trie_test.rs` — 6 integration tests
- `src/lib.rs` — added `pub mod engine;`

### Implementation notes
- Tokens obtained via `command.split_whitespace()` (splits on any whitespace, filters empties)
- Each token becomes a node in the trie via `HashMap<String, TrieNode>`
- `search_prefix` walks nodes depth-first tracking the deepest node with a `scheme_id`, returns `(scheme_id, token_count)`
- `search_exact` requires all tokens to match AND the final node to have a `scheme_id` set
- Pre-existing `line_parser.rs` and `line_parser_test.rs` had import issues (`crate::scheme::types` should be `crate::scheme`) — not part of this task

### Interesting discovery
- `src/engine/` directory already contained a `line_parser.rs` from a prior task, but the module wasn't registered in `mod.rs` (which didn't exist). The test file `tests/line_parser_test.rs` references `prunify::engine::line_parser::LineParser` — unrelated pre-existing work.

## Task 17: Line Parser (completed 2026-06-04)

### Implementation
- `src/engine/line_parser.rs` — `LineParser` struct with `apply_rules()` supporting all 3 match conditions:
  - `Regex { pattern }`: match full line against compiled regex
  - `Column { index, pattern }`: split line by whitespace, check column at index
  - `LineNumber { lines }`: 1-based line number filtering
- Both `Keep` (drop non-matching) and `Discard` (drop matching) actions supported
- Rules applied sequentially (output of rule N is input to rule N+1)
- Preserves trailing newline behavior of original output
- Returns empty string for empty output (no crash)
- All variants compile regex per-call (simple v1 approach)

### Tests
- `tests/line_parser_test.rs` — 5 integration tests all passing:
  - `test_keep_lines_matching_regex`: Keep lines matching `^error`
  - `test_discard_lines_matching_regex`: Discard lines matching `^error`
  - `test_multiple_rules_apply_in_order`: Two Keep rules applied sequentially
  - `test_empty_output`: Empty string returns empty
  - `test_no_matching_rules_keeps_all`: Discard rule with no matching lines keeps all

### Edge cases handled
- `is_empty()` early return for empty output
- `lines()` strips trailing newline, re-added if original had one
- `Column` with out-of-bounds index uses `map_or(false, ...)` — line kept (Keep) or not discarded (Discard)

## Task 18: Command Executor (completed 2026-06-04)

### Files created/changed
- `src/proxy/executor.rs` — `CommandExecutor` struct with `execute(command: &str) -> PrunifyResult<ExecutionResult>`:
  - Splits command string on whitespace via `split_whitespace()` into binary + args
  - Captures stdout/stderr via `Stdio::piped()`, exit code via `output.status.code().unwrap_or(-1)`
  - Command not found (`io::ErrorKind::NotFound`) → returns `Ok` with `exit_code: 127` (not Err)
  - Empty command string → returns `Err(PrunifyError::CommandFailed)`
  - Output preserved as-is (no trailing newline stripping/adding)
- `src/proxy/mod.rs` — added `pub mod executor;` + re-export of `CommandExecutor` and `ExecutionResult`
- `src/lib.rs` — added `pub mod proxy;` (already existed from TTY task, no change needed)
- `tests/executor_test.rs` — 6 integration tests (all pass):
  - `test_execute_simple_command`: `echo hello` → stdout="hello\n", stderr="", exit_code=0
  - `test_capture_stdout`: `echo hello stdout capture` → multi-word stdout
  - `test_capture_stderr`: `ls /nonexistent` → stderr non-empty, stdout empty, exit_code=2
  - `test_exit_code_propagation`: `false` → exit_code=1
  - `test_command_not_found`: nonexistent binary → exit_code=127
  - `test_output_with_no_newline`: `printf hello` → stdout="hello" without trailing newline

### Key decisions
- `CommandExecutor` is a stateless struct (no fields) — follows `SchemeStorage` pattern
- `ExecutionResult` is a simple struct (not a Result type) since both success and failure outcomes are represented via `exit_code`
- Commands that produce non-zero exit codes are NOT errors at the `PrunifyResult` level — the exit code is just data in `ExecutionResult`
- `CommandFailed` error variant is reserved for infrastructure failures (empty command, I/O errors other than NotFound)
- Tests use only commands available via PATH that work without shell quoting (`echo`, `false`, `ls`, `printf`) since `split_whitespace()` doesn't handle shell quoting

## Task: Output Marker (completed 2026-06-04)

### Files created/changed
- `src/proxy/marking.rs` — `OutputMarker` struct with `mark_pruned()`:
  - ExactMatch: no mark, output unchanged
  - PrefixMatch: appends `\n[PRUNED] (prefix match: N tokens — scheme may be suboptimal)\n`
  - Passthrough: appends `\n[UNKNOWN COMMAND] (no scheme found — output is raw)\n`
  - Empty output: returns just the mark text (no leading newline)
  - `no_mark=true`: output unchanged regardless of mode
  - Uses `format!()` for building strings; no ANSI colors
- `src/proxy/mod.rs` — added `pub mod marking;` and `pub use marking::OutputMarker;`
- `tests/marking_test.rs` — 4 integration tests (all pass):
  - `test_exact_match_returns_output_unchanged`: ExactMatch preserves output
  - `test_prefix_match_appends_pruned_mark`: PrefixMatch with token count in mark
  - `test_passthrough_appends_unknown_command_mark`: Passthrough shows [UNKNOWN COMMAND]
  - `test_no_mark_suppresses_all_marks`: no_mark=true bypasses marking

### Key decisions
- `matched_tokens` passed as separate arg (not extracted from `PrefixMatch(usize)`) for caller control
- Empty output edge case: mark text returned without leading `\n` separator
- `no_mark` gate checked before mode dispatch for early return
- Marks always end with `\n` (trailing newline for terminal cleanliness)
- `DispatchMode` imported via `crate::proxy::dispatcher::DispatchMode`

## Task: Recursion Guard (completed 2026-06-04)

### Files created/changed
- `src/proxy/recursion_guard.rs` — `RecursionGuard` struct with `is_recursive(command: &str) -> bool`:
  - Extracts first token via `command.split_whitespace().next()`
  - Checks direct match against `"prunify"` and `"prunify"` literals
  - Extracts file stem from first token (handles paths like `./target/debug/prunify`)
  - Also checks against `std::env::current_exe()` file stem (handles renamed binaries)
  - Returns `false` for empty/whitespace-only strings
- `src/proxy/mod.rs` — added `pub mod recursion_guard;` and `pub use recursion_guard::RecursionGuard;`
- `src/lib.rs` — added `pub mod proxy;`
- `tests/recursion_test.rs` — 4 integration tests (all pass):
  - `test_detect_self_invocation` — bare `prunify`, with args
  - `test_detect_nested_prunify` — `prunify` (debug binary name)
  - `test_normal_command_not_detected` — `ls`, `echo`, `git`, empty/whitespace
  - `test_different_path_prunify` — path-based first token vs arguments containing "prunify"

### Key decisions
- `RecursionGuard` is stateless (all methods are associated functions), matching the existing `TtyDetector` pattern in `tty.rs`
- Uses both literal check and `current_exe()` file stem check for maximum robustness
- Path-based first tokens (`./target/debug/prunify`) are detected via `Path::file_stem()`
- Commands that merely CONTAIN "prunify" in arguments (e.g., `echo 'use prunify'`) are NOT flagged — only the first token is checked

## Task: Signal Forwarding (completed 2026-06-04)

### Files created/changed
- `Cargo.toml` — added `ctrlc = "3"` dependency
- `src/proxy/signal_handler.rs` — signal forwarding module with:
  - `CHILD_PID: AtomicU32` static for thread-safe PID storage
  - `register_handler()` — installs SIGINT handler via `ctrlc::set_handler()`, forwards SIGINT to child via `libc::kill()`
  - `set_child_pid(pid: u32)` — stores child PID for handler access
  - `clear_child_pid()` — resets PID when child exits
- `src/proxy/executor.rs` — changed from `.output()` to `.spawn()` + `.wait_with_output()` + PID registration:
  - Calls `set_child_pid(child_pid)` after spawn, `clear_child_pid()` after wait
  - `child.id()` returns `u32` PID on Unix
- `src/proxy/mod.rs` — added `pub mod signal_handler;` and re-exports of `register_handler`, `set_child_pid`, `clear_child_pid`
- `src/main.rs` — calls `register_handler()` at top of `main()`, before any child spawns
- `tests/signal_test.sh` — shell test (2 tests):
  - SIGINT forwarded test: starts `sleep 30`, sends SIGINT, verifies exit in <10s (not 30s)
  - Normal execution test: verifies handler doesn't break regular commands

### Key decisions
- `register_handler()` called ONCE in `main()` — `ctrlc::set_handler()` errors on second call
- Changed from `Command::output()` (convenience = spawn + wait) to explicit `spawn()` + `wait_with_output()` to insert PID registration between them
- `AtomicU32` used for PID storage (thread-safe, no `unsafe` needed for the static)
- Signal handler silently ignores if `pid == 0` (child not yet spawned or already exited)
- `libc::kill()` return value ignored — ESRCH (-1) is valid if child already exited
- SIGKILL/SIGSTOP explicitly excluded (unhandleable by design)
- SIGTERM not explicitly forwarded (only SIGINT from Ctrl+C) — matches stated task scope
- Test uses elapsed-time assertion (<10s vs 30s sleep) rather than process-table inspection, which is simpler and more portable
