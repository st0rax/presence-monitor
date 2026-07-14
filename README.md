# presence-monitor

Eigenständiger Präsenz-Monitor für den Rechner von **storax**.
Präsenz wird über **ARP-MAC** (OnePlus 9 Pro in `arp -a`) und **Mikrofon-RMS**
festgestellt — siehe `device.phone_mac_prefix` und `mic.rms_threshold` (default
`0.01`, Env `PRESENCE_MIC_THRESH`). Jeder Statuswechsel
(anwesend ⇄ abwesend) wird zusätzlich über das Mikrofon verifiziert, wobei
ein 30-Sekunden-Mitschnitt gespeichert wird. Bei der Rückkehr (abwesend →
anwesend) erfolgt eine hörbare TTS-Begrüßung; bleibt diese 5 Minuten
unbeantwortet, schaltet das Programm in den **Panikmodus** und meldet seinen
Zustand per TTS.

Dieses Projekt ist **völlig eigenständig** und steht in keiner Verbindung zu
anderen Projekten (z. B. webagent/webagent-rs).

## Rust-Port (v2)

Ab `feat/rust-port` gibt es einen nativen Rust-Build (`presence-monitor.exe`),
der die PowerShell-Logik ersetzt. Build:

```bash
cargo build --release   # → target/release/presence-monitor.exe
```

CLI: `presence-monitor run` (Dauerbetrieb), `presence-monitor run --once`,
`presence-monitor self-check`. **Produktion (storax):** Rust-Build via Release
`v2.0.0`, Autostart über `start-monitor.ps1`. PowerShell (`presence_monitor.ps1`)
bleibt als Fallback.

## Funktionsweise

1. **Präsenz-Check** alle `check_interval_s` Sekunden (ARP + Mic-RMS).
   - present → storax ist zuhause
   - absent → storax ist nicht zuhause
2. Bei jedem **Übergang** wird ein **30-Sekunden-Mitschnitt** des Mikrofons
   aufgezeichnet und als WAV gespeichert (Verifikation der Erkennung).
3. Bei Übergang **absent → present**:
   - hörbare **TTS-Begrüßung** (fr-FR: „Frère Jacques“).
   - nach einer kurzen Pause (Echo-Abklingen) wartet das Programm bis zu
     `response_timeout_s` (300 s) auf eine Antwort von storax (Sprachpegel
     über `speech_threshold_db`).
   - **Antwort erkannt** → Log-Eintrag „antwort erkannt“.
   - **keine Antwort** innerhalb von 5 Minuten → **Panikmodus**: TTS meldet
     den Zustand auf Deutsch.
4. **Dauerhaftes Protokoll**: Text-Log, JSON-Metadaten pro Übergang und alle
   Audio-Dateien (Verifikation, TTS, Antwort-Clips) sind sauber strukturiert
   unter `logs/` abgelegt.

## Voraussetzungen

- Windows (PowerShell 5.1+)
- .NET Framework (für `System.Speech` / `System.Media`) — unter Windows
  standardmäßig vorhanden
- gebündeltes **ffmpeg** unter `tools/` (siehe unten)

## Einrichtung

1. Dieses Repository klonen/entpacken.
2. Die gebündelte `ffmpeg.zip` nach `tools/` entpacken, sodass
   `tools/ffmpeg-<version>-essentials_build/bin/ffmpeg.exe` existiert
   (enthalten im GitHub-Release unter „Assets“).
3. `config.json` bei Bedarf anpassen (Ziel-Host, Intervalle, Schwellen).

## Nutzung

```powershell
# Dauerhaft im Vordergrund überwachen:
powershell -ExecutionPolicy Bypass -File presence_monitor.ps1

# Einen einzelnen Zyklus ausführen:
powershell -ExecutionPolicy Bypass -File presence_monitor.ps1 -Once

# Selbsttest (Mikrofon + TTS, ohne Dauerlauf):
powershell -ExecutionPolicy Bypass -File presence_monitor.ps1 -SelfTest
```

## Autostart (dauerhaftes Logging)

