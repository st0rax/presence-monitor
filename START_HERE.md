# START HERE — presence-monitor

**Stand:** 2026-07-17 · Lies diese Datei zuerst, komplett, bevor du andere
Dokumente öffnest. Sie ist in sich geschlossen — du brauchst kein anderes
Repo und kein Vorwissen, um hier weiterzuarbeiten.

> 🔧 **Pflegepflicht:** Wer hier strukturell etwas ändert (neue Module,
> geänderter Test-/Release-Status, neue Config-Keys) aktualisiert diese
> Datei **als Teil derselben Änderung**, nicht als Nachtrag. Gilt unabhängig
> vom verwendeten Tool/Agenten.

---

## 0. Was ist das

„Bin ich zuhause?" via Netzwerk (OnePlus 9 Pro im ARP-Table + Ping) und
Mikrofon-RMS, mit TTS-Begrüßung bei Rückkehr und Panikmodus bei ausbleibender
Antwort. **Komplett eigenständig**, keine Verbindung zu `webagent`/
`webagent-rs` oder `bot2bot` — siehe `README.md` für die vollständige
Funktionsbeschreibung (Panikmodus-Ablauf, Logging-Struktur, Config-Tabelle);
die steht dort schon gut und wird hier nicht dupliziert.

## 1. Architektur

10 Module unter `src/`, Hardware-Zugriffe sauber als Traits injiziert (das
ist die stärkste konstruktive Eigenschaft des Projekts — macht den Kern
unit-testbar ohne echte Hardware):

| Modul | Zweck |
|---|---|
| `main.rs` | CLI-Parsing (clap), Einstieg |
| `config.rs` | `config.json` + `PRESENCE_*` Env-Overrides |
| `state.rs` | reine Zustandsmaschine (Statuswechsel, Verifikations-Logik) |
| `presence.rs` | Präsenz-Verdict: `(phone && wlan) \|\| mic_rms>threshold \|\| ping_present` |
| `monitor.rs` | `process_cycle` orchestriert einen Zyklus |
| `arp.rs`, `ping.rs` | Hardware-Probes (trait-injiziert) |
| `mic.rs` | Mikrofon-Aufnahme + RMS-Sampling (trait-injiziert) |
| `tts.rs` | Sprachausgabe, SAPI (trait-injiziert) |
| `clock.rs` | Zeit-Abstraktion |

Traits: `PhoneProbe`, `MicRecorder`, `MicLevelSampler`, `TtsEngine`,
`PresenceProbe`.

## 2. Build/Test

```powershell
cargo build --release   # → target/release/presence-monitor.exe
cargo test               # 29 Tests, alle Hardware-Pfade gemockt
```

Voraussetzungen: Windows (PowerShell 5.1+), .NET Framework (System.Speech/
System.Media, standardmäßig vorhanden), gebündeltes ffmpeg unter `tools/`
(siehe `README.md` §Einrichtung). Windows-only per Design (SAPI, `arp -a`) —
Portabilität ist kein Ziel.

## 3. Aktueller Stand (2026-07-17, nachgemessen)

v2.3.0. `cargo test`: **29/29 grün** (bei mir gerade selbst nachgemessen,
nicht nur behauptet). Kein `unsafe`, keine `unwrap()`-Explosion im
Happy-Path, `anyhow` für Fehler.

**Externer Review vorhanden** (`CODE_REVIEW.md`/`CLAUDE_PROPOSALS.md`, Qwen,
2026-07-16, Testzahl exakt bestätigt): keine kritischen Befunde, nur
Nice-to-haves:
- Kein Hardware-Integrationstest (mic/tts/arp) — nur manuell via
  `self-check`/`run --once` verifizierbar. Vorschlag: `#[ignore]`-Test oder
  README-Hinweis, dass Hardware-Pfade manuell getestet werden.
- `ping.rs`: Loopback-Fallback auf `127.0.0.1`, wenn `device.target` nicht
  erreichbar ist — ein erfolgreicher Loopback-Ping beweist keine echte
  Anwesenheit. Klären: entfernen oder `ping_present=false` erzwingen.
- `config.rs`: `PRESENCE_*`-Env-Overrides nicht in jedem Test-Pfad
  abgedeckt — ein Test, der z. B. `PRESENCE_MIC_THRESH` gegen den
  `config.json`-Default verifiziert, fehlt.

## 4. Nicht verwechseln

Die Python-Referenz `presence/presence_check.py`
(`github.com/st0rax/home-presence`, kanonisch unter `Desktop\presence\`) ist
**deprecated**, bleibt nur Verhaltensvorlage. ⚠️ Ein Duplikat unter
`Desktop\webagent\presence\presence_check.py` ist **ungetrackt**
(`webagent/.gitignore` ignoriert `presence/`) — Änderungen dort gehen
verloren, immer die kanonische Datei bearbeiten, falls die Python-Referenz
je wieder angefasst wird (eigentlich: nicht anfassen, siehe oben).

Zwei weitere, komplett unabhängige Projekte existieren daneben: `webagent`/
`webagent-rs` (`github.com/st0rax/webagent-rs`) und `bot2bot`
(`github.com/st0rax/bot2bot`). Keine Schnittmenge, keine Abhängigkeit —
jedes Projekt hat seine eigene `START_HERE.md`.
