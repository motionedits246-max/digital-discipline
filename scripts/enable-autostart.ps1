# This optional per-user operation makes the headless agent start at sign-in.
$ErrorActionPreference='Stop'
$root=(Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$exe=Join-Path $root 'target\release\digital-discipline-agent.exe'
if(-not(Test-Path $exe)){throw 'Build the agent in release mode before enabling autostart.'}
$registry='HKCU:\Software\Microsoft\Windows\CurrentVersion\Run'
New-ItemProperty -Path $registry -Name 'DigitalDisciplineAgent' -Value ('"'+$exe+'"') -PropertyType String -Force | Out-Null
Write-Host 'The agent will launch on your next Windows sign-in. This does not start the GUI.'
