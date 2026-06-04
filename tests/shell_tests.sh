#!/bin/bash
# shell_tests.sh — End-to-end integration tests for prunify
#
# Tests all 3 modes, built-in schemes, exit codes, flags, recursion guard,
# stderr passthrough, and the --no-mark / --scheme-dir options.
#
# Usage:
#   ./tests/shell_tests.sh
#
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PRUNIFY_BIN="$SCRIPT_DIR/../target/debug/prunify"
SCHEMES_DIR="$SCRIPT_DIR/../.prunify/schemes"
TEMP_DIR="/tmp/prunify-tests-$$"

PASS=0
FAIL=0

green() { printf "\033[32m%s\033[0m\n" "$1"; }
red()   { printf "\033[31m%s\033[0m\n" "$1"; }

# Cleanup temp dirs on exit
cleanup() { rm -rf "$TEMP_DIR"; }
trap cleanup EXIT

mkdir -p "$TEMP_DIR"

echo ""
echo "=== prunify shell tests ==="
echo ""

# -----------------------------------------------------------------------
# Test 1 — MODE 1 (exact match): ls -la scheme prunes total, ., ..
# -----------------------------------------------------------------------
test1() {
    local output
    output=$("$PRUNIFY_BIN" --no-mark ls -la 2>/dev/null)
    # The "total N" line must be removed
    echo "$output" | grep -q "^total" && return 1
    # The "." entry (line ending with " .") must be removed
    echo "$output" | grep -q " \.$" && return 1
    # The ".." entry (line ending with " ..") must be removed
    echo "$output" | grep -q " \.\.$" && return 1
    return 0
}
if test1; then
    green "  PASS: [1] ls -la exact match — total / ./.. pruned"
    PASS=$((PASS + 1))
else
    red "  FAIL: [1] ls -la exact match — total / ./.. pruned"
    FAIL=$((FAIL + 1))
fi

# -----------------------------------------------------------------------
# Test 2 — MODE 1 (exact match): git status scheme prunes "On branch"
# -----------------------------------------------------------------------
test2() {
    local git_dir="$TEMP_DIR/git-test-2"
    rm -rf "$git_dir" && mkdir -p "$git_dir"
    (
        cd "$git_dir" || exit 1
        git init >/dev/null 2>&1
        touch foo
        output=$("$PRUNIFY_BIN" --scheme-dir "$SCHEMES_DIR" --no-mark git status 2>/dev/null)
        echo "$output" | grep -q "On branch" && exit 1
        exit 0
    )
}
if test2; then
    green "  PASS: [2] git status exact match — 'On branch' pruned"
    PASS=$((PASS + 1))
else
    red "  FAIL: [2] git status exact match — 'On branch' pruned"
    FAIL=$((FAIL + 1))
fi

# -----------------------------------------------------------------------
# Test 3 — MODE 2 (prefix match): ls -la --color=auto returns [PRUNED]
# -----------------------------------------------------------------------
test3() {
    local output
    output=$("$PRUNIFY_BIN" ls -la --color=auto 2>/dev/null)
    echo "$output" | grep -q "\[PRUNED\]" || return 1
    return 0
}
if test3; then
    green "  PASS: [3] prefix match — [PRUNED] mark present"
    PASS=$((PASS + 1))
else
    red "  FAIL: [3] prefix match — [PRUNED] mark present"
    FAIL=$((FAIL + 1))
fi

# -----------------------------------------------------------------------
# Test 4 — MODE 3 (passthrough): unknown command outputs [UNKNOWN COMMAND]
# -----------------------------------------------------------------------
test4() {
    local output
    output=$("$PRUNIFY_BIN" echo hello 2>&1)
    echo "$output" | grep -q "\[UNKNOWN COMMAND\]" || return 1
    # The proxied command output should also be present
    echo "$output" | grep -q "hello" || return 1
    return 0
}
if test4; then
    green "  PASS: [4] unknown command — [UNKNOWN COMMAND] mark present"
    PASS=$((PASS + 1))
else
    red "  FAIL: [4] unknown command — [UNKNOWN COMMAND] mark present"
    FAIL=$((FAIL + 1))
fi

# -----------------------------------------------------------------------
# Test 5 — Exit code propagation: prunify exits with command's exit code
# -----------------------------------------------------------------------
test5() {
    "$PRUNIFY_BIN" false >/dev/null 2>&1
    local rc=$?
    # false exits with code 1
    [ "$rc" -eq 1 ] || return 1
    return 0
}
if test5; then
    green "  PASS: [5] exit code propagation — exit 1 from false"
    PASS=$((PASS + 1))
else
    red "  FAIL: [5] exit code propagation — exit 1 from false"
    FAIL=$((FAIL + 1))
fi

# -----------------------------------------------------------------------
# Test 6 — Recursion guard: prunify detects self-invocation
# -----------------------------------------------------------------------
test6() {
    local stderr
    # Capture stderr only (stdout goes to /dev/null)
    stderr=$("$PRUNIFY_BIN" prunify echo hello 2>&1 1>/dev/null)
    echo "$stderr" | grep -qi "recursion" || return 1
    return 0
}
if test6; then
    green "  PASS: [6] recursion guard — recursion detected"
    PASS=$((PASS + 1))
