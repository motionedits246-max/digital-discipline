<#
  Create a private GitHub repo for Digital Discipline, push current source,
  and optionally wait for and download the Windows installer artifact.
  Requires git + GitHub CLI (gh). GitHub authentication occurs locally.
  Does not print, store or request your GitHub personal-access token.
#>
param(
 [string]$RepoName = 'digital-discipline-windows',
 [switch]$SkipDownload
)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$root = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
function Assert-LastExit([string]$step) { if ($LASTEXITCODE -ne 0) { throw "$step failed (exit code $LASTEXITCODE)" } }
foreach ($cmd in @('git','gh')) {
 if (-not (Get-Command $cmd -ErrorAction SilentlyContinue)) { throw "$cmd not found. Install Git for Windows and GitHub CLI, reopen PowerShell, and try again." }
}
Push-Location $root
try {
  gh auth status --hostname github.com *> $null
  if ($LASTEXITCODE -ne 0) {
    Write-Host 'Sign in to GitHub in your browser...'
    gh auth login --hostname github.com --web --git-protocol https
    Assert-LastExit 'GitHub authentication'
  }
  $owner = (gh api user --jq .login).Trim()
  Assert-LastExit 'Get GitHub user'
  if ($owner -notmatch '^[A-Za-z0-9-]+$' -or $RepoName -notmatch '^[a-zA-Z0-9._-]+$') { throw 'Invalid owner/repository name' }
  $full = "$owner/$RepoName"

  # Avoid altering an unrelated existing repository.
  gh repo view $full --json name *> $null
  if ($LASTEXITCODE -eq 0) { throw "Repository $full already exists. Choose a different -RepoName; no code was pushed." }
  if (-not (Test-Path .git)) { git init -b main; Assert-LastExit 'git init' }
  if (-not (git config --local user.name)) {
    git config --local user.name $owner
    Assert-LastExit 'Set local Git author'
  }
  if (-not (git config --local user.email)) {
    $userId = (gh api user --jq .id).Trim()
    Assert-LastExit 'Get GitHub user ID'
    git config --local user.email "$userId+$owner@users.noreply.github.com"
    Assert-LastExit 'Set local Git no-reply author address'
  }
  git status --porcelain
  git add -A
  Assert-LastExit 'git add'
  git diff --cached --quiet
  if ($LASTEXITCODE -ne 0) {
    # Commit identity uses your GitHub profile's local no-reply identity.
    git commit -m 'Initial Digital Discipline Windows build source'
    Assert-LastExit 'git commit'
  }
  Write-Host "Creating private repo $full and pushing source..."
  gh repo create $full --private --source . --remote origin --push --description 'Windows-first local digital discipline desktop and Brave companion'
  Assert-LastExit 'Repository creation and push'
  Write-Host "Repository: https://github.com/$full"
  Write-Host "Build status: https://github.com/$full/actions"
  if ($SkipDownload) { exit 0 }
  Write-Host 'Looking for the GitHub Actions Windows build run...'
  $runId = $null
  for ($i=0; $i -lt 20; $i++) {
    $id = gh run list -R $full --workflow windows-ci.yml --limit 1 --json databaseId --jq '.[0].databaseId'
    if ($LASTEXITCODE -eq 0 -and $id -match '^\d+$') { $runId = $id; break }
    Start-Sleep -Seconds 5
  }
  if (-not $runId) { throw "Repository pushed, but no Actions run found yet. Open https://github.com/$full/actions" }
  gh run watch $runId -R $full --exit-status
  Assert-LastExit 'GitHub Actions build (see Actions logs for the failure)'
  $out = Join-Path $root 'github-build-output'
  New-Item -ItemType Directory -Path $out -Force | Out-Null
  gh run download $runId -R $full --name 'DigitalDiscipline-Windows-x64-v0.3.3' --dir $out
  Assert-LastExit 'Artifact download'
  Write-Host "Done. Installer and extension files are in $out"
  Get-ChildItem -Path $out -Recurse -File | Select-Object FullName,Length | Format-Table
} finally { Pop-Location }
