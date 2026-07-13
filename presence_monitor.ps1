#Requires -Version 5.1
[CmdletBinding()]
param(
    [switch]$SelfTest,
    [switch]$Once
)

$ErrorActionPreference = 'Stop'
$root = $PSScriptRoot
$config = Get-Content (Join-Path $root 'config.json') -Raw -Encoding UTF8 | ConvertFrom-Json

# --- Pfade ---------------------------------------------------------------
$logDir    = Join-Path $root $config.logging.log_dir
$audioDir  = Join-Path $root $config.logging.audio_dir
$transDir  = Join-Path $root $config.logging.transition_dir
$stateFile = Join-Path $root 'state.json'
$mainLog   = Join-Path $logDir 'presence.log'
@($logDir, $audioDir, $transDir) | ForEach-Object { if (-not (Test-Path $_)) { New-Item -ItemType Directory -Force -Path $_ | Out-Null } }

# --- ffmpeg (gebündelt unter tools/) -------------------------------------
$ffmpeg = Get-ChildItem (Join-Path $root 'tools') -Recurse -Filter ffmpeg.exe | Select-Object -First 1
if (-not $ffmpeg) { throw "ffmpeg.exe nicht gefunden. Bitte das gebündelte ffmpeg aus tools/ entpacken." }
$ffmpeg = $ffmpeg.FullName

function Write-Log($msg) {
    $ts = Get-Date -Format "yyyy-MM-ddTHH:mm:ss"
    "$ts $msg" | Tee-Object -FilePath $mainLog -Append | Write-Host
}

# ffmpeg schreibt Diagnose nach stderr; das wuerde unter 'Stop' als Fehler
# gewertet. Dieser Wrapper fuengt stdout+stderr ein, ohne zu brechen.
function Invoke-FFmpeg($ffArgs) {
    $prev = $ErrorActionPreference
    $ErrorActionPreference = 'Continue'
    try { & $ffmpeg @ffArgs 2>&1 }
    finally { $ErrorActionPreference = $prev }
}

# --- Mikrofon ------------------------------------------------------------
function Get-DefaultMic {
    $lines = Invoke-FFmpeg @('-hide_banner', '-list_devices', 'true', '-f', 'dshow', '-i', 'dummy')
    $devs = @()
    foreach ($l in $lines) {
        if ($l -match '"([^"]+)"\s*\(audio\)') { $devs += $Matches[1] }
    }
    if ($devs.Count -eq 0) { throw "Kein DirectShow-Audio-Geraet gefunden." }
    return $devs[0]
}

$micName = Get-DefaultMic
Write-Log ("Mikrofon: $micName")

function Record-Clip($seconds, $outFile) {
    Invoke-FFmpeg @('-y', '-hide_banner', '-loglevel', 'error', '-f', 'dshow', '-i', "audio=$micName", '-t', "$seconds", '-ac', '1', '-ar', '16000', $outFile)
    if (-not (Test-Path $outFile)) { throw "Aufnahme fehlgeschlagen: $outFile" }
}

function Get-ClipLevels($wavFile) {
    $out = Invoke-FFmpeg @('-hide_banner', '-i', $wavFile, '-af', 'volumedetect', '-f', 'null', '-')
    $m = ($out | Select-String 'max_volume:\s*([-\d.]+)\s*dB') | Select-Object -First 1
    if ($m) { return [float]$m.Matches.Groups[1].Value }
    return -100.0
}

function Record-VerifyClip($seconds) {
    $ts = Get-Date -Format "yyyyMMdd_HHmmss"
    $file = Join-Path $audioDir "verify_$ts.wav"
    Record-Clip $seconds $file
    $maxDb = Get-ClipLevels $file
    $meta = [pscustomobject]@{
        timestamp = (Get-Date).ToString("o")
        type      = "verify"
        audioFile = $file
        maxDb     = [math]::Round($maxDb, 2)
        durationS = $seconds
    }
    $meta | ConvertTo-Json | Set-Content (Join-Path $transDir "verify_$ts.json") -Encoding UTF8
    return $meta
}