else
    red "  FAIL: [6] recursion guard — recursion detected"
    FAIL=$((FAIL + 1))
fi

# -----------------------------------------------------------------------
# Test 7 — --no-mark flag suppresses [UNKNOWN COMMAND] and [PRUNED]
# -----------------------------------------------------------------------
test7() {
    local output
    output=$("$PRUNIFY_BIN" --no-mark echo hello 2>&1)
    # [UNKNOWN COMMAND] must NOT appear
    echo "$output" | grep -q "\[UNKNOWN COMMAND\]" && return 1
    # But the proxied output (hello) must still be present
    echo "$output" | grep -q "hello" || return 1
    return 0
}
if test7; then
    green "  PASS: [7] --no-mark — no [UNKNOWN COMMAND]"
    PASS=$((PASS + 1))
else
    red "  FAIL: [7] --no-mark — no [UNKNOWN COMMAND]"
    FAIL=$((FAIL + 1))
fi

# -----------------------------------------------------------------------
# Test 8 — --version flag
# -----------------------------------------------------------------------
test8() {
    local output
    output=$("$PRUNIFY_BIN" --version 2>&1)
    echo "$output" | grep -q "prunify 0.1.0" || return 1
    return 0
}
if test8; then
    green "  PASS: [8] --version — shows prunify 0.1.0"
    PASS=$((PASS + 1))
else
    red "  FAIL: [8] --version — shows prunify 0.1.0"
    FAIL=$((FAIL + 1))
fi

# -----------------------------------------------------------------------
# Test 9 — --help flag
# -----------------------------------------------------------------------
test9() {
    local output
    output=$("$PRUNIFY_BIN" --help 2>&1)
    echo "$output" | grep -qi "usage" || return 1
    echo "$output" | grep -qi "proxy" || return 1
    return 0
}
if test9; then
    green "  PASS: [9] --help — usage text displayed"
    PASS=$((PASS + 1))
else
    red "  FAIL: [9] --help — usage text displayed"
    FAIL=$((FAIL + 1))
fi

# -----------------------------------------------------------------------
# Test 10 — --scheme-dir custom path
# -----------------------------------------------------------------------
test10() {
    local custom_dir="$TEMP_DIR/custom-schemes"
    rm -rf "$custom_dir" && mkdir -p "$custom_dir"
    cp "$SCHEMES_DIR/ls-la.json" "$custom_dir/"
    local output
    output=$("$PRUNIFY_BIN" --scheme-dir "$custom_dir" --no-mark ls -la 2>/dev/null)
    # Should prune "total" line using the custom scheme
    echo "$output" | grep -q "^total" && return 1
    return 0
}
if test10; then
    green "  PASS: [10] --scheme-dir — custom path prunes output"
    PASS=$((PASS + 1))
else
    red "  FAIL: [10] --scheme-dir — custom path prunes output"
    FAIL=$((FAIL + 1))
fi

# -----------------------------------------------------------------------
# Test 11 — ps aux scheme: root processes removed
# -----------------------------------------------------------------------
test11() {
    local output
    output=$("$PRUNIFY_BIN" --no-mark ps aux 2>/dev/null)
    # Verify no root processes remain in pruned output
    echo "$output" | grep -q "^root" && return 1
    return 0
}
if test11; then
    green "  PASS: [11] ps aux — no root processes in output"
    PASS=$((PASS + 1))
else
    red "  FAIL: [11] ps aux — no root processes in output"
    FAIL=$((FAIL + 1))
fi

# -----------------------------------------------------------------------
# Test 12 — Stderr passthrough: errors from proxied command appear on stderr
# -----------------------------------------------------------------------
test12() {
    local output
    # Combine stdout+stderr to capture the error
    output=$("$PRUNIFY_BIN" ls /nonexistent 2>&1)
    echo "$output" | grep -qi "No such file" || return 1
    return 0
}
if test12; then
    green "  PASS: [12] stderr passthrough — error message present"
    PASS=$((PASS + 1))
else
    red "  FAIL: [12] stderr passthrough — error message present"
    FAIL=$((FAIL + 1))
fi

# -----------------------------------------------------------------------
# Test 13 — Exact match (mode 1) does NOT append any mark
# -----------------------------------------------------------------------
test13() {
    local output
    output=$("$PRUNIFY_BIN" ls -la 2>/dev/null)
    # Exact match must NOT have [PRUNED] or [UNKNOWN COMMAND]
    echo "$output" | grep -q "\[PRUNED\]" && return 1
    echo "$output" | grep -q "\[UNKNOWN COMMAND\]" && return 1
    return 0
}
if test13; then
    green "  PASS: [13] exact match — no marks appended"
    PASS=$((PASS + 1))
else
    red "  FAIL: [13] exact match — no marks appended"
    FAIL=$((FAIL + 1))
fi

