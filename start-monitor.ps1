#Requires -Version 5.1
# Startet presence-monitor.exe (Rust) im Hintergrund.
# Wird von HKCU-Run und install-task.ps1 verwendet.

$ErrorActionPreference = 'Stop'
$dir = $PSScriptRoot
$exe = Join-Path $dir 'presence-monitor.exe'

if (-not (Test-Path $exe)) {
    Write-Error "presence-monitor.exe fehlt. Release-Asset nach '$dir' legen oder: gh release download v2.0.0 --pattern presence-monitor.exe"
    exit 1
}

$running = Get-Process -Name 'presence-monitor' -ErrorAction SilentlyContinue
if ($running) {
    Write-Host "presence-monitor laeuft bereits (PID $($running.Id -join ', '))"
    exit 0
}

Start-Process -FilePath $exe -ArgumentList 'run' -WindowStyle Hidden -WorkingDirectory $dir
Write-Host "presence-monitor gestartet: $exe run"