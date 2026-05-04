#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/dev-env.sh"

if ! command -v cargo >/dev/null 2>&1; then
  echo "cargo is not installed or not on PATH. Install Rust first, then rerun this wrapper." >&2
  exit 1
fi

if [ "${1:-}" = "clippy" ]; then
  if ! command -v cargo-clippy >/dev/null 2>&1; then
    echo "cargo-clippy is not installed or not on PATH. Install the clippy component, then rerun this wrapper." >&2
    exit 1
  fi
  shift
  exec cargo-clippy "$@"
fi

exec cargo "$@"
