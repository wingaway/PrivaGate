#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

required_files=(
  "README.md"
  "LICENSE"
  "CONTRIBUTING.md"
  "SECURITY.md"
  "CITATION.cff"
  "Cargo.toml"
  "config/policy.sample.json"
  "tests/external_api_simulation/dataset.json"
)

missing=()
for path in "${required_files[@]}"; do
  if [ ! -e "$ROOT/$path" ]; then
    missing+=("$path")
  fi
done

if [ "${#missing[@]}" -gt 0 ]; then
  printf 'Missing required release files:\n' >&2
  printf '  %s\n' "${missing[@]}" >&2
  exit 1
fi

pattern='-----BEGIN [A-Z ]*PRIVATE KEY-----|Bearer[[:space:]]+[A-Za-z0-9._-]{20,}|AKIA[0-9A-Z]{16}|ghp_[A-Za-z0-9_]{20,}|sk-([A-Za-z0-9_-]{20,})|(password|passwd|api[_-]?key|secret)[[:space:]]*[:=][[:space:]]*['"'"'"]?[^[:space:]'"'"'"]{12,}'

findings="$(
  grep -RInE "$pattern" "$ROOT" \
    --exclude-dir=.git \
    --exclude-dir=target \
    --exclude-dir=.cargo-home \
    --exclude-dir=.rustup-home \
    --exclude-dir=.cache \
    --exclude-dir=data \
    --exclude-dir=__pycache__ \
    --exclude=prepublish-check.ps1 \
    --exclude=prepublish-check.sh \
    2>/dev/null |
  grep -Eiv 'synthetic|example|replace-with|placeholder|do not add real|不得提交|不得使用真实' || true
)"

if [ -n "$findings" ]; then
  printf '%s\n' "$findings" >&2
  printf 'Potential secret or sensitive release artifact found.\n' >&2
  exit 1
fi

for directory in data target .cargo-home .rustup-home .cache; do
  if [ -d "$ROOT/$directory" ]; then
    count="$(find "$ROOT/$directory" -type f 2>/dev/null | wc -l | tr -d ' ')"
    if [ "$count" != "0" ]; then
      printf 'local-only directory present and ignored: %s (%s files)\n' "$directory" "$count"
    fi
  fi
done

printf 'prepublish check passed\n'
