# Prunifier — Skill + Rust Proxy for Bash Output Pruning

## TL;DR

> **Quick Summary**: Build a Rust CLI proxy (`prunify`) that prunes verbose bash output using line-based JSON schemes, paired with an OpenCode skill that guides subagent-driven scheme generation. Three proxy modes: exact match (prune silently), prefix match via trie (prune + mark `[PRUNED]`), and unknown command (passthrough raw).
> 
> **Deliverables**:
> - Rust binary: `prunify` — CLI wrapper that proxies commands, applies pruning schemes, handles TTY/exit codes/recursion
> - JSON scheme system: Schema specification, 3 built-in schemes (git status, ls -la, ps aux), `.prunifier/schemes/` storage
> - Trie-based command matcher: Exact + prefix matching across full argument vectors
> - OpenCode skill: `SKILL.md` with workflow instructions for subagent-driven scheme generation
> - `.prunifier.yaml` config: Per-project overrides for schemes and settings
> - Test suite: TDD with Rust unit tests + shell integration tests
> 
> **Estimated Effort**: Large
> **Parallel Execution**: YES — 4 waves, max 8 concurrent
> **Critical Path**: T1 → T5 → T9 → T13 → T17 → T20 → F1-F4

---

## Context

### Original Request
Build a skill (using skill creator) and a Rust executable that proxies bash commands. The skill creates AST schemes to remove useless information from bash output. Three modes: exact scheme match (pruned output), common prefix match via trie (pruned + marked, triggers subagent optimization), and brand new command (raw output, triggers subagent analysis).

### Interview Summary
**Key Discussions**:
- **AST Scheme Format**: JSON schema with line-based selectors for v1 — parse output by line, apply regex/column-based keep/discard rules. Tree-based AST parsing deferred to v2+ (Metis recommendation: generic AST for arbitrary CLI output is a research project, not v1 scope).
- **Proxy Invocation**: Explicit CLI wrapper — `prunify <CMD>` (not a shell hook).
- **Pruning Authority**: Hybrid — universal defaults shipped with binary + per-project `.prunifier.yaml` overrides. Project override = complete replacement of default scheme (not deep merge).
- **Storage**: `.prunifier/schemes/` dotfile directory — 1 JSON file per command.
- **Command Scope**: Universal proxy — all commands accepted, unknown ones fall to mode 3 (passthrough).
- **Trie Matching**: Longest common prefix across full argument vector (e.g., `git status -s` shares prefix with `git status`).
- **Communication**: `[PRUNED]` mark in output for modes 2 and 3; skill reads output to detect need for subagent analysis.
- **Pipe Handling**: Prunify final output only — treat entire pipeline as one command.
- **Test Strategy**: TDD — RED-GREEN-REFACTOR for every implementation task.
- **Analysis Subagent**: Agent decides which subagent (explore/librarian/deep) to spawn for scheme generation.

### Metis Review
**Identified Gaps** (addressed):
- **AST parsing complexity**: Resolved — v1 uses line-based selectors only. AST deferred to v2+ with concrete trigger (≥10 schemes suggesting tree structure would reduce count).
- **Skill/binary boundary**: Defined — Rust binary handles all execution/parsing/pruning; OpenCode skill is documentation + subagent workflow instructions. Binary works standalone.
- **JSON Scheme schema undefined**: Resolved — schema definition is Task 1 (first task in plan).
- **Subagent workflow hand-waved**: Resolved — concrete workflow steps in Tasks 24-27.
- **Installation/distribution unclear**: Resolved — `cargo build`, `prunify` in PATH, skill copied to `.opencode/skills/`.
- **TTY/interactive commands**: Handled — TTY detection + transparent passthrough, no processing.
- **Missing edge cases**: All enumerated and covered in QA scenarios.

---

## Work Objectives

### Core Objective
Prunifier is a Rust CLI proxy (`prunify <CMD>`) that prunes verbose bash output using line-based selectors (v1) guided by per-command JSON schemes, paired with an OpenCode skill that documents workflows for scheme generation via subagents.

### Concrete Deliverables
- `Cargo.toml` + `src/main.rs` — Rust binary built with clap, anyhow, regex, trie-rs (or similar)
- `src/scheme.rs` — JSON scheme schema definition + parser/validator
- `src/trie_matcher.rs` — Trie data structure for command prefix matching
- `src/proxy.rs` — Command execution, output capture, pruning, TTY detection
- `src/modes.rs` — Three-mode dispatch logic
- `.prunifier/schemes/git-status.json` — Built-in scheme for git status
- `.prunifier/schemes/ls-la.json` — Built-in scheme for ls -la
- `.prunifier/schemes/ps-aux.json` — Built-in scheme for ps aux
- `.opencode/skills/prunifier/SKILL.md` — OpenCode skill workflow documentation
- `tests/` — Rust unit tests + integration shell tests
- `.prunifier.yaml` — Config schema specification

### Definition of Done
- [ ] `cargo build --release` succeeds with zero warnings
- [ ] `cargo test` → ALL pass (unit + integration)
- [ ] `prunify echo hello` outputs `hello` with exit 0
- [ ] `prunify ls -la` with scheme → pruned output without `total` line and `./..` entries
- [ ] `prunify ls -la --unknown-flag` → pruned output with `[PRUNED]` mark
- [ ] `prunify some-new-command` → raw output passthrough
- [ ] `prunify prunify echo hello` → recursion detected, bypassed
- [ ] `prunify ls /nonexistent` → exit code 2 propagated
- [ ] OpenCode skill: `SKILL.md` exists at `.opencode/skills/prunifier/SKILL.md`

### Must Have
- Line-based output pruning via JSON schemes (regex + column selectors)
- Three proxy modes: exact match, prefix match + `[PRUNED]`, passthrough
- Trie-based command prefix matching on full argument vectors
- TTY detection + transparent passthrough for interactive commands
- Exit code propagation from proxied command
- Recursion guard (detect `prunify prunify` — first token of proxied command is "prunify")
- 3 built-in schemes (git status, ls -la, ps aux)
- `.prunifier/schemes/` dotfile storage with per-project override
- `.prunifier.yaml` config file support
- OpenCode skill (SKILL.md) with subagent workflow instructions
- TDD with Rust unit tests and shell integration tests
- Agent-executed QA scenarios for every task

### Must NOT Have (Guardrails)
- Generic AST parser (v2+ only — requires ≥10 existing schemes to justify)
- Scheme auto-generation without human guidance
- Plugin system for custom parsers
- Web UI, TUI, or GUI
- Windows support (Linux/macOS only in v1)
- Shell hooks (PROMPT_COMMAND, preexec) — explicit `prunify` only
- Scheme marketplace, sharing, or remote registry
- Telemetry, analytics, or learning from usage
- More than 3 built-in schemes in v1
- Deep merge of project overrides with defaults (complete replacement only)
- Caching or logging of command output (potential sensitive data leak)
- Interactive command interception (TTY commands pass through untouched)

---

## Verification Strategy

> **ZERO HUMAN INTERVENTION** — ALL verification is agent-executed. No exceptions.

### Test Decision
- **Infrastructure exists**: NO (greenfield — needs setup)
- **Automated tests**: TDD (tests first)
- **Framework**: Rust built-in `#[test]` + `cargo test` + shell scripts for integration
- **Test setup included in plan**: Yes (Task 3)

### QA Policy
Every task MUST include agent-executed QA scenarios (see TODO template below).
Evidence saved to `.omo/evidence/task-{N}-{scenario-slug}.{ext}`.

- **CLI/Backend**: Use Bash — Run prunify commands, assert exit codes, grep output
- **Rust module**: Use `cargo test` — Unit tests with specific assertions
- **Integration**: Use shell scripts — End-to-end scenarios with `prunify` commands

---

## Execution Strategy

### Parallel Execution Waves

```
Wave 1 (Start Immediately — Foundation + Schema):
├── Task 1: Project scaffolding + Cargo init [quick]
├── Task 2: JSON scheme schema specification (design doc) [quick]
├── Task 3: Test infrastructure setup (cargo test harness + shell test framework) [quick]
├── Task 4: Error type definitions (anyhow + custom error enum) [quick]
├── Task 5: Scheme data types (Rust structs + serde) [quick]
├── Task 6: Config types (.prunifier.yaml schema types) [quick]
└── Task 7: Scheme storage module (read/write/validate scheme files) [quick]

Wave 2 (After Wave 1 — Core Engine, MAX PARALLEL):
├── Task 8: Trie matcher module (TDD, insert + search) [deep]
├── Task 9: Line parser module (split output, apply regex rules) [deep]
├── Task 10: Column selector module (tabular output column keep/discard) [deep]
├── Task 11: Scheme loader (load defaults + project overrides) [quick]
├── Task 12: Built-in schemes: git-status [quick]
├── Task 13: Built-in schemes: ls-la [quick]
├── Task 14: Built-in schemes: ps-aux [quick]
└── Task 15: Config loader (.prunifier.yaml reader) [quick]

Wave 3 (After Wave 2 — Proxy Engine + Integration):
├── Task 16: Command executor (std::process::Command + output capture + exit code) [deep]
├── Task 17: Three-mode dispatcher (exact / prefix / passthrough logic) [deep]
├── Task 18: TTY detector + passthrough [quick]
├── Task 19: Recursion guard (detect prunify self-invocation) [quick]
├── Task 20: CLI entry point (clap argument parsing + main) [unspecified-high]
├── Task 21: Output marking ([PRUNED] insertion logic) [quick]
└── Task 22: Integration tests (end-to-end shell scenarios) [unspecified-high]

Wave 4 (After Wave 3 — Skill + Edge Cases):
├── Task 23: OpenCode skill: SKILL.md (skill metadata + workflow docs) [writing]
├── Task 24: OpenCode skill: Mode-2 workflow (prefix match → subagent optimization) [writing]
├── Task 25: OpenCode skill: Mode-3 workflow (new command → subagent analysis) [writing]
├── Task 26: Edge case: Binary output handling [quick]
├── Task 27: Edge case: ANSI escape code stripping [quick]
├── Task 28: Edge case: Unicode/multibyte support [quick]
├── Task 29: Edge case: Signal passthrough (SIGINT/SIGTERM) [deep]
└── Task 30: Final integration test suite (all modes + all edge cases) [unspecified-high]

Wave FINAL (After ALL tasks — 4 parallel reviews, then user okay):
├── Task F1: Plan Compliance Audit (oracle)
├── Task F2: Code Quality Review (unspecified-high)
├── Task F3: Real Manual QA (unspecified-high)
└── Task F4: Scope Fidelity Check (deep)
-> Present results -> Get explicit user okay

Critical Path: T1 → T5 → T8/T9 → T16/T17 → T20 → T30 → F1-F4 → user okay
Parallel Speedup: ~65% faster than sequential
Max Concurrent: 8 (Wave 2)
```

### Dependency Matrix

- **1-7**: — — 8-15, 1
- **8**: 5 — 17, 2
- **9**: 5 — 17, 2
- **10**: 5 — 17, 2
- **11**: 5, 7 — 17, 2
- **12-14**: 5, 7 — 22, 2
- **15**: 6 — 11, 2
- **16**: — — 17, 20, 3
- **17**: 8, 9, 10, 11 — 20, 3
- **18**: — — 20, 3
- **19**: — — 20, 3
- **20**: 16, 17, 18, 19, 21 — 22, 3
- **21**: 17 — 20, 3
- **22**: 12-14, 20 — 30, 3
- **23-25**: — — —, 4
- **26-29**: 20 — 30, 4
- **30**: 22, 26-29 — F1-F4, 4

### Agent Dispatch Summary

- **Wave 1**: **7** — T1-T7 → `quick`
- **Wave 2**: **8** — T8 → `deep`, T9 → `deep`, T10 → `deep`, T11 → `quick`, T12-T14 → `quick`, T15 → `quick`
- **Wave 3**: **7** — T16 → `deep`, T17 → `deep`, T18-T19 → `quick`, T20 → `unspecified-high`, T21 → `quick`, T22 → `unspecified-high`
- **Wave 4**: **8** — T23-T25 → `writing`, T26-T28 → `quick`, T29 → `deep`, T30 → `unspecified-high`
- **FINAL**: **4** — F1 → `oracle`, F2 → `unspecified-high`, F3 → `unspecified-high`, F4 → `deep`

---

## TODOs

> Implementation + Test = ONE Task. Never separate.
> EVERY task MUST have: Recommended Agent Profile + Parallelization info + QA Scenarios.
> **A task WITHOUT QA Scenarios is INCOMPLETE. No exceptions.**
> **FORMAT**: Task labels MUST use bare numbers: `1.`, `2.`, `3.` — NOT `T1.`, `Task 1.`, `Phase 1:`.
> The /start-work progress counter requires exact format. Deviation = progress shows 0/0.
> Final Verification Wave labels MUST use `F1.`, `F2.`, etc. — NOT `T-F1.`, `F-1.`, `Final 1.`.

- [x] 1. Project scaffolding + Cargo init

  **What to do**:
  - Run `cargo init --name prunifier` in workspace root
  - Add dependencies to `Cargo.toml`: `clap = { version = "4", features = ["derive"] }`, `anyhow = "1"`, `serde = { version = "1", features = ["derive"] }`, `serde_json = "1"`, `regex = "1"`, `thiserror = "2"`
  - Set up directory structure: `src/`, `src/scheme/`, `src/config/`, `src/engine/`, `src/proxy/`, `tests/`
  - Create `.prunifier/schemes/` directory with `.gitkeep`
  - Add `src/main.rs` with minimal `fn main() { println!("prunifier v0.1.0"); }`
  - Add `src/lib.rs` (library root for testability)

  **Must NOT do**:
  - Do NOT add any dependency beyond the 6 listed (clap, anyhow, serde, serde_json, regex, thiserror)
  - Do NOT create `.prunifier.yaml` default file yet (that's Task 6)

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Pure file creation and Cargo setup — mechanical, no complex logic
  - **Skills**: None needed
  - **Skills Evaluated but Omitted**: None

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with Tasks 2-7)
  - **Blocks**: Tasks 2-30 (foundation for everything)
  - **Blocked By**: None (can start immediately)

  **References**:
  - Official docs: `https://doc.rust-lang.org/cargo/commands/cargo-init.html` — Cargo init command reference
  - Official docs: `https://docs.rs/clap/latest/clap/_derive/_tutorial/index.html` — Clap derive API for CLI args
  - Official docs: `https://docs.rs/anyhow/latest/anyhow/` — anyhow error handling
  - Official docs: `https://docs.rs/serde/latest/serde/derive.Deserialize.html` — Serde derive for JSON parsing

  **Acceptance Criteria**:
  - [ ] `cargo build` succeeds (produces `target/debug/prunifier` binary)
  - [ ] `cargo run` outputs "prunifier v0.1.0"
  - [ ] `ls .prunifier/schemes/` shows `.gitkeep`
  - [ ] All 6 dependencies present in `Cargo.toml` under `[dependencies]`

  **QA Scenarios**:

  ```
  Scenario: Cargo build succeeds on clean checkout
    Tool: Bash
    Preconditions: Workspace root, no target/ directory
    Steps:
      1. Run: cargo build 2>&1
      2. Assert: exit code is 0
      3. Assert: stderr does NOT contain "error" (case insensitive)
      4. Assert: target/debug/prunifier binary exists
    Expected Result: Binary compiles with zero errors
    Failure Indicators: Build fails, error messages in output
    Evidence: .omo/evidence/task-1-build-success.txt

  Scenario: Binary outputs version string
    Tool: Bash
    Preconditions: cargo build succeeded
    Steps:
      1. Run: cargo run 2>&1
      2. Assert: stdout contains "prunifier v0.1.0"
    Expected Result: Version string printed to stdout
    Failure Indicators: Wrong output, crash on startup
    Evidence: .omo/evidence/task-1-version-output.txt
  ```

  **Evidence to Capture**:
  - [ ] `task-1-build-success.txt` — cargo build output
  - [ ] `task-1-version-output.txt` — cargo run output

  **Commit**: YES (groups with Tasks 2-7 in Wave 1)
  - Message: `feat(prunifier): project scaffolding and cargo init`
  - Files: `Cargo.toml`, `Cargo.lock`, `src/main.rs`, `src/lib.rs`, `.prunifier/schemes/.gitkeep`
  - Pre-commit: `cargo build`

