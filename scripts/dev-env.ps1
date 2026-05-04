$ErrorActionPreference = "Stop"

$root = Split-Path -Parent $PSScriptRoot
$env:CARGO_HOME = Join-Path $root ".cargo-home"
$env:RUSTUP_HOME = Join-Path $root ".rustup-home"
$env:CARGO_TARGET_DIR = Join-Path $root "target"
$env:PROOFGATE_POLICY_PATH = Join-Path $root "config\policy.sample.json"
$env:PROOFGATE_MAPPING_LOG = Join-Path $root "data\local-mappings.jsonl"
$env:PROOFGATE_AUDIT_LOG = Join-Path $root "data\audit.jsonl"
if (-not $env:PROOFGATE_HMAC_KEY) {
  $env:PROOFGATE_HMAC_KEY = "replace-with-local-secret"
}

New-Item -ItemType Directory -Force -Path $env:CARGO_HOME | Out-Null
New-Item -ItemType Directory -Force -Path $env:RUSTUP_HOME | Out-Null
New-Item -ItemType Directory -Force -Path $env:CARGO_TARGET_DIR | Out-Null
New-Item -ItemType Directory -Force -Path (Split-Path -Parent $env:PROOFGATE_MAPPING_LOG) | Out-Null
New-Item -ItemType Directory -Force -Path (Split-Path -Parent $env:PROOFGATE_AUDIT_LOG) | Out-Null

$cargoBin = Join-Path $env:CARGO_HOME "bin"
if (($env:Path -split ';') -notcontains $cargoBin) {
  $env:Path = "$cargoBin;$env:Path"
}

Write-Host "CARGO_HOME=$env:CARGO_HOME"
Write-Host "RUSTUP_HOME=$env:RUSTUP_HOME"
Write-Host "CARGO_TARGET_DIR=$env:CARGO_TARGET_DIR"
Write-Host "PROOFGATE_POLICY_PATH=$env:PROOFGATE_POLICY_PATH"
Write-Host "PROOFGATE_MAPPING_LOG=$env:PROOFGATE_MAPPING_LOG"
Write-Host "PROOFGATE_AUDIT_LOG=$env:PROOFGATE_AUDIT_LOG"
