# MISSION — presence-monitor

**Wahrheitsquelle für Status/Architektur:** `START_HERE.md`. Bei Widerspruch
gewinnt `START_HERE.md`. Diese Datei ist der **aktuelle Arbeitsfokus** —
ändert sich häufiger, wird bei Themenwechsel überschrieben statt angehäuft.

---

## Arbeitsweise (verbindlich, projektübergreifend gültig)

1. **Aufwand = Aufgabengröße.** Kein Design-Doc für eine Ein-Zeilen-Änderung.
2. **Liefere funktionierenden Code, nicht Dokumente über Arbeit.**
3. **Kleine, verifizierte Commits.** Änderung real ausführen/testen und
   belegen, nie „fertig" ohne Beleg behaupten.
4. **Vor Vertrauen auf eine Zahl/Behauptung: selbst nachmessen.**

## Aktueller Fokus (Stand 2026-07-17)

Sauberes Projekt, keine kritischen Befunde. Offene Punkte sind alle
niedrige Priorität, absteigend:

1. **Loopback-Fallback in `ping.rs` klären** — bei nicht erreichbarem
   `device.target` fällt das Programm auf `127.0.0.1` zurück; ein
   erfolgreicher Loopback-Ping beweist keine echte Anwesenheit. Entweder
   Fallback entfernen oder `ping_present` in dem Fall konsistent `false`
   setzen.
2. **Hardware-Integrationstest dokumentieren oder ergänzen** — `mic.rs`
   (echte Aufnahme) und `tts.rs` (SAPI) sind nur via `self-check`/
   `run --once` manuell testbar. Entweder ein `#[ignore]`-Test, der unter
   Windows läuft und im CI übersprungen wird, oder im README explizit
   vermerken, dass diese Pfade nur manuell getestet sind.
3. **Env-Override-Test ergänzen** — `PRESENCE_*`-Overrides sind nicht in
   jedem Test-Pfad abgedeckt. Ein Unit-Test, der z. B.
   `PRESENCE_MIC_THRESH` gegen den `config.json`-Default verifiziert.

Details: `CODE_REVIEW.md`/`CLAUDE_PROPOSALS.md` (externer Review,
2026-07-16).

## Nicht jetzt

Portabilität auf andere Plattformen — bewusst kein Ziel (siehe
`CONVENTIONS.md`).