# -----------------------------------------------------------------------
# Test 14 — No arguments shows error
# -----------------------------------------------------------------------
test14() {
    local output rc
    output=$("$PRUNIFY_BIN" 2>&1) || rc=$?
    # Should exit with non-zero and show usage
    [ "${rc:-0}" -ne 0 ] || return 1
    echo "$output" | grep -qi "error" || return 1
    return 0
}
if test14; then
    green "  PASS: [14] no args — error and non-zero exit"
    PASS=$((PASS + 1))
else
    red "  FAIL: [14] no args — error and non-zero exit"
    FAIL=$((FAIL + 1))
fi

# -----------------------------------------------------------------------
# Test 15 — Binary passthrough: byte-identical output with --no-mark
# -----------------------------------------------------------------------
test15() {
    local bin_file="$TEMP_DIR/binary-test.bin"
    local ref_hash prun_hash

    # Generate 256 bytes of random binary data
    dd if=/dev/urandom of="$bin_file" bs=256 count=1 2>/dev/null

    # Direct checksum of the raw binary
    ref_hash=$(md5sum < "$bin_file" | cut -d' ' -f1)

    # Checksum after piping through prunify --no-mark
    prun_hash=$("$PRUNIFY_BIN" --no-mark cat "$bin_file" 2>/dev/null | md5sum | cut -d' ' -f1)

    [ "$ref_hash" = "$prun_hash" ] || return 1
    return 0
}
if test15; then
    green "  PASS: [15] binary passthrough — byte-identical output"
    PASS=$((PASS + 1))
else
    red "  FAIL: [15] binary passthrough — output differs"
    FAIL=$((FAIL + 1))
fi

# -----------------------------------------------------------------------
# Test 16 — ANSI passthrough: escape sequences preserved without corruption
# -----------------------------------------------------------------------
test16() {
    local direct_hash prun_hash

    # Generate known ANSI output, checksum directly
    direct_hash=$(printf '\x1b[31mRED\x1b[0m\n' | md5sum | cut -d' ' -f1)

    # Same input through prunify --no-mark cat
    prun_hash=$(printf '\x1b[31mRED\x1b[0m\n' | "$PRUNIFY_BIN" --no-mark cat 2>/dev/null | md5sum | cut -d' ' -f1)

    [ "$direct_hash" = "$prun_hash" ] || return 1
    return 0
}
if test16; then
    green "  PASS: [16] ANSI passthrough — escape sequences preserved"
    PASS=$((PASS + 1))
else
    red "  FAIL: [16] ANSI passthrough — escape sequences corrupted"
    FAIL=$((FAIL + 1))
fi

# -----------------------------------------------------------------------
# Test 17 — Unicode passthrough: multi-byte characters preserved
# -----------------------------------------------------------------------
test17() {
    local output
    output=$("$PRUNIFY_BIN" --no-mark echo "✅ ファイル 中文 test" 2>/dev/null)
    echo "$output" | grep -q "✅"      || return 1
    echo "$output" | grep -q "ファイル" || return 1
    echo "$output" | grep -q "中文"     || return 1
    echo "$output" | grep -q "test"     || return 1
    return 0
}
if test17; then
    green "  PASS: [17] unicode passthrough — characters preserved"
    PASS=$((PASS + 1))
else
    red "  FAIL: [17] unicode passthrough — characters missing"
    FAIL=$((FAIL + 1))
fi

# -----------------------------------------------------------------------
# Test 18 — Large output: 5000 lines through prunify
# -----------------------------------------------------------------------
test18() {
    local line_count
    line_count=$("$PRUNIFY_BIN" --no-mark seq 1 5000 2>/dev/null | wc -l)
    [ "$line_count" -eq 5000 ] || return 1
    return 0
}
if test18; then
    green "  PASS: [18] large output — 5000 lines preserved"
    PASS=$((PASS + 1))
else
    red "  FAIL: [18] large output — expected 5000 lines, got ${line_count:-0}"
    FAIL=$((FAIL + 1))
fi

# -----------------------------------------------------------------------
# Test 19 — Empty output: true command produces nothing
# -----------------------------------------------------------------------
test19() {
    local output rc
    output=$("$PRUNIFY_BIN" --no-mark true 2>/dev/null) || rc=$?
    [ -z "$output" ]     || return 1
    [ "${rc:-0}" -eq 0 ] || return 1
    return 0
}
if test19; then
    green "  PASS: [19] empty output — stdout empty, exit 0"
    PASS=$((PASS + 1))
else
    red "  FAIL: [19] empty output — unexpected stdout or non-zero exit"
    FAIL=$((FAIL + 1))
fi

# -----------------------------------------------------------------------
# Test 20 — Signal integration: SIGINT forwarded to child
# -----------------------------------------------------------------------
test20() {
    bash "$SCRIPT_DIR/signal_test.sh"
}
if test20; then
    green "  PASS: [20] signal integration — signal_test.sh passes"
    PASS=$((PASS + 1))
else
    red "  FAIL: [20] signal integration — signal_test.sh failed"
    FAIL=$((FAIL + 1))
fi

# -----------------------------------------------------------------------
# Summary
# -----------------------------------------------------------------------
echo ""
echo "=== results: $PASS passed, $FAIL failed ==="
exit "$FAIL"