- [x] 2. JSON scheme schema specification

  **What to do**:
  - Create `SCHEMA.md` documenting the JSON scheme format with examples
  - Define JSON Schema (the meta-schema that validates scheme files) as `src/scheme/schema.json`
  - Design the scheme structure: `{ "command": string, "version": 1, "rules": [...] }`
  - Each rule: `{ "action": "keep"|"discard", "match": { "type": "regex"|"column"|"line_number", "pattern": string, "column_index": number? } }`
  - Document column-based selector for tabular output: specify `separator` (regex for column split), rules can reference `column_index`
  - Write examples: prune total line, prune `.`/`..` entries, keep only PID+COMMAND from ps aux
  - Write a JSON Schema validator test that rejects invalid schemes

  **Must NOT do**:
  - Do NOT implement the Rust parser yet (that's Task 5)
  - Do NOT create scheme files for commands yet (that's Tasks 12-14)
  - Do NOT design tree-based/AST selectors — line-based only per v1 guardrail

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Pure documentation + JSON Schema definition — no Rust implementation
  - **Skills**: None needed
  - **Skills Evaluated but Omitted**: None

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with Tasks 1, 3-7)
  - **Blocks**: Tasks 5, 12-14 (scheme types and built-in schemes depend on schema)
  - **Blocked By**: Task 1 (Cargo init for file location)

  **References**:
  - Official docs: `https://json-schema.org/understanding-json-schema/` — JSON Schema specification
  - Pattern: `ls -la` output format — columns: permissions, links, owner, group, size, date, time, name

  **Acceptance Criteria**:
  - [ ] `SCHEMA.md` exists with clear examples for regex and column rules
  - [ ] `src/scheme/schema.json` is valid JSON Schema (passes any JSON Schema validator)
  - [ ] Documented example: `git status` pruning rule (strip "On branch" and blank lines, keep file lists)
  - [ ] Documented example: `ls -la` pruning rule (strip "total", strip `.`/`..` entries)
  - [ ] Documented example: `ps aux` pruning rule (keep PID + COMMAND columns only)

  **QA Scenarios**:

  ```
  Scenario: Schema file is valid JSON Schema
    Tool: Bash
    Preconditions: src/scheme/schema.json exists
    Steps:
      1. Run: python3 -c "import json; json.load(open('src/scheme/schema.json')); print('VALID')"
      2. Assert: output contains "VALID"
      3. Run: cat src/scheme/schema.json | python3 -c "import json,sys; s=json.load(sys.stdin); assert '\$schema' in s or 'type' in s; print('HAS_SCHEMA')"
      4. Assert: output contains "HAS_SCHEMA"
    Expected Result: Valid JSON with type definitions
    Failure Indicators: Invalid JSON, missing type field
    Evidence: .omo/evidence/task-2-schema-valid.json

  Scenario: SCHEMA.md documents all required fields
    Tool: Bash
    Preconditions: SCHEMA.md exists
    Steps:
      1. Run: grep -c "command" SCHEMA.md
      2. Assert: count >= 1
      3. Run: grep -c "rules" SCHEMA.md
      4. Assert: count >= 1
      5. Run: grep -c "action" SCHEMA.md
      6. Assert: count >= 1
    Expected Result: All key fields documented
    Failure Indicators: Missing documentation for core fields
    Evidence: .omo/evidence/task-2-schema-docs.txt
  ```

  **Evidence to Capture**:
  - [ ] `task-2-schema-valid.json` — schema validation output
  - [ ] `task-2-schema-docs.txt` — grep results from SCHEMA.md

  **Commit**: YES (Wave 1 group)
  - Message: `feat(prunifier): JSON scheme schema specification`
  - Files: `SCHEMA.md`, `src/scheme/schema.json`
  - Pre-commit: `python3 -c "import json; json.load(open('src/scheme/schema.json'))"`

- [x] 3. Test infrastructure setup

  **What to do**:
  - Create `tests/integration_test.rs` with first test: `#[test] fn test_prunify_echo()`
  - Create `tests/shell_tests.sh` — bash script for integration scenarios
  - Add test helper module `tests/common/mod.rs` with utility functions (temp_dir, scheme_fixture, run_prunify)
  - Configure `Cargo.toml` with `[[test]]` section if needed
  - Write first TDD test: `test_scheme_validator_rejects_invalid_json()` in `tests/integration_test.rs` (should FAIL — expected, implementation comes later)
  - Add `cargo test` and `./tests/shell_tests.sh` to CI verification commands

  **Must NOT do**:
  - Do NOT implement the validator yet — this is TDD, tests should fail initially
  - Do NOT add external test frameworks (just Rust's built-in `#[test]`)

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: File creation and test scaffolding — straightforward setup
  - **Skills**: None needed
  - **Skills Evaluated but Omitted**: None

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with Tasks 1-2, 4-7)
  - **Blocks**: Tasks 5, 8-10, 16-17, 22, 30 (tests validate implementations)
  - **Blocked By**: Task 1 (directory structure)

  **References**:
  - Official docs: `https://doc.rust-lang.org/book/ch11-01-writing-tests.html` — Rust test writing
  - Official docs: `https://doc.rust-lang.org/cargo/reference/cargo-targets.html#tests` — Cargo test targets

  **Acceptance Criteria**:
  - [ ] `cargo test` runs and shows 1 test (FAIL — expected as TDD)
  - [ ] `tests/shell_tests.sh` is executable (`chmod +x`) and runs without syntax errors
  - [ ] `tests/common/mod.rs` exports `run_prunify()` and `scheme_fixture()` functions
  - [ ] `tests/integration_test.rs` has `mod common;` import

  **QA Scenarios**:

  ```
  Scenario: Cargo test discovers integration tests
    Tool: Bash
    Preconditions: test files exist
    Steps:
      1. Run: cargo test --test integration_test 2>&1
      2. Assert: output contains "running 1 test" (the TDD failing test)
      3. Assert: exit code != 0 (test fails — expected for TDD)
    Expected Result: Test framework discovers and runs the test (failing is OK)
    Failure Indicators: "error: no test target named", compilation error
    Evidence: .omo/evidence/task-3-test-discovery.txt

  Scenario: Shell test script is executable
    Tool: Bash
    Preconditions: tests/shell_tests.sh exists
    Steps:
      1. Run: test -x tests/shell_tests.sh && echo "EXECUTABLE" || echo "NOT_EXECUTABLE"
      2. Assert: output is "EXECUTABLE"
    Expected Result: Shell script has execute permission
    Failure Indicators: "NOT_EXECUTABLE"
    Evidence: .omo/evidence/task-3-shell-exec.txt
  ```

  **Evidence to Capture**:
  - [ ] `task-3-test-discovery.txt` — cargo test output
  - [ ] `task-3-shell-exec.txt` — permission check result

  **Commit**: YES (Wave 1 group)
  - Message: `test(prunifier): test infrastructure setup with TDD skeleton`
  - Files: `tests/integration_test.rs`, `tests/shell_tests.sh`, `tests/common/mod.rs`
  - Pre-commit: `cargo test 2>&1 | head -20`

- [x] 4. Error type definitions

  **What to do**:
  - Create `src/error.rs` with `PrunifierError` enum using `thiserror`
  - Variants: `SchemeNotFound(String)`, `InvalidScheme(String)`, `CommandFailed(String, i32)`, `IoError(#[from] std::io::Error)`, `JsonError(#[from] serde_json::Error)`, `RegexError(#[from] regex::Error)`, `ConfigError(String)`, `RecursionDetected`
  - Implement `std::fmt::Display` via `thiserror` derive
  - Add type alias `pub type PrunifierResult<T> = Result<T, PrunifierError>`
  - Re-export from `src/lib.rs`: `pub mod error; pub use error::{PrunifierError, PrunifierResult};`

  **Must NOT do**:
  - Do NOT add any variant not listed above
  - Do NOT implement custom Display or Error manually (use thiserror derive only)

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Single-file enum definition with derive macros — trivial
  - **Skills**: None needed
  - **Skills Evaluated but Omitted**: None

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with Tasks 1-3, 5-7)
  - **Blocks**: Tasks 5-30 (error type used everywhere)
  - **Blocked By**: Task 1 (project structure)

  **References**:
  - Official docs: `https://docs.rs/thiserror/latest/thiserror/` — thiserror derive macro syntax

  **Acceptance Criteria**:
  - [ ] `cargo check` passes (no compilation errors)
  - [ ] `PrunifierError` has exactly 8 variants as listed
  - [ ] `PrunifierResult<T>` type alias compiles
  - [ ] Error types re-exported from `src/lib.rs`

  **QA Scenarios**:

  ```
  Scenario: Error enum compiles and derives Debug/Display
    Tool: Bash
    Preconditions: src/error.rs exists
    Steps:
      1. Run: cargo check 2>&1
      2. Assert: exit code is 0
      3. Run: cargo doc --no-deps -p prunifier 2>&1
      4. Assert: exit code is 0 (docs generate without errors)
    Expected Result: Error types compile cleanly
    Failure Indicators: Compilation errors, missing derives
    Evidence: .omo/evidence/task-4-error-compile.txt
  ```

  **Evidence to Capture**:
  - [ ] `task-4-error-compile.txt` — cargo check output

  **Commit**: YES (Wave 1 group)
  - Message: `feat(prunifier): error type definitions with thiserror`
  - Files: `src/error.rs`, `src/lib.rs`
  - Pre-commit: `cargo check`

- [x] 5. Scheme data types (Rust structs + serde)

  **What to do** (TDD — write test first, see it fail, then implement):
  - **RED**: Write `tests/scheme_types_test.rs` with tests: `test_deserialize_valid_scheme()`, `test_reject_invalid_action()`, `test_reject_missing_command()`, `test_column_rule_requires_index()`
  - **GREEN**: Create `src/scheme/types.rs` with `Scheme`, `Rule`, `MatchCondition`, `Action` structs/enums
  - `Scheme { command: String, version: u32, rules: Vec<Rule> }`
  - `Rule { action: Action, match_condition: MatchCondition, description: Option<String> }`
  - `Action` enum: `Keep`, `Discard`
  - `MatchCondition` enum: `Regex { pattern: String }`, `Column { index: usize, pattern: String }`, `LineNumber { lines: Vec<usize> }`
  - Implement `serde::Deserialize` for all types via `#[derive(Deserialize)]`
  - Implement `Scheme::validate()` method that checks rules are non-empty and patterns are valid regex
  - Re-export from `src/scheme/mod.rs`

  **Must NOT do**:
  - Do NOT implement the JSON schema validator yet (that's the Scheme storage module in Task 7)
  - Do NOT add custom Deserialize implementations (use derive only)

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Straightforward Rust struct/enum definitions with serde derives
  - **Skills**: None needed
  - **Skills Evaluated but Omitted**: None

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with Tasks 1-4, 6-7)
  - **Blocks**: Tasks 7-14, 17 (everything that reads/writes schemes)
  - **Blocked By**: Task 1 (project structure), Task 2 (schema spec), Task 3 (test harness)

  **References**:
  - Official docs: `https://docs.rs/serde/latest/serde/derive.Deserialize.html` — Serde Deserialize derive
  - Official docs: `https://docs.rs/regex/latest/regex/struct.Regex.html` — Regex::new() for validation
  - Spec: `SCHEMA.md` from Task 2 — Schema structure reference

  **Acceptance Criteria** (if TDD):
  - [ ] `cargo test tests::scheme_types_test` → all 4 tests PASS
  - [ ] `Scheme::validate()` returns Ok for valid scheme, Err for invalid
  - [ ] Valid JSON: `{"command":"ls","version":1,"rules":[{"action":"keep","match_condition":{"type":"Regex","pattern":"README"}}]}` deserializes without error
  - [ ] Invalid JSON: missing `command` field → deserialize fails

  **QA Scenarios**:

  ```
  Scenario: Valid scheme JSON deserializes correctly
    Tool: Bash (cargo test)
    Preconditions: tests/scheme_types_test.rs exists
    Steps:
      1. Run: cargo test test_deserialize_valid_scheme -- --nocapture 2>&1
      2. Assert: test passes (exit code 0 from test binary)
      3. Assert: test output does NOT contain "FAILED"
    Expected Result: Valid JSON scheme parses into Scheme struct
    Failure Indicators: Test FAILED, panic in deserialization
    Evidence: .omo/evidence/task-5-scheme-types-test.txt

  Scenario: Invalid scheme JSON is rejected
    Tool: Bash (cargo test)
    Preconditions: tests/scheme_types_test.rs exists
    Steps:
      1. Run: cargo test test_reject_invalid_action test_reject_missing_command -- --nocapture 2>&1
      2. Assert: both tests pass
    Expected Result: Invalid schemes fail to deserialize
    Failure Indicators: Tests FAIL, invalid data accepted silently
    Evidence: .omo/evidence/task-5-scheme-rejection-test.txt
  ```

  **Evidence to Capture**:
  - [ ] `task-5-scheme-types-test.txt` — cargo test output for valid scheme
  - [ ] `task-5-scheme-rejection-test.txt` — cargo test output for invalid scheme

  **Commit**: YES (Wave 1 group)
  - Message: `feat(prunifier): scheme data types with serde deserialization`
  - Files: `src/scheme/types.rs`, `src/scheme/mod.rs`, `tests/scheme_types_test.rs`
  - Pre-commit: `cargo test tests::scheme_types_test`

