# Exercise real Windows collection lifecycle with isolated per-test user storage.
$ErrorActionPreference='Stop'
$runtimeRoot=(Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$runtimeExpectedVersion=(Get-Content (Join-Path $runtimeRoot 'apps/desktop/package.json') -Raw | ConvertFrom-Json).version
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
 if($ready.health.version -ne $runtimeExpectedVersion -or $ready.health.storage_version -lt 15){throw 'Agent version or migration mismatch'}
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
 $third=Start-TestAgent
 $ready=Wait-Agent
 function Agent-Post([string]$route,$body){Invoke-RestMethod ("http://127.0.0.1:48761"+$route) -Method Post -Headers $auth -ContentType 'application/json' -Body ($body|ConvertTo-Json -Depth 20 -Compress)}
 $enrolled=Agent-Post '/browser-pairs' @{browser='Brave';extension_id=('a'*32)}
 $companion=Agent-Post '/companion' @{}
 $extensionManifest=Get-Content (Join-Path $companion.folder 'manifest.json') -Raw | ConvertFrom-Json
 if($extensionManifest.version -ne $companion.version -or -not (Test-Path (Join-Path $companion.folder 'src/queue-policy.js'))){throw 'Managed companion update is incomplete'}
 $pairedAfterUpdate=Invoke-RestMethod 'http://127.0.0.1:48761/browser-pairs' -Headers $auth
 if(-not @($pairedAfterUpdate.pairs|Where-Object {$_.extension_id -eq ('a'*32)}).Count){throw 'Companion update lost browser enrollment'}
 $profile=Agent-Post '/workflows/profiles' @{name='CI study';apps=@('new-editor.exe');domains=@('www.youtube.com')}
 $task=Agent-Post '/workflows/tasks' @{title='Finish lesson exercises';profile_id=$profile.id}
 $focus=Agent-Post '/focus/start' @{goal_id=$null;task_id=$task.id;profile_id=$profile.id;minutes=1;difficulty='Strict';block_domains=@('www.youtube.com');cycle=@{work_minutes=1;break_minutes=1;cycles=2;break_policy='soften'};learning=@{video_ids=@('Example123');override_rule_ids=@('focus');hide_home=$true;hide_related=$true;hide_shorts=$true;stop_autoplay=$true}}
 $state=Invoke-RestMethod 'http://127.0.0.1:48761/live' -Headers $auth
 if($state.focus.context.intended_outcome -ne 'Finish lesson exercises' -or $state.focus.cycle.state -ne 'work'){throw 'Task or work-cycle context missing'}
 Agent-Post ("/focus/cycles/"+$focus.id) @{action='pause';version=$state.focus.cycle.version;minutes=$null}|Out-Null
 Agent-Post '/privacy/session' @{minutes=1}|Out-Null
 Agent-Post '/shutdown' @{}|Out-Null
 if(-not $third.WaitForExit(6000)){throw 'Workflow agent did not stop'}
 $fourth=Start-TestAgent
 $ready=Wait-Agent
 $state=Invoke-RestMethod 'http://127.0.0.1:48761/live' -Headers $auth
 if($state.focus.id -ne $focus.id -or $state.focus.cycle.state -ne 'paused' -or -not $state.private_session.active){throw 'Focus/private state was lost on restart'}
 Agent-Post '/privacy/session' @{minutes=0}|Out-Null
 Agent-Post ("/focus/cycles/"+$focus.id) @{action='resume';version=$state.focus.cycle.version;minutes=$null}|Out-Null
 $policies=Invoke-RestMethod 'http://127.0.0.1:48761/extension/rules' -Headers $auth
 if(-not @($policies.blocks|Where-Object {$_.allow_video_ids -contains 'Example123'}).Count){throw 'Scoped lesson exception was not produced'}
 Agent-Post '/policy/exceptions' @{rule_id=$focus.id;domain='www.youtube.com';minutes=2;reason='study';profile_id=$null}|Out-Null
 $policies=Invoke-RestMethod 'http://127.0.0.1:48761/extension/rules' -Headers $auth
 if(@($policies.blocks|Where-Object {$_.id -eq $focus.id}).Count){throw 'Scoped unlock did not release its selected rule'}
 if(-not @($policies.blocks|Where-Object {$_.path_prefix -eq '/shorts/'}).Count){throw 'Scoped unlock incorrectly released unrelated Shorts restriction'}
 $snapshot=Agent-Post '/export/snapshot' @{}
 $snapshotFile=Join-Path $runtimeTemp ("DigitalDiscipline/exports/"+$snapshot.snapshot_id+'.jsonl')
 $footer=Get-Content $snapshotFile -Tail 1|ConvertFrom-Json
 if(-not $footer.complete -or $footer.truncated){throw 'Complete snapshot footer invalid'}
 Agent-Post '/focus/end' @{}|Out-Null
 $reviews=Invoke-RestMethod 'http://127.0.0.1:48761/focus/reviews' -Headers $auth
 if($reviews.reviews[0].context.intended_outcome -ne 'Finish lesson exercises'){throw 'Task review context missing'}
 $details=Invoke-RestMethod 'http://127.0.0.1:48761/health/details' -Headers $auth
 if(-not $details.resources.available -or $details.resources.working_set_bytes -le 0){throw 'Windows process resource measurement unavailable'}
 $backupSecret='test backup passphrase'
 Agent-Post '/backup/schedule' @{enabled=$true;passphrase=$backupSecret;history=$false}|Out-Null
 Agent-Post '/backup/schedule' @{enabled=$false;history=$false}|Out-Null
 $backup=Agent-Post '/backup' @{passphrase=$backupSecret;history=$false}
 $preview=Agent-Post '/backup/preview' @{filename=$backup.filename;passphrase=$backupSecret}
 if($preview.digest.Length -ne 64 -or $preview.history){throw 'Encrypted backup preview invalid'}
 $restored=Agent-Post '/backup/restore' @{filename=$backup.filename;passphrase=$backupSecret;digest=$preview.digest}
 if(-not $restored.recovery){throw 'Encrypted restore did not record its recovery copy'}
 Agent-Post '/shutdown' @{}|Out-Null
 if(-not $fourth.WaitForExit(6000)){throw 'Final workflow agent did not stop'}
 Write-Host 'PASS: migration, duplicate process, pause persistence, task/profile context, focus-cycle/private restart, scoped Learning Mode/unlock, managed companion update/enrollment, complete snapshot, review and graceful shutdown'
} finally {
 foreach($p in $runtimeProcesses){if(-not $p.HasExited){$p.Kill();$p.WaitForExit(3000) | Out-Null};$p.Dispose()}
 Remove-Item $runtimeTemp -Recurse -Force -ErrorAction SilentlyContinue
}
