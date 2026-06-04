#!/bin/bash
# signal_test.sh — Verify SIGINT and SIGTERM forwarding to child process
#
# Starts prunify with a long-running command (sleep 30), sends a signal
# (SIGINT or SIGTERM), and verifies the signal reaches the child (process
# exits quickly, not after the full sleep duration).
#
# Usage:
#   ./tests/signal_test.sh
#
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PRUNIFY_BIN="$SCRIPT_DIR/../target/debug/prunify"

PASS=0
FAIL=0

green() { printf "\033[32m%s\033[0m\n" "$1"; }
red()   { printf "\033[31m%s\033[0m\n" "$1"; }

echo ""
echo "=== prunify signal forwarding test ==="
echo ""

# -----------------------------------------------------------------------
# Test 1 — SIGINT reaches the child process
# -----------------------------------------------------------------------
test1() {
    local prunify_pid start_time end_time elapsed

    # Start prunify with a 30-second sleep in the background
    "$PRUNIFY_BIN" sleep 30 &
    prunify_pid=$!

    # Give it a moment to spawn the child and register the PID
    sleep 1

    # Send SIGINT to prunify — handler must forward to child
    start_time=$(date +%s)
    kill -INT "$prunify_pid"

    # Wait for prunify to exit (should return quickly, not 30s from sleep)
    wait "$prunify_pid" 2>/dev/null || true
    end_time=$(date +%s)

    elapsed=$((end_time - start_time))

    # If the signal was properly forwarded, sleep was killed and prunify
    # exited in well under 30 seconds. Use 10s as a generous upper bound.
    if [ "$elapsed" -lt 10 ]; then
        return 0
    else
        echo "  (took ${elapsed}s — expected <10s)"
        return 1
    fi
}
if test1; then
    green "  PASS: [1] SIGINT forwarded — child exited in <10s"
    PASS=$((PASS + 1))
else
    red "  FAIL: [1] SIGINT forwarded — child slept full duration"
    FAIL=$((FAIL + 1))
fi

# -----------------------------------------------------------------------
# Test 2 — Normal execution (no signal) completes normally
# -----------------------------------------------------------------------
test2() {
    local output

    output=$("$PRUNIFY_BIN" echo "signal-test-ok" 2>/dev/null)
    echo "$output" | grep -q "signal-test-ok" || return 1
    return 0
}
if test2; then
    green "  PASS: [2] normal execution unaffected by signal handler"
    PASS=$((PASS + 1))
else
    red "  FAIL: [2] normal execution broken by signal handler"
    FAIL=$((FAIL + 1))
fi

# -----------------------------------------------------------------------
# Test 3 — SIGTERM reaches the child process
# -----------------------------------------------------------------------
test3() {
    local prunify_pid start_time end_time elapsed

    # Start prunify with a 30-second sleep in the background
    "$PRUNIFY_BIN" sleep 30 &
    prunify_pid=$!

    # Give it a moment to spawn the child and register the PID
    sleep 1

    # Send SIGTERM to prunify — handler must forward to child
    start_time=$(date +%s)
    kill -TERM "$prunify_pid"

    # Wait for prunify to exit (should return quickly, not 30s from sleep)
    wait "$prunify_pid" 2>/dev/null || true
    end_time=$(date +%s)

    elapsed=$((end_time - start_time))

    # If the signal was properly forwarded, sleep was killed and prunify
    # exited in well under 30 seconds. Use 10s as a generous upper bound.
    if [ "$elapsed" -lt 10 ]; then
        return 0
    else
        echo "  (took ${elapsed}s — expected <10s)"
        return 1
    fi
}
if test3; then
    green "  PASS: [3] SIGTERM forwarded — child exited in <10s"
    PASS=$((PASS + 1))
else
    red "  FAIL: [3] SIGTERM forwarded — child slept full duration"
    FAIL=$((FAIL + 1))
fi

# -----------------------------------------------------------------------
# Summary
# -----------------------------------------------------------------------
echo ""
echo "=== results: $PASS passed, $FAIL failed ==="
exit "$FAIL"
