#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

export CARGO_HOME="${CARGO_HOME:-"$ROOT/.cargo-home"}"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-"$ROOT/target"}"

if [ -d "$ROOT/.rustup-home/toolchains" ]; then
  export RUSTUP_HOME="${RUSTUP_HOME:-"$ROOT/.rustup-home"}"
fi

export PROOFGATE_POLICY_PATH="${PROOFGATE_POLICY_PATH:-"$ROOT/config/policy.sample.json"}"
export PROOFGATE_MAPPING_LOG="${PROOFGATE_MAPPING_LOG:-"$ROOT/data/local-mappings.jsonl"}"
export PROOFGATE_AUDIT_LOG="${PROOFGATE_AUDIT_LOG:-"$ROOT/data/audit.jsonl"}"
export PROOFGATE_REVIEW_MODE="${PROOFGATE_REVIEW_MODE:-"off"}"
export PROOFGATE_HMAC_KEY="${PROOFGATE_HMAC_KEY:-"replace-with-local-secret"}"

mkdir -p "$CARGO_HOME" "$CARGO_TARGET_DIR"
mkdir -p "$(dirname "$PROOFGATE_MAPPING_LOG")" "$(dirname "$PROOFGATE_AUDIT_LOG")"

if [ -d "$CARGO_HOME/bin" ]; then
  export PATH="$CARGO_HOME/bin:$PATH"
fi

echo "CARGO_HOME=$CARGO_HOME"
if [ "${RUSTUP_HOME:-}" != "" ]; then
  echo "RUSTUP_HOME=$RUSTUP_HOME"
fi
echo "CARGO_TARGET_DIR=$CARGO_TARGET_DIR"
echo "PROOFGATE_POLICY_PATH=$PROOFGATE_POLICY_PATH"
echo "PROOFGATE_MAPPING_LOG=$PROOFGATE_MAPPING_LOG"
echo "PROOFGATE_AUDIT_LOG=$PROOFGATE_AUDIT_LOG"
echo "PROOFGATE_REVIEW_MODE=$PROOFGATE_REVIEW_MODE"
