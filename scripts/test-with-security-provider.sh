#!/usr/bin/env bash
# test-with-security-provider.sh — Run the full Songbird test suite with a live security provider.
#
# Discovers the security provider binary from plasmidBin, starts it on a temp
# Unix socket, runs `cargo test --workspace --all-features`, then reports results.
#
# Usage:
#   ./scripts/test-with-security-provider.sh              # Full test suite
#   ./scripts/test-with-security-provider.sh -- -p crate  # Pass extra args to cargo test
#
# Environment:
#   SECURITY_PROVIDER_BIN — explicit path to security provider binary (skips discovery)
#   BEARDOG_BIN           — deprecated alias (fallback if SECURITY_PROVIDER_BIN unset)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
WORKSPACE_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# --- Resolve plasmidBin --------------------------------------------------
PLASMID_BIN="${ECOPRIMALS_PLASMID_BIN:-""}"
if [ -z "$PLASMID_BIN" ]; then
    candidate="$WORKSPACE_ROOT/../../infra/plasmidBin"
    if [ -d "$candidate" ]; then
        PLASMID_BIN="$(cd "$candidate" && pwd)"
    fi
fi

# --- Discover or fetch security provider binary ---------------------------
PROVIDER="${SECURITY_PROVIDER_BIN:-${BEARDOG_BIN:-""}}"
if [ -z "$PROVIDER" ] && [ -n "$PLASMID_BIN" ]; then
    # Detect host triple for plasmidBin triple-scoped layout
    HOST_TRIPLE="${CARGO_BUILD_TARGET:-$(rustc -vV 2>/dev/null | awk '/^host:/{print $2}')}"
    HOST_TRIPLE="${HOST_TRIPLE:-x86_64-unknown-linux-gnu}"

    # Triple-scoped discovery (current layout)
    if [ -f "$PLASMID_BIN/primals/$HOST_TRIPLE/beardog" ]; then
        PROVIDER="$PLASMID_BIN/primals/$HOST_TRIPLE/beardog"
    elif [ -f "$PLASMID_BIN/primals/$HOST_TRIPLE/security-provider" ]; then
        PROVIDER="$PLASMID_BIN/primals/$HOST_TRIPLE/security-provider"
    # Legacy flat layout fallback
    elif [ -f "$PLASMID_BIN/primals/security-provider" ]; then
        PROVIDER="$PLASMID_BIN/primals/security-provider"
    elif [ -f "$PLASMID_BIN/primals/beardog" ]; then
        echo "WARN: using legacy flat binary name 'beardog' — migrate to triple-scoped layout"
        PROVIDER="$PLASMID_BIN/primals/beardog"
    fi
fi

if [ -z "$PROVIDER" ] || [ ! -f "$PROVIDER" ]; then
    if command -v membrane >/dev/null 2>&1; then
        echo "--- Fetching security provider via membrane plasmid.fetch ---"
        membrane plasmid.fetch --capability crypto.delegate 2>/dev/null || true
        # Re-discover after fetch
        if [ -n "$PLASMID_BIN" ] && [ -f "$PLASMID_BIN/primals/${HOST_TRIPLE:-unknown}/beardog" ]; then
            PROVIDER="$PLASMID_BIN/primals/${HOST_TRIPLE:-unknown}/beardog"
        fi
    fi
fi

if [ -z "$PROVIDER" ] || [ ! -f "$PROVIDER" ]; then
    echo "ERROR: security provider binary not found."
    echo "  Set \$SECURITY_PROVIDER_BIN, or place it in:"
    echo "    infra/plasmidBin/primals/<triple>/beardog"
    echo "  Or install 'membrane' CLI and run: membrane plasmid.fetch --capability crypto.delegate"
    exit 1
fi

chmod +x "$PROVIDER"
echo "--- security provider binary: $PROVIDER ---"

# --- Start security provider on a temp socket --------------------------------
SOCKET_DIR="$(mktemp -d)"
SOCKET_PATH="$SOCKET_DIR/security-provider-test.sock"
PROVIDER_PID=""

cleanup() {
    if [ -n "$PROVIDER_PID" ]; then
        kill "$PROVIDER_PID" 2>/dev/null || true
        wait "$PROVIDER_PID" 2>/dev/null || true
    fi
    rm -f "$SOCKET_PATH"
    rmdir "$SOCKET_DIR" 2>/dev/null || true
}
trap cleanup EXIT

"$PROVIDER" --socket "$SOCKET_PATH" --mode json-rpc &
PROVIDER_PID=$!

echo "--- Waiting for security provider (PID $PROVIDER_PID) on $SOCKET_PATH ---"
for i in $(seq 1 50); do
    if [ -S "$SOCKET_PATH" ]; then
        echo "--- security provider ready (${i}00 ms) ---"
        break
    fi
    if ! kill -0 "$PROVIDER_PID" 2>/dev/null; then
        echo "ERROR: security provider exited before socket was created"
        exit 1
    fi
    sleep 0.1
done

if [ ! -S "$SOCKET_PATH" ]; then
    echo "ERROR: security provider did not create socket within 5 seconds"
    exit 1
fi

# --- Export environment for tests ------------------------------------------
export SECURITY_PROVIDER_BIN="$PROVIDER"
export SECURITY_PROVIDER_SOCKET="$SOCKET_PATH"
# Deprecated aliases for backward compatibility
export BEARDOG_BIN="$PROVIDER"
export BEARDOG_SOCKET="$SOCKET_PATH"
export BEARDOG_SOCKET_PATH="$SOCKET_PATH"
export NEURAL_API_SOCKET="$SOCKET_PATH"

# --- Run tests -------------------------------------------------------------
echo ""
echo "=== Running tests with live security provider ==="
echo "  SECURITY_PROVIDER_SOCKET=$SOCKET_PATH"
echo ""

cd "$WORKSPACE_ROOT"

EXTRA_ARGS=""
if [ "${1:-}" = "--" ]; then
    shift
    EXTRA_ARGS="$*"
fi

IGNORED_BEFORE=$(cargo test --workspace --all-features -- --list 2>/dev/null | grep -c "ignored" || echo "0")

set +e
cargo test --workspace --all-features $EXTRA_ARGS 2>&1
TEST_EXIT=$?
set -e

IGNORED_AFTER=$(cargo test --workspace --all-features -- --list 2>/dev/null | grep -c "ignored" || echo "0")

echo ""
echo "=== Results ==="
echo "  Exit code: $TEST_EXIT"
echo "  Ignored tests (before security provider): ~$IGNORED_BEFORE"
echo "  Ignored tests (with security provider):   ~$IGNORED_AFTER"
echo ""

exit $TEST_EXIT
