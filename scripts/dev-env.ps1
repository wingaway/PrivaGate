$ErrorActionPreference = "Stop"

$root = Split-Path -Parent $PSScriptRoot
$env:CARGO_HOME = Join-Path $root ".cargo-home"
$env:RUSTUP_HOME = Join-Path $root ".rustup-home"
$env:CARGO_TARGET_DIR = Join-Path $root "target"
$env:PRIVAGATE_POLICY_PATH = Join-Path $root "config\policy.sample.json"
$env:PRIVAGATE_MAPPING_LOG = Join-Path $root "data\local-mappings.jsonl"
$env:PRIVAGATE_AUDIT_LOG = Join-Path $root "data\audit.jsonl"
$env:PRIVAGATE_REVIEW_LOG = Join-Path $root "data\manual-review.jsonl"

if (-not $env:PRIVAGATE_HMAC_KEY) {
  $env:PRIVAGATE_HMAC_KEY = "replace-with-local-test-secret"
}
if (-not $env:PRIVAGATE_REVIEW_MODE) {
  $env:PRIVAGATE_REVIEW_MODE = "off"
}
if (-not $env:PRIVAGATE_MODEL_ADAPTER) {
  $env:PRIVAGATE_MODEL_ADAPTER = "disabled"
}

if (-not $env:LOCAL_MODEL_BASE_URL) {
  $env:LOCAL_MODEL_BASE_URL = "https://example-local-compatible-api/v1"
}
if (-not $env:LOCAL_MODEL_API_KEY) {
  $env:LOCAL_MODEL_API_KEY = "replace-with-local-simulation-key"
}
if (-not $env:LOCAL_MODEL_NAME) {
  $env:LOCAL_MODEL_NAME = "replace-with-local-simulation-model"
}

if (-not $env:EXTERNAL_MODEL_BASE_URL) {
  $env:EXTERNAL_MODEL_BASE_URL = "https://example-external-compatible-api/v1"
}
if (-not $env:EXTERNAL_MODEL_API_KEY) {
  $env:EXTERNAL_MODEL_API_KEY = "replace-with-external-simulation-key"
}
if (-not $env:EXTERNAL_MODEL_NAME) {
  $env:EXTERNAL_MODEL_NAME = "replace-with-external-simulation-model"
}

New-Item -ItemType Directory -Force -Path $env:CARGO_HOME | Out-Null
New-Item -ItemType Directory -Force -Path $env:RUSTUP_HOME | Out-Null
New-Item -ItemType Directory -Force -Path $env:CARGO_TARGET_DIR | Out-Null
New-Item -ItemType Directory -Force -Path (Split-Path -Parent $env:PRIVAGATE_MAPPING_LOG) | Out-Null
New-Item -ItemType Directory -Force -Path (Split-Path -Parent $env:PRIVAGATE_AUDIT_LOG) | Out-Null
New-Item -ItemType Directory -Force -Path (Split-Path -Parent $env:PRIVAGATE_REVIEW_LOG) | Out-Null

$cargoBin = Join-Path $env:CARGO_HOME "bin"
if (($env:Path -split ';') -notcontains $cargoBin) {
  $env:Path = "$cargoBin;$env:Path"
}

Write-Host "CARGO_HOME=$env:CARGO_HOME"
Write-Host "RUSTUP_HOME=$env:RUSTUP_HOME"
Write-Host "CARGO_TARGET_DIR=$env:CARGO_TARGET_DIR"
Write-Host "PRIVAGATE_POLICY_PATH=$env:PRIVAGATE_POLICY_PATH"
Write-Host "PRIVAGATE_MAPPING_LOG=$env:PRIVAGATE_MAPPING_LOG"
Write-Host "PRIVAGATE_AUDIT_LOG=$env:PRIVAGATE_AUDIT_LOG"
Write-Host "PRIVAGATE_REVIEW_LOG=$env:PRIVAGATE_REVIEW_LOG"
Write-Host "PRIVAGATE_REVIEW_MODE=$env:PRIVAGATE_REVIEW_MODE"
Write-Host "PRIVAGATE_MODEL_ADAPTER=$env:PRIVAGATE_MODEL_ADAPTER"
