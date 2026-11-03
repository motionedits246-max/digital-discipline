param(
 [Parameter(Mandatory=$true)][ValidatePattern('^[a-p]{32}$')][string]$ExtensionId,
 [ValidateSet('Brave','Chrome','Edge')][string]$Browser='Brave'
)
$ErrorActionPreference='Stop'
$root=(Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$exe=Join-Path $root 'target\release\digital-discipline-agent.exe'
if(-not (Test-Path $exe)){throw "Build the agent first: cargo build --release -p digital-discipline-agent"}
$dir=Join-Path $env:LOCALAPPDATA 'DigitalDiscipline'
New-Item -ItemType Directory -Force $dir | Out-Null
$file=Join-Path $dir ("com.digitaldiscipline.bridge.$Browser.json")
$hostJson = @{
  name='com.digitaldiscipline.bridge'
  description='Digital Discipline local browser bridge'
  path=$exe
  type='stdio'
  allowed_origins=@("chrome-extension://$ExtensionId/")
} | ConvertTo-Json -Depth 5
[System.IO.File]::WriteAllText($file, $hostJson, (New-Object System.Text.UTF8Encoding($false)))
$registry= switch($Browser){
 'Brave' {'HKCU:\Software\BraveSoftware\Brave-Browser\NativeMessagingHosts\com.digitaldiscipline.bridge'}
 'Chrome' {'HKCU:\Software\Google\Chrome\NativeMessagingHosts\com.digitaldiscipline.bridge'}
 'Edge' {'HKCU:\Software\Microsoft\Edge\NativeMessagingHosts\com.digitaldiscipline.bridge'}
}
New-Item -Path $registry -Force | Out-Null
Set-Item -Path $registry -Value $file
Write-Host "Native host registered for $Browser. Re-open the extension popup to sync."
Write-Host "Host manifest: $file"
