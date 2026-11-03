$registry='HKCU:\Software\Microsoft\Windows\CurrentVersion\Run'
if(Get-ItemProperty -Path $registry -Name 'DigitalDisciplineAgent' -ErrorAction SilentlyContinue){
 Remove-ItemProperty -Path $registry -Name 'DigitalDisciplineAgent'
 Write-Host 'Digital Discipline agent startup entry removed.'
}
