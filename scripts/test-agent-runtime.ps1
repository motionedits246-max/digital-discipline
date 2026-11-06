# Exercise real Windows collection lifecycle with isolated per-test user storage.
$ErrorActionPreference='Stop'
$runtimeRoot=(Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$runtimeTemp=Join-Path ([IO.Path]::GetTempPath()) ('dd-runtime-'+[Guid]::NewGuid())
New-Item -ItemType Directory -Path $runtimeTemp | Out-Null
$runtimeProcesses=[Collections.Generic.List[Diagnostics.Process]]::new()
function Start-TestAgent {
 $info=[Diagnostics.ProcessStartInfo]::new()
 $info.FileName=Join-Path $runtimeRoot 'target/release/digital-discipline-agent.exe'
 $info.UseShellExecute=$false
 $info.CreateNoWindow=$true
 $info.Environment['LOCALAPPDATA']=$runtimeTemp
 $p=[Diagnostics.Process]::Start($info)
 $runtimeProcesses.Add($p)
 return $p
}
function Wait-Agent {
 for($i=0;$i -lt 100;$i++){
  try {
   $t=(Get-Content (Join-Path $runtimeTemp 'DigitalDiscipline/agent.token') -Raw).Trim()
   $h=Invoke-RestMethod 'http://127.0.0.1:48761/health' -Headers @{Authorization="Bearer $t"} -TimeoutSec 2
   if($h.application -eq 'digital-discipline-agent'){return @{token=$t;health=$h}}
  } catch {}
  Start-Sleep -Milliseconds 100
 }
 throw 'Agent did not become healthy'
}
try {
 $first=Start-TestAgent
 $ready=Wait-Agent
 if($ready.health.version -ne '0.4.0' -or $ready.health.storage_version -lt 10){throw 'Agent version or migration mismatch'}
 $duplicate=Start-TestAgent
 if(-not $duplicate.WaitForExit(5000) -or $duplicate.ExitCode -eq 0){throw 'Duplicate agent did not exit without collecting'}
 $auth=@{Authorization="Bearer $($ready.token)"}
 Invoke-RestMethod 'http://127.0.0.1:48761/tracking/pause' -Method Post -Headers $auth -ContentType 'application/json' -Body '{"paused":true}' | Out-Null
 Invoke-RestMethod 'http://127.0.0.1:48761/shutdown' -Method Post -Headers $auth -ContentType 'application/json' -Body '{}' | Out-Null
 if(-not $first.WaitForExit(6000)){throw 'Graceful shutdown timed out'}
 $second=Start-TestAgent
 $ready=Wait-Agent
 $state=Invoke-RestMethod 'http://127.0.0.1:48761/live' -Headers $auth
 if(-not $state.tracking_paused){throw 'Pause preference was lost across restart'}
 Invoke-RestMethod 'http://127.0.0.1:48761/shutdown' -Method Post -Headers $auth -ContentType 'application/json' -Body '{}' | Out-Null
 if(-not $second.WaitForExit(6000)){throw 'Restarted agent did not shut down'}
 Write-Host 'PASS: fresh migration, authenticated version, duplicate process, persisted pause, graceful stop and restart'
} finally {
 foreach($p in $runtimeProcesses){if(-not $p.HasExited){$p.Kill();$p.WaitForExit(3000) | Out-Null};$p.Dispose()}
 Remove-Item $runtimeTemp -Recurse -Force -ErrorAction SilentlyContinue
}