- [x] 6. Config types (.prunifier.yaml schema types)

  **What to do**:
  - Create `src/config/types.rs` with `PrunifierConfig` struct
  - Fields: `scheme_dir: Option<PathBuf>` (default: `.prunifier/schemes/`), `verbose: Option<bool>`, `no_color: Option<bool>`, `strict: Option<bool>` (reject unknown commands instead of passthrough)
  - Implement `serde::Deserialize` with `#[serde(default)]` for optional fields
  - Implement `Default` for `PrunifierConfig` with sensible defaults
  - Add `PrunifierConfig::load()` placeholder (returns default — real loading in Task 15)
  - Create `tests/config_types_test.rs`: test deserialize minimal yaml, test defaults, test invalid field ignored
  - Add `serde_yaml = "0.9"` to Cargo.toml

  **Must NOT do**:
  - Do NOT implement file reading from disk yet (that's Task 15)
  - Do NOT add more than 4 config fields

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Struct definition with serde — similar to Task 5, straightforward
  - **Skills**: None needed
  - **Skills Evaluated but Omitted**: None

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with Tasks 1-5, 7)
  - **Blocks**: Task 15 (config loader uses these types)
  - **Blocked By**: Task 1 (project structure)

  **References**:
  - Official docs: `https://docs.rs/serde_yaml/latest/serde_yaml/` — serde_yaml usage
  - Pattern: `.prunifier.yaml` — top-level YAML config file in project root

  **Acceptance Criteria**:
  - [ ] `cargo test tests::config_types_test` → all tests PASS
  - [ ] YAML `scheme_dir: ./custom-schemes` deserializes correctly
  - [ ] Empty YAML `{}` uses all defaults
  - [ ] `PrunifierConfig::default().scheme_dir` is `None` (meaning use built-in default)

  **QA Scenarios**:

  ```
  Scenario: Config types compile and deserialize
    Tool: Bash (cargo test)
    Preconditions: tests/config_types_test.rs exists
    Steps:
      1. Run: cargo test config_types_test -- --nocapture 2>&1
      2. Assert: all tests pass
    Expected Result: Config types work correctly
    Failure Indicators: Test FAILED, deserialization error
    Evidence: .omo/evidence/task-6-config-types-test.txt
  ```

  **Evidence to Capture**:
  - [ ] `task-6-config-types-test.txt` — cargo test output

  **Commit**: YES (Wave 1 group)
  - Message: `feat(prunifier): config types for .prunifier.yaml`
  - Files: `src/config/types.rs`, `src/config/mod.rs`, `tests/config_types_test.rs`, `Cargo.toml`
  - Pre-commit: `cargo test config_types_test`

- [x] 7. Scheme storage module (read/write/validate scheme files)

  **What to do** (TDD):
  - **RED**: Write `tests/scheme_storage_test.rs` with tests for: `test_load_valid_scheme()`, `test_load_missing_file()`, `test_load_invalid_json()`, `test_load_scheme_wrong_version()`
  - **GREEN**: Create `src/scheme/storage.rs` with `SchemeStorage` struct
  - Methods: `load(path: &Path) -> PrunifierResult<Scheme>`, `load_all(dir: &Path) -> PrunifierResult<Vec<Scheme>>`, `validate_scheme_file(path: &Path) -> PrunifierResult<()>`
  - `load_all`: reads all `.json` files from directory, skips non-JSON files, collects valid schemes
  - `validate_scheme_file`: checks file exists, is valid JSON, conforms to scheme schema
  - Error on: file not found → `SchemeNotFound`, invalid JSON → `InvalidScheme`, wrong version → `InvalidScheme`
  - Add test fixture: `tests/fixtures/valid-scheme.json`, `tests/fixtures/invalid-schema.json`, `tests/fixtures/empty-dir/`

  **Must NOT do**:
  - Do NOT scan recursively into subdirectories (flat `.prunifier/schemes/` only)
  - Do NOT implement `.prunifier.yaml` reading (that's Task 15)

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: File I/O + JSON parsing — well-defined, mechanical
  - **Skills**: None needed
  - **Skills Evaluated but Omitted**: None

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with Tasks 1-6)
  - **Blocks**: Tasks 11-14, 17 (scheme loading for proxy)
  - **Blocked By**: Task 5 (Scheme type must exist)

  **References**:
  - Official docs: `https://doc.rust-lang.org/std/fs/fn.read_to_string.html` — std::fs file reading
  - Official docs: `https://docs.rs/serde_json/latest/serde_json/fn.from_str.html` — serde_json deserialization
  - Pattern: `src/scheme/types.rs:Scheme` — Scheme type from Task 5

  **Acceptance Criteria** (if TDD):
  - [ ] `cargo test tests::scheme_storage_test` → all 4 tests PASS
  - [ ] `SchemeStorage::load()` on valid fixture returns Ok(Scheme)
  - [ ] `SchemeStorage::load()` on missing file returns Err(SchemeNotFound)
  - [ ] `SchemeStorage::load_all()` on directory with 3 scheme files returns 3 schemes
  - [ ] `SchemeStorage::load_all()` on directory with non-JSON files skips them

  **QA Scenarios**:

  ```
  Scenario: Load valid scheme from file
    Tool: Bash (cargo test)
    Preconditions: tests/fixtures/valid-scheme.json exists
    Steps:
      1. Run: cargo test test_load_valid_scheme -- --nocapture 2>&1
      2. Assert: test passes
    Expected Result: Valid scheme file loads successfully
    Failure Indicators: Test FAILED
    Evidence: .omo/evidence/task-7-storage-load-test.txt

  Scenario: Missing file returns error
    Tool: Bash (cargo test)
    Preconditions: tests/fixtures/nonexistent.json does NOT exist
    Steps:
      1. Run: cargo test test_load_missing_file -- --nocapture 2>&1
      2. Assert: test passes (returns Err)
    Expected Result: Missing file produces SchemeNotFound error
    Failure Indicators: Test FAILED (panic on missing file)
    Evidence: .omo/evidence/task-7-storage-missing-test.txt
  ```

  **Evidence to Capture**:
  - [ ] `task-7-storage-load-test.txt` — valid load test output
  - [ ] `task-7-storage-missing-test.txt` — missing file test output

  **Commit**: YES (Wave 1 group)
  - Message: `feat(prunifier): scheme storage module with file loading`
  - Files: `src/scheme/storage.rs`, `tests/scheme_storage_test.rs`, `tests/fixtures/valid-scheme.json`, `tests/fixtures/invalid-schema.json`
  - Pre-commit: `cargo test scheme_storage_test`

