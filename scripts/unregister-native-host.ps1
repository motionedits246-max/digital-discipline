param([ValidateSet('Brave','Chrome','Edge')][string]$Browser='Brave')
$names=@{Brave='BraveSoftware\Brave-Browser';Chrome='Google\Chrome';Edge='Microsoft\Edge'}
$registry="HKCU:\Software\$($names[$Browser])\NativeMessagingHosts\com.digitaldiscipline.bridge"
$owned=Join-Path $env:LOCALAPPDATA ("DigitalDiscipline\com.digitaldiscipline.bridge.$Browser.json")
if(Test-Path $registry){
 $current=(Get-Item $registry).GetValue('')
 if($current -eq $owned){Remove-Item $registry -Force;Write-Host 'Owned registry registration removed.'}
 else{Write-Warning 'Registration points elsewhere; not removing an unknown entry.'}
}
Write-Host 'This script does not delete history. Use Settings & Privacy to delete history explicitly.'
