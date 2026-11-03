$ErrorActionPreference='Stop'
$root=(Resolve-Path (Join-Path $PSScriptRoot '..')).Path
Push-Location $root
try {
  if(-not (Get-Command cargo -ErrorAction SilentlyContinue)){throw 'Rust toolchain missing: install rustup and Visual Studio Build Tools (C++ workload).'}
  if(-not (Get-Command npm -ErrorAction SilentlyContinue)){throw 'Node/npm missing: install Node.js LTS.'}
  $testFiles = @(Get-ChildItem -Path tests -Filter '*.test.mjs' | ForEach-Object { $_.FullName })
  node --test $testFiles
  if($LASTEXITCODE -ne 0){throw 'Browser and bundle tests failed.'}
  python tests/migration-contract.py
  if($LASTEXITCODE -ne 0){throw 'Database migration tests failed.'}
  cargo test -p digital-discipline-agent
  if($LASTEXITCODE -ne 0){throw 'Agent tests failed.'}
  cargo build -p digital-discipline-agent --release
  if($LASTEXITCODE -ne 0){throw 'Agent build failed.'}
  & (Join-Path $root 'scripts\start-agent.ps1')
  Push-Location (Join-Path $root 'apps\desktop')
  try {
    npm install
    if($LASTEXITCODE -ne 0){throw 'npm install failed.'}
    npm run tauri:dev
  } finally {Pop-Location}
} finally {Pop-Location}
