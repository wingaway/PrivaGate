param(
  [Parameter(ValueFromRemainingArguments = $true)]
  [string[]]$CargoArgs
)

$ErrorActionPreference = "Stop"

. (Join-Path $PSScriptRoot "dev-env.ps1")

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
  throw "cargo is not installed or not on PATH. Install Rust first, then rerun this wrapper."
}

if ($CargoArgs.Count -gt 0 -and $CargoArgs[0] -eq "clippy") {
  if (-not (Get-Command cargo-clippy -ErrorAction SilentlyContinue)) {
    throw "cargo-clippy is not installed or not on PATH. Install the clippy component, then rerun this wrapper."
  }
  $ClippyArgs = @($CargoArgs | Select-Object -Skip 1)
  if ($ClippyArgs -notcontains "--") {
    $LintFlagIndex = -1
    for ($Index = 0; $Index -lt $ClippyArgs.Count; $Index++) {
      if ($ClippyArgs[$Index] -match '^-([ADWF])$') {
        $LintFlagIndex = $Index
        break
      }
    }
    if ($LintFlagIndex -ge 0) {
      $Before = @()
      if ($LintFlagIndex -gt 0) {
        $Before = @($ClippyArgs[0..($LintFlagIndex - 1)])
      }
      $After = @($ClippyArgs[$LintFlagIndex..($ClippyArgs.Count - 1)])
      $ClippyArgs = $Before + @("--") + $After
    }
  }
  & cargo-clippy @ClippyArgs
  exit $LASTEXITCODE
}

& cargo @CargoArgs
exit $LASTEXITCODE
