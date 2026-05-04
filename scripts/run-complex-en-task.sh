#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/dev-env.sh"

missing=()
for name in EXTERNAL_MODEL_BASE_URL EXTERNAL_MODEL_API_KEY EXTERNAL_MODEL_NAME; do
  if [ -z "${!name:-}" ]; then
    missing+=("$name")
  fi
done

if [ "${#missing[@]}" -gt 0 ]; then
  printf 'Missing required external model config: %s\n' "${missing[*]}" >&2
  exit 1
fi

python "$SCRIPT_DIR/../tests/complex_text_task/run_complex_en_medical_claim_task.py" "$@"