# --- TTS (System.Speech) -------------------------------------------------
function Invoke-TTS($text, $lang) {
    Add-Type -AssemblyName System.Speech
    Add-Type -AssemblyName System.Media
    $synth = New-Object System.Speech.Synthesis.SpeechSynthesizer
    $voice = $synth.GetInstalledVoices() | Where-Object { $_.VoiceInfo.Culture.Name -like "$lang*" } | Select-Object -First 1
    if ($voice) { try { $synth.SelectVoice($voice.VoiceInfo.Name) } catch { } }
    $ts = Get-Date -Format "yyyyMMdd_HHmmss"
    $file = Join-Path $audioDir "tts_$ts.wav"
    $synth.SetOutputToWaveFile($file)
    $synth.Speak($text)
    $synth.Dispose()
    # laut abspielen (fuer Protokoll zusaetzlich als WAV gespeichert)
    $player = New-Object System.Media.SoundPlayer($file)
    $player.PlaySync()
    $player.Dispose()
    return $file
}

function Test-DevicePresent($target, $timeoutMs) {
    $null = ping.exe -n 1 -w $timeoutMs $target
    return ($LASTEXITCODE -eq 0)
}

function Wait-ForResponse($timeoutS, $chunkS, $thresholdDb) {
    $elapsed = 0
    while ($elapsed -lt $timeoutS) {
        $ts = Get-Date -Format "yyyyMMdd_HHmmss"
        $chunk = Join-Path $audioDir "resp_$ts.wav"
        Record-Clip $chunkS $chunk
        $maxDb = Get-ClipLevels $chunk
        if ($maxDb -gt $thresholdDb) {
            Write-Log ("antwort-erkannt (clip: $chunk, max=$maxDb dB)")
            return $true
        }
        Remove-Item $chunk -ErrorAction SilentlyContinue
        $elapsed += $chunkS
    }
    return $false
}

# --- State ---------------------------------------------------------------
if (Test-Path $stateFile) { $state = Get-Content $stateFile -Raw -Encoding UTF8 | ConvertFrom-Json }
else { $state = [pscustomobject]@{ present = $null; lastTransition = $null } }

function Process-Cycle {
    $present = Test-DevicePresent $config.device.target $config.device.ping_timeout_ms
    $stateNow = if ($present) { 'present' } else { 'absent' }
    Write-Log "ping $($config.device.target) -> $stateNow"
    if ($null -eq $state.present) {
        $state.present = $present
        $state.lastTransition = (Get-Date).ToString("o")
        $state | ConvertTo-Json | Set-Content $stateFile -Encoding UTF8
        Write-Log "initial state = $stateNow (kein Uebergang)"
        return
    }
    if ($present -ne $state.present) {
        $from = if ($state.present) { 'present' } else { 'absent' }
        $to = $stateNow
        Write-Log "UEBERGANG $from -> $to"
        $meta = Record-VerifyClip $config.mic.verify_seconds
        Write-Log ("verify clip: $($meta.audioFile) max=$($meta.maxDb)dB")
        if ($to -eq 'present') {
            $g = Invoke-TTS $config.tts.greeting_text $config.tts.greeting_language
            Write-Log ("begruessung (TTS): $g")
            Write-Log ("warte $($config.mic.response_cooldown_s)s bis Mikrofon-Echo der Begruessung abgeklungen ist")
            Start-Sleep -Seconds $config.mic.response_cooldown_s
            $answered = Wait-ForResponse $config.mic.response_timeout_s $config.mic.chunk_seconds $config.mic.speech_threshold_db
            if ($answered) {
                Write-Log "antwort erkannt (Storax hat geantwortet)"
            } else {
                $p = Invoke-TTS $config.tts.panic_text $config.tts.panic_language
                Write-Log ("PANICMODE: $p")
            }
        }
        $state.present = $present
        $state.lastTransition = (Get-Date).ToString("o")
        $state | ConvertTo-Json | Set-Content $stateFile -Encoding UTF8
    }
}

# --- Entrypoints ---------------------------------------------------------
if ($SelfTest) {
    try {
        Write-Log "SELFTEST start"
        $m = Record-VerifyClip ([int]3)
        Write-Log ("selftest clip: $($m.audioFile) max=$($m.maxDb)dB")
        $gt = Invoke-TTS $config.tts.greeting_text $config.tts.greeting_language
        Write-Log ("selftest greeting: $gt")
        Write-Log "SELFTEST ende"
    } catch {
        Write-Log ("SELFTEST FEHLER: " + $_.Exception.ToString())
        throw
    }
    return
}

if ($Once) {
    Process-Cycle
} else {
    while ($true) {
        Process-Cycle
        Start-Sleep -Seconds $config.device.check_interval_s
    }
}
