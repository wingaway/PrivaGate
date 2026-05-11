#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

export CARGO_HOME="${CARGO_HOME:-"$ROOT/.cargo-home"}"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-"$ROOT/target"}"

if [ -d "$ROOT/.rustup-home/toolchains" ]; then
  export RUSTUP_HOME="${RUSTUP_HOME:-"$ROOT/.rustup-home"}"
fi

export PRIVAGATE_POLICY_PATH="${PRIVAGATE_POLICY_PATH:-"$ROOT/config/policy.sample.json"}"
export PRIVAGATE_MAPPING_LOG="${PRIVAGATE_MAPPING_LOG:-"$ROOT/data/local-mappings.jsonl"}"
export PRIVAGATE_AUDIT_LOG="${PRIVAGATE_AUDIT_LOG:-"$ROOT/data/audit.jsonl"}"
export PRIVAGATE_REVIEW_LOG="${PRIVAGATE_REVIEW_LOG:-"$ROOT/data/manual-review.jsonl"}"
export PRIVAGATE_REVIEW_MODE="${PRIVAGATE_REVIEW_MODE:-"off"}"
export PRIVAGATE_MODEL_ADAPTER="${PRIVAGATE_MODEL_ADAPTER:-"disabled"}"
export PRIVAGATE_HMAC_KEY="${PRIVAGATE_HMAC_KEY:-"replace-with-local-secret"}"

mkdir -p "$CARGO_HOME" "$CARGO_TARGET_DIR"
mkdir -p "$(dirname "$PRIVAGATE_MAPPING_LOG")" "$(dirname "$PRIVAGATE_AUDIT_LOG")" "$(dirname "$PRIVAGATE_REVIEW_LOG")"

if [ -d "$CARGO_HOME/bin" ]; then
  export PATH="$CARGO_HOME/bin:$PATH"
fi

echo "CARGO_HOME=$CARGO_HOME"
if [ "${RUSTUP_HOME:-}" != "" ]; then
  echo "RUSTUP_HOME=$RUSTUP_HOME"
fi
echo "CARGO_TARGET_DIR=$CARGO_TARGET_DIR"
echo "PRIVAGATE_POLICY_PATH=$PRIVAGATE_POLICY_PATH"
echo "PRIVAGATE_MAPPING_LOG=$PRIVAGATE_MAPPING_LOG"
echo "PRIVAGATE_AUDIT_LOG=$PRIVAGATE_AUDIT_LOG"
echo "PRIVAGATE_REVIEW_LOG=$PRIVAGATE_REVIEW_LOG"
echo "PRIVAGATE_REVIEW_MODE=$PRIVAGATE_REVIEW_MODE"
echo "PRIVAGATE_MODEL_ADAPTER=$PRIVAGATE_MODEL_ADAPTER"
