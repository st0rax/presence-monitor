#Requires -Version 5.1
[CmdletBinding()]
param()

# Installiert presence-monitor als geplante Aufgabe, die bei jeder Anmeldung
# im Hintergrund startet und so fuer durchgehendes Logging sorgt.
# Benoetigt Administrator-Rechte (Register-ScheduledTask).
# Einfache, admin-freie Alternative: HKCU-Run-Eintrag (siehe README).
# Deinstallation:  Unregister-ScheduledTask -TaskName 'PresenceMonitor' -Confirm:$false

$ErrorActionPreference = 'Stop'
$script = Join-Path $PSScriptRoot 'presence_monitor.ps1'

$action = New-ScheduledTaskAction -Execute 'powershell.exe' -Argument (
    "-ExecutionPolicy Bypass -WindowStyle Hidden -File `"$script`""
)
$trigger = New-ScheduledTaskTrigger -AtLogOn
$settings = New-ScheduledTaskSettingsSet `
    -AllowStartIfOnBatteries `
    -DontStopIfGoingOnBatteries `
    -StartWhenAvailable `
    -RestartCount 3 `
    -RestartInterval (New-TimeSpan -Minutes 1)

Register-ScheduledTask -TaskName 'PresenceMonitor' `
    -Action $action -Trigger $trigger -Settings $settings `
    -Description 'Ping-basierter Praesenz-Monitor (OnePlus 9 Pro)' -Force

Start-ScheduledTask -TaskName 'PresenceMonitor'
Write-Host "PresenceMonitor als geplante Aufgabe installiert und gestartet."
