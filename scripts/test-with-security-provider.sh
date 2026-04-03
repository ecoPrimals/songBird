#!/usr/bin/env bash
# test-with-security-provider.sh — Run the full Songbird test suite with a live security provider.
#
# Fetches beardog from plasmidBin if not present, starts it on a temp Unix socket,
# runs `cargo test --workspace --all-features`, then reports results.
#
# Usage:
#   ./scripts/test-with-security-provider.sh              # Full test suite
#   ./scripts/test-with-security-provider.sh -- -p crate  # Pass extra args to cargo test
#
# Environment:
#   BEARDOG_BIN  — explicit path to security provider binary (skips discovery)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
WORKSPACE_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# --- Resolve plasmidBin --------------------------------------------------
PLASMID_BIN="${ECOPRIMALS_PLASMID_BIN:-""}"
if [ -z "$PLASMID_BIN" ]; then
    # Walk up from workspace looking for infra/plasmidBin
    candidate="$WORKSPACE_ROOT/../../infra/plasmidBin"
    if [ -d "$candidate" ]; then
        PLASMID_BIN="$(cd "$candidate" && pwd)"
    fi
fi

# --- Discover or fetch beardog binary -------------------------------------
BEARDOG="${BEARDOG_BIN:-""}"
if [ -z "$BEARDOG" ] && [ -n "$PLASMID_BIN" ]; then
    BEARDOG="$PLASMID_BIN/primals/beardog"
fi

if [ -z "$BEARDOG" ] || [ ! -f "$BEARDOG" ]; then
    if [ -n "$PLASMID_BIN" ] && [ -f "$PLASMID_BIN/fetch.sh" ]; then
        echo "--- Fetching beardog via plasmidBin/fetch.sh ---"
        bash "$PLASMID_BIN/fetch.sh" --primal beardog
        BEARDOG="$PLASMID_BIN/primals/beardog"
    fi
fi

if [ -z "$BEARDOG" ] || [ ! -f "$BEARDOG" ]; then
    echo "ERROR: beardog binary not found."
    echo "  Set \$BEARDOG_BIN, place it in infra/plasmidBin/primals/beardog,"
    echo "  or ensure plasmidBin/fetch.sh can download it."
    exit 1
fi

chmod +x "$BEARDOG"
echo "--- beardog binary: $BEARDOG ---"

# --- Start beardog on a temp socket ----------------------------------------
SOCKET_DIR="$(mktemp -d)"
SOCKET_PATH="$SOCKET_DIR/beardog-test.sock"
BEARDOG_PID=""

cleanup() {
    if [ -n "$BEARDOG_PID" ]; then
        kill "$BEARDOG_PID" 2>/dev/null || true
        wait "$BEARDOG_PID" 2>/dev/null || true
    fi
    rm -f "$SOCKET_PATH"
    rmdir "$SOCKET_DIR" 2>/dev/null || true
}
trap cleanup EXIT

"$BEARDOG" --socket "$SOCKET_PATH" --mode json-rpc &
BEARDOG_PID=$!

echo "--- Waiting for beardog (PID $BEARDOG_PID) on $SOCKET_PATH ---"
for i in $(seq 1 50); do
    if [ -S "$SOCKET_PATH" ]; then
        echo "--- beardog ready (${i}00 ms) ---"
        break
    fi
    if ! kill -0 "$BEARDOG_PID" 2>/dev/null; then
        echo "ERROR: beardog exited before socket was created"
        exit 1
    fi
    sleep 0.1
done

if [ ! -S "$SOCKET_PATH" ]; then
    echo "ERROR: beardog did not create socket within 5 seconds"
    exit 1
fi

# --- Export environment for tests ------------------------------------------
export BEARDOG_BIN="$BEARDOG"
export BEARDOG_SOCKET="$SOCKET_PATH"
export BEARDOG_SOCKET_PATH="$SOCKET_PATH"
export NEURAL_API_SOCKET="$SOCKET_PATH"

# --- Run tests -------------------------------------------------------------
echo ""
echo "=== Running tests with live BearDog ==="
echo "  BEARDOG_SOCKET=$SOCKET_PATH"
echo ""

cd "$WORKSPACE_ROOT"

# Capture any extra args after --
EXTRA_ARGS=""
if [ "${1:-}" = "--" ]; then
    shift
    EXTRA_ARGS="$*"
fi

# Count baseline ignored tests
IGNORED_BEFORE=$(cargo test --workspace --all-features -- --list 2>/dev/null | grep -c "ignored" || echo "0")

# Run full suite
set +e
cargo test --workspace --all-features $EXTRA_ARGS 2>&1
TEST_EXIT=$?
set -e

# Count remaining ignored tests
IGNORED_AFTER=$(cargo test --workspace --all-features -- --list 2>/dev/null | grep -c "ignored" || echo "0")

echo ""
echo "=== Results ==="
echo "  Exit code: $TEST_EXIT"
echo "  Ignored tests (before BearDog): ~$IGNORED_BEFORE"
echo "  Ignored tests (with BearDog):   ~$IGNORED_AFTER"
echo ""

exit $TEST_EXIT
