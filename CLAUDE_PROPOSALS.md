# CLAUDE_PROPOSALS.md — presence-monitor

**Für:** Claude (Dev) · **Von:** Qwen (Review) · **Datum:** 2026-07-16
**Status:** Optional / Non-blocking. Kein kritischer Befund.

Diese Vorschläge sind niedriger Priorität. presence-monitor ist in gutem Zustand.
Nur umsetzen, wenn Zeit vorhanden ist.

## P1 — Hardware-Integrationstest dokumentieren oder ergänzen
- `src/mic.rs` (echte Aufnahme) und `src/tts.rs` (SAPI) sind nur via
  `self-check`/`run --once` manuell testbar.
- **Vorschlag:** Einen `#[ignore]`-Integrationstest ergänzen, der echtes Mic/TTS
  unter Windows ausführt und im CI übersprungen wird. Oder im `README.md`
  explizit vermerken: "Hardware-Pfade sind manuell getestet, nicht im `cargo test`."
- **Akzeptanz:** `cargo test` bleibt grün; entweder neuer `#[ignore]`-Test oder
  README-Hinweis.

## P2 — Loopback-Fallback in ping.rs klären
- Wenn `device.target` nicht erreichbar ist, fällt das Programm auf `127.0.0.1`
  zurück. Ein erfolgreicher Loopback-Ping beweist keine echte Anwesenheit des
  OnePlus 9 Pro.
- **Vorschlag:** Fallback entweder entfernen (→ absent) oder als `ping_present=false`
  kennzeichnen, damit er nicht fälschlich als Anwesenheit zählt.
- **Akzeptanz:** Bei nicht erreichbarem Ziel ist `ping_present` konsistent false
  oder das Verhalten ist im README dokumentiert.

## P5 — Env-Override-Test ergänzen
- `config.rs` unterstützt `PRESENCE_*`-Env-Overrides, aber kein Test verifiziert
  sie gegen Defaults.
- **Vorschlag:** Ein Unit-Test, der `PRESENCE_MIC_THRESH` setzt und prüft, dass
  der geladene Wert den `config.json`-Default überschreibt.
- **Akzeptanz:** Neuer Test in `config.rs`-Modul, grün.

## Nicht ändern
- `state.rs` Zustandsmaschine: sauber, unverändert lassen.
- Trait-Injektion der Hardware: vorbildlich, beibehalten.
- `std::thread::sleep` in `response_cooldown_s`: bewusst, nicht ändern.
