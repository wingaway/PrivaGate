$ErrorActionPreference = "Stop"

$Root = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$ExcludedDirectories = @(
  "\.git\",
  "\target\",
  "\.cargo-home\",
  "\.rustup-home\",
  "\.cache\",
  "\data\",
  "\__pycache__\"
)

$RequiredFiles = @(
  "README.md",
  "LICENSE",
  "CONTRIBUTING.md",
  "CODE_OF_CONDUCT.md",
  "GOVERNANCE.md",
  "SECURITY.md",
  "CITATION.cff",
  ".github\PULL_REQUEST_TEMPLATE.md",
  ".github\ISSUE_TEMPLATE\bug_report.yml",
  ".github\ISSUE_TEMPLATE\feature_request.yml",
  ".github\ISSUE_TEMPLATE\research_question.yml",
  ".github\ISSUE_TEMPLATE\detector_request.yml",
  "Cargo.toml",
  "config\policy.sample.json",
  "tests\external_api_simulation\dataset.json",
  "docs\CONTRIBUTOR_TASKS.md",
  "docs\RFC_PROCESS.md",
  "docs\rfcs\0000-template.md"
)

$Missing = @()
foreach ($RelativePath in $RequiredFiles) {
  if (-not (Test-Path -LiteralPath (Join-Path $Root $RelativePath))) {
    $Missing += $RelativePath
  }
}

$Patterns = @(
  "-----BEGIN [A-Z ]*PRIVATE KEY-----",
  "Bearer\s+[A-Za-z0-9._\-]{20,}",
  "AKIA[0-9A-Z]{16}",
  "ghp_[A-Za-z0-9_]{20,}",
  "sk-(?!synthetic)[A-Za-z0-9_\-]{20,}",
  "(?i)(password|passwd|api[_-]?key|secret)\s*[:=]\s*['""]?(?!\.\.\.|replace|example|local|test|synthetic)[A-Za-z0-9_./:+\-]{12,}"
)

$Files = Get-ChildItem -Path $Root -Recurse -File -Force | Where-Object {
  $Path = $_.FullName
  if ($Path.EndsWith("\scripts\prepublish-check.ps1") -or $Path.EndsWith("\scripts\prepublish-check.sh")) {
    return $false
  }
  foreach ($Excluded in $ExcludedDirectories) {
    if ($Path.Contains($Excluded)) {
      return $false
    }
  }
  return $true
}

$Findings = @()
foreach ($File in $Files) {
  $Matches = Select-String -LiteralPath $File.FullName -Pattern $Patterns -ErrorAction SilentlyContinue
  foreach ($Match in $Matches) {
    if ($Match.Line -match "(?i)synthetic|example|replace-with|placeholder|do not add real|不得提交|不得使用真实") {
      continue
    }
    $Findings += [PSCustomObject]@{
      Path = $File.FullName.Substring($Root.Length + 1)
      LineNumber = $Match.LineNumber
      Line = $Match.Line.Trim()
    }
  }
}

if ($Missing.Count -gt 0) {
  Write-Error ("Missing required release files: " + ($Missing -join ", "))
}

if ($Findings.Count -gt 0) {
  $Findings | Format-Table -AutoSize
  Write-Error "Potential secret or sensitive release artifact found."
}

$GeneratedDirectories = @("data", "target", ".cargo-home", ".rustup-home", ".cache")
foreach ($Directory in $GeneratedDirectories) {
  $Path = Join-Path $Root $Directory
  if (Test-Path -LiteralPath $Path) {
    $Count = (Get-ChildItem -LiteralPath $Path -Recurse -File -Force -ErrorAction SilentlyContinue | Measure-Object).Count
    if ($Count -gt 0) {
      Write-Output "local-only directory present and ignored: $Directory ($Count files)"
    }
  }
}

Write-Output "prepublish check passed"
