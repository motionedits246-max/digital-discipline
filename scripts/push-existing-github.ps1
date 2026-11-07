<#
Uploads Digital Discipline source to the user's EXISTING private GitHub repository,
waits for the Windows Actions installer workflow, and downloads its artifact.
Requires Git for Windows and GitHub CLI (gh), and interactive user authentication.
No PAT, token, or password should be pasted into this script.
#>
param(
  [string]$Repo = 'motionedits246-max/digital-discipline-windows',
  [switch]$SkipDownload
)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$root = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
function Check([string]$step) {
  if ($LASTEXITCODE -ne 0) { throw "$step failed (exit code $LASTEXITCODE)" }
}
foreach ($program in @('git','gh')) {
  if (-not (Get-Command $program -ErrorAction SilentlyContinue)) {
    throw "$program is missing. Install Git for Windows and GitHub CLI, then reopen PowerShell."
  }
}
if ($Repo -notmatch '^[A-Za-z0-9-]+/[A-Za-z0-9._-]+$') { throw 'Invalid owner/repo' }
Push-Location $root
try {
  gh auth status --hostname github.com 2>$null | Out-Null
  if ($LASTEXITCODE -ne 0) {
    gh auth login --hostname github.com --web --git-protocol https
    Check 'GitHub sign-in'
  }
  gh auth setup-git
  Check 'GitHub Git credential setup'
  gh repo view $Repo --json name,isPrivate --jq '.name + " private=" + (.isPrivate|tostring)'
  Check 'Access to existing GitHub repository'
  $remoteUrl = "https://github.com/$Repo.git"

  # GitHub already contains README.md and .github/workflows/windows-ci.yml.
  # Clone the existing default branch to preserve remote history and avoid a force push.
  $scratch = Join-Path ([System.IO.Path]::GetTempPath()) ('dd-github-' + [Guid]::NewGuid().ToString('N'))
  New-Item -Path $scratch -ItemType Directory -Force | Out-Null
  try {
    $repoDir = Join-Path $scratch 'repo'
    git clone --branch main $remoteUrl $repoDir
    Check 'Clone existing repository'
    $sourcePaths = @('agent','apps','docs','extension','scripts','tests','Cargo.toml','.gitignore','README.md','GITHUB-BUILD-INSTRUCTIONS.md')
    foreach ($item in $sourcePaths) {
      $src = Join-Path $root $item
      if (-not (Test-Path $src)) { throw "Required project file/folder missing: $src" }
      $dst = Join-Path $repoDir $item
      if (Test-Path $dst) { Remove-Item $dst -Recurse -Force }
      Copy-Item -Path $src -Destination $dst -Recurse -Force
    }
    # Preserve the existing GitHub Actions workflow in the remote repository.
    # It runs scripts/build-windows.ps1 on windows-2022.
    Push-Location $repoDir
    try {
      $who = (gh api user --jq .login).Trim()
      Check 'Read GitHub username'
      $uid = (gh api user --jq .id).Trim()
      Check 'Read GitHub user id'
      git config user.name $who
      git config user.email "$uid+$who@users.noreply.github.com"
      git add --all
      Check 'Stage project source'
      git diff --cached --quiet
      if ($LASTEXITCODE -eq 0) {
        Write-Host 'No source changes to push.'
      } else {
        git commit -m 'Upload Digital Discipline v0.3.3 Windows application source'
        Check 'Commit source'
        git push origin main
        Check 'Push source to private repository'
      }
    } finally { Pop-Location }
  } finally {
    if (Test-Path $scratch) { Remove-Item $scratch -Recurse -Force -ErrorAction SilentlyContinue }
  }
  Write-Host "Source pushed: https://github.com/$Repo"
  Write-Host "Build runs: https://github.com/$Repo/actions"
  if ($SkipDownload) { return }
  # Dispatch explicitly; this is independent of a push-triggered run.
  gh workflow run windows-ci.yml --repo $Repo --ref main
  Check 'Trigger Windows installer workflow'
  $runId = $null
  for ($i = 0; $i -lt 12; $i++) {
    $id = (gh run list -R $Repo --workflow windows-ci.yml --event workflow_dispatch --limit 1 --json databaseId --jq '.[0].databaseId')
    if ($LASTEXITCODE -eq 0 -and $id -match '^\d+$') { $runId = $id; break }
    Start-Sleep -Seconds 5
  }
  if (-not $runId) { throw "No build run visible. Check https://github.com/$Repo/actions" }
  Write-Host "Watching Windows build $runId ..."
  gh run watch $runId --repo $Repo --exit-status
  Check 'Windows installer build'
  $dest = Join-Path $root 'github-build-output'
  New-Item -Path $dest -ItemType Directory -Force | Out-Null
  gh run download $runId --repo $Repo --name 'DigitalDiscipline-Windows-x64-v0.3.3' --dir $dest
  Check 'Download Windows installer artifact'
  $installers = @(Get-ChildItem $dest -Filter '*setup*.exe' -Recurse)
  if ($installers.Count -eq 0) { throw 'Workflow succeeded but installer executable not found in downloaded artifacts' }
  Write-Host 'Installer downloaded:'
  $installers | ForEach-Object { Write-Host $_.FullName }
} finally { Pop-Location }
