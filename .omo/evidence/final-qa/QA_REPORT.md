# Final QA Report — prunifier v0.1.0

## Manual Scenarios (13)

| # | Scenario | Status | Notes |
|---|----------|--------|-------|
| 1 | `cargo build --release` | ✅ PASS | Build succeeds (2 warnings: unused import, dead_code) |
| 2 | `prunify echo hello` | ✅ PASS | Shows "hello" + [UNKNOWN COMMAND] mark |
| 3 | `prunify --no-mark echo hello` | ✅ PASS | Shows "hello" with no marks |
| 4 | `prunify --version` | ✅ PASS | Shows "prunifier 0.1.0" |
| 5 | `prunify --help` | ✅ PASS | Shows usage with proxy description |
| 6 | `prunify ls -la` | ✅ PASS | "total" line, "." and ".." entries pruned |
| 7 | `prunify ls -la --color=auto` | ✅ PASS | [PRUNED] mark present (prefix match) |
| 8 | `prunify git status` | ✅ PASS | "On branch" headers pruned |
| 9 | `prunify ps aux` | ✅ PASS | All root processes pruned (container: all are root) |
| 10 | `prunify sh -c "exit 42"` | ❌ FAIL | Expected exit 42, got 0. Bug: `join(" ")+split_whitespace()` round-trip breaks quoted args — `sh -c "exit 42"` becomes `sh -c exit 42` where `exit` runs without arg (exits 0). |
| 11 | `prunify prunify echo hello` | ✅ PASS | Recursion guard activates |
| 12 | `prunify ls /nonexistent` | ✅ PASS | "No such file" on stderr, exit code 2 |
| 13 | `prunify --scheme-dir .prunifier/schemes --no-mark ls -la` | ✅ PASS | Works with explicit scheme dir |

**Manual: 12/13 PASS**

## Shell Tests (tests/shell_tests.sh)

| Test | Description | Status |
|------|-------------|--------|
| [1] | ls -la exact match — total / ./.. pruned | ✅ PASS |
| [2] | git status exact match — 'On branch' pruned | ✅ PASS |
| [3] | prefix match — [PRUNED] mark present | ✅ PASS |
| [4] | unknown command — [UNKNOWN COMMAND] mark present | ✅ PASS |
| [5] | exit code propagation — exit 1 from false | ✅ PASS |
| [6] | recursion guard — recursion detected | ✅ PASS |
| [7] | --no-mark — no [UNKNOWN COMMAND] | ✅ PASS |
| [8] | --version — shows prunifier 0.1.0 | ✅ PASS |
| [9] | --help — usage text displayed | ✅ PASS |
| [10] | --scheme-dir — custom path prunes output | ✅ PASS |
| [11] | ps aux — no root processes in output | ✅ PASS |
| [12] | stderr passthrough — error message present | ✅ PASS |
| [13] | exact match — no marks appended | ✅ PASS |
| [14] | no args — error and non-zero exit | ✅ PASS |
| [15] | binary passthrough — byte-identical output | ✅ PASS |
| [16] | ANSI passthrough — escape sequences preserved | ✅ PASS |
| [17] | unicode passthrough — characters preserved | ✅ PASS |
| [18] | large output — 5000 lines preserved | ✅ PASS |
| [19] | empty output — stdout empty, exit 0 | ✅ PASS |
| [20] | signal integration — signal_test.sh passes | ✅ PASS |

**Shell Tests: 20/20 PASS**

## Signal Tests (tests/signal_test.sh)

| Test | Description | Status |
|------|-------------|--------|
| [1] | SIGINT forwarded — child exited in <10s | ✅ PASS |
| [2] | normal execution unaffected by signal handler | ✅ PASS |

**Signal Tests: 2/2 PASS**

## Exit Code Note

Exit code propagation works correctly for single commands (false→1, ls /nonexistent→2, ls nonexistent_dir→2). 
But for `sh -c "exit 42"`, the argument join/split round-trip (join(" ") then split_whitespace()) loses the 
quoted argument grouping. The string "exit 42" (one arg to sh -c) becomes two separate args "exit" and "42".
This causes `sh` to execute `exit` (no arg → exits 0) instead of `exit 42`.

Not detected by shell_tests.sh because test[5] uses `false` (single binary, no argument quoting issue).
