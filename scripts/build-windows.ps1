# Build the agent, run tests, compile the desktop, and package Windows release artifacts.
# This script changes nothing outside the repo until the installer is launched by the user.
$ErrorActionPreference='Stop'
$root=(Resolve-Path (Join-Path $PSScriptRoot '..')).Path
Push-Location $root
try {
 if(-not(Get-Command cargo -ErrorAction SilentlyContinue)){throw 'Install Rust stable (MSVC target) from rustup.rs'}
 if(-not(Get-Command npm -ErrorAction SilentlyContinue)){throw 'Install Node.js LTS and npm'}
 Write-Host '[1/6] Browser policy, notifications, encrypted bundles'
 $testFiles = @(Get-ChildItem -Path tests -Filter '*.test.mjs' | ForEach-Object { $_.FullName })
 node --test $testFiles
 if($LASTEXITCODE -ne 0){throw 'Extension tests failed'}
 Write-Host '[1b/6] SQLite migration smoke test'
 python 'tests/migration-contract.py'
 if($LASTEXITCODE -ne 0){throw 'SQL schema tests failed'}
 python 'tests/browser-pair-migration.py'
 if($LASTEXITCODE -ne 0){throw 'Browser pairing migration test failed'}
 python 'tests/workflow-migration.py'
 if($LASTEXITCODE -ne 0){throw 'Workflow migration test failed'}
 Write-Host '[2/6] Rust unit tests'
 cargo test --locked -p digital-discipline-agent
 if($LASTEXITCODE -ne 0){throw 'Agent tests failed'}
 Write-Host '[3/6] Build independent Rust agent'
 cargo build --locked -p digital-discipline-agent --release
 if($LASTEXITCODE -ne 0){throw 'Agent build failed'}
 if($env:DD_DEV_CERT_THUMBPRINT){
   Write-Host '[3b/6] Sign agent with development certificate (NOT publicly trusted)'
   $cert=Get-Item ("Cert:\CurrentUser\My\" + $env:DD_DEV_CERT_THUMBPRINT)
   $agentExe=Join-Path $root 'target/release/digital-discipline-agent.exe'
   $sig=Set-AuthenticodeSignature -FilePath $agentExe -Certificate $cert -HashAlgorithm SHA256
   if(-not $sig.SignerCertificate -or $sig.SignerCertificate.Thumbprint -ne $cert.Thumbprint){throw 'Agent Authenticode signature missing'}
   Write-Host "Agent signed: $($sig.SignerCertificate.Thumbprint)"
 }
 Push-Location 'apps/desktop'
 try {
  Write-Host '[4/6] Install locked app dependencies'
  npm ci --no-audit --no-fund
  if($LASTEXITCODE -ne 0){throw 'npm dependency install failed'}
  Write-Host '[5/6] Build application frontend'
  npm run build
  if($LASTEXITCODE -ne 0){throw 'Frontend compilation failed'}
  Write-Host '[6/6] Build Tauri NSIS installer'
  npm run tauri:build
  if($LASTEXITCODE -ne 0){throw 'Tauri installer failed'}
 } finally {Pop-Location}
 $dest=Join-Path $root 'release-output'
 New-Item -ItemType Directory -Force $dest | Out-Null
 Copy-Item (Join-Path $root 'target/release/digital-discipline-agent.exe') $dest -Force
 $desktopExe=Join-Path $root 'target/release/digital-discipline-desktop.exe'
 if(Test-Path $desktopExe){
   if($env:DD_DEV_CERT_THUMBPRINT){
     $cert=Get-Item ("Cert:\CurrentUser\My\" + $env:DD_DEV_CERT_THUMBPRINT)
     $desktopSignature=Set-AuthenticodeSignature -FilePath $desktopExe -Certificate $cert -HashAlgorithm SHA256
     if(-not $desktopSignature.SignerCertificate -or $desktopSignature.SignerCertificate.Thumbprint -ne $env:DD_DEV_CERT_THUMBPRINT){throw 'Desktop executable signing failed'}
     Write-Host "Desktop application executable signed with development certificate"
   }
   Copy-Item $desktopExe $dest -Force
 }
 $installers=Get-ChildItem (Join-Path $root 'target/release/bundle/nsis') -Filter '*.exe' -ErrorAction SilentlyContinue
 if(-not $installers){throw 'No NSIS installer generated'}
 $installers | Copy-Item -Destination $dest -Force
 if(Test-Path (Join-Path $root 'release-output/extension.zip')){Remove-Item (Join-Path $root 'release-output/extension.zip')}
 Compress-Archive -Path (Join-Path $root 'extension/*') -DestinationPath (Join-Path $root 'release-output/extension.zip')
 if($env:DD_DEV_CERT_THUMBPRINT){
   foreach($exe in @(Get-ChildItem $dest -Filter '*.exe')){
     $sig=Get-AuthenticodeSignature $exe.FullName
     if(-not $sig.SignerCertificate -or $sig.SignerCertificate.Thumbprint -ne $env:DD_DEV_CERT_THUMBPRINT){throw "Unsigned or mismatched development certificate: $($exe.Name)"}
     Write-Host "DEV signature verified (untrusted root expected): $($exe.Name) ($($sig.Status))"
   }
 }
 Write-Host "Output: $dest (development self-signed signatures do NOT establish Windows publisher trust)"
} finally {Pop-Location}
