$ErrorActionPreference = "Stop"

. (Join-Path $PSScriptRoot "dev-env.ps1")

if (-not $env:EXTERNAL_MODEL_BASE_URL -or -not $env:EXTERNAL_MODEL_API_KEY -or -not $env:EXTERNAL_MODEL_NAME) {
  throw "Set EXTERNAL_MODEL_BASE_URL, EXTERNAL_MODEL_API_KEY, and EXTERNAL_MODEL_NAME before running this task."
}

python (Join-Path $PSScriptRoot "..\tests\complex_text_task\run_complex_zh_supply_chain_task.py") @args
