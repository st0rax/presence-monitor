# CODE_REVIEW.md — presence-monitor (Rust v2.3.0)

**Datum:** 2026-07-16
**Reviewer:** Qwen (Code-Review, read-only)
**Build/Test-Status:** `cargo test` → 29 Tests PASS; `cargo build --release` erzeugt `presence-monitor.exe`
**Referenz:** Dieses Projekt ist eigenständig (kein Bezug zu webagent/webagent-rs). Python-Fallback `presence/` ist deprecated.

## Architektur-Überblick

10 Module unter `src/`:
- `main.rs` — CLI-Parsing (clap), Einstieg
- `config.rs` — `config.json` + `PRESENCE_*` Env-Overrides
- `state.rs` — reine Zustandsmaschine (Statuswechsel, Verifikations-Logik)
- `presence.rs` — Präsenz-Verdict: `(phone && wlan) || mic_rms>threshold || ping_present`
- `monitor.rs` — `process_cycle` orchestriert einen Zyklus
- `arp.rs`, `ping.rs` — Hardware-Probes (trait-injiziert)
- `mic.rs` — Mikrofon-Aufnahme + RMS-Sampling (trait-injiziert)
- `tts.rs` — Sprach-Ausgabe (SAPI, trait-injiziert)
- `clock.rs` — Zeit-Abstraktion

**Hardware-Seams sind sauber als Traits injiziert** (`PhoneProbe`, `MicRecorder`,
`MicLevelSampler`, `TtsEngine`, `PresenceProbe`). Das macht den Kern gut
unit-testbar und ist das stärkste konstruktive Merkmal des Projekts.

## Stärken

- Klare Trennung zwischen reiner Logik (`state.rs`) und Hardware (`*.rs` Traits).
- Kein `unsafe`, keine `unwrap()`-Explosion im Happy-Path (anyhow für Fehler).
- Dokumentation (`README.md`) ist sehr detailliert; messbare Ziele sind definiert.
- `cargo test` grün (29).

## Befunde (Minor)

| # | Schwere | Bereich | Befund |
|---|---------|---------|--------|
| P1 | Low | Tests | Kein **Integrationstest** mit echter Hardware (mic/tts/arp). Alle 29 Tests nutzen Mocks/Traits. Reale TTS-Wiedergabe & Mikrofon-RMS sind nur manuell via `self-check`/`run --once` verifizierbar. |
| P2 | Low | ping.rs | Legacy-Target `device.target` ist nur teilweise genutzt; bei nicht erreichbarem Host fällt das Programm auf `127.0.0.1` zurück (Loopback-Ping beweist keine echte Anwesenheit). Dokumentieren oder entfernen. |
| P3 | Info | monitor.rs | `response_cooldown_s` nutzt blockierendes `std::thread::sleep`. Bewusst so (ECHO-Abklingen) — kein Handlungsbedarf, nur als Info für Portabilität. |
| P4 | Info | Windows-only | SAPI (TTS) und `arp -a` sind Windows-spezifisch. Portabilität ist kein Ziel (laut README), daher kein Befund, nur Hinweis. |
| P5 | Low | config.rs | `config.json` wird geladen, aber Env-Overrides (`PRESENCE_*`) sind nicht in jedem Test-Pfad abgedeckt. Empfehlenswert: mind. ein Test, der Env-Override gegen Default verifiziert. |

## Sicherheit

Keine externen Netzwerk-Aufrufe außer lokalem ARP/Ping. Kein Shell-Exec.
Keine Secrets im Code. **Sauber.**

## Fazit

Solides, sauberes Rust-Projekt mit vorbildlicher Hardware-Abstraktion. Keine
kritischen Befunde. Empfehlung: optional einen Hardware-Integrationstest
hinzufügen (oder im README explizit als "manuell" markieren) und das
Loopback-Fallback in `ping.rs` klären.
