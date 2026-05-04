#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/dev-env.sh"

missing=()
for name in \
  LOCAL_MODEL_BASE_URL \
  LOCAL_MODEL_API_KEY \
  LOCAL_MODEL_NAME \
  EXTERNAL_MODEL_BASE_URL \
  EXTERNAL_MODEL_API_KEY \
  EXTERNAL_MODEL_NAME
do
  if [ -z "${!name:-}" ]; then
    missing+=("$name")
  fi
done

if [ "${#missing[@]}" -gt 0 ]; then
  printf 'Set required model config before running this simulation: %s\n' "${missing[*]}" >&2
  exit 1
fi

python3 "$SCRIPT_DIR/../tests/external_api_simulation/run_external_api_simulation.py" "$@"
