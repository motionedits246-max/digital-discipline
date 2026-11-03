$ErrorActionPreference='Stop'
$root=(Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$exe=Join-Path $root 'target\release\digital-discipline-agent.exe'
if(-not (Test-Path $exe)){throw 'Build the agent first: cargo build --release -p digital-discipline-agent'}
Start-Process -FilePath $exe -WindowStyle Hidden
Write-Host 'Local agent launched. It does not depend on the desktop GUI.'
