# Upgrade in place while preserving the unpacked extension directory/ID and chrome.storage.local event queue.
# User must click Reload on brave://extensions after this finishes.
param([Parameter(Mandatory=$true)][string]$ExistingExtensionFolder)
$ErrorActionPreference='Stop'
$source=Join-Path (Resolve-Path (Join-Path $PSScriptRoot '..')).Path 'extension'
$dest=(Resolve-Path -LiteralPath $ExistingExtensionFolder).Path
if($dest -eq $source){Write-Host 'Existing extension is the source extension folder. No copy needed: Reload in Brave.';exit 0}
$manifest=Join-Path $dest 'manifest.json'
if(-not(Test-Path -LiteralPath $manifest)){throw 'Destination is not an unpacked browser extension folder (manifest.json missing).'}
$info=Get-Content -LiteralPath $manifest -Raw | ConvertFrom-Json
if($info.name -notlike '*Digital Discipline*'){throw 'This folder is not the Digital Discipline companion. No files changed.'}
$backup=Join-Path $env:TEMP ('dd-companion-backup-'+(Get-Date -Format yyyyMMdd-HHmmss))
Copy-Item -Path $dest -Destination $backup -Recurse -Force
Copy-Item -Path (Join-Path $source '*') -Destination $dest -Recurse -Force
Write-Host "Updated extension files IN PLACE at $dest"
Write-Host "Backup at $backup"
Write-Host 'Next: brave://extensions > Digital Discipline > Reload. Accept the localhost permission if prompted.'
Write-Host 'Your Brave extension ID and browser-managed queued event storage were not deliberately changed.'