- [x] 8. Trie matcher module (TDD: insert + search)

  **What to do** (TDD):
  - **RED**: Write `tests/trie_test.rs`: `test_insert_and_exact_match()`, `test_prefix_match()`, `test_no_match()`, `test_longest_prefix()`, `test_multiple_commands()`, `test_empty_trie()`
  - **GREEN**: Create `src/engine/trie.rs` with `CommandTrie` struct
  - Implement a trie where each node represents a command token (words split by whitespace)
  - `insert(command: &str, scheme_id: &str)` — tokenize and store
  - `search_exact(command: &str) -> Option<&str>` — exact full-command match
  - `search_prefix(command: &str) -> Option<(&str, usize)>` — longest common prefix match, returns (scheme_id, matched_tokens_count)
  - Tokenization: split command string by whitespace, preserve order
  - The trie root has children keyed by the first token

  **Must NOT do**:
  - Do NOT use external trie crate unless absolutely necessary (implement custom — it's ~100 lines)
  - Do NOT handle argument normalization or flag reordering (exact token sequence only)
  - Do NOT implement fuzzy matching or edit distance

  **Recommended Agent Profile**:
  - **Category**: `deep`
    - Reason: Custom trie data structure implementation requires careful design and correctness
  - **Skills**: None needed
  - **Skills Evaluated but Omitted**: None

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2 (with Tasks 9-15)
  - **Blocks**: Task 17 (dispatcher uses trie for mode selection)
  - **Blocked By**: Task 5 (Scheme type for scheme_id reference)

  **References**:
  - Pattern: Trie data structure — each node has `children: HashMap<String, TrieNode>` and optional `scheme_id`
  - Pattern: Longest prefix match — walk trie until no child matches, return last node with scheme_id

  **Acceptance Criteria** (if TDD):
  - [ ] `cargo test tests::trie_test` → all 6 tests PASS
  - [ ] `search_exact("git status")` returns Some after inserting "git status"
  - [ ] `search_prefix("git status --porcelain")` returns Some with matched_tokens=2 after inserting "git status"
  - [ ] `search_exact("docker ps")` returns None if only "git" commands inserted
  - [ ] `search_prefix("ls")` returns the scheme for "ls -la" with matched_tokens=1

  **QA Scenarios**:

  ```
  Scenario: Exact match returns correct scheme
    Tool: Bash (cargo test)
    Preconditions: tests/trie_test.rs exists
    Steps:
      1. Run: cargo test test_insert_and_exact_match -- --nocapture 2>&1
      2. Assert: test passes
    Expected Result: Trie correctly stores and retrieves exact command matches
    Failure Indicators: Test FAILED
    Evidence: .omo/evidence/task-8-trie-exact-test.txt

  Scenario: Prefix match returns scheme and match depth
    Tool: Bash (cargo test)
    Preconditions: tests/trie_test.rs exists
    Steps:
      1. Run: cargo test test_longest_prefix -- --nocapture 2>&1
      2. Assert: test passes, matched_tokens_count > 0
    Expected Result: Trie finds longest common prefix
    Failure Indicators: Test FAILED, wrong match depth
    Evidence: .omo/evidence/task-8-trie-prefix-test.txt
  ```

  **Evidence to Capture**:
  - [ ] `task-8-trie-exact-test.txt` — exact match test output
  - [ ] `task-8-trie-prefix-test.txt` — prefix match test output

  **Commit**: YES (Wave 2 group)
  - Message: `feat(prunifier): trie matcher for command prefix matching`
  - Files: `src/engine/trie.rs`, `src/engine/mod.rs`, `tests/trie_test.rs`
  - Pre-commit: `cargo test trie_test`

- [x] 9. Line parser module (split output, apply regex rules)

  **What to do** (TDD):
  - **RED**: Write `tests/line_parser_test.rs`: `test_keep_lines_matching_regex()`, `test_discard_lines_matching_regex()`, `test_multiple_rules_apply_in_order()`, `test_empty_output()`, `test_no_matching_rules_keeps_all()`
  - **GREEN**: Create `src/engine/line_parser.rs` with `LineParser` struct
  - `apply_rules(output: &str, rules: &[Rule]) -> PrunifierResult<String>` — split by newline, apply each rule in order, return pruned string
  - Rule application: for `Action::Keep`, drop lines that DON'T match; for `Action::Discard`, drop lines that DO match
  - Handle empty lines (preserve unless explicitly discarded)
  - Preserve trailing newline behavior of original output
  - Support `Regex` and `LineNumber` match conditions

  **Must NOT do**:
  - Do NOT handle Column selectors in this module (that's Task 10)
  - Do NOT strip ANSI codes here (that's Task 27)
  - Do NOT modify the original output's line endings beyond pruning

  **Recommended Agent Profile**:
  - **Category**: `deep`
    - Reason: String processing with regex, edge cases around newlines and empty output
  - **Skills**: None needed
  - **Skills Evaluated but Omitted**: None

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2 (with Tasks 8, 10-15)
  - **Blocks**: Task 17 (dispatcher uses parser for pruning)
  - **Blocked By**: Task 5 (Rule/Action types)

  **References**:
  - Official docs: `https://docs.rs/regex/latest/regex/struct.Regex.html` — Regex methods
  - Pattern: `src/scheme/types.rs:Rule` — Rule type from Task 5

  **Acceptance Criteria** (if TDD):
  - [ ] `cargo test tests::line_parser_test` → all 5 tests PASS
  - [ ] Input "line1\nline2\nline3" with discard rule matching "line2" → output is "line1\nline3"
  - [ ] Input with no matching rules → output unchanged
  - [ ] Empty input → empty output (no crash)
  - [ ] Rules apply in order: first rule runs against full output, second against result of first

  **QA Scenarios**:

  ```
  Scenario: Discard rule removes matching lines
    Tool: Bash (cargo test)
    Preconditions: tests/line_parser_test.rs exists
    Steps:
      1. Run: cargo test test_discard_lines_matching_regex -- --nocapture 2>&1
      2. Assert: test passes, pruned output lacks discarded lines
    Expected Result: Lines matching discard regex are removed
    Failure Indicators: Test FAILED, lines not removed
    Evidence: .omo/evidence/task-9-line-parser-discard.txt

  Scenario: Multiple rules chain correctly
    Tool: Bash (cargo test)
    Preconditions: tests/line_parser_test.rs exists
    Steps:
      1. Run: cargo test test_multiple_rules_apply_in_order -- --nocapture 2>&1
      2. Assert: test passes, output reflects chained pruning
    Expected Result: Rules apply sequentially
    Failure Indicators: Test FAILED, rules applied in wrong order
    Evidence: .omo/evidence/task-9-line-parser-chain.txt
  ```

  **Evidence to Capture**:
  - [ ] `task-9-line-parser-discard.txt` — discard test output
  - [ ] `task-9-line-parser-chain.txt` — chained rules test output

  **Commit**: YES (Wave 2 group)
  - Message: `feat(prunifier): line parser with regex-based pruning rules`
  - Files: `src/engine/line_parser.rs`, `tests/line_parser_test.rs`
  - Pre-commit: `cargo test line_parser_test`

- [x] 10. Column selector module (tabular output pruning)

  **What to do** (TDD):
  - **RED**: Write `tests/column_selector_test.rs`: `test_keep_specific_columns()`, `test_discard_specific_columns()`, `test_whitespace_separator()`, `test_variable_column_count()`, `test_column_index_out_of_bounds()`
  - **GREEN**: Create `src/engine/column_selector.rs` with `ColumnSelector` struct
  - `apply_rules(output: &str, rules: &[Rule]) -> PrunifierResult<String>` — for each line, split by separator, apply column-based keep/discard
  - Default separator: whitespace (one or more spaces/tabs)
  - For `Action::Keep` with `Column` condition: keep only specified column
  - For `Action::Discard` with `Column` condition: remove specified column, rejoin remaining
  - Handle variable-width columns gracefully (line with fewer columns than index → keep line as-is)
  - Preserve column alignment where possible (pad to original width)

  **Must NOT do**:
  - Do NOT handle regex-based line selection here (that's Task 9)
  - Do NOT attempt to reflow or rewrap text

  **Recommended Agent Profile**:
  - **Category**: `deep`
    - Reason: Column-based parsing with alignment preservation and edge cases
  - **Skills**: None needed
  - **Skills Evaluated but Omitted**: None

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2 (with Tasks 8-9, 11-15)
  - **Blocks**: Task 17 (dispatcher uses column selector for tabular output)
  - **Blocked By**: Task 5 (Rule/Action types)

  **References**:
  - Official docs: `https://doc.rust-lang.org/std/primitive.str.html#method.split_whitespace` — str::split_whitespace
  - Pattern: `ls -la` output — space-separated columns with variable widths
  - Pattern: `ps aux` output — space-separated columns, COMMAND column may contain spaces

  **Acceptance Criteria** (if TDD):
  - [ ] `cargo test tests::column_selector_test` → all 5 tests PASS
  - [ ] Input "a b c\n1 2 3" with keep column 0 → output "a\n1"
  - [ ] Input "a b c\n1 2 3" with discard column 1 → output "a c\n1 3"
  - [ ] Line with fewer columns than index → line preserved unchanged (no panic)

  **QA Scenarios**:

  ```
  Scenario: Keep specific column from tabular output
    Tool: Bash (cargo test)
    Preconditions: tests/column_selector_test.rs exists
    Steps:
      1. Run: cargo test test_keep_specific_columns -- --nocapture 2>&1
      2. Assert: test passes, output contains only specified column
    Expected Result: Only specified column is kept
    Failure Indicators: Test FAILED, wrong columns in output
    Evidence: .omo/evidence/task-10-column-keep.txt

  Scenario: Out-of-bounds column index handled gracefully
    Tool: Bash (cargo test)
    Preconditions: tests/column_selector_test.rs exists
    Steps:
      1. Run: cargo test test_column_index_out_of_bounds -- --nocapture 2>&1
      2. Assert: test passes, no panic, line preserved
    Expected Result: Lines with fewer columns are preserved, not dropped
    Failure Indicators: Test FAILED due to panic
    Evidence: .omo/evidence/task-10-column-oob.txt
  ```

  **Evidence to Capture**:
  - [ ] `task-10-column-keep.txt` — column keep test output
  - [ ] `task-10-column-oob.txt` — out-of-bounds test output

  **Commit**: YES (Wave 2 group)
  - Message: `feat(prunifier): column selector for tabular output pruning`
  - Files: `src/engine/column_selector.rs`, `tests/column_selector_test.rs`
  - Pre-commit: `cargo test column_selector_test`

- [x] 11. Scheme loader (load defaults + project overrides)

  **What to do** (TDD):
  - **RED**: Write `tests/scheme_loader_test.rs`: `test_load_defaults()`, `test_project_override_replaces_default()`, `test_no_project_config_uses_defaults()`, `test_empty_scheme_dir()`
  - **GREEN**: Create `src/scheme/loader.rs` with `SchemeLoader` struct
  - `new(default_dir: PathBuf) -> Self` — sets default schemes directory
  - `load(config: &PrunifierConfig) -> PrunifierResult<HashMap<String, Scheme>>` — loads all schemes
  - Priority: project override (from `config.scheme_dir`) completely replaces default for same command
  - Return map keyed by command string (e.g., "git status", "ls -la")
  - Use `SchemeStorage::load_all()` from Task 7 for file reading
  - Handle case where neither default nor project dir has schemes (empty map is OK)

  **Must NOT do**:
  - Do NOT deep merge project overrides with defaults (complete replacement per guardrail)
  - Do NOT handle `.prunifier.yaml` reading here (use PrunifierConfig passed in)

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Orchestration layer over existing SchemeStorage — straightforward composition
  - **Skills**: None needed
  - **Skills Evaluated but Omitted**: None

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2 (with Tasks 8-10, 12-15)
  - **Blocks**: Task 17 (dispatcher uses loaded schemes)
  - **Blocked By**: Tasks 5, 7 (Scheme type + storage), Task 6 (PrunifierConfig type)

  **References**:
  - Pattern: `src/scheme/storage.rs:SchemeStorage` — Storage from Task 7
  - Pattern: `src/config/types.rs:PrunifierConfig` — Config type from Task 6

  **Acceptance Criteria** (if TDD):
  - [ ] `cargo test tests::scheme_loader_test` → all 4 tests PASS
  - [ ] Loading defaults from dir with 3 scheme files → map has 3 entries
  - [ ] Project override for "git status" replaces default "git status" scheme
  - [ ] Default for "ls -la" still present when only "git status" is overridden

  **QA Scenarios**:

  ```
  Scenario: Project override replaces default scheme
    Tool: Bash (cargo test)
    Preconditions: tests/scheme_loader_test.rs exists
    Steps:
      1. Run: cargo test test_project_override_replaces_default -- --nocapture 2>&1
      2. Assert: test passes, project version used
    Expected Result: Project scheme takes precedence over default
    Failure Indicators: Test FAILED, default scheme used instead of override
    Evidence: .omo/evidence/task-11-loader-override.txt

  Scenario: No project config uses defaults only
    Tool: Bash (cargo test)
    Preconditions: tests/scheme_loader_test.rs exists
    Steps:
      1. Run: cargo test test_no_project_config_uses_defaults -- --nocapture 2>&1
      2. Assert: test passes, all default schemes loaded
    Expected Result: Default schemes loaded when no project config
    Failure Indicators: Test FAILED, empty scheme map
    Evidence: .omo/evidence/task-11-loader-defaults.txt
  ```

  **Evidence to Capture**:
  - [ ] `task-11-loader-override.txt` — override test output
  - [ ] `task-11-loader-defaults.txt` — defaults test output

  **Commit**: YES (Wave 2 group)
  - Message: `feat(prunifier): scheme loader with default + override priority`
  - Files: `src/scheme/loader.rs`, `tests/scheme_loader_test.rs`
  - Pre-commit: `cargo test scheme_loader_test`

- [x] 12. Built-in scheme: git-status

  **What to do**:
  - Create `.prunifier/schemes/git-status.json` with pruning rules for `git status` output
  - Parse `git status` output format: section headers ("On branch X", "Changes not staged", etc.), blank lines, file lists
  - Rules: discard "On branch" line, discard blank lines, discard "nothing to commit" messages, keep only file change lines (modified/deleted/renamed/untracked)
  - Add `description` field on each rule explaining what it does
  - Write test in `tests/scheme_loader_test.rs` or new `tests/builtin_schemes_test.rs` that validates the scheme loads and applies to sample `git status` output

  **Must NOT do**:
  - Do NOT attempt to cover all `git status` variants (--porcelain, --short) — v1 covers default output only
  - Do NOT modify the scheme format (use schema from Task 2)

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Creating a JSON file with regex patterns — no Rust code required
  - **Skills**: None needed
  - **Skills Evaluated but Omitted**: None

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2 (with Tasks 8-11, 13-15)
  - **Blocks**: Task 22 (integration tests use built-in schemes)
  - **Blocked By**: Tasks 5, 7 (Scheme types + storage for validation)

  **References**:
  - Pattern: `git status` default output — sections: branch info, staged changes, unstaged changes, untracked files
  - Spec: `SCHEMA.md` from Task 2 — scheme format specification

  **Acceptance Criteria**:
  - [ ] `.prunifier/schemes/git-status.json` is valid JSON per `src/scheme/schema.json`
  - [ ] Scheme has at least 3 rules (discard branch line, discard blank lines, keep file changes)
  - [ ] Sample `git status` output with 3 modified files → pruned output shows only the 3 file lines
  - [ ] Sample `git status` output with clean working tree → pruned output is empty (or single "clean" line)

  **QA Scenarios**:

  ```
  Scenario: git-status scheme prunes branch header
    Tool: Bash
    Preconditions: cargo build succeeded, git-status scheme exists
    Steps:
      1. Create test: echo -e "On branch main\n\nChanges not staged for commit:\n\tmodified: src/main.rs\n\tmodified: README.md" > /tmp/test-git-status.txt
      2. Run: (implementation test — verify scheme JSON is valid and rules match)
      3. Run: python3 -c "import json; s=json.load(open('.prunifier/schemes/git-status.json')); assert len(s['rules'])>=3; print('OK')"
      4. Assert: output is "OK"
    Expected Result: Scheme has at least 3 rules and is valid JSON
    Failure Indicators: Invalid JSON, fewer than 3 rules
    Evidence: .omo/evidence/task-12-git-status-scheme.txt
  ```

  **Evidence to Capture**:
  - [ ] `task-12-git-status-scheme.txt` — scheme validation output

  **Commit**: YES (Wave 2 group)
  - Message: `feat(prunifier): built-in scheme for git status`
  - Files: `.prunifier/schemes/git-status.json`
  - Pre-commit: `python3 -c "import json; json.load(open('.prunifier/schemes/git-status.json')); print('VALID')"`

- [x] 13. Built-in scheme: ls-la

  **What to do**:
  - Create `.prunifier/schemes/ls-la.json` with pruning rules for `ls -la` output
  - Parse `ls -la` output format: "total N" header line, then rows with columns (permissions, links, owner, group, size, date, time, name)
  - Rules: discard "total" line (regex `^total\s`), discard lines ending with ` .` or ` ..` (current/parent dir entries), keep everything else
  - Add `description` field on each rule
  - Write validation test that scheme loads and applies to sample `ls -la` output

  **Must NOT do**:
  - Do NOT handle `ls` variants without `-la` flag (e.g., plain `ls`, `ls -l`, `ls -a` are different schemes)
  - Do NOT add column-based selectors for ls unless spec demands it (line-based regex is sufficient here)

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Creating a JSON scheme file with regex patterns — mechanical
  - **Skills**: None needed
  - **Skills Evaluated but Omitted**: None

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2 (with Tasks 8-12, 14-15)
  - **Blocks**: Task 22 (integration tests)
  - **Blocked By**: Tasks 5, 7 (Scheme types + storage)

  **References**:
  - Pattern: `ls -la` output example — see interview discussion for format
  - Spec: `SCHEMA.md` from Task 2

  **Acceptance Criteria**:
  - [ ] `.prunifier/schemes/ls-la.json` is valid JSON per schema
  - [ ] Scheme has at least 2 rules (discard total, discard . and ..)
  - [ ] Sample `ls -la` output with 5 files + . + .. → pruned output shows 5 files, no total line, no . or ..

  **QA Scenarios**:

  ```
  Scenario: ls-la scheme prunes total and dot entries
    Tool: Bash
    Preconditions: ls-la scheme exists
    Steps:
      1. Run: python3 -c "
import json
s=json.load(open('.prunifier/schemes/ls-la.json'))
rules = [r['action'] for r in s['rules']]
assert 'discard' in rules, 'No discard rules'
print(f'OK: {len(rules)} rules')
"
      2. Assert: output contains "OK"
    Expected Result: Scheme has discard rules
    Failure Indicators: Invalid JSON, no discard rules
    Evidence: .omo/evidence/task-13-ls-la-scheme.txt
  ```

  **Evidence to Capture**:
  - [ ] `task-13-ls-la-scheme.txt` — scheme validation output

  **Commit**: YES (Wave 2 group)
  - Message: `feat(prunifier): built-in scheme for ls -la`
  - Files: `.prunifier/schemes/ls-la.json`
  - Pre-commit: `python3 -c "import json; json.load(open('.prunifier/schemes/ls-la.json')); print('VALID')"`

- [x] 14. Built-in scheme: ps-aux

  **What to do**:
  - Create `.prunifier/schemes/ps-aux.json` with pruning rules for `ps aux` output
  - Parse `ps aux` output format: header row + columns (USER, PID, %CPU, %MEM, VSZ, RSS, TTY, STAT, START, TIME, COMMAND)
  - Rules: discard header row, keep only PID and COMMAND columns (using column selector from Task 10)
  - Use `Column` match condition with whitespace separator, keep columns at index 1 (PID) and 10 (COMMAND)
  - Add `description` field explaining the column selection
  - Write validation test

  **Must NOT do**:
  - Do NOT add regex rules for ps (column selector is the right approach here)
  - Do NOT handle BSD `ps` variants — Linux `ps aux` format only

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: JSON scheme creation — mechanical
  - **Skills**: None needed
  - **Skills Evaluated but Omitted**: None

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2 (with Tasks 8-13, 15)
  - **Blocks**: Task 22 (integration tests)
  - **Blocked By**: Tasks 5, 7 (Scheme types + storage)

  **References**:
  - Pattern: `ps aux` output — USER PID %CPU %MEM VSZ RSS TTY STAT START TIME COMMAND
  - Pattern: Column 1 = PID, Column 10 = COMMAND (0-indexed)

  **Acceptance Criteria**:
  - [ ] `.prunifier/schemes/ps-aux.json` is valid JSON per schema
  - [ ] Scheme uses `Column` match condition type for PID and COMMAND
  - [ ] Scheme has a discard rule for the header row (line number 0 or regex)

  **QA Scenarios**:

  ```
  Scenario: ps-aux scheme uses column selectors
    Tool: Bash
    Preconditions: ps-aux scheme exists
    Steps:
      1. Run: python3 -c "
import json
s=json.load(open('.prunifier/schemes/ps-aux.json'))
has_column = any(r.get('match_condition',{}).get('type')=='Column' for r in s['rules'])
assert has_column, 'No column rules'
print('OK: uses column selectors')
"
      2. Assert: output contains "OK"
    Expected Result: Scheme uses Column match conditions
    Failure Indicators: No column rules found
    Evidence: .omo/evidence/task-14-ps-aux-scheme.txt
  ```

  **Evidence to Capture**:
  - [ ] `task-14-ps-aux-scheme.txt` — scheme validation output

  **Commit**: YES (Wave 2 group)
  - Message: `feat(prunifier): built-in scheme for ps aux`
  - Files: `.prunifier/schemes/ps-aux.json`
  - Pre-commit: `python3 -c "import json; json.load(open('.prunifier/schemes/ps-aux.json')); print('VALID')"`

- [x] 15. Config loader (.prunifier.yaml reader)

  **What to do** (TDD):
  - **RED**: Write `tests/config_loader_test.rs`: `test_load_yaml_config()`, `test_missing_config_uses_defaults()`, `test_invalid_yaml_errors()`, `test_partial_config_merges_defaults()`
  - **GREEN**: Create `src/config/loader.rs` with `ConfigLoader`
  - `load(path: Option<&Path>) -> PrunifierResult<PrunifierConfig>` — load from given path or default location (project root `.prunifier.yaml`)
  - Use `serde_yaml::from_str` for parsing
  - On missing file: return `PrunifierConfig::default()` (not an error)
  - On invalid YAML: return `ConfigError` with parse error message
  - Validate config values: `scheme_dir` must be valid path if set, `verbose` must be true/false

  **Must NOT do**:
  - Do NOT create a default `.prunifier.yaml` file — config is optional
  - Do NOT support nested config structures beyond the 4 fields defined

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: YAML file reading + serde deserialization — well-defined
  - **Skills**: None needed
  - **Skills Evaluated but Omitted**: None

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2 (with Tasks 8-14)
  - **Blocks**: Task 20 (CLI reads config)
  - **Blocked By**: Task 6 (PrunifierConfig type)

  **References**:
  - Official docs: `https://docs.rs/serde_yaml/latest/serde_yaml/fn.from_str.html` — YAML parsing
  - Pattern: `src/config/types.rs:PrunifierConfig` — Config type from Task 6

  **Acceptance Criteria** (if TDD):
  - [ ] `cargo test tests::config_loader_test` → all 4 tests PASS
  - [ ] Loading valid YAML `scheme_dir: ./my-schemes` → config.scheme_dir = Some("./my-schemes")
  - [ ] Missing `.prunifier.yaml` → returns default config (not error)
  - [ ] Invalid YAML (malformed) → returns ConfigError

  **QA Scenarios**:

  ```
  Scenario: Missing config returns defaults
    Tool: Bash (cargo test)
    Preconditions: tests/config_loader_test.rs exists
    Steps:
      1. Run: cargo test test_missing_config_uses_defaults -- --nocapture 2>&1
      2. Assert: test passes, config equals default
    Expected Result: No config file = default behavior (not crash)
    Failure Indicators: Test FAILED due to file-not-found panic
    Evidence: .omo/evidence/task-15-config-defaults.txt

  Scenario: Valid YAML config loads correctly
    Tool: Bash (cargo test)
    Preconditions: YAML fixture exists
    Steps:
      1. Run: cargo test test_load_yaml_config -- --nocapture 2>&1
      2. Assert: test passes, scheme_dir is "custom-schemes"
    Expected Result: YAML fields deserialize to struct fields
    Failure Indicators: Test FAILED, wrong values
    Evidence: .omo/evidence/task-15-config-load.txt
  ```

  **Evidence to Capture**:
  - [ ] `task-15-config-defaults.txt` — defaults test output
  - [ ] `task-15-config-load.txt` — load test output

  **Commit**: YES (Wave 2 group)
  - Message: `feat(prunifier): config loader for .prunifier.yaml`
  - Files: `src/config/loader.rs`, `tests/config_loader_test.rs`
  - Pre-commit: `cargo test config_loader_test`

- [x] 16. Command executor (std::process::Command + output capture + exit code)

  **What to do** (TDD):
  - **RED**: Write `tests/executor_test.rs`: `test_execute_simple_command()`, `test_capture_stdout()`, `test_capture_stderr()`, `test_exit_code_propagation()`, `test_command_not_found()`, `test_output_with_no_newline()`
  - **GREEN**: Create `src/proxy/executor.rs` with `CommandExecutor`
  - `execute(command: &str) -> PrunifierResult<ExecutionResult>` where `ExecutionResult { stdout: String, stderr: String, exit_code: i32 }`
  - Parse command string into binary + args (split by whitespace, handle quoted strings)
  - Use `std::process::Command` to spawn, capture stdout and stderr separately
  - Propagate exit code on both success and failure (command fails → exit code is non-zero, but executor returns Ok with the exit code)
  - Handle command-not-found: return `CommandFailed` with exit code 127
  - Handle output with no trailing newline (preserve it)
  - Set `stdout(Stdio::piped())` and `stderr(Stdio::piped())` for capture

  **Must NOT do**:
  - Do NOT handle TTY detection here (that's Task 18)
  - Do NOT handle signal propagation (that's Task 29)
  - Do NOT add timeout logic
  - Do NOT handle pipes (`|`) — prunify receives the full pipeline as one argument

  **Recommended Agent Profile**:
  - **Category**: `deep`
    - Reason: Subprocess management with proper error handling, exit codes, and edge cases
  - **Skills**: None needed
  - **Skills Evaluated but Omitted**: None

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 3 (with Tasks 17-19, 21)
  - **Blocks**: Tasks 17, 20 (dispatcher and CLI use executor)
  - **Blocked By**: Task 4 (error types), Task 1 (project structure)

  **References**:
  - Official docs: `https://doc.rust-lang.org/std/process/struct.Command.html` — std::process::Command
  - Official docs: `https://doc.rust-lang.org/std/process/struct.ExitStatus.html` — ExitStatus for code extraction
  - Pattern: `src/error.rs:PrunifierError::CommandFailed` — Error type for command failures

  **Acceptance Criteria** (if TDD):
  - [ ] `cargo test tests::executor_test` → all 6 tests PASS
  - [ ] `execute("echo hello")` → stdout="hello\n", stderr="", exit_code=0
  - [ ] `execute("ls /nonexistent")` → stderr contains error, exit_code=2
  - [ ] `execute("nonexistent_command_xyz")` → stderr contains "not found", exit_code=127
  - [ ] `execute("printf hello")` → stdout="hello" (no trailing newline), exit_code=0

  **QA Scenarios**:

  ```
  Scenario: Simple command executes and captures output
    Tool: Bash (cargo test)
    Preconditions: tests/executor_test.rs exists
    Steps:
      1. Run: cargo test test_execute_simple_command -- --nocapture 2>&1
      2. Assert: test passes, stdout contains expected text
    Expected Result: Command output captured correctly
    Failure Indicators: Test FAILED, output mismatch
    Evidence: .omo/evidence/task-16-executor-simple.txt

  Scenario: Exit code propagated on failure
    Tool: Bash (cargo test)
    Preconditions: tests/executor_test.rs exists
    Steps:
      1. Run: cargo test test_exit_code_propagation -- --nocapture 2>&1
      2. Assert: test passes, exit_code != 0
    Expected Result: Non-zero exit code from failed command is preserved
    Failure Indicators: Test FAILED, exit code 0 on failure
    Evidence: .omo/evidence/task-16-executor-exit-code.txt
  ```

  **Evidence to Capture**:
  - [ ] `task-16-executor-simple.txt` — simple command test
  - [ ] `task-16-executor-exit-code.txt` — exit code propagation test

  **Commit**: YES (Wave 3 group)
  - Message: `feat(prunifier): command executor with output capture and exit codes`
  - Files: `src/proxy/executor.rs`, `src/proxy/mod.rs`, `tests/executor_test.rs`
  - Pre-commit: `cargo test executor_test`

- [x] 17. Three-mode dispatcher (exact / prefix / passthrough logic)

  **What to do** (TDD):
  - **RED**: Write `tests/dispatcher_test.rs`: `test_mode1_exact_match_prunes()`, `test_mode2_prefix_match_prunes_and_marks()`, `test_mode3_no_match_passthrough()`, `test_mode2_mark_includes_pruned_tag()`, `test_dispatcher_preserves_exit_code()`
  - **GREEN**: Create `src/proxy/dispatcher.rs` with `Dispatcher` struct
  - `new(trie: CommandTrie, schemes: HashMap<String, Scheme>) -> Self` — load from trie and scheme map
  - `dispatch(command: &str, output: &str) -> PrunifierResult<(String, DispatchMode)>` where `DispatchMode` enum: `ExactMatch`, `PrefixMatch(usize)`, `Passthrough`
  - Mode 1 (ExactMatch): command exactly matches trie entry → apply scheme, return pruned output
  - Mode 2 (PrefixMatch): trie finds prefix match but not exact → apply closest scheme, prepend `[PRUNED] (prefix match: N tokens)` to output
  - Mode 3 (Passthrough): no trie match at all → return output unchanged, prepend `[UNKNOWN COMMAND]` mark
  - Apply pruning: use `LineParser` (Task 9) for regex/line rules, `ColumnSelector` (Task 10) for column rules
  - The dispatcher does NOT execute commands — it receives already-executed output from the executor

  **Must NOT do**:
  - Do NOT execute commands inside the dispatcher (execution is separate)
  - Do NOT handle TTY or recursion guard here (those are upstream in the CLI)

  **Recommended Agent Profile**:
  - **Category**: `deep`
    - Reason: Core orchestration logic combining trie, scheme loading, line parser, and column selector
  - **Skills**: None needed
  - **Skills Evaluated but Omitted**: None

  **Parallelization**:
  - **Can Run In Parallel**: NO (depends on most Wave 2 modules)
  - **Parallel Group**: Wave 3 (sequential with Tasks 16, 18-21)
  - **Blocks**: Task 20 (CLI uses dispatcher)
  - **Blocked By**: Tasks 8 (trie), 9 (line parser), 10 (column selector), 11 (scheme loader), 16 (executor)

  **References**:
  - Pattern: `src/engine/trie.rs:CommandTrie` — Trie from Task 8
  - Pattern: `src/engine/line_parser.rs:LineParser` — Parser from Task 9
  - Pattern: `src/engine/column_selector.rs:ColumnSelector` — Selector from Task 10
  - Pattern: `src/scheme/types.rs:Scheme` — Scheme type from Task 5

  **Acceptance Criteria** (if TDD):
  - [ ] `cargo test tests::dispatcher_test` → all 5 tests PASS
  - [ ] Exact match for "git status" → output pruned per git-status scheme, no mark
  - [ ] Prefix match for "git status --short" (only "git status" in trie) → output pruned + `[PRUNED]` mark
  - [ ] No match for "docker ps" → output unchanged + `[UNKNOWN COMMAND]` mark
  - [ ] `DispatchMode::PrefixMatch(2)` returned when 2 tokens matched

  **QA Scenarios**:

  ```
  Scenario: Exact match prunes without marking
    Tool: Bash (cargo test)
    Preconditions: tests/dispatcher_test.rs exists
    Steps:
      1. Run: cargo test test_mode1_exact_match_prunes -- --nocapture 2>&1
      2. Assert: test passes, output pruned, no [PRUNED] tag
    Expected Result: Mode 1 applies scheme silently
    Failure Indicators: Test FAILED, [PRUNED] appearing in exact match
    Evidence: .omo/evidence/task-17-dispatcher-exact.txt

  Scenario: Prefix match adds [PRUNED] mark
    Tool: Bash (cargo test)
    Preconditions: tests/dispatcher_test.rs exists
    Steps:
      1. Run: cargo test test_mode2_prefix_match_prunes_and_marks -- --nocapture 2>&1
      2. Assert: test passes, output contains "[PRUNED]"
    Expected Result: Mode 2 marks output as pruned with prefix warning
    Failure Indicators: Test FAILED, [PRUNED] missing
    Evidence: .omo/evidence/task-17-dispatcher-prefix.txt

  Scenario: Unknown command passes through
    Tool: Bash (cargo test)
    Preconditions: tests/dispatcher_test.rs exists
    Steps:
      1. Run: cargo test test_mode3_no_match_passthrough -- --nocapture 2>&1
      2. Assert: test passes, output unchanged except for [UNKNOWN COMMAND]
    Expected Result: Mode 3 returns raw output
    Failure Indicators: Test FAILED, output modified when it shouldn't be
    Evidence: .omo/evidence/task-17-dispatcher-passthrough.txt
  ```

  **Evidence to Capture**:
  - [ ] `task-17-dispatcher-exact.txt` — mode 1 test
  - [ ] `task-17-dispatcher-prefix.txt` — mode 2 test
  - [ ] `task-17-dispatcher-passthrough.txt` — mode 3 test

  **Commit**: YES (Wave 3 group)
  - Message: `feat(prunifier): three-mode dispatcher with trie routing and pruning`
  - Files: `src/proxy/dispatcher.rs`, `tests/dispatcher_test.rs`
  - Pre-commit: `cargo test dispatcher_test`

- [x] 18. TTY detector + passthrough

  **What to do** (TDD):
  - **RED**: Write `tests/tty_test.rs`: `test_detect_tty_stdout()`, `test_detect_non_tty_stdout()`, `test_tty_passthrough_skips_pruning()`
  - **GREEN**: Create `src/proxy/tty.rs` with `TtyDetector`
  - `is_tty() -> bool` — check if stdout is a TTY using `libc::isatty(libc::STDOUT_FILENO)` on Linux/macOS
  - `should_passthrough(command: &str) -> bool` — returns true if command appears interactive (check for known interactive binaries: vim, nano, htop, top, less, more, emacs, screen, tmux)
  - When TTY detected or command is interactive: the proxy should NOT capture stdout (use `Stdio::inherit()`), letting the command take over the terminal
  - This module is used by the CLI entry point BEFORE calling the executor

  **Must NOT do**:
  - Do NOT add Windows-specific TTY detection (Linux/macOS only per guardrail)
  - Do NOT try to detect TTY by parsing command output (check the FD directly)

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Simple isatty check + known-interactive command list — straightforward
  - **Skills**: None needed
  - **Skills Evaluated but Omitted**: None

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 3 (with Tasks 16-17, 19, 21)
  - **Blocks**: Task 20 (CLI uses TTY detection before execution)
  - **Blocked By**: None (standalone module, no dependencies)

  **References**:
  - Official docs: `https://man7.org/linux/man-pages/man3/isatty.3.html` — isatty man page
  - Pattern: Use `libc = "0.2"` crate for `libc::isatty`

  **Acceptance Criteria** (if TDD):
  - [ ] `cargo test tests::tty_test` → all 3 tests PASS
  - [ ] `is_tty()` on piped output → false
  - [ ] `is_tty()` when run in real terminal → true (tested via integration, not unit)
  - [ ] `should_passthrough("vim")` → true
  - [ ] `should_passthrough("ls")` → false

  **QA Scenarios**:

  ```
  Scenario: Non-TTY output detected correctly
    Tool: Bash (cargo test)
    Preconditions: tests/tty_test.rs exists
    Steps:
      1. Run: cargo test test_detect_non_tty_stdout -- --nocapture 2>&1
      2. Assert: test passes, is_tty() returns false (cargo test pipes output)
    Expected Result: Piped stdout detected as non-TTY
    Failure Indicators: Test FAILED
    Evidence: .omo/evidence/task-18-tty-nontty.txt

  Scenario: Interactive commands trigger passthrough
    Tool: Bash (cargo test)
    Preconditions: tests/tty_test.rs exists
    Steps:
      1. Run: cargo test test_tty_passthrough_skips_pruning -- --nocapture 2>&1
      2. Assert: test passes, should_passthrough("vim") is true
    Expected Result: Known interactive commands always passthrough
    Failure Indicators: Test FAILED, vim not recognized as interactive
    Evidence: .omo/evidence/task-18-tty-passthrough.txt
  ```

  **Evidence to Capture**:
  - [ ] `task-18-tty-nontty.txt` — non-TTY test
  - [ ] `task-18-tty-passthrough.txt` — passthrough test

  **Commit**: YES (Wave 3 group)
  - Message: `feat(prunifier): TTY detector with interactive command passthrough`
  - Files: `src/proxy/tty.rs`, `tests/tty_test.rs`, `Cargo.toml` (add libc dep)
  - Pre-commit: `cargo test tty_test`

- [x] 19. Recursion guard (detect prunify self-invocation)

  **What to do** (TDD):
  - **RED**: Write `tests/recursion_test.rs`: `test_detect_self_invocation()`, `test_detect_nested_prunify()`, `test_normal_command_not_detected()`, `test_different_path_prunify()`
  - **GREEN**: Create `src/proxy/recursion_guard.rs` with `RecursionGuard`
  - `is_recursive(command: &str) -> bool` — check if the command string contains `prunify` or starts with the prunify binary path
  - Detection: check if the command binary is named "prunify" (case sensitive), or if the full command starts with the prunify executable path
  - When recursion detected: print warning to stderr ("prunify: recursion detected — bypassing proxy") and exit 0
  - This runs in the CLI BEFORE any processing (before executor, before dispatcher)

  **Must NOT do**:
  - Do NOT use shell-based detection (check the command string in Rust, not bash)
  - Do NOT block all commands containing "prunify" in arguments (e.g., `echo "use prunify"` should work)

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Simple string check on command name — trivial
  - **Skills**: None needed
  - **Skills Evaluated but Omitted**: None

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 3 (with Tasks 16-18, 21)
  - **Blocks**: Task 20 (CLI calls recursion guard first)
  - **Blocked By**: None (standalone)

  **References**:
  - Pattern: `std::env::current_exe()` to get the prunify binary path for comparison

  **Acceptance Criteria** (if TDD):
  - [ ] `cargo test tests::recursion_test` → all 4 tests PASS
  - [ ] `is_recursive("prunify ls -la")` → true
  - [ ] `is_recursive("./target/release/prunify ls -la")` → true
  - [ ] `is_recursive("echo 'use prunify for this'")` → false
  - [ ] `is_recursive("ls -la")` → false

  **QA Scenarios**:

  ```
  Scenario: Self-invocation detected and blocked
    Tool: Bash (cargo test)
    Preconditions: tests/recursion_test.rs exists
    Steps:
      1. Run: cargo test test_detect_self_invocation -- --nocapture 2>&1
      2. Assert: test passes, is_recursive returns true
    Expected Result: Recursive prunify invocation detected
    Failure Indicators: Test FAILED, recursion NOT detected
    Evidence: .omo/evidence/task-19-recursion-detect.txt

  Scenario: Normal commands pass through
    Tool: Bash (cargo test)
    Preconditions: tests/recursion_test.rs exists
    Steps:
      1. Run: cargo test test_normal_command_not_detected -- --nocapture 2>&1
      2. Assert: test passes, is_recursive returns false
    Expected Result: Non-prunify commands not flagged
    Failure Indicators: Test FAILED, false positives
    Evidence: .omo/evidence/task-19-recursion-normal.txt
  ```

  **Evidence to Capture**:
  - [ ] `task-19-recursion-detect.txt` — detection test
  - [ ] `task-19-recursion-normal.txt` — false positive test

  **Commit**: YES (Wave 3 group)
  - Message: `feat(prunifier): recursion guard for self-invocation detection`
  - Files: `src/proxy/recursion_guard.rs`, `tests/recursion_test.rs`
  - Pre-commit: `cargo test recursion_test`

- [x] 20. CLI entry point (clap argument parsing + main)

  **What to do** (TDD — integration test, not unit):
  - **RED**: Write `tests/cli_test.rs`: `test_prunify_flag_passthrough()`, `test_help_flag()`, `test_no_args_shows_usage()`, `test_version_flag()`
  - **GREEN**: Create `src/cli.rs` with clap `#[derive(Parser)]` struct
  - CLI args: positional `<CMD>...` (all args after prunify's own flags form the proxied command), `--scheme-dir <DIR>` (optional override), `--verbose`, `--no-mark` (disable [PRUNED] marking), `--strict` (error on unknown commands)
  - `main()` in `src/main.rs`: parse args, check recursion guard, check TTY, load config, load schemes, populate trie, execute command, dispatch, print output, exit with code
  - Integration: `prunify ls -la` → dispatch through all modes
  - Command parsing: all positional args after prunify's own flags form the proxied command string (no `--` separator needed)
  - Update `Cargo.toml` with `[[bin]]` section, name = "prunify"

  **Must NOT do**:
  - Do NOT put logic in main() — main delegates to modules
  - Do NOT add subcommands to the CLI (single command proxy only)

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
    - Reason: Complex integration of all modules into a cohesive CLI entry point
  - **Skills**: None needed
  - **Skills Evaluated but Omitted**: None

  **Parallelization**:
  - **Can Run In Parallel**: NO (depends on all Wave 3 modules)
  - **Parallel Group**: Wave 3 (sequential — last task in wave)
  - **Blocks**: Task 22 (integration tests), Tasks 26-30 (edge cases + final tests)
  - **Blocked By**: Tasks 16 (executor), 17 (dispatcher), 18 (TTY), 19 (recursion), 21 (marking)

  **References**:
  - Official docs: `https://docs.rs/clap/latest/clap/_derive/_tutorial/index.html` — Clap derive tutorial
  - Pattern: `src/proxy/dispatcher.rs:Dispatcher` — Dispatcher from Task 17
  - Pattern: `src/proxy/executor.rs:CommandExecutor` — Executor from Task 16

  **Acceptance Criteria** (if TDD):
  - [ ] `cargo test tests::cli_test` → all 4 tests PASS
  - [ ] `prunify --help` prints usage with all flags documented
  - [ ] `prunify echo hello` outputs "hello" with exit 0
  - [ ] `prunify --version` prints "prunifier v0.1.0"
  - [ ] `prunify ls /nonexistent` exits with code 2

  **QA Scenarios**:

  ```
  Scenario: Basic prunify command works end-to-end
    Tool: Bash
    Preconditions: cargo build succeeded
    Steps:
      1. Run: ./target/debug/prunify echo "hello world" 2>&1
      2. Assert: exit code is 0
      3. Assert: stdout contains "hello world"
    Expected Result: Simple command passes through unchanged
    Failure Indicators: Non-zero exit, output mismatch
    Evidence: .omo/evidence/task-20-cli-basic.txt

  Scenario: Help flag shows usage
    Tool: Bash
    Preconditions: cargo build succeeded
    Steps:
      1. Run: ./target/debug/prunify --help 2>&1
      2. Assert: exit code is 0
      3. Assert: stdout contains "--command" or "[COMMAND]"
      4. Assert: stdout contains "--scheme-dir"
    Expected Result: Help text documents all flags
    Failure Indicators: Missing help text, exit != 0
    Evidence: .omo/evidence/task-20-cli-help.txt

  Scenario: Exit code propagation
    Tool: Bash
    Preconditions: cargo build succeeded
    Steps:
      1. Run: ./target/debug/prunify ls /nonexistent_path_12345 2>&1
      2. Capture: EXIT_CODE=$?
      3. Assert: EXIT_CODE != 0
      4. Assert: stderr contains "No such file" or similar
    Expected Result: Failed command exit code propagated
    Failure Indicators: Exit code 0 on command failure
    Evidence: .omo/evidence/task-20-cli-exit-code.txt
  ```

  **Evidence to Capture**:
  - [ ] `task-20-cli-basic.txt` — basic command test
  - [ ] `task-20-cli-help.txt` — help output
  - [ ] `task-20-cli-exit-code.txt` — exit code test

  **Commit**: YES (Wave 3 group)
  - Message: `feat(prunifier): CLI entry point with clap and full proxy pipeline`
  - Files: `src/cli.rs`, `src/main.rs`
  - Pre-commit: `cargo build && ./target/debug/prunify --help`

- [x] 21. Output marking ([PRUNED] insertion logic)

  **What to do** (TDD):
  - **RED**: Write `tests/marking_test.rs`: `test_pruned_mark_appended()`, `test_no_mark_on_exact_match()`, `test_unknown_command_mark()`, `test_no_mark_flag_suppresses()`
  - **GREEN**: Create `src/proxy/marking.rs` with `OutputMarker`
  - `mark_pruned(output: &str, mode: DispatchMode, matched_tokens: usize) -> String` — append appropriate mark
  - Mode 1 (ExactMatch): no mark, return output as-is
  - Mode 2 (PrefixMatch): append `\n[PRUNED] (prefix match: N tokens — scheme may be suboptimal)\n` to END of output
  - Mode 3 (Passthrough): append `\n[UNKNOWN COMMAND] (no scheme found — output is raw)\n` to END of output
  - Respect `--no-mark` flag: when set, never add marks (but still prune in modes 1-2)
  - Marks go to stdout (appended at end, after all pruned content — downstream pipes consume useful data first)

  **Must NOT do**:
  - Do NOT put marks at the START of stdout (only at end)
  - Do NOT put marks on stderr (stdout only, so grep/pipe consumers see them)
  - Do NOT colorize marks in v1 (ANSI colors are Task 27)

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: String prepending based on simple enum matching — trivial
  - **Skills**: None needed
  - **Skills Evaluated but Omitted**: None

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 3 (with Tasks 16-19)
  - **Blocks**: Task 20 (CLI uses marking)
  - **Blocked By**: Task 17 (needs DispatchMode enum)

  **References**:
  - Pattern: `src/proxy/dispatcher.rs:DispatchMode` — Dispatch mode enum from Task 17

  **Acceptance Criteria** (if TDD):
  - [ ] `cargo test tests::marking_test` → all 4 tests PASS
  - [ ] Prefix match → stdout ends with `[PRUNED]`
  - [ ] Exact match → stdout does NOT contain `[PRUNED]`
  - [ ] Unknown command → stdout ends with `[UNKNOWN COMMAND]`
  - [ ] `--no-mark` flag → stdout contains no marks for any mode

  **QA Scenarios**:

  ```
  Scenario: [PRUNED] mark appended to end of pruned output
    Tool: Bash (cargo test)
    Preconditions: tests/marking_test.rs exists
    Steps:
      1. Run: cargo test test_pruned_mark_appended -- --nocapture 2>&1
      2. Assert: test passes, stdout ends with "[PRUNED]"
    Expected Result: Prefix match output has mark at the end
    Failure Indicators: Test FAILED, mark missing or at wrong position
    Evidence: .omo/evidence/task-21-marking-pruned.txt

  Scenario: Exact match has no mark
    Tool: Bash (cargo test)
    Preconditions: tests/marking_test.rs exists
    Steps:
      1. Run: cargo test test_no_mark_on_exact_match -- --nocapture 2>&1
      2. Assert: test passes, [PRUNED] NOT in stdout
    Expected Result: Exact match is silent
    Failure Indicators: Test FAILED, [PRUNED] appearing on exact match
    Evidence: .omo/evidence/task-21-marking-exact.txt
  ```

  **Evidence to Capture**:
  - [ ] `task-21-marking-pruned.txt` — prefix mark test
  - [ ] `task-21-marking-exact.txt` — exact match no-mark test

  **Commit**: YES (Wave 3 group)
  - Message: `feat(prunifier): output marking with [PRUNED] and [UNKNOWN COMMAND] tags`
  - Files: `src/proxy/marking.rs`, `tests/marking_test.rs`
  - Pre-commit: `cargo test marking_test`

- [x] 22. Integration tests (end-to-end shell scenarios)

  **What to do**:
  - Expand `tests/shell_tests.sh` with end-to-end scenarios using the built binary
  - Scenarios: `test_mode1_git_status_pruned()`, `test_mode1_ls_la_pruned()`, `test_mode2_prefix_match_marked()`, `test_mode3_unknown_passthrough()`, `test_exit_code_propagation()`, `test_recursion_guard()`, `test_tty_passthrough_skips_pruning()`, `test_project_override_scheme()`, `test_no_mark_flag()`
  - Each test: setup (create test dir, write config if needed), run `prunify <CMD>`, assert exit code, assert stdout/stderr content
  - Use temp directories for test isolation (`mktemp -d`)
  - Run with `bats` or plain bash assertions (`[ "$output" = "expected" ] || exit 1`)

  **Must NOT do**:
  - Do NOT require `bats` framework (plain bash is fine for v1)
  - Do NOT modify production code from integration tests

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
    - Reason: Comprehensive end-to-end shell scripts covering all modes and edge cases
  - **Skills**: None needed
  - **Skills Evaluated but Omitted**: None

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Wave 3 (sequential — depends on CLI being functional)
  - **Blocks**: Task 30 (final integration suite extends these)
  - **Blocked By**: Tasks 12-14 (built-in schemes), 20 (CLI entry point)

  **References**:
  - Pattern: `tests/shell_tests.sh` from Task 3 — existing shell test scaffold
  - Pattern: `.prunifier/schemes/git-status.json` — Built-in scheme from Task 12

  **Acceptance Criteria**:
  - [ ] `./tests/shell_tests.sh` runs all 9 scenarios
  - [ ] All 9 scenarios PASS (exit 0 from the script)
  - [ ] Mode 1 scenario: `prunify ls -la` output does NOT contain "total" or " ." or " .."
  - [ ] Mode 2 scenario: `prunify ls -la --color=auto` stdout ends with `[PRUNED]`
  - [ ] Mode 3 scenario: `prunify echo hello` output contains "hello"

  **QA Scenarios**:

  ```
  Scenario: Full integration — git status with scheme
    Tool: Bash
    Preconditions: cargo build succeeded, schemes exist
    Steps:
      1. cd /tmp && mkdir -p test-prunifier-git && cd test-prunifier-git
      2. git init && touch file.txt
      3. Run: /root/prunifier/target/debug/prunify git status 2>&1
      4. Assert: exit code is 0
      5. Assert: stdout does NOT contain "On branch" (pruned)
      6. Assert: stdout contains "file.txt" or "Untracked"
    Expected Result: Git status output pruned correctly
    Failure Indicators: "On branch" visible, exit != 0
    Evidence: .omo/evidence/task-22-integration-git-status.txt

  Scenario: Exit code propagated through proxy
    Tool: Bash
    Preconditions: cargo build succeeded
    Steps:
      1. Run: /root/prunifier/target/debug/prunify sh -c "exit 42" 2>&1
      2. Capture: EXIT=$?
      3. Assert: EXIT is 42
    Expected Result: Exit code 42 propagated
    Failure Indicators: Exit code 0 instead of 42
    Evidence: .omo/evidence/task-22-integration-exit-code.txt
  ```

  **Evidence to Capture**:
  - [ ] `task-22-integration-git-status.txt` — git status end-to-end
  - [ ] `task-22-integration-exit-code.txt` — exit code test

  **Commit**: YES (Wave 3 group)
  - Message: `test(prunifier): end-to-end integration tests for all three modes`
  - Files: `tests/shell_tests.sh`
  - Pre-commit: `bash tests/shell_tests.sh`

- [x] 23. OpenCode skill: SKILL.md (skill metadata + basic workflow)

  **What to do**:
  - Create `.opencode/skills/prunifier/SKILL.md` with YAML frontmatter + markdown body
  - Frontmatter: `name: prunifier`, `description: Proxy bash commands through prunify to prune verbose output using AST schemes`, `triggers: prunify, prunifier, prune output, trim output`
  - Document: what Prunifier is, how to install (`cargo build --release`, add to PATH), how to invoke (`prunify <CMD>`)
  - Document: the 3 modes and what each means
  - Document: scheme file format (with examples from SCHEMA.md)
  - Document: `.prunifier.yaml` config options
  - Document: how to create a new scheme (point to Tasks 24-25 for subagent workflows)
  - Add note: "The prunify binary works standalone without this skill. This skill provides workflow guidance."

  **Must NOT do**:
  - Do NOT include executable code in the skill (skills are markdown-only)
  - Do NOT claim the skill can auto-generate schemes without subagent involvement

  **Recommended Agent Profile**:
  - **Category**: `writing`
    - Reason: Pure documentation with structured format — no code
  - **Skills**: None needed
  - **Skills Evaluated but Omitted**: None

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 4 (with Tasks 24-28)
  - **Blocks**: None (documentation only)
  - **Blocked By**: Task 2 (SCHEMA.md for scheme format details), Task 20 (CLI for usage docs)

  **References**:
  - Pattern: `~/.cache/opencode/skills/security-research/SKILL.md` — OpenCode skill format (YAML frontmatter + markdown body)
  - Pattern: `SCHEMA.md` from Task 2 — scheme format reference
  - Pattern: `.prunifier.yaml` config options from Tasks 6, 15

  **Acceptance Criteria**:
  - [ ] `.opencode/skills/prunifier/SKILL.md` exists with valid YAML frontmatter
  - [ ] Frontmatter has `name`, `description`, `triggers` fields
  - [ ] Body documents: installation, invocation, 3 modes, scheme format, config
  - [ ] Body includes a note that binary works standalone
  - [ ] Body references SCHEMA.md for detailed scheme format

  **QA Scenarios**:

  ```
  Scenario: Skill file has valid frontmatter
    Tool: Bash
    Preconditions: .opencode/skills/prunifier/SKILL.md exists
    Steps:
      1. Run: head -10 .opencode/skills/prunifier/SKILL.md
      2. Assert: output contains "---" (YAML delimiter)
      3. Assert: output contains "name:"
      4. Assert: output contains "triggers:"
    Expected Result: Valid YAML frontmatter present
    Failure Indicators: Missing frontmatter, missing required fields
    Evidence: .omo/evidence/task-23-skill-frontmatter.txt

  Scenario: Skill documents installation steps
    Tool: Bash
    Preconditions: SKILL.md exists
    Steps:
      1. Run: grep -c "cargo build" .opencode/skills/prunifier/SKILL.md
      2. Assert: count >= 1
      3. Run: grep -ci "install" .opencode/skills/prunifier/SKILL.md
      4. Assert: count >= 1
    Expected Result: Installation instructions present
    Failure Indicators: No installation docs
    Evidence: .omo/evidence/task-23-skill-install.txt
  ```

  **Evidence to Capture**:
  - [ ] `task-23-skill-frontmatter.txt` — frontmatter check
  - [ ] `task-23-skill-install.txt` — installation docs check

  **Commit**: YES (Wave 4 group)
  - Message: `docs(prunifier): OpenCode skill with workflow documentation`
  - Files: `.opencode/skills/prunifier/SKILL.md`
  - Pre-commit: `head -10 .opencode/skills/prunifier/SKILL.md | grep -q "name:"`

- [x] 24. OpenCode skill: Mode-2 workflow (prefix match → subagent optimization)

  **What to do**:
  - Add a "Mode 2: Prefix Match Workflow" section to `.opencode/skills/prunifier/SKILL.md`
  - Document the workflow: when agent sees `[PRUNED]` appended to stdout, it should:
    1. Run the command WITHOUT `prunify` to see the full raw output
    2. Analyze the full output to identify what's signal vs noise
    3. Compare with the applied (prefix-matched) scheme's pruning to see what was missed
    4. Choose a subagent (explore/librarian/deep — agent decides) to draft an optimized scheme JSON
    5. Write the new scheme to `.prunifier/schemes/<command-slug>.json`
    6. Validate with `prunify <command>` — verify `[PRUNED]` disappears (exact match now)
  - Include a concrete example: `git status --short` prefix-matched to `git status` scheme → subagent creates `git-status-short.json`
  - Note: "The agent using this skill decides which subagent to spawn. explore is good for analysis, librarian for external research, deep for complex scheme design."

  **Must NOT do**:
  - Do NOT claim automatic optimization without subagent/human involvement
  - Do NOT modify the Rust binary to trigger subagents (skill is documentation-only)

  **Recommended Agent Profile**:
  - **Category**: `writing`
    - Reason: Documentation of a workflow — no code involved
  - **Skills**: None needed
  - **Skills Evaluated but Omitted**: None

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 4 (with Tasks 23, 25-28)
  - **Blocks**: None
  - **Blocked By**: Task 23 (SKILL.md must exist)

  **References**:
  - Pattern: `.opencode/skills/prunifier/SKILL.md` from Task 23 — existing skill file

  **Acceptance Criteria**:
  - [ ] SKILL.md has "Mode 2: Prefix Match Workflow" section header
  - [ ] Section documents 6-step workflow (numbered or bulleted)
  - [ ] Section includes concrete example with `git status --short`
  - [ ] Section mentions subagent types (explore, librarian, deep) and when to use each

  **QA Scenarios**:

  ```
  Scenario: Mode 2 workflow documented
    Tool: Bash
    Preconditions: SKILL.md exists
    Steps:
      1. Run: grep -c "Mode 2" .opencode/skills/prunifier/SKILL.md
      2. Assert: count >= 1
      3. Run: grep -ci "prefix match" .opencode/skills/prunifier/SKILL.md
      4. Assert: count >= 1
      5. Run: grep -c "subagent" .opencode/skills/prunifier/SKILL.md
      6. Assert: count >= 1
    Expected Result: Mode 2 workflow fully documented
    Failure Indicators: Missing mode 2 section, no subagent mention
    Evidence: .omo/evidence/task-24-skill-mode2.txt
  ```

  **Evidence to Capture**:
  - [ ] `task-24-skill-mode2.txt` — mode 2 documentation check

  **Commit**: YES (Wave 4 group)
  - Message: `docs(prunifier): mode-2 workflow for prefix match subagent optimization`
  - Files: `.opencode/skills/prunifier/SKILL.md`
  - Pre-commit: `grep -q "Mode 2" .opencode/skills/prunifier/SKILL.md`

- [x] 25. OpenCode skill: Mode-3 workflow (new command → subagent analysis)

  **What to do**:
  - Add a "Mode 3: New Command Workflow" section to `.opencode/skills/prunifier/SKILL.md`
  - Document the workflow: when agent sees `[UNKNOWN COMMAND]` in stderr, it should:
    1. Review the raw output (passed through unchanged)
    2. Decide if this command's output would benefit from pruning (not all commands need it)
    3. If pruning would help: choose a subagent (explore/librarian/deep) to analyze the output
    4. The subagent should identify repetitive/metadata/noise patterns and draft a scheme JSON
    5. Write the scheme to `.prunifier/schemes/<command-slug>.json`
    6. Test with `prunify <command>` — confirm output is pruned usefully
    7. If the scheme is high-quality and the command is common, suggest contributing it back upstream
  - Include a concrete example: new command `docker ps` → subagent analyzes columnar output, creates scheme keeping CONTAINER_ID + NAMES columns
  - Note: "Not all commands need pruning. Some produce minimal output already. The subagent should evaluate whether pruning improves agent efficiency."

  **Must NOT do**:
  - Do NOT claim automatic scheme generation (subagent does the analysis)
  - Do NOT promise upstream contribution acceptance

  **Recommended Agent Profile**:
  - **Category**: `writing`
    - Reason: Documentation workflow — code-free
  - **Skills**: None needed
  - **Skills Evaluated but Omitted**: None

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 4 (with Tasks 23-24, 26-28)
  - **Blocks**: None
  - **Blocked By**: Task 23 (SKILL.md must exist)

  **References**:
  - Pattern: `.opencode/skills/prunifier/SKILL.md` from Task 23 — existing skill file

  **Acceptance Criteria**:
  - [ ] SKILL.md has "Mode 3: New Command Workflow" section header
  - [ ] Section documents 7-step workflow (numbered or bulleted)
  - [ ] Section includes concrete example with `docker ps`
  - [ ] Section mentions that not all commands need pruning
  - [ ] Section addresses scheme quality evaluation criteria

  **QA Scenarios**:

  ```
  Scenario: Mode 3 workflow documented
    Tool: Bash
    Preconditions: SKILL.md exists
    Steps:
      1. Run: grep -c "Mode 3" .opencode/skills/prunifier/SKILL.md
      2. Assert: count >= 1
      3. Run: grep -ci "new command" .opencode/skills/prunifier/SKILL.md
      4. Assert: count >= 1
      5. Run: grep -ci "docker" .opencode/skills/prunifier/SKILL.md
      6. Assert: count >= 1
    Expected Result: Mode 3 workflow fully documented with example
    Failure Indicators: Missing mode 3 section, no example
    Evidence: .omo/evidence/task-25-skill-mode3.txt
  ```

  **Evidence to Capture**:
  - [ ] `task-25-skill-mode3.txt` — mode 3 documentation check

  **Commit**: YES (Wave 4 group)
  - Message: `docs(prunifier): mode-3 workflow for new command subagent analysis`
  - Files: `.opencode/skills/prunifier/SKILL.md`
  - Pre-commit: `grep -q "Mode 3" .opencode/skills/prunifier/SKILL.md`

- [x] 26. Edge case: Binary output handling

  **What to do** (TDD):
  - **RED**: Write `tests/binary_output_test.rs`: `test_binary_output_passthrough()`, `test_text_output_still_pruned()`, `test_null_bytes_in_output()`
  - **GREEN**: Create `src/proxy/binary_detector.rs` with `BinaryDetector`
  - `is_binary(data: &[u8]) -> bool` — check for null bytes or high concentration of non-printable characters (>30% non-ASCII non-UTF8 in first 8KB)
  - When binary detected: skip all pruning, pass output through as raw bytes
  - When NOT binary: proceed with normal text-based pruning
  - Important: check BEFORE converting to String (use `&[u8]` from executor output). If binary, write raw bytes to stdout (no String conversion)
  - Update executor to return `Vec<u8>` for stdout/stderr instead of String (or provide both)

  **Must NOT do**:
  - Do NOT attempt to "prune" binary data — detect and passthrough only
  - Do NOT convert binary data to String (corruption risk)

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Simple heuristic check on byte content — small module
  - **Skills**: None needed
  - **Skills Evaluated but Omitted**: None

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 4 (with Tasks 23-25, 27-29)
  - **Blocks**: Task 30 (final integration tests)
  - **Blocked By**: Task 20 (CLI must be functional for integration testing)

  **References**:
  - Pattern: `src/proxy/executor.rs:ExecutionResult` from Task 16 — may need to add raw bytes field

  **Acceptance Criteria** (if TDD):
  - [ ] `cargo test tests::binary_output_test` → all 3 tests PASS
  - [ ] `cat /dev/urandom | head -c 1024` piped through prunify → output is byte-identical
  - [ ] Regular text `echo hello` → still pruned normally (no false binary detection)
  - [ ] Output with single null byte → detected as binary

  **QA Scenarios**:

  ```
  Scenario: Binary file output passes through unchanged
    Tool: Bash
    Preconditions: cargo build succeeded
    Steps:
      1. dd if=/dev/urandom bs=1024 count=1 of=/tmp/test-bin.bin 2>/dev/null
      2. ORIGINAL_HASH=$(md5sum /tmp/test-bin.bin | cut -d' ' -f1)
      3. Run: ./target/debug/prunify cat /tmp/test-bin.bin > /tmp/test-bin-out.bin 2>/dev/null
      4. OUTPUT_HASH=$(md5sum /tmp/test-bin-out.bin | cut -d' ' -f1)
      5. Assert: ORIGINAL_HASH equals OUTPUT_HASH
    Expected Result: Binary data is byte-identical after passthrough
    Failure Indicators: Hash mismatch (data corrupted)
    Evidence: .omo/evidence/task-26-binary-passthrough.txt

  Scenario: Text output still gets pruned
    Tool: Bash
    Preconditions: cargo build succeeded, ls-la scheme exists
    Steps:
      1. Run: ./target/debug/prunify ls -la /root/prunifier 2>&1
      2. Assert: output does NOT contain "total" (pruned correctly)
    Expected Result: Normal text pruning still works
    Failure Indicators: Binary detection false-positive prevents pruning
    Evidence: .omo/evidence/task-26-text-still-pruned.txt
  ```

  **Evidence to Capture**:
  - [ ] `task-26-binary-passthrough.txt` — binary passthrough test
  - [ ] `task-26-text-still-pruned.txt` — text pruning still works test

  **Commit**: YES (Wave 4 group)
  - Message: `feat(prunifier): binary output detection and passthrough`
  - Files: `src/proxy/binary_detector.rs`, `tests/binary_output_test.rs`
  - Pre-commit: `cargo test binary_output_test`

- [x] 27. Edge case: ANSI escape code stripping

  **What to do** (TDD):
  - **RED**: Write `tests/ansi_test.rs`: `test_strip_ansi_codes()`, `test_preserve_non_ansi_text()`, `test_colored_ls_output_stripped()`, `test_stripped_before_pruning()`
  - **GREEN**: Create `src/engine/ansi_stripper.rs` with `AnsiStripper`
  - `strip(input: &str) -> String` — remove all ANSI escape sequences (CSI sequences: `\x1b[...m`, OSC sequences, etc.)
  - Use regex: `\x1b\[[0-9;]*[a-zA-Z]` for CSI sequences
  - Strip BEFORE line parser runs (so regex rules match clean text, not ANSI-wrapped text)
  - Call stripper in the dispatcher pipeline (after binary detection, before line parsing)
  - Handle common case: `ls --color=auto` producing ANSI-colored output

  **Must NOT do**:
  - Do NOT re-add color after pruning (pruned output is plain text)
  - Do NOT try to preserve semantic color meaning (just strip)

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Regex-based string replacement — simple
  - **Skills**: None needed
  - **Skills Evaluated but Omitted**: None

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 4 (with Tasks 23-26, 28-29)
  - **Blocks**: Task 30 (final integration tests)
  - **Blocked By**: None (standalone utility)

  **References**:
  - Pattern: ANSI escape code format — `\x1b[` followed by numbers/semicolons, ending in a letter

  **Acceptance Criteria** (if TDD):
  - [ ] `cargo test tests::ansi_test` → all 4 tests PASS
  - [ ] Input `\x1b[31mhello\x1b[0m` → output `hello`
  - [ ] Input without ANSI → output unchanged
  - [ ] ANSI-stripped output then pruned by line parser correctly

  **QA Scenarios**:

  ```
  Scenario: ANSI codes removed from colored output
    Tool: Bash (cargo test)
    Preconditions: tests/ansi_test.rs exists
    Steps:
      1. Run: cargo test test_strip_ansi_codes -- --nocapture 2>&1
      2. Assert: test passes, ANSI sequences absent from output
    Expected Result: ANSI escape codes stripped
    Failure Indicators: Test FAILED, ANSI codes in output
    Evidence: .omo/evidence/task-27-ansi-strip.txt

  Scenario: Colored ls output pruned correctly after stripping
    Tool: Bash
    Preconditions: cargo build succeeded, ls-la scheme exists
    Steps:
      1. Run: ./target/debug/prunify ls -la --color=always /root/prunifier 2>&1
      2. Assert: output does NOT contain "total" (pruning worked after ANSI strip)
      3. Assert: output does NOT contain escape sequences (ANSI stripped)
    Expected Result: Colored ls output is both stripped and pruned
    Failure Indicators: "total" visible, ANSI codes visible
    Evidence: .omo/evidence/task-27-ansi-ls-pruned.txt
  ```

  **Evidence to Capture**:
  - [ ] `task-27-ansi-strip.txt` — unit test output
  - [ ] `task-27-ansi-ls-pruned.txt` — integration test output

  **Commit**: YES (Wave 4 group)
  - Message: `feat(prunifier): ANSI escape code stripping before pruning`
  - Files: `src/engine/ansi_stripper.rs`, `tests/ansi_test.rs`
  - Pre-commit: `cargo test ansi_test`

- [x] 28. Edge case: Unicode/multibyte support

  **What to do** (TDD):
  - **RED**: Write `tests/unicode_test.rs`: `test_unicode_filenames_in_output()`, `test_emoji_in_output()`, `test_multibyte_regex_matching()`, `test_column_split_with_unicode()`
  - **GREEN**: Verify existing line parser and column selector handle Unicode correctly (UTF-8 strings in Rust handle this natively)
  - Add targeted tests for: Chinese filenames in `ls`, emoji in command output, CJK characters in `ps COMMAND` column
  - If any test fails: fix the parser/splitter to use proper Unicode-aware operations (e.g., `chars()` not raw byte indexing)
  - Ensure regex patterns in scheme JSON are valid UTF-8 (serde enforces this)
  - Verify `split_whitespace()` handles Unicode whitespace correctly

  **Must NOT do**:
  - Do NOT add ICU or external Unicode libraries (Rust's std handles UTF-8 natively)
  - Do NOT change the scheme format to support Unicode patterns differently

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Adding Unicode test cases to verify Rust's native UTF-8 handling — mostly pass-through
  - **Skills**: None needed
  - **Skills Evaluated but Omitted**: None

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 4 (with Tasks 23-27, 29)
  - **Blocks**: Task 30 (final integration tests)
  - **Blocked By**: Tasks 9 (line parser), 10 (column selector) — the modules being verified

  **References**:
  - Official docs: `https://doc.rust-lang.org/std/primitive.str.html` — Rust str type (UTF-8 native)

  **Acceptance Criteria** (if TDD):
  - [ ] `cargo test tests::unicode_test` → all 4 tests PASS
  - [ ] `ls` output with filename "ファイル.txt" → column selector finds and keeps it correctly
  - [ ] Regex pattern matching emoji ✅ → works as expected
  - [ ] `ps` output with CJK COMMAND name → column split works correctly

  **QA Scenarios**:

  ```
  Scenario: Unicode filenames in ls output pruned correctly
    Tool: Bash
    Preconditions: cargo build succeeded
    Steps:
      1. mkdir /tmp/test-unicode && cd /tmp/test-unicode
      2. touch "résumé.txt" "ファイル.txt" "😀.txt"
      3. Run: /root/prunifier/target/debug/prunify ls -la 2>&1
      4. Assert: output contains "résumé.txt"
      5. Assert: output contains "ファイル.txt"
      6. Assert: output does NOT contain "total" (pruned)
    Expected Result: Unicode filenames preserved after pruning
    Failure Indicators: Unicode corrupted, "total" not pruned
    Evidence: .omo/evidence/task-28-unicode-ls.txt

  Scenario: Emoji in output handled
    Tool: Bash
    Preconditions: cargo build succeeded
    Steps:
      1. Run: /root/prunifier/target/debug/prunify echo "✅ Tests pass 😀" 2>&1
      2. Assert: output contains "✅ Tests pass 😀"
    Expected Result: Emoji preserved in passthrough
    Failure Indicators: Emoji corrupted or missing
    Evidence: .omo/evidence/task-28-unicode-emoji.txt
  ```

  **Evidence to Capture**:
  - [ ] `task-28-unicode-ls.txt` — unicode filenames test
  - [ ] `task-28-unicode-emoji.txt` — emoji passthrough test

  **Commit**: YES (Wave 4 group)
  - Message: `test(prunifier): unicode and multibyte character handling`
  - Files: `tests/unicode_test.rs`
  - Pre-commit: `cargo test unicode_test`

- [x] 29. Edge case: Signal passthrough (SIGINT/SIGTERM)

  **What to do**:
  - Create `src/proxy/signal_handler.rs` with signal forwarding logic
  - When prunify receives SIGINT (Ctrl+C), forward it to the child process
  - Register signal handler with `ctrlc` crate (add to Cargo.toml: `ctrlc = "3"`)
  - On SIGINT: send SIGINT to child process PID, wait for child to exit, then exit with child's exit code (or 130 if child doesn't exit)
  - On prunify's own shutdown: ensure child process is reaped (drop the Command handle or wait)
  - Write integration test: `tests/signal_test.sh` — start `prunify sleep 30`, send SIGINT to prunify PID, verify both prunify and sleep terminate
  - Update executor to track child PID for signal forwarding

  **Must NOT do**:
  - Do NOT install global signal handlers that interfere with Rust's runtime
  - Do NOT handle SIGKILL (unhandleable) or SIGSTOP

  **Recommended Agent Profile**:
  - **Category**: `deep`
    - Reason: Signal handling, process management, and race conditions — requires careful implementation
  - **Skills**: None needed
  - **Skills Evaluated but Omitted**: None

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 4 (with Tasks 23-28, 30)
  - **Blocks**: Task 30 (final integration tests cover signals)
  - **Blocked By**: Task 16 (executor handles child processes)

  **References**:
  - Official docs: `https://docs.rs/ctrlc/latest/ctrlc/` — ctrlc crate for signal handling
  - Official docs: `https://doc.rust-lang.org/std/process/struct.Child.html#method.kill` — Child::kill() for signal forwarding

  **Acceptance Criteria**:
  - [ ] `cargo test` — signal handler unit tests pass
  - [ ] `prunify sleep 30 &` → sending SIGINT to prunify PID → both prunify and sleep terminate
  - [ ] `prunify sleep 30 &` → sending SIGTERM to prunify PID → both prunify and sleep terminate
  - [ ] Exit code after SIGINT is 130 (128 + 2, standard signal exit convention)

  **QA Scenarios**:

  ```
  Scenario: SIGINT forwarded to child process
    Tool: Bash
    Preconditions: cargo build succeeded
    Steps:
      1. Run: ./target/debug/prunify sleep 30 &
      2. Capture: PRUNIFY_PID=$!
      3. Run: sleep 1  # let sleep start
      4. Run: kill -INT $PRUNIFY_PID
      5. Run: wait $PRUNIFY_PID 2>/dev/null
      6. Capture: EXIT_CODE=$?
      7. Assert: EXIT_CODE is 130 (or non-zero)
      8. Assert: sleep 30 process is no longer running
    Expected Result: SIGINT forwarded, both processes terminate
    Failure Indicators: Sleep still running after SIGINT, exit code 0
    Evidence: .omo/evidence/task-29-signal-sigint.txt

  Scenario: Normal exit still works (signal handler doesn't break clean exit)
    Tool: Bash
    Preconditions: cargo build succeeded
    Steps:
      1. Run: ./target/debug/prunify echo hello
      2. Assert: exit code is 0
    Expected Result: Normal commands still work after signal handler setup
    Failure Indicators: Non-zero exit on normal command
    Evidence: .omo/evidence/task-29-signal-normal.txt
  ```

  **Evidence to Capture**:
  - [ ] `task-29-signal-sigint.txt` — SIGINT forwarding test
  - [ ] `task-29-signal-normal.txt` — normal exit still works

  **Commit**: YES (Wave 4 group)
  - Message: `feat(prunifier): signal forwarding (SIGINT/SIGTERM) to child process`
  - Files: `src/proxy/signal_handler.rs`, `Cargo.toml` (add ctrlc dep)
  - Pre-commit: `cargo test`

- [x] 30. Final integration test suite (all modes + all edge cases)

  **What to do**:
  - Expand `tests/shell_tests.sh` into a comprehensive test suite covering ALL acceptance criteria
  - Add scenarios: mode 1 exact match for all 3 built-in schemes, mode 2 prefix match with marking, mode 3 unknown command passthrough, binary output passthrough, ANSI stripping + pruning, unicode in output, signal forwarding, recursion guard, TTY detection (non-TTY mode), exit code propagation for all modes, `--no-mark` flag, `--scheme-dir` override, invalid scheme handling (error message, not crash), empty output, large output (1000+ lines)
  - Each test: setup → run → assert → cleanup
  - Add summary at end: `echo "PASS: $PASS | FAIL: $FAIL | TOTAL: $TOTAL"`
  - Add `make test-integration` target or document in README how to run
  - Run from CI: `cargo build --release && bash tests/shell_tests.sh`

  **Must NOT do**:
  - Do NOT skip any mode or edge case scenario
  - Do NOT write tests that depend on specific system state (use temp dirs)

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
    - Reason: Comprehensive test suite requiring understanding of all modes and edge cases
  - **Skills**: None needed
  - **Skills Evaluated but Omitted**: None

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Wave 4 (sequential — depends on ALL prior tasks)
  - **Blocks**: F1-F4 (verification wave validates against these tests)
  - **Blocked By**: Tasks 22 (integration test scaffold), 26 (binary), 27 (ANSI), 28 (unicode), 29 (signals)

  **References**:
  - Pattern: `tests/shell_tests.sh` from Task 22 — existing shell test structure
  - Pattern: `.prunifier/schemes/git-status.json` — Built-in scheme
  - Pattern: `.prunifier/schemes/ls-la.json` — Built-in scheme

  **Acceptance Criteria**:
  - [ ] `bash tests/shell_tests.sh` runs all scenarios
  - [ ] ALL scenarios PASS (exit 0 from script)
  - [ ] At least 20 distinct test scenarios
  - [ ] Coverage: all 3 modes, all 3 built-in schemes, all edge cases (binary, ANSI, unicode, signals, recursion)
  - [ ] Summary line at end: "PASS: N | FAIL: 0 | TOTAL: N"

  **QA Scenarios**:

  ```
  Scenario: Full test suite runs without failures
    Tool: Bash
    Preconditions: cargo build --release succeeded
    Steps:
      1. Run: bash tests/shell_tests.sh 2>&1
      2. Assert: exit code is 0
      3. Assert: output contains "FAIL: 0"
      4. Assert: output contains "PASS:" with count >= 20
    Expected Result: All integration tests pass
    Failure Indicators: Non-zero exit, any test failure
    Evidence: .omo/evidence/task-30-full-suite.txt

  Scenario: Individual mode-1 test works
    Tool: Bash
    Preconditions: shell_tests.sh has mode1 test
    Steps:
      1. Run: bash tests/shell_tests.sh 2>&1 | grep -i "mode1\|exact.*match\|test_mode1"
      2. Assert: output shows passing status
    Expected Result: Mode 1 tests individually pass
    Failure Indicators: Mode 1 test failures
    Evidence: .omo/evidence/task-30-mode1-detail.txt
  ```

  **Evidence to Capture**:
  - [ ] `task-30-full-suite.txt` — complete test suite output
  - [ ] `task-30-mode1-detail.txt` — mode 1 specific results

  **Commit**: YES (Wave 4 group)
  - Message: `test(prunifier): comprehensive integration test suite covering all modes and edge cases`
  - Files: `tests/shell_tests.sh`
  - Pre-commit: `bash tests/shell_tests.sh`

---

## Final Verification Wave (MANDATORY — after ALL implementation tasks)

> 4 review agents run in PARALLEL. ALL must APPROVE. Present consolidated results to user and get explicit "okay" before completing.
>
> **Do NOT auto-proceed after verification. Wait for user's explicit approval before marking work complete.**
> **Never mark F1-F4 as checked before getting user's okay.** Rejection or user feedback -> fix -> re-run -> present again -> wait for okay.

- [x] F1. **Plan Compliance Audit** — `oracle`
  Read the plan end-to-end. For each "Must Have": verify implementation exists (read file, run cargo test, run prunify command). For each "Must NOT Have": search codebase for forbidden patterns — reject with file:line if found. Check evidence files exist in .omo/evidence/. Compare deliverables against plan.
  Output: `Must Have [N/N] | Must NOT Have [N/N] | Tasks [N/N] | VERDICT: APPROVE/REJECT`

- [x] F2. **Code Quality Review** — `unspecified-high`
  Run `cargo clippy -- -D warnings` + `cargo test` + `cargo fmt --check`. Review all changed files for: `unwrap()` outside tests, empty catches, println!/eprintln! in prod code, commented-out code, unused imports. Check AI slop: excessive comments, over-abstraction, generic names.
  Output: `Build [PASS/FAIL] | Clippy [PASS/FAIL] | Tests [N pass/N fail] | Format [PASS/FAIL] | VERDICT: APPROVE/REJECT`

- [x] F3. **Real Manual QA** — `unspecified-high`
  Start from clean state (`cargo build --release`). Execute EVERY QA scenario from EVERY task — follow exact steps, capture evidence. Test cross-task integration (all three modes working together). Test edge cases: empty state, binary output, unicode, ANSI. Save to `.omo/evidence/final-qa/`.
  Output: `Scenarios [N/N pass] | Integration [N/N] | Edge Cases [N tested] | VERDICT: APPROVE/REJECT`

- [x] F4. **Scope Fidelity Check** — `deep`
  For each task: read "What to do", read actual diff (git log/diff). Verify 1:1 — everything in spec was built (no missing), nothing beyond spec was built (no creep). Check "Must NOT do" compliance. Detect cross-task contamination: Task N touching Task M's files. Flag unaccounted changes.
  Output: `Tasks [N/N compliant] | Contamination [CLEAN/N issues] | Unaccounted [CLEAN/N files] | VERDICT: APPROVE/REJECT`

---

## Commit Strategy

- **Wave 1**: `feat(prunifier): project scaffolding, types, and schema definitions` — Cargo.{toml,lock}, src/types/*.rs
- **Wave 2**: `feat(prunifier): core engine — trie matcher, line parser, schemes, config` — src/engine/*.rs, .prunifier/schemes/*.json
- **Wave 3**: `feat(prunifier): proxy engine — executor, dispatcher, TTY guard, CLI` — src/proxy/*.rs, src/main.rs
- **Wave 4**: `feat(prunifier): skill, edge cases, final integration` — .opencode/skills/*, src/edge/*.rs, tests/
- **FINAL**: `chore(prunifier): verification wave fixes` — various (from F1-F4 findings)

---

## Success Criteria

### Verification Commands
```bash
cargo build --release                  # Expected: exit 0, binary at target/release/prunify
cargo test                             # Expected: ALL pass
cargo clippy -- -D warnings            # Expected: exit 0, zero warnings
./target/release/prunify echo hello # Expected: "hello" on stdout, exit 0
```

### Final Checklist
- [ ] All "Must Have" present (9 items)
- [ ] All "Must NOT Have" absent (12 items)
- [ ] All 30 implementation tasks complete
- [ ] All 4 Final Verification tasks APPROVE
- [ ] User explicitly approves F1-F4 results
- [ ] `cargo build --release` clean
- [ ] `cargo test` all pass
- [ ] At least 3 built-in schemes operational