Damit der Monitor auch ohne offene Session **laufend** protokolliert:

- **Einfach (ohne Admin):** HKCU-Run startet `start-monitor.ps1` bei Anmeldung
  (Rust-Binary). Ist eingerichtet auf diesem Rechner:
  ```
  powershell.exe -ExecutionPolicy Bypass -WindowStyle Hidden -File C:\Users\storax\Desktop\presence-monitor\start-monitor.ps1
  ```
  Manuell starten: `powershell -ExecutionPolicy Bypass -File start-monitor.ps1`
  Entfernen: `reg delete "HKCU\Software\Microsoft\Windows\CurrentVersion\Run" /v PresenceMonitor /f`

- **Robust (mit Auto-Neustart, benötigt Admin):** `install-task.ps1` legt eine
  geplante Aufgabe an, die bei Anmeldung startet und bei Absturz neu startet.
  Als Administrator ausführen:
  ```
  powershell -ExecutionPolicy Bypass -File install-task.ps1
  ```
  Deinstallieren: `Unregister-ScheduledTask -TaskName 'PresenceMonitor' -Confirm:$false`

## Logging-Struktur

```
logs/
  presence.log            # laufendes Text-Protokoll (alle Ereignisse)
  audio/
    verify_YYYYMMDD_HHMMSS.wav   # 30s Verifikations-Mitschnitt pro Übergang
    tts_YYYYMMDD_HHMMSS.wav      # Begrüßung / Panikmodus-Ansage
    resp_YYYYMMDD_HHMMSS.wav     # erkannte Antwort-Clips (sonst gelöscht)
  transitions/
    verify_YYYYMMDD_HHMMSS.json  # Metadaten (Zeitstempel, max dB, Dauer)
state.json                # letzter bekannter Präsenzstatus
```

## Konfiguration (config.json)

| Schlüssel | Bedeutung |
| --- | --- |
| `device.target` | Anzupingender Host (OnePlus 9 Pro) |
| `device.ping_timeout_ms` | Ping-Timeout |
| `device.check_interval_s` | Wartezeit zwischen den Prüfungen |
| `mic.verify_seconds` | Länge des Verifikations-Mitschnitts (30 s) |
| `mic.response_timeout_s` | Antwort-Frist nach Begrüßung (300 s = 5 min) |
| `mic.response_cooldown_s` | Pause nach Begrüßung vor dem Lauschen |
| `mic.speech_threshold_db` | Pegel-Schwelle, ab der „Antwort“ gilt |
| `mic.chunk_seconds` | Länge der Lausch-Abschnitte |
| `tts.greeting_language` / `tts.panic_language` | Sprache der Ansagen |
| `tts.greeting_text` / `tts.panic_text` | Ansage-Text |

## Plan & überprüfbare Ziele

| # | Ziel | Überprüfbar durch |
| --- | --- | --- |
| 1 | Präsenz-Erkennung via Ping des OnePlus 9 Pro | `presence.log` zeigt `ping oneplus9pro.local -> present/absent` |
| 2 | Bei jedem Statuswechsel 30s Mikrofon-Verifikation | `logs/audio/verify_*.wav` + `logs/transitions/verify_*.json` existieren pro Übergang |
| 3 | Keine Idle-/Tastatur-Erkennung (nur Ping) | Quellcode enthält keine `GetLastInputInfo`/Idle-Logik |
| 4 | Begrüßung bei Rückkehr hörbar per TTS | `tts_*.wav` wird erzeugt **und** über Lautsprecher abgespielt |
| 5 | Antwort-Pflicht von storax innerhalb 5 min | `presence.log` enthält „antwort erkannt“ oder „PANICMODE“ nach Begrüßung |
| 6 | Panikmodus bei ausbleibender Antwort | `presence.log` Eintrag `PANICMODE` + `tts_*.wav` (de-DE) |
| 7 | Lückenloses, nachvollziehbares Protokoll | `logs/` enthält Log + Audio + JSON, chronologisch sortiert |
| 8 | Selbsttest ohne Dauerlauf möglich | `-SelfTest` läuft fehlerfrei durch |
