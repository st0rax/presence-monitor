# START HERE — presence-monitor

> Willkommen in **presence-monitor**. Dieses Repo folgt dem **Bazaar-Modell**:
> mehrere Agents (LLM) arbeiten freiwillig an einer gemeinsamen Aufgabentafel,
> koordiniert über einen immer grünen Stamm. Dein Job: **einen Task claimen,
> umsetzen, testen, klein mergen** — und den Zustand so hinterlassen, dass der
> nächste (oder der Mensch) ohne dein Kopf-Wissen weiter kann.

Diese Datei ist der **Bazaar-Einstieg**. Sie ist **nicht** die einzige Datei
und nicht „in sich geschlossen“. Dummy (`st0rax/dummy-bazaar`) ist nur die
Prozess-Vorlage: Satz „einzige Einstiegsdatei“, Dummy-Platzhalter und
`*@webagent.local` **nicht** hierher kopieren. Dieses Repo **ist** Rust —
`cargo test` / `cargo test --lib` ist hier das richtige Gate.

> 🔧 **Pflegepflicht:** Wer hier strukturell etwas ändert (neue Module,
> geänderter Test-/Release-Status, neue Config-Keys, Bazaar-Dateien)
> aktualisiert diese Datei **als Teil derselben Änderung**, nicht als Nachtrag.

## Schutz und Pflicht-Lese (diese Reihenfolge)

Bestehende Schutz- und Projektdateien gelten **zuerst**. Bei Widerspruch
gewinnt die **strengere** Regel — nicht Dummy-Gewohnheit.

1. `AGENTS.md` — verbindliche Arbeitsdirektive (Schutz, zwölf Regeln)
2. `GOALS.md` — Nordstern **G-001** (kein TASKBOARD-Claim, niemand claimt ihn)
3. `docs/WORK_CONTRACT.md` — freiwilliger Bazaar-Leitfaden
4. `docs/TASKBOARD.json` — **einzige** Claim-Tafel (JSON ist Wahrheit)
5. Dann nach Bedarf: `MISSION.md` (aktueller Fokus), `README.md` (Funktion,
   Config, Logging), `CONVENTIONS.md`, `src/`

Markdown-Boards nicht anlegen und nicht mit JSON „synchron halten“.
Ein Markdown-Spiegel darf hinterherhinken — JSON gewinnt.

## In 60 Sekunden (Bazaar)

1. **Zustand:** `docs/TASKBOARD.json` — erste Zelle `free`, deren `depends_on`
   alle `done` sind. Das ist dein Kandidat. Niedrigste ID bei Gleichstand.
2. **Claim nur in JSON:** `status=claimed`, `owner`, `branch`, `claimed_at`.
   Eine JSON-`id`, ein Entwickler. **G-001 ist kein Claim.**
3. **Branch** von `master`: `docs|feature|fix|chore|refactor|test/<id>-<kurz>`.
4. **Eine Sache** pro Zweig, so klein wie möglich.
5. **Verifizieren** mit dem `verification`-Feld der Zelle — in diesem Repo
   typisch `cargo test` und/oder `cargo test --lib` plus CI
   (`.github/workflows/ci.yml`).
6. **PR gegen `master`.** `master` bleibt grün. Nicht force-pushen.
7. **Done** nur mit Beleg: `status=done`, `proof_path`, `done_at`. Behauptung
   ohne Beleg gilt nicht.

## 0. Was ist das

„Bin ich zuhause?" via Netzwerk (OnePlus 9 Pro im ARP-Table + Ping) und
Mikrofon-RMS, mit TTS-Begrüßung bei Rückkehr und Panikmodus bei ausbleibender
Antwort. **Komplett eigenständig**, keine Verbindung zu `webagent`/
`webagent-rs` oder `bot2bot` — siehe `README.md` für die vollständige
Funktionsbeschreibung (Panikmodus-Ablauf, Logging-Struktur, Config-Tabelle);
die steht dort schon gut und wird hier nicht dupliziert.

## Kern vs. Rest

- **Monitor (Kern):** `src/` (10 Module), `Cargo.toml`, `config.json`.
  Hardware-Zugriffe als Traits injiziert — Kern ist unit-testbar ohne echte
  Hardware.
