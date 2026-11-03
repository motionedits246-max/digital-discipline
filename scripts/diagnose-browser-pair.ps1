# Digital Discipline v0.3.2 native messaging diagnostic for an installed Windows desktop.
# Read-only checks. Does not read browser history, secrets, or change registry values.
param(
  [ValidateSet('Brave','Chrome','Edge')][string]$Browser='Brave',
  [string]$ExtensionId='' 
)
if($ExtensionId -ne '' -and $ExtensionId -notmatch '^[a-p]{32}$'){ throw 'Extension ID must be 32 letters from a to p.' }
$ErrorActionPreference='Stop'
$hostName='com.digitaldiscipline.bridge'
$vendor=@{Brave='BraveSoftware\Brave-Browser';Chrome='Google\Chrome';Edge='Microsoft\Edge'}[$Browser]
$key='HKCU\Software\'+$vendor+'\NativeMessagingHosts\'+$hostName
$manifest=Join-Path $env:LOCALAPPDATA "DigitalDiscipline\$hostName.$Browser.json"
$pass=0;$fail=0
function Check([string]$label,[bool]$good,[string]$detail=''){
  if($good){$script:pass++;Write-Host "[PASS] $label" -ForegroundColor Green}
  else {$script:fail++;Write-Host "[FAIL] $label" -ForegroundColor Yellow}
  if($detail){Write-Host "       $detail"}
}
Write-Host "Digital Discipline · Native Pairing Doctor ($Browser)" -ForegroundColor Cyan
Write-Host "User: $env:USERNAME | Registry scope: HKCU"
$regOut=(& reg.exe query $key /ve 2>&1 | Out-String)
$found=$LASTEXITCODE -eq 0
Check 'Browser native host registry entry' $found
Check 'Manifest JSON exists' (Test-Path $manifest) $manifest
$parsed=$null
if(Test-Path $manifest){try{$parsed=Get-Content $manifest -Raw | ConvertFrom-Json}catch{Write-Host "Manifest JSON parsing error: $($_.Exception.Message)"}}
Check 'Manifest is valid and uses stdio' ($null -ne $parsed -and $parsed.name -eq $hostName -and $parsed.type -eq 'stdio')
if($found){Check 'Registry points to current manifest' ($regOut.ToLowerInvariant().Contains($manifest.ToLowerInvariant()))}
if($parsed){
  Check 'Native host binary exists' (Test-Path $parsed.path) $parsed.path
  if($ExtensionId){Check 'Current Extension ID allowed' (@($parsed.allowed_origins) -contains "chrome-extension://$ExtensionId/")}
  Write-Host ('Allowed origins: '+(@($parsed.allowed_origins).Count))
}
$listener=$false
try{$tcp=New-Object Net.Sockets.TcpClient;$result=$tcp.BeginConnect('127.0.0.1',48761,$null,$null);$listener=$result.AsyncWaitHandle.WaitOne(400);if($listener){$tcp.EndConnect($result)};$tcp.Dispose()}catch{$listener=$false}
Check 'Agent local service reachable on 127.0.0.1:48761' $listener
Write-Host "Tests: $pass passed, $fail failed."
Write-Host 'IMPORTANT: All green only checks Windows registration. The extension popup must also show a successful handshake.'
Write-Host 'If not connected, open brave://extensions > Digital Discipline > Service worker > Inspect and read the native messaging error.'
Write-Host 'Do not uninstall/reinstall the extension with queued events: its unsent data may be deleted.'
