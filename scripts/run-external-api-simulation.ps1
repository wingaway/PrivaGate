$ErrorActionPreference = "Stop"

. (Join-Path $PSScriptRoot "dev-env.ps1")

if (-not $env:LOCAL_MODEL_BASE_URL -or -not $env:LOCAL_MODEL_API_KEY -or -not $env:LOCAL_MODEL_NAME) {
  throw "Set LOCAL_MODEL_BASE_URL, LOCAL_MODEL_API_KEY, and LOCAL_MODEL_NAME before running this simulation."
}

if (-not $env:EXTERNAL_MODEL_BASE_URL -or -not $env:EXTERNAL_MODEL_API_KEY -or -not $env:EXTERNAL_MODEL_NAME) {
  throw "Set EXTERNAL_MODEL_BASE_URL, EXTERNAL_MODEL_API_KEY, and EXTERNAL_MODEL_NAME before running this simulation."
}

python (Join-Path $PSScriptRoot "..\tests\external_api_simulation\run_external_api_simulation.py") @args