- **Bazaar:** `GOALS.md`, `docs/WORK_CONTRACT.md`, `docs/TASKBOARD.json`,
  `docs/GIT_AGENTS.md` (Identitäten `*@presence.local`).
- **Schutz / Fokus:** `AGENTS.md` (nicht flachklappen), `MISSION.md`
  (aktueller Fokus, wechselt öfter — Claims stehen in JSON, nicht hier).
- **Betrieb:** `start-monitor.ps1`, `install-task.ps1`, PowerShell-Fallback
  `presence_monitor.ps1` (nicht empfohlen).
- **Historie:** `docs/legacy-python-reference.md` — Python ist deprecated.

**Wahrheit bei Widerspruch:** `AGENTS.md` (Schutz) → diese Datei →
`docs/TASKBOARD.json` (Claims) → `README.md` / `MISSION.md`. Ältere Docs
verlieren.

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

Identitäten: `docs/GIT_AGENTS.md` (`*@presence.local`). Nicht
`*@hombot.local`, nicht `*@webagent.local`, nicht `*@bot2bot.local`.

## 3. Aktueller Stand (ehrlich)

v2.3.0. `cargo test`: **29/29 grün** (Stand 2026-07-17, nachgemessen in
`START_HERE.md`/`CODE_REVIEW.md`, nicht hier neu gemessen). Kein `unsafe`,
keine `unwrap()`-Explosion im Happy-Path, `anyhow` für Fehler.

**Externer Review** (`CODE_REVIEW.md`/`CLAUDE_PROPOSALS.md`, Qwen,
2026-07-16): keine kritischen Befunde. Offene Nice-to-haves stehen als
Zellen in `docs/TASKBOARD.json` (JSON ist die Tafel, nicht MISSION):

- **PM-011** Loopback-Fallback: leeres `device.target` wird in `monitor.rs`
  auf `127.0.0.1` gesetzt — ein erfolgreicher Loopback-Ping beweist keine
  echte Anwesenheit.
- **PM-012** Kein Hardware-Integrationstest (mic/tts/arp) — nur manuell via
  `self-check`/`run --once`.
- **PM-013** `PRESENCE_*`-Env-Overrides nicht in jedem Test-Pfad abgedeckt.

**CI existiert:** `.github/workflows/ci.yml` (`cargo fmt`, `clippy -D warnings`,
`cargo test --verbose`, `cargo build --release`) auf `windows-latest`,
`on.push` / `on.pull_request`. Default-Branch ist `master`.

## 4. Nicht verwechseln

Die Python-Referenz `presence/presence_check.py`
(`github.com/st0rax/home-presence`, kanonisch unter `Desktop\presence\`) ist
**deprecated**. Ihre verbliebenen Verhaltensdetails sind jetzt im aktuellen
Repository unter [`docs/legacy-python-reference.md`](docs/legacy-python-reference.md)
festgehalten; dieser Rust-Bestand ist die alleinige Implementierungs- und
Dokumentationsquelle. ⚠️ Ein Duplikat unter
`Desktop\webagent\presence\presence_check.py` ist **ungetrackt**
(`webagent/.gitignore` ignoriert `presence/`) — Änderungen dort gehen
verloren und dürfen nicht als Quelle verwendet werden.

Zwei weitere, komplett unabhängige Projekte existieren daneben: `webagent`/
`webagent-rs` (`github.com/st0rax/webagent-rs`) und `bot2bot`
(`github.com/st0rax/bot2bot`). Keine Schnittmenge, keine Abhängigkeit —
jedes Projekt hat seine eigene `START_HERE.md`.

## Tabu

- Dummy, HomBot (`st0rax/hombot-uberbot`), `webagent-rs`, jewtube, bot2bot
  von hier aus **nicht** patchen.
- Kein Dummy-`PLAN.md` / DOSSIER / PENSIEVE. Kein HomBot-Eis / UART /
  `STATUS_LIVE`.
- Keine Secrets, Tokens, Host-Pfade mit Credentials in git.
- Verkettete TASKBOARD-Zellen nie parallel ziehen (`depends_on` = Kante).
- `AGENTS.md` und `MISSION.md` nicht flachklappen.
